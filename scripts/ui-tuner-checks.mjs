import assert from "node:assert/strict";
import { join } from "node:path";

const cases = [
  {
    id: "tuned",
    state: "detected",
    note: "A4",
    frequency: "440.00 Hz",
    cents: "0.0 cents",
    pointer: "0",
  },
  {
    id: "flat",
    state: "detected",
    note: "A4",
    frequency: "427.47 Hz",
    cents: "-50.0 cents",
    pointer: "-50",
  },
  {
    id: "sharp",
    state: "detected",
    note: "A4",
    frequency: "452.89 Hz",
    cents: "+50.0 cents",
    pointer: "50",
  },
  {
    id: "below",
    state: "detected",
    note: "A4",
    frequency: "421.35 Hz",
    cents: "-75.0 cents",
    pointer: "-50",
  },
  {
    id: "above",
    state: "detected",
    note: "A4",
    frequency: "459.48 Hz",
    cents: "+75.0 cents",
    pointer: "50",
  },
  {
    id: "note",
    state: "detected",
    note: "C4",
    frequency: "261.63 Hz",
    cents: "0.0 cents",
    pointer: "0",
  },
  { id: "listening", state: "listening" },
  { id: "silence", state: "no-pitch" },
  { id: "uncertain", state: "low-confidence" },
  { id: "error", state: "error" },
  { id: "idle", state: "idle" },
];

