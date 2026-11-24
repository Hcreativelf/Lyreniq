import assert from "node:assert/strict";
import { copyFile } from "node:fs/promises";
import { join } from "node:path";

const workspaces = [
  ["tuner", "实时调音", "开始监听"],
  ["files", "文件分析", "导入音频"],
  ["tone", "参考音", "播放参考音"],
  ["settings", "设置", null],
];

export async function verifyShell({ page, runDir, record, assertBackground }) {
  const matrix = [];
  const captures = [];
  for (const [width, height] of [
    [1024, 700],
    [1440, 900],
  ]) {
    // This changes the actual hidden native window, not a browser viewport mock.
    await page.evaluate(
      async (size) => {
        await window.__TAURI_INTERNALS__.invoke("plugin:window|set_size", {
          label: "main",
          value: { Logical: size },
        });
      },
      { width, height },
    );
    await page.waitForFunction((size) => innerWidth === size.width && innerHeight === size.height, {
      width,
      height,
    });
    for (const theme of ["dark", "light"]) {
      const currentTheme = await page.locator(".app-shell").getAttribute("data-theme");
      if (currentTheme !== theme)
        await page
          .getByRole("button", {
            name: theme === "light" ? "切换为浅色主题" : "切换为深色主题",
            exact: true,
          })
          .click();
      for (const [id, label, action] of workspaces) {
        const nav = page.getByRole("navigation", { name: "工作区导航" });
        await nav.getByRole("button", { name: label, exact: true }).click();
        await page.getByRole("heading", { name: label, level: 1, exact: true }).waitFor();
        assert.equal(
          await nav.getByRole("button", { name: label, exact: true }).getAttribute("aria-current"),
          "page",
        );
        if (action)
          assert.equal(
            await page.getByRole("button", { name: action, exact: true }).isDisabled(),
            true,
          );
        const layout = await page.evaluate(() => {
          const shell = document.querySelector(".app-shell");
          const main = document.querySelector("main");
          const focusables = [
            ...document.querySelectorAll("nav button, .theme-toggle, .reconnect, .theme-option"),
          ];
          return {
            width: innerWidth,
            height: innerHeight,
            devicePixelRatio,
            scrollWidth: document.documentElement.scrollWidth,
            scrollHeight: document.documentElement.scrollHeight,
            mainOverflow:
              main.scrollWidth > main.clientWidth || main.scrollHeight > main.clientHeight,
            clippedControls: focusables.filter((element) => {
              const r = element.getBoundingClientRect();
              return r.left < 0 || r.top < 0 || r.right > innerWidth || r.bottom > innerHeight;
            }).length,
            theme: shell.dataset.theme,
          };
        });
        assert.equal(layout.theme, theme);
        assert.ok(
          layout.scrollWidth <= width && layout.scrollHeight <= height,
          "Document overflow",
        );
        assert.equal(layout.mainOverflow, false, "Workspace must fit without clipping");
        assert.equal(layout.clippedControls, 0);
        await assertBackground();
        const filename = `${theme}-${width}x${height}-${id}.png`;
        await page.screenshot({ path: join(runDir, filename), animations: "disabled" });
        captures.push(filename);
        matrix.push({ workspace: id, theme, ...layout, screenshot: filename });
      }
      record(
        `${theme} ${width}x${height}: all four native workspaces fit; actions stay unavailable`,
      );
    }
  }
  await copyFile(join(runDir, "dark-1024x700-tuner.png"), join(runDir, "native-window.png"));

  // Page-level CDP input never injects operating-system keyboard events.
  await page.locator("body").evaluate((body) => {
    body.tabIndex = -1;
    body.focus();
  });
  await page.keyboard.press("Tab");
  assert.equal(
    await page
      .getByRole("link", { name: "跳到主要内容" })
      .evaluate((el) => el === document.activeElement),
    true,
  );
  await page.keyboard.press("Enter");
  assert.equal(await page.locator("main").evaluate((el) => el === document.activeElement), true);
  await page.locator("body").evaluate((body) => body.focus());
  await page.keyboard.press("Tab");
  await page.keyboard.press("Tab");
  assert.equal(
    await page
      .getByRole("button", { name: "实时调音", exact: true })
      .evaluate((el) => el === document.activeElement),
    true,
  );
  await page.keyboard.press("Tab");
  await page.keyboard.press("Enter");
  assert.equal(
    await page
      .getByRole("heading", { name: "文件分析", level: 1 })
      .evaluate((el) => el === document.activeElement),
    true,
  );
  await page.getByRole("navigation").getByRole("button", { name: "设置", exact: true }).click();
  await page.keyboard.press("Tab");
  await page.keyboard.press("Space");
  assert.equal(await page.locator(".app-shell").getAttribute("data-theme"), "dark");
  await page.keyboard.press("Tab");
  await page.keyboard.press("Space");
  const light = page.getByRole("button", { name: "浅色主题", exact: true });
  assert.equal(await light.getAttribute("aria-pressed"), "true");
  const focus = await light.evaluate((element) => ({
    focused: document.activeElement === element,
    visible: element.matches(":focus-visible"),
    width: getComputedStyle(element).outlineWidth,
    style: getComputedStyle(element).outlineStyle,
  }));
  assert.equal(focus.focused, true);
  assert.equal(focus.visible, true);
  assert.equal(focus.style, "solid");
  assert.ok(Number.parseFloat(focus.width) >= 2);
  await page.screenshot({ path: join(runDir, "keyboard-focus.png"), animations: "disabled" });
  captures.push("keyboard-focus.png");
  await assertBackground();
  record(
    "CDP keyboard: skip link, Tab/Enter navigation, heading focus, Space theme selection and visible focus ring",
  );

  const contrast = [];
  for (const theme of ["dark", "light"]) {
    const toggle = theme === "dark" ? "切换为深色主题" : "切换为浅色主题";
    if ((await page.locator(".app-shell").getAttribute("data-theme")) !== theme)
      await page.getByRole("button", { name: toggle, exact: true }).click();
    const ratios = await page.locator(".app-shell").evaluate((shell) => {
      const css = getComputedStyle(shell);
      function luminance(hex) {
        const channels = hex
          .trim()
          .replace("#", "")
          .match(/../g)
          .map((part) => Number.parseInt(part, 16) / 255)
          .map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
        return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
      }
      return ["--text", "--muted", "--subtle"].flatMap((foreground) =>
        ["--bg", "--surface", "--sidebar"].map((background) => {
          const a = luminance(css.getPropertyValue(foreground));
          const b = luminance(css.getPropertyValue(background));
          return {
            foreground,
            background,
            ratio: (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05),
          };
        }),
      );
    });
    for (const sample of ratios)
      assert.ok(
        sample.ratio >= 4.5,
        `${theme} ${sample.foreground}/${sample.background} contrast ${sample.ratio}`,
      );
    contrast.push({ theme, samples: ratios });
  }
  record("both themes: primary, muted and secondary text tokens meet 4.5:1 on workspace surfaces");
  const scaling = [];
  const session = await page.context().newCDPSession(page);
  try {
    for (const scale of [1.25, 1.5]) {
      await session.send("Emulation.setDeviceMetricsOverride", {
        width: 1024,
        height: 700,
        deviceScaleFactor: scale,
        mobile: false,
      });
      await page
        .getByRole("navigation")
        .getByRole("button", { name: "实时调音", exact: true })
        .click();
      const metrics = await page.evaluate(() => ({
        width: innerWidth,
        height: innerHeight,
        devicePixelRatio,
        overflow:
          document.documentElement.scrollWidth > innerWidth ||
          document.documentElement.scrollHeight > innerHeight ||
          document.querySelector("main").scrollHeight > document.querySelector("main").clientHeight,
      }));
      assert.equal(metrics.width, 1024);
      assert.equal(metrics.height, 700);
      assert.equal(metrics.devicePixelRatio, scale);
      assert.equal(metrics.overflow, false);
      const filename = `renderer-scale-${scale}.png`;
      await page.screenshot({ path: join(runDir, filename), animations: "disabled" });
      captures.push(filename);
      scaling.push({
        ...metrics,
        screenshot: filename,
        mode: "CDP renderer emulation, not Windows display settings",
      });
      await assertBackground();
    }
  } finally {
    await session.send("Emulation.clearDeviceMetricsOverride");
    await session.detach();
  }
  record(
    "renderer-only 125%/150% scale checks fit at logical 1024x700; Windows display settings unchanged",
  );
  await assertBackground();
  return { matrix, captures, keyboardFocus: focus, contrast, scaling };
}
