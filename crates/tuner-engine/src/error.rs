use audio_decode::DecodeError;
use std::fmt;
use tuner_core::CoreError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineError {
    Core(CoreError),
    Decode(DecodeError),
    Cancelled,
    Io(String),
    InvalidConfig(&'static str),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(e) => write!(f, "DSP core error: {e}"),
            Self::Decode(e) => write!(f, "audio decode error: {e}"),
            Self::Cancelled => f.write_str("analysis cancelled"),
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::InvalidConfig(msg) => write!(f, "invalid engine configuration: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<CoreError> for EngineError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<DecodeError> for EngineError {
    fn from(value: DecodeError) -> Self {
        Self::Decode(value)
    }
}
