use crate::{error::positive_finite, CoreError, TuningResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PitchStatus {
    Idle,
    Listening,
    NoPitch,
    LowConfidence,
    Detected,
    Error,
}

/// V1 has zero or one pitch. Inactive/invalid states cannot carry stale results.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Unified, serializable telemetry frame produced by Rust as the single source of truth for UI.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunerSnapshot {
    session_id: u64,
    sequence: u64,
    timestamp_s: f64,
    actual_sample_rate: u32,
    status: PitchStatus,
    raw_pitch: Option<PitchEstimate>,
    display_frequency_hz: Option<f64>,
    is_held: bool,
    tuning: Option<TuningResult>,
    rms_dbfs: Option<f32>,
    dropped_frames: u64,
    reference_a4_hz: f64,
    tolerance_cents: f64,
}

#[derive(Clone, Debug)]
pub struct TunerSnapshotInit {
    pub session_id: u64,
    pub sequence: u64,
    pub timestamp_s: f64,
    pub actual_sample_rate: u32,
    pub status: PitchStatus,
    pub raw_pitch: Option<PitchEstimate>,
    pub display_frequency_hz: Option<f64>,
    pub is_held: bool,
    pub tuning: Option<TuningResult>,
    pub rms_dbfs: Option<f32>,
    pub dropped_frames: u64,
    pub reference_a4_hz: f64,
    pub tolerance_cents: f64,
}

impl TunerSnapshot {
    pub fn new(init: TunerSnapshotInit) -> Result<Self, CoreError> {
        if !init.timestamp_s.is_finite() || init.timestamp_s < 0.0 {
            return Err(CoreError::InvalidValue("timestamp_s"));
        }
        if !init.reference_a4_hz.is_finite() || !(400.0..=480.0).contains(&init.reference_a4_hz) {
            return Err(CoreError::InvalidValue("reference_a4_hz (400..=480)"));
        }
        if !init.tolerance_cents.is_finite() || !(0.0..=50.0).contains(&init.tolerance_cents) {
            return Err(CoreError::InvalidValue("tolerance_cents (0..=50)"));
        }
        if let Some(rms) = init.rms_dbfs {
            if !rms.is_finite() || rms > 0.0 {
                return Err(CoreError::InvalidValue("rms_dbfs"));
            }
            if matches!(init.status, PitchStatus::Idle | PitchStatus::Error) {
                return Err(CoreError::InvalidFrame(
                    "inactive snapshots must clear signal metrics",
                ));
            }
        }
        if init.status == PitchStatus::Detected && init.raw_pitch.is_none() {
            return Err(CoreError::InvalidFrame(
                "detected snapshot must carry raw_pitch",
            ));
        }
        if init.status != PitchStatus::Detected && init.raw_pitch.is_some() {
            return Err(CoreError::InvalidFrame(
                "non-detected snapshot cannot carry raw_pitch",
            ));
        }
        if let Some(hz) = init.display_frequency_hz {
            positive_finite(hz, "display_frequency_hz")?;
            if matches!(
                init.status,
                PitchStatus::Idle | PitchStatus::Error | PitchStatus::NoPitch
            ) {
                return Err(CoreError::InvalidFrame(
                    "inactive or silent snapshot cannot carry display_frequency_hz",
                ));
            }
        } else if init.is_held || init.tuning.is_some() {
            return Err(CoreError::InvalidFrame(
                "tuning or held state requires display_frequency_hz",
            ));
        }
        Ok(Self {
            session_id: init.session_id,
            sequence: init.sequence,
            timestamp_s: init.timestamp_s,
            actual_sample_rate: init.actual_sample_rate,
            status: init.status,
            raw_pitch: init.raw_pitch,
            display_frequency_hz: init.display_frequency_hz,
            is_held: init.is_held,
            tuning: init.tuning,
            rms_dbfs: init.rms_dbfs,
            dropped_frames: init.dropped_frames,
            reference_a4_hz: init.reference_a4_hz,
            tolerance_cents: init.tolerance_cents,
        })
    }

    pub const fn session_id(&self) -> u64 {
        self.session_id
    }
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
    pub const fn timestamp_s(&self) -> f64 {
        self.timestamp_s
    }
    pub const fn actual_sample_rate(&self) -> u32 {
        self.actual_sample_rate
    }
    pub const fn status(&self) -> PitchStatus {
        self.status
    }
    pub const fn raw_pitch(&self) -> Option<PitchEstimate> {
        self.raw_pitch
    }
    pub const fn display_frequency_hz(&self) -> Option<f64> {
        self.display_frequency_hz
    }
    pub const fn is_held(&self) -> bool {
        self.is_held
    }
    pub fn tuning(&self) -> Option<&TuningResult> {
        self.tuning.as_ref()
    }
    pub const fn rms_dbfs(&self) -> Option<f32> {
        self.rms_dbfs
    }
    pub const fn dropped_frames(&self) -> u64 {
        self.dropped_frames
    }
    pub const fn reference_a4_hz(&self) -> f64 {
        self.reference_a4_hz
    }
    pub const fn tolerance_cents(&self) -> f64 {
        self.tolerance_cents
    }
}
