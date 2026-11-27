//! Log-frequency (cents-domain) pitch tracker separating raw DSP output from
//! smooth, flicker-free display pitch with outlier rejection and transient hold.
use crate::{CoreError, PitchDetection, PitchStatus};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchTrackerConfig {
    /// Cents jump threshold that immediately resets smoothing (new note).
    pub jump_reset_cents: f64,
    /// EMA smoothing factor for tiny jitter (< 3 cents). Lower = steadier needle.
    pub alpha_stable: f64,
    /// EMA smoothing factor for larger motion (tuning peg / vibrato). Higher = faster response.
    pub alpha_fast: f64,
    /// Number of consecutive LowConfidence frames to hold last display pitch before clearing.
    /// Silence (NoPitch) and errors ALWAYS clear immediately.
    pub hold_frames: u8,
}

impl Default for PitchTrackerConfig {
    /// Balanced real-time tuning preset: rock-steady around 0 cents, responsive on note change.
    fn default() -> Self {
        Self {
            jump_reset_cents: 50.0,
            alpha_stable: 0.30,
            alpha_fast: 0.80,
            hold_frames: 2,
        }
    }
}

impl PitchTrackerConfig {
    /// Fast-tracking preset with minimal hangover.
    pub fn responsive() -> Self {
        Self {
            jump_reset_cents: 45.0,
            alpha_stable: 0.45,
            alpha_fast: 0.90,
            hold_frames: 1,
        }
    }

    /// Strict 3-frame median with zero hold and no EMA lag (deterministic / offline-friendly).
    pub fn strict_median() -> Self {
        Self {
            jump_reset_cents: 50.0,
            alpha_stable: 1.0,
            alpha_fast: 1.0,
            hold_frames: 0,
        }
    }

    pub fn validate(self) -> Result<Self, CoreError> {
        if !self.jump_reset_cents.is_finite() || !(10.0..=600.0).contains(&self.jump_reset_cents) {
            return Err(CoreError::InvalidValue("jump_reset_cents (10..=600)"));
        }
        if !self.alpha_stable.is_finite() || !(0.05..=1.0).contains(&self.alpha_stable) {
            return Err(CoreError::InvalidValue("alpha_stable (0.05..=1.0)"));
        }
        if !self.alpha_fast.is_finite()
            || !(0.05..=1.0).contains(&self.alpha_fast)
            || self.alpha_fast < self.alpha_stable
        {
            return Err(CoreError::InvalidValue("alpha_fast (alpha_stable..=1.0)"));
        }
        if self.hold_frames > 10 {
            return Err(CoreError::InvalidValue("hold_frames (0..=10)"));
        }
        Ok(self)
    }
}

/// Raw detection stays intact; display frequency and hold flag are strictly separate.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedPitch {
    raw: PitchDetection,
    display_frequency_hz: Option<f64>,
    is_held: bool,
}

impl TrackedPitch {
    pub const fn raw(self) -> PitchDetection {
        self.raw
    }
    pub const fn display_frequency_hz(self) -> Option<f64> {
        self.display_frequency_hz
    }
    pub const fn is_held(self) -> bool {
        self.is_held
    }
}

/// Log2-frequency median + adaptive EMA pitch tracker.
/// Call `reset()` on session start/stop, source change, or sample-rate change.
pub struct PitchTracker {
    config: PitchTrackerConfig,
    history_log2: [f64; 3],
    count: usize,
    next: usize,
    last_raw_log2: Option<f64>,
    smoothed_log2: Option<f64>,
    missed_frames: u8,
}

impl Default for PitchTracker {
    fn default() -> Self {
        Self::new(PitchTrackerConfig::default()).expect("default tracker config must be valid")
    }
}

impl PitchTracker {
    pub fn new(config: PitchTrackerConfig) -> Result<Self, CoreError> {
        let config = config.validate()?;
        Ok(Self {
            config,
            history_log2: [0.0; 3],
            count: 0,
            next: 0,
            last_raw_log2: None,
            smoothed_log2: None,
            missed_frames: 0,
        })
    }

    pub const fn config(&self) -> PitchTrackerConfig {
        self.config
    }

    pub fn reset(&mut self) {
        self.history_log2 = [0.0; 3];
        self.count = 0;
        self.next = 0;
        self.last_raw_log2 = None;
        self.smoothed_log2 = None;
        self.missed_frames = 0;
    }

    pub fn process(
        &mut self,
        detection: Result<PitchDetection, CoreError>,
    ) -> Result<TrackedPitch, CoreError> {
        let raw = match detection {
            Ok(value) => value,
            Err(error) => {
                self.reset();
                return Err(error);
            }
        };

        if raw.status() != PitchStatus::Detected {
            // Transient LowConfidence can briefly hold the displayed reading to avoid flicker;
            // Silence (NoPitch) and inactive states always clear immediately.
            if raw.status() == PitchStatus::LowConfidence
                && self.config.hold_frames > 0
                && self.missed_frames < self.config.hold_frames
            {
                if let Some(held_log2) = self.smoothed_log2 {
                    self.missed_frames += 1;
                    return Ok(TrackedPitch {
                        raw,
                        display_frequency_hz: Some(held_log2.exp2()),
                        is_held: true,
                    });
                }
            }
            self.reset();
            return Ok(TrackedPitch {
                raw,
                display_frequency_hz: None,
                is_held: false,
            });
        }

        let hz = raw.pitches()[0].frequency_hz();
        let log2_hz = hz.log2();
        self.missed_frames = 0;

        if self.last_raw_log2.is_some_and(|prev| {
            let delta_cents = 1200.0 * (log2_hz - prev).abs();
            delta_cents > self.config.jump_reset_cents
        }) {
            self.reset();
        }

        self.last_raw_log2 = Some(log2_hz);
        self.history_log2[self.next] = log2_hz;
        self.next = (self.next + 1) % 3;
        self.count = (self.count + 1).min(3);

        let median_log2 = if self.count == 3 {
            let mut sorted = self.history_log2;
            sorted.sort_by(f64::total_cmp);
            sorted[1]
        } else {
            log2_hz
        };

        let (next_smoothed, display_hz) = match self.smoothed_log2 {
            None => (
                median_log2,
                if self.count < 3 {
                    hz
                } else {
                    median_log2.exp2()
                },
            ),
            Some(prev_display) => {
                let delta_cents = 1200.0 * (median_log2 - prev_display).abs();
                if delta_cents > self.config.jump_reset_cents {
                    (median_log2, median_log2.exp2())
                } else {
                    // Ramp EMA factor smoothly from alpha_stable (at <= 2 cents)
                    // to alpha_fast (at >= 18 cents).
                    let blend = ((delta_cents - 2.0) / 16.0).clamp(0.0, 1.0);
                    let alpha = self.config.alpha_stable
                        + blend * (self.config.alpha_fast - self.config.alpha_stable);
                    let val = prev_display + alpha * (median_log2 - prev_display);
                    (val, val.exp2())
                }
            }
        };

        self.smoothed_log2 = Some(next_smoothed);
        Ok(TrackedPitch {
            raw,
            display_frequency_hz: Some(display_hz),
            is_held: false,
        })
    }
}
