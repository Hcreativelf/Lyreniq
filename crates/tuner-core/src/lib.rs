//! Device-independent YIN detection, pitch contracts and tuning math.
//!
//! No device, desktop or audio playback implementation lives here.
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
mod smoothing;
pub mod test_signal;
mod tuning;
mod yin;

pub use error::CoreError;
pub use model::{FrequencyRange, PitchEstimate, PitchFrame, PitchStatus};
pub use smoothing::{PitchSmoother, SmoothedPitch};
pub use tuning::{
    cents_difference, frequency_to_midi, midi_to_frequency, MidiNote, NoteSpelling,
    TargetSelection, TuningConfig, TuningDirection, TuningResult,
};
pub use yin::{PitchDetection, YinConfig, YinDetector, MAX_YIN_WINDOW_SAMPLES};
