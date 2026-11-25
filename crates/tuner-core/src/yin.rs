//! Direct, fixed-window YIN reference implementation. Run outside audio callbacks.
use crate::{CoreError, FrequencyRange, PitchEstimate, PitchStatus};

/// Bounds scratch memory; this is not a real-time execution budget.
pub const MAX_YIN_WINDOW_SAMPLES: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct YinConfig {
    frequency_range: FrequencyRange,
    window_ms: u32,
    threshold: f64,
    min_rms_dbfs: f64,
}

impl Default for YinConfig {
    fn default() -> Self {
        Self {
            frequency_range: FrequencyRange::default(),
            window_ms: 300,
            threshold: 0.1,
            min_rms_dbfs: -60.0,
        }
    }
}

impl YinConfig {
    pub fn new(
        frequency_range: FrequencyRange,
        window_ms: u32,
        threshold: f64,
        min_rms_dbfs: f64,
    ) -> Result<Self, CoreError> {
        if !(60..=300).contains(&window_ms) {
            return Err(CoreError::InvalidValue("YIN window_ms (60..=300)"));
        }
        if !threshold.is_finite() || threshold <= 0.0 || threshold >= 1.0 {
            return Err(CoreError::InvalidValue("YIN threshold (0..1 exclusive)"));
        }
        if !min_rms_dbfs.is_finite() || !(-160.0..=0.0).contains(&min_rms_dbfs) {
            return Err(CoreError::InvalidValue("YIN min_rms_dbfs (-160..=0)"));
        }
        Ok(Self {
            frequency_range,
            window_ms,
            threshold,
            min_rms_dbfs,
        })
    }

    pub const fn frequency_range(self) -> FrequencyRange {
        self.frequency_range
    }
    pub const fn window_ms(self) -> u32 {
        self.window_ms
    }
    pub const fn threshold(self) -> f64 {
        self.threshold
    }
    pub const fn min_rms_dbfs(self) -> f64 {
        self.min_rms_dbfs
    }

    /// Exact PCM length at the actual rate; rejects incompatible lag/window geometry.
    pub fn sample_count(self, sample_rate: u32) -> Result<usize, CoreError> {
        Ok(self.layout(sample_rate)?.samples)
    }

    fn layout(self, sample_rate: u32) -> Result<LagLayout, CoreError> {
        self.frequency_range.validate_for_sample_rate(sample_rate)?;
        let count = u64::from(sample_rate) * u64::from(self.window_ms) / 1000;
        if !(8..=MAX_YIN_WINDOW_SAMPLES as u64).contains(&count) {
            return Err(CoreError::InvalidValue("YIN window sample count"));
        }
        let samples = count as usize;
        let rate = f64::from(sample_rate);
        let max_lag = (rate / self.frequency_range.min_hz()).ceil();
        // Leave a right neighbor for interpolation and at least one longest
        // period in the fixed comparison span. Check before float-to-int cast.
        if !max_lag.is_finite() || max_lag + 1.0 > (samples / 2) as f64 {
            return Err(CoreError::InvalidValue("YIN window too short for min_hz"));
        }
        Ok(LagLayout {
            samples,
            min_lag: (rate / self.frequency_range.max_hz()).floor() as usize,
            max_lag: max_lag as usize,
        })
    }
}

struct LagLayout {
    samples: usize,
    min_lag: usize,
    max_lag: usize,
}

/// Zero or one pitch, without fabricated session IDs or timestamps.
/// Only Detected carries a pitch; confidence is periodicity, not probability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchDetection {
    pitch: Option<PitchEstimate>,
    status: PitchStatus,
    rms_dbfs: Option<f32>,
}

impl PitchDetection {
    pub fn pitches(&self) -> &[PitchEstimate] {
        self.pitch.as_slice()
    }
    pub const fn status(self) -> PitchStatus {
        self.status
    }
    /// RMS after DC removal; None for incomplete windows or exact silence.
    pub const fn rms_dbfs(self) -> Option<f32> {
        self.rms_dbfs
    }

    fn without_pitch(status: PitchStatus, rms_dbfs: Option<f32>) -> Self {
        Self {
            pitch: None,
            status,
            rms_dbfs,
        }
    }
}

/// O(comparison_samples * max_lag), with bounded, reused f64 scratch buffers.
/// No smoothing or previous estimate is retained. Not for audio callbacks.
///
/// ```
/// use tuner_core::{test_signal::sine, FrequencyRange, YinConfig, YinDetector};
/// let config = YinConfig::new(FrequencyRange::new(100.0, 1000.0)?, 100, 0.1, -60.0)?;
/// let actual_rate = 48_000;
/// let pcm = sine(actual_rate, 442.0, 0.5, config.sample_count(actual_rate)?)?;
/// let result = YinDetector::new(config).detect(&pcm, actual_rate)?;
/// assert_eq!(result.pitches().len(), 1);
/// assert!((result.pitches()[0].frequency_hz() - 442.0).abs() < 0.1);
/// # Ok::<(), tuner_core::CoreError>(())
/// ```
#[derive(Default)]
pub struct YinDetector {
    config: YinConfig,
    centered: Vec<f64>,
    difference: Vec<f64>,
    normalized: Vec<f64>,
}

impl YinDetector {
    pub fn new(config: YinConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }
    pub const fn config(&self) -> YinConfig {
        self.config
    }