export async function verifyTuner({ page, runDir, record, assertBackground }) {
  const matrix = [];
  const captures = [];
  for (const [width, height] of [
    [1024, 700],
    [1440, 900],
  ]) {
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
      if ((await page.locator(".app-shell").getAttribute("data-theme")) !== theme)
        await page
          .getByRole("button", {
            name: theme === "dark" ? "切换为深色主题" : "切换为浅色主题",
            exact: true,
          })
          .click();
      await page
        .getByRole("navigation")
        .getByRole("button", { name: "实时调音", exact: true })
        .click();
      assert.equal(await page.locator(".tuner-workspace").getAttribute("data-mode"), "default");
      assert.equal((await page.getByLabel("当前音名", { exact: true }).textContent()).trim(), "—");
      assert.equal(await page.locator("[data-testid=tuning-needle]").count(), 0);
      assert.equal(await page.getByRole("meter").count(), 0);
      assert.equal(
        await page.getByRole("button", { name: "开始监听", exact: true }).isDisabled(),
        true,
      );
      const entry = page.getByRole("button", { name: "体验 Demo", exact: true });
      await entry.focus();
      await entry.press("Enter");
      const scenario = page.getByLabel("Demo 场景", { exact: true });
      await page.waitForFunction(() => document.activeElement?.id === "demo-scenario");
      await scenario.press("Home");
      await scenario.press("ArrowDown");
      await page.waitForFunction(
        () =>
          document.querySelector('output[aria-label="音分偏差"]')?.textContent.trim() ===
          "-50.0 cents",
      );
      assert.equal(await scenario.evaluate((el) => el === document.activeElement), true);
      const focus = await scenario.evaluate((el) => ({
        width: getComputedStyle(el).outlineWidth,
        style: getComputedStyle(el).outlineStyle,
      }));
      assert.ok(Number.parseFloat(focus.width) >= 2 && focus.style === "solid");
      if (width === 1024 && theme === "dark") {
        await page.screenshot({
          path: join(runDir, "demo-keyboard-focus.png"),
          animations: "disabled",
        });
        captures.push("demo-keyboard-focus.png");
      }
      for (const sample of cases) {
        await scenario.selectOption(sample.id);
        await page.waitForFunction(
          (expected) =>
            document.querySelector(".tuner-workspace")?.dataset.state === expected.state &&
            document.querySelector('output[aria-label="当前音名"]')?.textContent.trim() ===
              (expected.note ?? "—"),
          sample,
        );
        assert.equal(
          await page.locator(".tuner-workspace").getAttribute("data-state"),
          sample.state,
        );
        assert.equal(await page.locator(".tuner-workspace").getAttribute("data-mode"), "demo");
        assert.equal(await page.getByText("Demo · 模拟信号", { exact: true }).isVisible(), true);
        assert.equal(
          await page
            .getByText("当前为模拟数据，不代表麦克风测量结果。", { exact: true })
            .isVisible(),
          true,
        );
        const note = (await page.getByLabel("当前音名", { exact: true }).textContent()).trim();
        const frequency = (await page.getByLabel("当前频率", { exact: true }).textContent()).trim();
        const cents = (await page.getByLabel("音分偏差", { exact: true }).textContent()).trim();
        assert.equal(note, sample.note ?? "—");
        assert.equal(frequency, sample.frequency ?? "— Hz");
        assert.equal(cents, sample.cents ?? "— cents");
        const needle = page.locator("[data-testid=tuning-needle]");
        if (sample.pointer !== undefined)
          assert.equal(await needle.getAttribute("data-cents"), sample.pointer);
        else assert.equal(await needle.count(), 0);
        if (sample.id === "above" || sample.id === "below")
          assert.equal(
            await page.getByText("超出指针量程，数值完整保留", { exact: true }).isVisible(),
            true,
          );
        const layout = await page.evaluate(() => ({
          width: innerWidth,
          height: innerHeight,
          overflow:
            document.documentElement.scrollWidth > innerWidth ||
            document.documentElement.scrollHeight > innerHeight ||
            document.querySelector("main").scrollWidth >
              document.querySelector("main").clientWidth ||
            document.querySelector("main").scrollHeight >
              document.querySelector("main").clientHeight,
          clippedControls: [...document.querySelectorAll("button, select")].filter((el) => {
            const r = el.getBoundingClientRect();
            return r.left < 0 || r.top < 0 || r.right > innerWidth || r.bottom > innerHeight;
          }).length,
        }));
        assert.equal(layout.overflow, false, `${theme}/${width}/${sample.id} overflow`);
        assert.equal(layout.clippedControls, 0);
        let screenshot;
        if (width === 1024 || sample.id === "tuned") {
          screenshot = `demo-${theme}-${width}x${height}-${sample.id}.png`;
          await page.screenshot({ path: join(runDir, screenshot), animations: "disabled" });
          captures.push(screenshot);
        }
        matrix.push({
          theme,
          ...layout,
          scenario: sample.id,
          state: sample.state,
          note,
          frequency,
          cents,
          pointer: sample.pointer ?? null,
          screenshot,
        });
        await assertBackground();
        if (sample.id === "error") {
          await page.getByRole("button", { name: "重试 Demo", exact: true }).click();
          await page.waitForFunction(
            () =>
              document.querySelector('output[aria-label="当前音名"]')?.textContent.trim() === "A4",
          );
          assert.equal(
            (await page.getByLabel("当前音名", { exact: true }).textContent()).trim(),
            "A4",
          );
        }
      }
      const exit = page.getByRole("button", { name: "退出 Demo", exact: true });
      await exit.focus();
      await exit.press("Enter");
      await page.waitForFunction(
        () =>
          document.querySelector(".tuner-workspace")?.dataset.mode === "default" &&
          document.activeElement?.textContent.includes("体验 Demo"),
      );
      assert.equal(await entry.evaluate((el) => el === document.activeElement), true);
      assert.equal(
        (await page.getByLabel("当前频率", { exact: true }).textContent()).trim(),
        "— Hz",
      );
      assert.equal(await page.getByRole("meter").count(), 0);
      assert.equal(await page.getByLabel("Demo 场景", { exact: true }).count(), 0);
      await assertBackground();
      record(
        `${theme} ${width}x${height}: 11 explicit Demo states, unclamped numbers, cleared stale pitch, retry and keyboard exit passed`,
      );
    }
  }
  // Navigating away disposes the Demo session; returning must start empty.
  await page.getByRole("button", { name: "体验 Demo", exact: true }).click();
  await page.getByRole("navigation").getByRole("button", { name: "文件分析", exact: true }).click();
  await page.getByRole("navigation").getByRole("button", { name: "实时调音", exact: true }).click();
  assert.equal(await page.locator(".tuner-workspace").getAttribute("data-mode"), "default");
  assert.equal((await page.getByLabel("当前音名", { exact: true }).textContent()).trim(), "—");
  await assertBackground();
  record("leaving the tuner disposes Demo; returning never revives simulated measurements");
  return { matrix, captures, source: "opt-in fixed Demo fixtures; no microphone or DSP" };
}
