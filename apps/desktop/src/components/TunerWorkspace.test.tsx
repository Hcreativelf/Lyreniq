import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { TunerState } from "../services/audio-backend";
import { type DemoBackend, demoStates } from "../services/demo-backend";
import { TunerWorkspace } from "./TunerWorkspace";

describe("tuner display", () => {
  it("defaults to unavailable input with no invented pitch or signal", () => {
    const { container } = render(<TunerWorkspace />);
    expect(container.firstChild).toHaveAttribute("data-mode", "default");
    expect(screen.getByLabelText("当前音名")).toHaveTextContent("—");
    expect(screen.getByLabelText("当前频率")).toHaveTextContent("— Hz");
    expect(screen.queryByTestId("tuning-needle")).not.toBeInTheDocument();
    expect(screen.queryByRole("meter")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Demo 场景")).not.toBeInTheDocument();
  });

  it.each([
    ["flat", "-50.0", -50, "音高偏低"],
    ["tuned", "0.0", 0, "音准合适"],
    ["sharp", "+50.0", 50, "音高偏高"],
    ["below", "-75.0", -50, "音高偏低"],
    ["above", "+75.0", 50, "音高偏高"],
  ])(
    "keeps %s numeric cents while constraining only the needle",
    async (scenario, numeric, pointer, direction) => {
      const user = userEvent.setup();
      render(<TunerWorkspace />);
      await user.click(screen.getByRole("button", { name: "体验 Demo" }));
      await user.selectOptions(screen.getByLabelText("Demo 场景"), String(scenario));
      expect(screen.getByText("Demo · 模拟信号")).toBeVisible();
      expect(screen.getByText("当前为模拟数据，不代表麦克风测量结果。")).toBeVisible();
      expect(screen.getByLabelText("音分偏差")).toHaveTextContent(`${numeric} cents`);
      expect(screen.getByTestId("tuning-needle")).toHaveAttribute("data-cents", String(pointer));
      expect(screen.getByText(String(direction))).toBeVisible();
      if (scenario === "above" || scenario === "below")
        expect(screen.getByText("超出指针量程，数值完整保留")).toBeVisible();
    },
  );

  it("clears pitch in non-detected states and replaces the old note on a note change", async () => {
    const user = userEvent.setup();
    render(<TunerWorkspace />);
    await user.click(screen.getByRole("button", { name: "体验 Demo" }));
    for (const scenario of ["listening", "silence", "uncertain", "error", "idle"]) {
      await user.selectOptions(screen.getByLabelText("Demo 场景"), "note");
      expect(screen.getByLabelText("当前音名")).toHaveTextContent("C4");
      expect(screen.getByLabelText("当前频率")).toHaveTextContent("261.63 Hz");
      await user.selectOptions(screen.getByLabelText("Demo 场景"), scenario);
      expect(screen.getByLabelText("当前音名")).toHaveTextContent("—");
      expect(screen.getByLabelText("当前频率")).toHaveTextContent("— Hz");
      expect(screen.queryByTestId("tuning-needle")).not.toBeInTheDocument();
    }
  });

  it("recovers a Demo error and clears every simulated value on exit", async () => {
    const user = userEvent.setup();
    render(<TunerWorkspace />);
    await user.click(screen.getByRole("button", { name: "体验 Demo" }));
    await user.selectOptions(screen.getByLabelText("Demo 场景"), "error");
    await user.click(screen.getByRole("button", { name: "重试 Demo" }));
    expect(screen.getByLabelText("Demo 场景")).toHaveFocus();
    expect(screen.getByLabelText("当前音名")).toHaveTextContent("A4");
    expect(screen.getByRole("meter", { name: "置信度" })).toHaveAttribute("value", "0.96");
    expect(screen.getByRole("meter", { name: "输入电平" })).toHaveAttribute("value", "-18");
    await user.click(screen.getByRole("button", { name: "退出 Demo" }));
    expect(screen.getByRole("button", { name: "体验 Demo" })).toHaveFocus();
    expect(screen.getByLabelText("当前音名")).toHaveTextContent("—");
    expect(screen.queryByRole("meter")).not.toBeInTheDocument();
    expect(screen.queryByText("Demo · 模拟信号")).not.toBeInTheDocument();
  });

  it("unsubscribes, stops and ignores a late frame from the old backend", async () => {
    let publish: (state: TunerState) => void = () => {};
    const unsubscribe = vi.fn();
    const stop = vi.fn(async () => {});
    const backend: DemoBackend = {
      source: "demo",
      subscribe(listener) {
        publish = listener;
        return unsubscribe;
      },
      async start() {
        publish(demoStates.tuned);
      },
      stop,
      selectScenario() {},
    };
    const user = userEvent.setup();
    render(<TunerWorkspace demoFactory={() => backend} />);
    await user.click(screen.getByRole("button", { name: "体验 Demo" }));
    expect(screen.getByLabelText("当前音名")).toHaveTextContent("A4");
    await user.click(screen.getByRole("button", { name: "退出 Demo" }));
    act(() => publish(demoStates.sharp));
    expect(screen.getByLabelText("当前音名")).toHaveTextContent("—");
    expect(unsubscribe).toHaveBeenCalledOnce();
    expect(stop).toHaveBeenCalledOnce();
  });

  it("removes a previous reading on invalid backend values", async () => {
    let publish: (state: TunerState) => void = () => {};
    const backend: DemoBackend = {
      source: "demo",
      subscribe(listener) {
        publish = listener;
        return () => {};
      },
      async start() {
        publish(demoStates.tuned);
      },
      async stop() {},
      selectScenario() {},
    };
    const user = userEvent.setup();
    render(<TunerWorkspace demoFactory={() => backend} />);
    await user.click(screen.getByRole("button", { name: "体验 Demo" }));
    act(() =>
      publish({
        status: "detected",
        reading: { ...demoStates.tuned.reading, frequencyHz: Number.NaN },
      }),
    );
    expect(screen.getByText("收到无效信号，已清除读数。")).toBeVisible();
    expect(screen.getByLabelText("当前频率")).toHaveTextContent("— Hz");
    expect(screen.queryByTestId("tuning-needle")).not.toBeInTheDocument();
  });
});
