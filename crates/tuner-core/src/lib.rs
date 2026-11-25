//! Device-independent pitch contracts and twelve-tone equal-temperament math.
//!
//! No device, desktop, audio playback or pitch-detection implementation lives here.
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
pub mod test_signal;
mod tuning;

pub use error::CoreError;
pub use model::{FrequencyRange, PitchEstimate, PitchFrame, PitchStatus};
pub use tuning::{
    cents_difference, frequency_to_midi, midi_to_frequency, MidiNote, NoteSpelling,
    TargetSelection, TuningConfig, TuningDirection, TuningResult,
};
