export type SignalMetrics = Readonly<{ rmsDbfs: number; confidence: number }>;
export type PitchReading = SignalMetrics &
  Readonly<{
    noteName: string;
    octave: number;
    frequencyHz: number;
    targetHz: number;
    cents: number;
  }>;

// Presentation contract only. Rust DSP/IPC contracts are introduced in later tasks.
export type TunerState =
  | Readonly<{ status: "idle" }>
  | Readonly<{ status: "listening" | "no-pitch" | "low-confidence"; signal: SignalMetrics }>
  | Readonly<{ status: "detected"; reading: PitchReading }>
  | Readonly<{ status: "error"; code: "input-unavailable" | "invalid-frame" }>;

export interface AudioBackend {
  readonly source: "unavailable" | "demo";
  subscribe(listener: (state: TunerState) => void): () => void;
  start(): Promise<void>;
  stop(): Promise<void>;
}

export function createUnavailableBackend(): AudioBackend {
  return {
    source: "unavailable",
    subscribe(listener) {
      listener({ status: "idle" });
      return () => {};
    },
    async start() {
      throw new Error("Live audio input is not implemented");
    },
    async stop() {},
  };
}

export function validateTunerState(state: TunerState): TunerState {
  const signal =
    state.status === "detected" ? state.reading : "signal" in state ? state.signal : null;
  if (
    signal &&
    (!Number.isFinite(signal.rmsDbfs) ||
      signal.rmsDbfs > 0 ||
      !Number.isFinite(signal.confidence) ||
      signal.confidence < 0 ||
      signal.confidence > 1)
  ) {
    return { status: "error", code: "invalid-frame" };
  }
  if (state.status === "detected") {
    const r = state.reading;
    if (
      !Number.isFinite(r.frequencyHz) ||
      r.frequencyHz <= 0 ||
      !Number.isFinite(r.targetHz) ||
      r.targetHz <= 0 ||
      !Number.isFinite(r.cents) ||
      !Number.isInteger(r.octave) ||
      r.octave < -1 ||
      r.octave > 9 ||
      !/^[A-G](?:#|b)?$/.test(r.noteName)
    ) {
      return { status: "error", code: "invalid-frame" };
    }
  }
  return state;
}

export function pointerCents(cents: number): number {
  if (!Number.isFinite(cents)) throw new Error("Pointer requires finite cents");
  return Math.max(-50, Math.min(50, cents));
}

export function tuningDirection(cents: number): "flat" | "tuned" | "sharp" {
  return cents < -3 ? "flat" : cents > 3 ? "sharp" : "tuned";
}

export function formatCents(cents: number): string {
  const rounded = Math.abs(cents) < 0.05 ? 0 : cents;
  return `${rounded > 0 ? "+" : ""}${rounded.toFixed(1)}`;
}
