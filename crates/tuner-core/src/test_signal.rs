//! Deterministic, bounded in-memory PCM fixtures; no audio playback or device I/O.
use crate::{error::positive_finite, CoreError};
use std::f64::consts::TAU;

pub const MAX_FIXTURE_SAMPLES: usize = 480_000;
pub const MAX_HARMONICS: usize = 32;

/// Fixed 64-bit LCG, mapped from its high 32 bits to [-amplitude, amplitude].
/// Fixture noise only; not a cryptographic or statistical-quality RNG API.
pub fn white_noise(seed: u64, amplitude: f64, sample_count: usize) -> Result<Vec<f32>, CoreError> {
    if !amplitude.is_finite() || !(0.0..=1.0).contains(&amplitude) {
        return Err(CoreError::InvalidValue("noise amplitude (0..=1)"));
    }
    if sample_count > MAX_FIXTURE_SAMPLES {
        return Err(CoreError::FixtureTooLarge);
    }
    let mut state = seed;
    let mut pcm = Vec::new();
    pcm.try_reserve_exact(sample_count)
        .map_err(|_| CoreError::AllocationFailed)?;
    for _ in 0..sample_count {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let unit = f64::from((state >> 32) as u32) / f64::from(u32::MAX);
        pcm.push(((2.0 * unit - 1.0) * amplitude) as f32);
    }
    Ok(pcm)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Harmonic {
    multiple: u32,
    amplitude: f64,
}

impl Harmonic {
    pub fn new(multiple: u32, amplitude: f64) -> Result<Self, CoreError> {
        if multiple == 0 {
            return Err(CoreError::InvalidValue("harmonic multiple"));
        }
        if !amplitude.is_finite() || !(0.0..=1.0).contains(&amplitude) {
            return Err(CoreError::InvalidValue("harmonic amplitude (0..=1)"));
        }
        Ok(Self {
            multiple,
            amplitude,
        })
    }
    pub const fn multiple(self) -> u32 {
        self.multiple
    }
    pub const fn amplitude(self) -> f64 {
        self.amplitude
    }
}

pub fn sine(
    sample_rate: u32,
    frequency_hz: f64,
    amplitude: f64,
    sample_count: usize,
) -> Result<Vec<f32>, CoreError> {
    harmonic_tone(
        sample_rate,
        frequency_hz,
        &[Harmonic::new(1, amplitude)?],
        sample_count,
    )
}

/// All harmonics use phase zero; amplitudes sum to at most one (no normalization).
pub fn harmonic_tone(
    sample_rate: u32,
    fundamental_hz: f64,
    harmonics: &[Harmonic],
    sample_count: usize,
) -> Result<Vec<f32>, CoreError> {
    positive_finite(fundamental_hz, "fundamental_hz")?;
    if sample_rate == 0 {
        return Err(CoreError::InvalidValue("sample_rate"));
    }
    if harmonics.is_empty() || harmonics.len() > MAX_HARMONICS {
        return Err(CoreError::InvalidValue("harmonic count (1..=32)"));
    }
    if sample_count > MAX_FIXTURE_SAMPLES {
        return Err(CoreError::FixtureTooLarge);
    }
    if harmonics.iter().map(|h| h.amplitude).sum::<f64>() > 1.0 {
        return Err(CoreError::InvalidValue("total harmonic amplitude (<=1)"));
    }
    for harmonic in harmonics {
        let hz = fundamental_hz * f64::from(harmonic.multiple);
        if !hz.is_finite() || hz >= f64::from(sample_rate) / 2.0 {
            return Err(CoreError::InvalidValue("harmonic must be below Nyquist"));
        }
    }
    let mut pcm = Vec::new();
    pcm.try_reserve_exact(sample_count)
        .map_err(|_| CoreError::AllocationFailed)?;
    for n in 0..sample_count {
        let sample = harmonics
            .iter()
            .map(|harmonic| {
                let cycles = (fundamental_hz * f64::from(harmonic.multiple)
                    / f64::from(sample_rate))
                    * n as f64;
                harmonic.amplitude * (TAU * cycles).sin()
            })
            .sum::<f64>();
        pcm.push(sample as f32);
    }
    Ok(pcm)
}
