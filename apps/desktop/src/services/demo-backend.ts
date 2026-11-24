import type { AudioBackend, PitchReading, TunerState } from "./audio-backend";

const a4: PitchReading = {
  noteName: "A",
  octave: 4,
  frequencyHz: 440,
  targetHz: 440,
  cents: 0,
  confidence: 0.96,
  rmsDbfs: -18,
};
const detected = (reading: Partial<PitchReading>): Extract<TunerState, { status: "detected" }> => ({
  status: "detected",
  reading: { ...a4, ...reading },
});

// Fixed, opt-in display fixtures. They do not analyze or generate audio.
export const demoStates = {
  tuned: detected({}),
  flat: detected({ frequencyHz: 427.474054, cents: -50 }),
  sharp: detected({ frequencyHz: 452.892984, cents: 50 }),
  below: detected({ frequencyHz: 421.345444, cents: -75 }),
  above: detected({ frequencyHz: 459.480464, cents: 75 }),
  note: detected({ noteName: "C", frequencyHz: 261.625565, targetHz: 261.625565 }),
  listening: { status: "listening", signal: { rmsDbfs: -32, confidence: 0.2 } },
  silence: { status: "no-pitch", signal: { rmsDbfs: -90, confidence: 0 } },
  uncertain: { status: "low-confidence", signal: { rmsDbfs: -27, confidence: 0.3 } },
  error: { status: "error", code: "input-unavailable" },
  idle: { status: "idle" },
} as const satisfies Record<string, TunerState>;

export type DemoScenario = keyof typeof demoStates;
export interface DemoBackend extends AudioBackend {
  readonly source: "demo";
  selectScenario(scenario: DemoScenario): void;
}

export function createDemoBackend(): DemoBackend {
  let state: TunerState = { status: "idle" };
  let scenario: DemoScenario = "tuned";
  let running = false;
  const listeners = new Set<(state: TunerState) => void>();
  function emit(next: TunerState) {
    state = next;
    for (const listener of listeners) listener(state);
  }
  return {
    source: "demo",
    subscribe(listener) {
      listeners.add(listener);
      listener(state);
      return () => {
        listeners.delete(listener);
      };
    },
    async start() {
      running = true;
      emit(demoStates[scenario]);
    },
    async stop() {
      running = false;
      emit({ status: "idle" });
    },
    selectScenario(next) {
      scenario = next;
      if (running) emit(demoStates[scenario]);
    },
  };
}
