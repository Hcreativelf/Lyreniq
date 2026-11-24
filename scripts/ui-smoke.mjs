import assert from "node:assert/strict";
import { execFile, spawn } from "node:child_process";
import { createWriteStream } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { dirname, join, relative } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { chromium } from "playwright-core";
import { verifyShell } from "./ui-shell-checks.mjs";
import { verifyTuner } from "./ui-tuner-checks.mjs";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const runDir = join(root, ".local", "ui-smoke", new Date().toISOString().replace(/[:.]/g, "-"));
const runFile = promisify(execFile);
const checks = [];
const record = (message) => {
  checks.push(message);
  console.log(`[PASS] ${message}`);
};

async function freePort() {
  const server = createServer();
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}

async function waitUntil(action, timeout = 300_000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const result = await action();
    if (result) return result;
    await delay(250);
  }
  throw new Error(`Timed out after ${timeout} ms`);
}

async function portReleased(port) {
  const server = createServer();
  return new Promise((resolve) => {
    server.once("error", () => resolve(false));
    server.listen(port, "127.0.0.1", () => server.close(() => resolve(true)));
  });
}

await mkdir(runDir, { recursive: true });
const base = JSON.parse(
  await readFile(join(root, "apps/desktop/src-tauri/tauri.conf.json"), "utf8"),
);
const pkg = JSON.parse(await readFile(join(root, "package.json"), "utf8"));
assert.equal(
  base.app.windows.length,
  1,
  "Review the background harness before adding more windows",
);
const vitePort = await freePort();
let cdpPort = await freePort();
while (cdpPort === vitePort) cdpPort = await freePort();
const origin = `http://127.0.0.1:${vitePort}`;
const endpoint = `http://127.0.0.1:${cdpPort}`;
const profile = join(runDir, "app-data");
const configPath = join(runDir, "tauri-background.json");
const screenshot = join(runDir, "native-window.png");
const debugArgs = `--remote-debugging-port=${cdpPort} --remote-debugging-address=127.0.0.1`;
const testConfig = {
  build: { beforeDevCommand: `pnpm dev --port ${vitePort}`, devUrl: origin },
  app: {
    appDirectoriesOverride: profile,
    windows: [
      { ...base.app.windows[0], visible: false, focus: false, additionalBrowserArgs: debugArgs },
    ],
    security: {
      devCsp: base.app.security.devCsp.replaceAll(":1420", `:${vitePort}`),
      capabilities: [
        {
          identifier: "ui-smoke-only",
          windows: ["main"],
          permissions: [
            "core:window:allow-is-visible",
            "core:window:allow-is-focused",
            "core:window:allow-close",
            "core:window:allow-set-size",
          ],
        },
      ],
    },
  },
};
await writeFile(configPath, `${JSON.stringify(testConfig, null, 2)}\n`);

