//! Device-independent DSP 2.0 core for Lyreniq:
//! FFT spectrum analysis, FFT-accelerated Fast YIN & multi-scale pitch detection,
//! log-frequency pitch tracker, tuning math, instrument profiles and phase-continuous synth.
//!
//! No device, desktop or audio playback I/O implementation lives here.
//!
//! ```
//! use tuner_core::{MidiNote, TargetSelection, TuningConfig};
//! let result = TuningConfig::default()
//!     .tune(442.0, TargetSelection::Locked(MidiNote::new(69).unwrap()))
//!     .unwrap();
//! assert_eq!(result.note_name(), "A4");
//! assert!((result.cents() - 7.85141504).abs() < 1e-8);
//! ```
#![forbid(unsafe_code)]

mod error;
mod model;
mod profile;
mod smoothing;
mod spectrum;
mod synth;
pub mod test_signal;
mod tuning;
mod yin;

pub use error::CoreError;
pub use model::{
    FrequencyRange, PitchEstimate, PitchFrame, PitchStatus, TunerSnapshot, TunerSnapshotInit,
};
pub use profile::{DetectionMode, InstrumentProfile, InstrumentProfileId};
pub use smoothing::{PitchTracker, PitchTrackerConfig, TrackedPitch};
pub use spectrum::{
    HarmonicPeak, LogSpectrum, SpectrumAnalyzer, SpectrumConfig, SpectrumFrame, MAX_FFT_SIZE,
    MIN_FFT_SIZE,
};
pub use synth::{ToneGenerator, ToneWaveform, DEFAULT_SAFE_TONE_GAIN};
pub use tuning::{
    cents_difference, frequency_to_midi, midi_to_frequency, MidiNote, NoteSpelling,
    TargetSelection, TuningConfig, TuningDirection, TuningResult,
};
pub use yin::{
    MultiScalePitchDetector, PitchDetection, YinConfig, YinDetector, MAX_YIN_WINDOW_SAMPLES,
};