    /// Accepts normalized mono PCM [-1, 1] at its actual sample rate.
    /// Empty/short windows yield NoPitch, never an estimate from an earlier call.
    /// Oversized windows, non-finite PCM and incompatible rates/configs are errors.
    pub fn detect(&mut self, pcm: &[f32], sample_rate: u32) -> Result<PitchDetection, CoreError> {
        let layout = self.config.layout(sample_rate)?;
        if pcm.len() > layout.samples {
            return Err(CoreError::InvalidValue("PCM exceeds configured YIN window"));
        }
        if pcm.iter().any(|x| !x.is_finite() || x.abs() > 1.0) {
            return Err(CoreError::InvalidValue("normalized finite PCM (-1..=1)"));
        }
        if pcm.len() < layout.samples {
            return Ok(PitchDetection::without_pitch(PitchStatus::NoPitch, None));
        }

        resize_buffer(&mut self.centered, pcm.len())?;
        let mean = pcm.iter().map(|&x| f64::from(x)).sum::<f64>() / pcm.len() as f64;
        let mut energy = 0.0;
        for (sample, &input) in self.centered.iter_mut().zip(pcm) {
            *sample = f64::from(input) - mean;
            energy += *sample * *sample;
        }
        let mean_square = energy / pcm.len() as f64;
        let rms_dbfs = (mean_square > 0.0).then(|| (10.0 * mean_square.log10()).min(0.0));
        let rms_metric = rms_dbfs.map(|rms| rms as f32);
        if rms_dbfs.is_none_or(|rms| rms <= self.config.min_rms_dbfs) {
            return Ok(PitchDetection::without_pitch(
                PitchStatus::NoPitch,
                rms_metric,
            ));
        }

        let last_lag = layout.max_lag + 1;
        resize_buffer(&mut self.difference, last_lag + 1)?;
        resize_buffer(&mut self.normalized, last_lag + 1)?;
        difference_function(
            &self.centered,
            layout.samples - last_lag,
            &mut self.difference,
        );
        normalize_difference(&self.difference, &mut self.normalized);

        // Choose the first qualifying valley, not the global minimum at a
        // multiple of the period. No below-threshold valley means uncertainty.
        for lag in layout.min_lag..=layout.max_lag {
            let value = self.normalized[lag];
            if value >= self.normalized[lag - 1] || value > self.normalized[lag + 1] {
                continue;
            }
            let (_, depth) = interpolate(&self.normalized, lag);
            if depth >= self.config.threshold {
                continue;
            }
            // Raw difference interpolation avoids CMNDF's period bias (YIN §II.E).
            let (period, _) = interpolate(&self.difference, lag);
            let frequency = f64::from(sample_rate) / period;
            if frequency < self.config.frequency_range.min_hz()
                || frequency > self.config.frequency_range.max_hz()
            {
                return Ok(PitchDetection::without_pitch(
                    PitchStatus::NoPitch,
                    rms_metric,
                ));
            }
            return Ok(PitchDetection {
                pitch: Some(PitchEstimate::new(
                    frequency,
                    (1.0 - depth).clamp(0.0, 1.0) as f32,
                )?),
                status: PitchStatus::Detected,
                rms_dbfs: rms_metric,
            });
        }
        Ok(PitchDetection::without_pitch(
            PitchStatus::LowConfidence,
            rms_metric,
        ))
    }
}

fn resize_buffer(buffer: &mut Vec<f64>, len: usize) -> Result<(), CoreError> {
    if buffer.len() < len {
        buffer
            .try_reserve_exact(len - buffer.len())
            .map_err(|_| CoreError::AllocationFailed)?;
    }
    buffer.resize(len, 0.0);
    Ok(())
}

fn difference_function(pcm: &[f64], comparison_samples: usize, difference: &mut [f64]) {
    difference[0] = 0.0;
    for (lag, value) in difference.iter_mut().enumerate().skip(1) {
        *value = pcm[..comparison_samples]
            .iter()
            .zip(&pcm[lag..lag + comparison_samples])
            .map(|(a, b)| (a - b).powi(2))
            .sum();
    }
}

fn normalize_difference(difference: &[f64], normalized: &mut [f64]) {
    normalized[0] = 1.0;
    let mut sum = 0.0;
    for lag in 1..difference.len() {
        sum += difference[lag];
        normalized[lag] = if sum > 0.0 {
            difference[lag] * lag as f64 / sum
        } else {
            1.0
        };
    }
}

fn interpolate(values: &[f64], lag: usize) -> (f64, f64) {
    let (left, center, right) = (values[lag - 1], values[lag], values[lag + 1]);
    let curvature = left - 2.0 * center + right;
    if curvature > 0.0 {
        let offset = (left - right) / (2.0 * curvature);
        if offset.abs() <= 1.0 {
            return (lag as f64 + offset, center - 0.25 * (left - right) * offset);
        }
    }
    (lag as f64, center)
}

#[cfg(test)]
mod tests {
    use super::{difference_function, interpolate, normalize_difference};

    #[test]
    fn difference_and_cmndf_match_hand_computed_fixed_span() {
        let mut difference = [0.0; 5];
        difference_function(&[0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0], 3, &mut difference);
        assert_eq!(difference, [0.0, 3.0, 4.0, 3.0, 0.0]);
        let mut normalized = [0.0; 5];
        normalize_difference(&difference, &mut normalized);
        assert_eq!(normalized, [1.0, 1.0, 8.0 / 7.0, 0.9, 0.0]);
        normalize_difference(&[0.0; 5], &mut normalized);
        assert_eq!(normalized, [1.0; 5]);
    }

    #[test]
    fn interpolation_finds_quadratic_vertex_and_handles_flat_values() {
        let (lag, depth) = interpolate(&[1.5625, 0.0625, 0.5625], 1);
        assert_eq!((lag, depth), (1.25, 0.0));
        assert_eq!(interpolate(&[1.0; 3], 1), (1.0, 1.0));
    }
}
