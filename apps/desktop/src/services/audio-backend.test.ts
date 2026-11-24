import { describe, expect, it, vi } from "vitest";
import { createUnavailableBackend, validateTunerState } from "./audio-backend";
import { createDemoBackend, demoStates } from "./demo-backend";

describe("audio display adapter boundary", () => {
  it("keeps the production adapter idle and rejects capture instead of returning Demo data", async () => {
    const backend = createUnavailableBackend();
    const listener = vi.fn();
    const unsubscribe = backend.subscribe(listener);
    expect(backend.source).toBe("unavailable");
    expect(listener).toHaveBeenCalledExactlyOnceWith({ status: "idle" });
    await expect(backend.start()).rejects.toThrow("not implemented");
    await backend.stop();
    unsubscribe();
  });

  it("starts/stops the chosen fixture and stops delivery after unsubscribe", async () => {
    const backend = createDemoBackend();
    const listener = vi.fn();
    const unsubscribe = backend.subscribe(listener);
    backend.selectScenario("sharp");
    expect(listener).toHaveBeenLastCalledWith({ status: "idle" });
    await backend.start();
    expect(listener).toHaveBeenLastCalledWith(demoStates.sharp);
    await backend.stop();
    expect(listener).toHaveBeenLastCalledWith({ status: "idle" });
    unsubscribe();
    const count = listener.mock.calls.length;
    await backend.start();
    expect(listener).toHaveBeenCalledTimes(count);
  });

  it.each([
    { frequencyHz: Number.NaN },
    { frequencyHz: -1 },
    { targetHz: 0 },
    { cents: Number.POSITIVE_INFINITY },
    { confidence: 1.1 },
    { rmsDbfs: Number.NEGATIVE_INFINITY },
    { noteName: "wrong" },
    { octave: 100 },
  ])("rejects invalid display reading %j", (patch) => {
    expect(
      validateTunerState({
        status: "detected",
        reading: { ...demoStates.tuned.reading, ...patch },
      }),
    ).toEqual({ status: "error", code: "invalid-frame" });
  });
});
