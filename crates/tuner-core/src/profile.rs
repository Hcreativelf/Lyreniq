//! Data-driven instrument presets and custom target note profiles.
use crate::{CoreError, FrequencyRange, MidiNote, TargetSelection};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstrumentProfileId {
    Chromatic,
    GuitarStandard,
    GuitarDropD,
    Bass4,
    Bass5,
    Ukulele,
    Violin,
    Viola,
    Cello,
    Piano,
    Custom,
}

/// Real-time latency vs low-frequency reach hint for an instrument profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DetectionMode {
    Fast,
    Balanced,
    Bass,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentProfile {
    id: InstrumentProfileId,
    label: String,
    targets: Vec<MidiNote>,
    frequency_range: FrequencyRange,
    detection_mode: DetectionMode,
}

impl InstrumentProfile {
    pub fn builtin(id: InstrumentProfileId) -> Result<Self, CoreError> {
        let profile = match id {
            InstrumentProfileId::Chromatic => Self {
                id,
                label: "Chromatic".into(),
                targets: Vec::new(),
                frequency_range: FrequencyRange::new(20.0, 5000.0)?,
                detection_mode: DetectionMode::Balanced,
            },
            InstrumentProfileId::GuitarStandard => Self {
                id,
                label: "Guitar (Standard E)".into(),
                targets: notes(&[40, 45, 50, 55, 59, 64])?, // E2 A2 D3 G3 B3 E4
                frequency_range: FrequencyRange::new(65.0, 1200.0)?,
                detection_mode: DetectionMode::Fast,
            },
            InstrumentProfileId::GuitarDropD => Self {
                id,
                label: "Guitar (Drop D)".into(),
                targets: notes(&[38, 45, 50, 55, 59, 64])?, // D2 A2 D3 G3 B3 E4
                frequency_range: FrequencyRange::new(58.0, 1200.0)?,
                detection_mode: DetectionMode::Fast,
            },
            InstrumentProfileId::Bass4 => Self {
                id,
                label: "Bass (4-String)".into(),
                targets: notes(&[28, 33, 38, 43])?, // E1 A1 D2 G2
                frequency_range: FrequencyRange::new(30.0, 600.0)?,
                detection_mode: DetectionMode::Bass,
            },
            InstrumentProfileId::Bass5 => Self {
                id,
                label: "Bass (5-String)".into(),
                targets: notes(&[23, 28, 33, 38, 43])?, // B0 E1 A1 D2 G2
                frequency_range: FrequencyRange::new(24.0, 600.0)?,
                detection_mode: DetectionMode::Bass,
            },
            InstrumentProfileId::Ukulele => Self {
                id,
                label: "Ukulele (GCEA)".into(),
                targets: notes(&[67, 60, 64, 69])?, // G4 C4 E4 A4
                frequency_range: FrequencyRange::new(200.0, 1600.0)?,
                detection_mode: DetectionMode::Fast,
            },
            InstrumentProfileId::Violin => Self {
                id,
                label: "Violin".into(),
                targets: notes(&[55, 62, 69, 76])?, // G3 D4 A4 E5
                frequency_range: FrequencyRange::new(160.0, 2400.0)?,
                detection_mode: DetectionMode::Fast,
            },
            InstrumentProfileId::Viola => Self {
                id,
                label: "Viola".into(),
                targets: notes(&[48, 55, 62, 69])?, // C3 G3 D4 A4
                frequency_range: FrequencyRange::new(110.0, 1800.0)?,
                detection_mode: DetectionMode::Fast,
            },
            InstrumentProfileId::Cello => Self {
                id,
                label: "Cello".into(),
                targets: notes(&[36, 43, 50, 57])?, // C2 G2 D3 A3
                frequency_range: FrequencyRange::new(55.0, 1200.0)?,
                detection_mode: DetectionMode::Balanced,
            },
            InstrumentProfileId::Piano => Self {
                id,
                label: "Piano (12-TET)".into(),
                targets: Vec::new(),
                frequency_range: FrequencyRange::new(25.0, 4500.0)?,
                detection_mode: DetectionMode::Balanced,
            },
            InstrumentProfileId::Custom => {
                return Err(CoreError::InvalidValue(
                    "use InstrumentProfile::custom for custom targets",
                ));
            }
        };
        Ok(profile)
    }

    pub fn custom(
        label: impl Into<String>,
        targets: Vec<MidiNote>,
        frequency_range: FrequencyRange,
    ) -> Result<Self, CoreError> {
        let label = label.into();
        if label.trim().is_empty() {
            return Err(CoreError::InvalidValue("profile label"));
        }
        if targets.is_empty() || targets.len() > 64 {
            return Err(CoreError::InvalidValue("custom target count (1..=64)"));
        }
        let detection_mode = if frequency_range.min_hz() < 45.0 {
            DetectionMode::Bass
        } else if frequency_range.min_hz() >= 60.0 {
            DetectionMode::Fast
        } else {
            DetectionMode::Balanced
        };
        Ok(Self {
            id: InstrumentProfileId::Custom,
            label,
            targets,
            frequency_range,
            detection_mode,
        })
    }

    pub fn builtin_all() -> Vec<Self> {
        [
            InstrumentProfileId::Chromatic,
            InstrumentProfileId::GuitarStandard,
            InstrumentProfileId::GuitarDropD,
            InstrumentProfileId::Bass4,
            InstrumentProfileId::Bass5,
            InstrumentProfileId::Ukulele,
            InstrumentProfileId::Violin,
            InstrumentProfileId::Viola,
            InstrumentProfileId::Cello,
            InstrumentProfileId::Piano,
        ]
        .into_iter()
        .map(|id| Self::builtin(id).expect("built-in instrument profile must be valid"))
        .collect()
    }

    pub const fn id(&self) -> InstrumentProfileId {
        self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn targets(&self) -> &[MidiNote] {
        &self.targets
    }
    pub const fn frequency_range(&self) -> FrequencyRange {
        self.frequency_range
    }
    pub const fn detection_mode(&self) -> DetectionMode {
        self.detection_mode
    }

    pub fn target_selection(&self, locked: Option<MidiNote>) -> TargetSelection<'_> {
        if let Some(note) = locked {
            return TargetSelection::Locked(note);
        }
        if self.targets.is_empty() {
            TargetSelection::Chromatic
        } else {
            TargetSelection::Nearest(&self.targets)
        }
    }
}

fn notes(midi: &[i32]) -> Result<Vec<MidiNote>, CoreError> {
    midi.iter().map(|&n| MidiNote::new(n)).collect()
}
