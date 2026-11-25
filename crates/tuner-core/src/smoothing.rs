use crate::{cents_difference, CoreError, PitchDetection, PitchStatus};

/// Raw detection remains intact; only the optional display frequency is filtered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmoothedPitch {
    raw: PitchDetection,
    display_frequency_hz: Option<f64>,
}
impl SmoothedPitch {
    pub const fn raw(self) -> PitchDetection {
        self.raw
    }
    pub const fn display_frequency_hz(self) -> Option<f64> {
        self.display_frequency_hz
    }
}

/// Three-frame median, reset on invalid frames, errors or >50-cent pitch jumps.
/// At a 20 ms hop the steady-state median spans 40 ms and may lag one hop.
/// Lifecycle/session/rate changes must call reset; no timestamps are fabricated.
#[derive(Default)]
pub struct PitchSmoother {
    history: [f64; 3],
    count: usize,
    next: usize,
    last_hz: Option<f64>,
}
impl PitchSmoother {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn process(
        &mut self,
        detection: Result<PitchDetection, CoreError>,
    ) -> Result<SmoothedPitch, CoreError> {
        let raw = match detection {
            Ok(value) => value,
            Err(error) => {
                self.reset();
                return Err(error);
            }
        };
        if raw.status() != PitchStatus::Detected {
            self.reset();
            return Ok(SmoothedPitch {
                raw,
                display_frequency_hz: None,
            });
        }
        let hz = raw.pitches()[0].frequency_hz();
        if self.last_hz.is_some_and(|previous| {
            cents_difference(hz, previous).is_ok_and(|cents| cents.abs() > 50.0)
        }) {
            self.reset();
        }
        self.last_hz = Some(hz);
        self.history[self.next] = hz;
        self.next = (self.next + 1) % 3;
        self.count = (self.count + 1).min(3);
        let display = if self.count == 3 {
            let mut sorted = self.history;
            sorted.sort_by(f64::total_cmp);
            sorted[1]
        } else {
            hz
        };
        Ok(SmoothedPitch {
            raw,
            display_frequency_hz: Some(display),
        })
    }
}
