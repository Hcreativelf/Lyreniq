use crate::{error::positive_finite, CoreError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrequencyRange {
    min_hz: f64,
    max_hz: f64,
}

impl Default for FrequencyRange {
    fn default() -> Self {
        Self {
            min_hz: 20.0,
            max_hz: 5000.0,
        }
    }
}

impl FrequencyRange {
    pub fn new(min_hz: f64, max_hz: f64) -> Result<Self, CoreError> {
        positive_finite(min_hz, "min_hz")?;
        positive_finite(max_hz, "max_hz")?;
        if min_hz >= max_hz {
            return Err(CoreError::InvalidValue("frequency range order"));
        }
        Ok(Self { min_hz, max_hz })
    }
    pub const fn min_hz(self) -> f64 {
        self.min_hz
    }
    pub const fn max_hz(self) -> f64 {
        self.max_hz
    }
    pub fn validate_for_sample_rate(self, sample_rate: u32) -> Result<(), CoreError> {
        if sample_rate == 0 || self.max_hz >= f64::from(sample_rate) / 2.0 {
            return Err(CoreError::InvalidValue("sample_rate/Nyquist limit"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchEstimate {
    frequency_hz: f64,
    confidence: f32,
}

impl PitchEstimate {
    pub fn new(frequency_hz: f64, confidence: f32) -> Result<Self, CoreError> {
        positive_finite(frequency_hz, "frequency_hz")?;
        if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
            return Err(CoreError::InvalidValue("confidence (0..=1)"));
        }
        Ok(Self {
            frequency_hz,
            confidence,
        })
    }
    pub const fn frequency_hz(self) -> f64 {
        self.frequency_hz
    }
    pub const fn confidence(self) -> f32 {
        self.confidence
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchStatus {
    Idle,
    Listening,
    NoPitch,
    LowConfidence,
    Detected,
    Error,
}

/// V1 has zero or one pitch. Inactive/invalid states cannot carry stale results.
#[derive(Clone, Debug, PartialEq)]
pub struct PitchFrame {
    session_id: u64,
    timestamp_s: f64,
    pitches: Vec<PitchEstimate>,
    status: PitchStatus,
    rms_dbfs: Option<f32>,
}

impl PitchFrame {
    pub fn new(
        session_id: u64,
        timestamp_s: f64,
        pitches: Vec<PitchEstimate>,
        status: PitchStatus,
        rms_dbfs: Option<f32>,
    ) -> Result<Self, CoreError> {
        if !timestamp_s.is_finite() || timestamp_s < 0.0 {
            return Err(CoreError::InvalidValue("timestamp_s"));
        }
        if (status == PitchStatus::Detected && pitches.len() != 1)
            || (status != PitchStatus::Detected && !pitches.is_empty())
        {
            return Err(CoreError::InvalidFrame(
                "only detected frames may contain one pitch",
            ));
        }
        if let Some(rms) = rms_dbfs {
            if !rms.is_finite() || rms > 0.0 {
                return Err(CoreError::InvalidValue("rms_dbfs"));
            }
            if matches!(status, PitchStatus::Idle | PitchStatus::Error) {
                return Err(CoreError::InvalidFrame(
                    "inactive frames must clear signal metrics",
                ));
            }
        }
        Ok(Self {
            session_id,
            timestamp_s,
            pitches,
            status,
            rms_dbfs,
        })
    }
    pub const fn session_id(&self) -> u64 {
        self.session_id
    }
    pub const fn timestamp_s(&self) -> f64 {
        self.timestamp_s
    }
    pub fn pitches(&self) -> &[PitchEstimate] {
        &self.pitches
    }
    pub const fn status(&self) -> PitchStatus {
        self.status
    }
    pub const fn rms_dbfs(&self) -> Option<f32> {
        self.rms_dbfs
    }
}