let app;
let exitResult;
let browser;
let failure;
let nativeWindowState = null;
let shellResults = null;
let tunerResults = null;
const log = createWriteStream(join(runDir, "native.log"));
try {
  const powershell = join(process.env.SystemRoot, "System32/WindowsPowerShell/v1.0/powershell.exe");
  app = spawn(
    powershell,
    ["-NoProfile", "-File", join(root, "scripts/dev.ps1"), "-BackgroundTestConfig", configPath],
    {
      cwd: root,
      windowsHide: true,
      stdio: ["ignore", "pipe", "pipe"],
      env: {
        ...process.env,
        CARGO_TARGET_DIR: join(root, "target/ui-smoke"),
        WEBVIEW2_USER_DATA_FOLDER: profile,
        CARGO_TERM_COLOR: "never",
        CARGO_TERM_PROGRESS_WHEN: "never",
      },
    },
  );
  app.stdout.on("data", (data) => log.write(data));
  app.stderr.on("data", (data) => log.write(data));
  app.on("error", (error) => {
    exitResult = { error: error.message };
  });
  app.on("close", (code, signal) => {
    exitResult = { code, signal };
  });
  console.log(`Starting isolated hidden Tauri window; log: ${relative(root, runDir)}/native.log`);
  await waitUntil(async () => {
    if (exitResult) throw new Error(`Native launch exited early: ${JSON.stringify(exitResult)}`);
    try {
      const response = await fetch(`${endpoint}/json/version`, {
        signal: AbortSignal.timeout(1000),
      });
      return response.ok;
    } catch {
      return false;
    }
  });
  browser = await chromium.connectOverCDP(endpoint);
  const page = await waitUntil(
    async () =>
      browser
        .contexts()
        .flatMap((context) => context.pages())
        .find((item) => item.url().startsWith(origin)),
    30_000,
  );
  await page.getByRole("heading", { name: "实时调音", exact: true, level: 1 }).waitFor();
  await page.getByText(`Lyreniq ${pkg.version} · windows`, { exact: true }).waitFor();
  record("real Tauri page rendered and Rust runtime_label succeeded");

  async function assertBackground() {
    const state = await page.evaluate(async () => {
      const invoke = window.__TAURI_INTERNALS__.invoke;
      return {
        visible: await invoke("plugin:window|is_visible", { label: "main" }),
        focused: await invoke("plugin:window|is_focused", { label: "main" }),
      };
    });
    nativeWindowState = state;
    assert.equal(state.visible, false, "Test window must stay hidden");
    assert.equal(state.focused, false, "Test window must not take focus");
  }
  await assertBackground();
  record("native window reports hidden and unfocused");
  const nativeReply = page.waitForResponse((response) => {
    const url = new URL(response.url());
    return (
      url.hostname === "ipc.localhost" &&
      url.pathname === "/runtime_label" &&
      response.request().method() === "POST"
    );
  });
  await page.getByRole("button", { name: "重新验证桌面连接", exact: true }).click();
  const response = await nativeReply;
  assert.equal(response.status(), 200);
  assert.equal(response.headers()["tauri-response"], "ok");
  const reply = response.headers()["content-type"].includes("application/json")
    ? await response.json()
    : await response.text();
  assert.equal(reply, `Lyreniq ${pkg.version} · windows`);
  await page.getByText(`Lyreniq ${pkg.version} · windows`, { exact: true }).waitFor();
  await assertBackground();
  record("reconnect button called Rust while window stayed hidden and unfocused");
  shellResults = await verifyShell({ page, runDir, record, assertBackground });
  tunerResults = await verifyTuner({ page, runDir, record, assertBackground });

  await page
    .evaluate(() => window.__TAURI_INTERNALS__.invoke("plugin:window|close", { label: "main" }))
    .catch((error) => {
      if (!/closed|destroyed/i.test(error.message)) throw error;
    });
  await waitUntil(async () => exitResult, 15_000);
  assert.equal(exitResult.code, 0, "Tauri/dev script must exit normally after closing its window");
  await waitUntil(
    async () => (await portReleased(cdpPort)) && (await portReleased(vitePort)),
    15_000,
  );
  record("native window closed normally; dev script exited 0 and both ports were released");
} catch (error) {
  failure = error;
  console.error(error.stack);
} finally {
  if (browser) await browser.close().catch(() => {});
  if (app?.pid && app.exitCode === null && !exitResult) {
    // Only terminate this harness's own process tree after an unsuccessful test.
    await runFile("taskkill.exe", ["/PID", String(app.pid), "/T", "/F"], {
      windowsHide: true,
    }).catch(() => {});
    await waitUntil(async () => exitResult, 5_000).catch(() => {});
  }
  await new Promise((resolve) => log.end(resolve));
  const result = {
    task: "T04",
    status: failure ? "failed" : "passed",
    checks,
    appVersion: pkg.version,
    mode: "hidden native Tauri/WebView2 over loopback CDP",
    requestedWindowVisible: false,
    requestedWindowFocused: false,
    nativeWindowState,
    shell: shellResults,
    tuner: tunerResults,
    resultDirectory: relative(root, runDir),
    screenshot: relative(root, screenshot),
    exit: exitResult ?? null,
    error: failure?.message ?? null,
    unverified: ["native titlebar interaction", "DPI/display scaling matrix", "audio devices"],
  };
  await writeFile(join(runDir, "result.json"), `${JSON.stringify(result, null, 2)}\n`);
  await writeFile(
    join(root, ".local/ui-smoke/latest.json"),
    `${JSON.stringify(result, null, 2)}\n`,
  );
  console.log(`UI result: ${result.status}; ${relative(root, runDir)}/result.json`);
}
if (failure) process.exitCode = 1;
