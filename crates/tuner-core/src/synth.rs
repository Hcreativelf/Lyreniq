//! Phase-continuous, allocation-free reference & test tone generator.
//! Safe to invoke directly inside real-time audio output callbacks.
use crate::{error::positive_finite, CoreError};
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

/// Safe default low output gain (~ -16.5 dBFS) to protect ears and speakers.
pub const DEFAULT_SAFE_TONE_GAIN: f32 = 0.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToneWaveform {
    /// Pure sine wave (strictly single harmonic).
    Sine,
    /// Gentle musical tone (fundamental + subtle 2nd & 3rd harmonics, normalized to <= 1.0).
    SoftHarmonic,
}

/// Real-time-safe oscillator with smooth linear gain ramping (anti-click/pop)
/// and smooth frequency slewing. Zero allocations during `fill_buffer`.
pub struct ToneGenerator {
    sample_rate: u32,
    phase: f64,
    current_hz: f64,
    target_hz: f64,
    current_gain: f64,
    target_gain: f64,
    active_gain: f64,
    playing: bool,
    waveform: ToneWaveform,
    gain_ramp_step: f64,
    freq_slew_coeff: f64,
}

impl ToneGenerator {
    pub fn new(sample_rate: u32, frequency_hz: f64, gain: f32) -> Result<Self, CoreError> {
        if sample_rate == 0 {
            return Err(CoreError::InvalidValue("sample_rate"));
        }
        validate_tone_frequency(frequency_hz, sample_rate)?;
        validate_gain(gain)?;
        let rate = f64::from(sample_rate);
        // 15 ms full-scale linear ramp eliminates start/stop clicks and pops.
        let ramp_samples = (rate * 0.015).max(1.0);
        // ~4 ms time constant for smooth pitch glides without zipper noise.
        let freq_slew_coeff = 1.0 - (-1.0 / (rate * 0.004)).exp();
        Ok(Self {
            sample_rate,
            phase: 0.0,
            current_hz: frequency_hz,
            target_hz: frequency_hz,
            current_gain: 0.0,
            target_gain: 0.0,
            active_gain: f64::from(gain),
            playing: false,
            waveform: ToneWaveform::Sine,
            gain_ramp_step: 1.0 / ramp_samples,
            freq_slew_coeff,
        })
    }

    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub const fn target_frequency_hz(&self) -> f64 {
        self.target_hz
    }
    pub const fn active_gain(&self) -> f32 {
        self.active_gain as f32
    }
    pub const fn waveform(&self) -> ToneWaveform {
        self.waveform
    }
    pub const fn is_playing(&self) -> bool {
        self.playing
    }
    /// True once stopped and the fade-out envelope has reached zero.
    pub fn is_silent(&self) -> bool {
        !self.playing && self.current_gain <= 1e-7
    }

    pub fn set_frequency_hz(&mut self, frequency_hz: f64) -> Result<(), CoreError> {
        validate_tone_frequency(frequency_hz, self.sample_rate)?;
        self.target_hz = frequency_hz;
        if self.is_silent() {
            self.current_hz = frequency_hz;
        }
        Ok(())
    }

    pub fn set_gain(&mut self, gain: f32) -> Result<(), CoreError> {
        validate_gain(gain)?;
        self.active_gain = f64::from(gain);
        if self.playing {
            self.target_gain = self.active_gain;
        }
        Ok(())
    }

    pub fn set_waveform(&mut self, waveform: ToneWaveform) {
        self.waveform = waveform;
    }

    pub fn start(&mut self) {
        self.playing = true;
        self.target_gain = self.active_gain;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.target_gain = 0.0;
    }

    /// Fills `output` with mono samples in `[-1.0, 1.0]`. Strictly allocation-free.
    pub fn fill_buffer(&mut self, output: &mut [f32]) {
        let inv_rate = 1.0 / f64::from(self.sample_rate);
        let nyquist = f64::from(self.sample_rate) * 0.5;
        for slot in output.iter_mut() {
            // Smooth gain ramp (anti-pop)
            if self.current_gain < self.target_gain {
                self.current_gain = (self.current_gain + self.gain_ramp_step).min(self.target_gain);
            } else if self.current_gain > self.target_gain {
                self.current_gain = (self.current_gain - self.gain_ramp_step).max(self.target_gain);
            }

            if self.current_gain <= 1e-7 && !self.playing {
                self.current_gain = 0.0;
                *slot = 0.0;
                continue;
            }

            // Smooth frequency slew
            self.current_hz += (self.target_hz - self.current_hz) * self.freq_slew_coeff;

            let sample = match self.waveform {
                ToneWaveform::Sine => self.phase.sin(),
                ToneWaveform::SoftHarmonic => {
                    let mut sum = 0.72 * self.phase.sin();
                    if self.current_hz * 2.0 < nyquist {
                        sum += 0.20 * (2.0 * self.phase).sin();
                    }
                    if self.current_hz * 3.0 < nyquist {
                        sum += 0.08 * (3.0 * self.phase).sin();
                    }
                    sum
                }
            };

            *slot = (sample * self.current_gain).clamp(-1.0, 1.0) as f32;
            self.phase = (self.phase + TAU * self.current_hz * inv_rate).rem_euclid(TAU);
        }
    }
}

fn validate_tone_frequency(frequency_hz: f64, sample_rate: u32) -> Result<(), CoreError> {
    positive_finite(frequency_hz, "frequency_hz")?;
    let nyquist = f64::from(sample_rate) / 2.0;
    if frequency_hz < 20.0 || frequency_hz >= nyquist {
        return Err(CoreError::InvalidValue("tone frequency (20 Hz..Nyquist)"));
    }
    Ok(())
}

fn validate_gain(gain: f32) -> Result<(), CoreError> {
    if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
        return Err(CoreError::InvalidValue("tone gain (0..=1)"));
    }
    Ok(())
}
