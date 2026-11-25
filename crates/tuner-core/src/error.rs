use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreError {
    InvalidValue(&'static str),
    MidiOutOfRange,
    EmptyTargets,
    InvalidFrame(&'static str),
    FixtureTooLarge,
    AllocationFailed,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue(field) => write!(f, "invalid value: {field}"),
            Self::MidiOutOfRange => f.write_str("MIDI note must be in 0..=127"),
            Self::EmptyTargets => f.write_str("at least one target note is required"),
            Self::InvalidFrame(reason) => write!(f, "invalid pitch frame: {reason}"),
            Self::FixtureTooLarge => f.write_str("test signal exceeds the sample limit"),
            Self::AllocationFailed => {
                f.write_str("could not allocate the sample or analysis buffer")
            }
        }
    }
}

impl std::error::Error for CoreError {}

pub(crate) fn positive_finite(value: f64, field: &'static str) -> Result<(), CoreError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(CoreError::InvalidValue(field));
    }
    Ok(())
}
