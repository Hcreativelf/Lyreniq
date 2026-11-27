//! Reusable FFT magnitude spectrum analyzer with Hann window coherent-gain normalization,
//! relative dBFS scaling, harmonic queries and log-frequency binning for UI rendering.
use crate::CoreError;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;
use std::sync::Arc;

pub const MIN_FFT_SIZE: usize = 512;
pub const MAX_FFT_SIZE: usize = 16_384;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectrumConfig {
    fft_size: usize,
    floor_dbfs: f32,
}

impl Default for SpectrumConfig {
    fn default() -> Self {
        Self {
            fft_size: 4096,
            floor_dbfs: -100.0,
        }
    }
}

impl SpectrumConfig {
    pub fn new(fft_size: usize, floor_dbfs: f32) -> Result<Self, CoreError> {
        if !(MIN_FFT_SIZE..=MAX_FFT_SIZE).contains(&fft_size) || !fft_size.is_power_of_two() {
            return Err(CoreError::InvalidValue(
                "fft_size (power of two in 512..=16384)",
            ));
        }
        if !floor_dbfs.is_finite() || !(-160.0..=-20.0).contains(&floor_dbfs) {
            return Err(CoreError::InvalidValue("floor_dbfs (-160..=-20)"));
        }
        Ok(Self {
            fft_size,
            floor_dbfs,
        })
    }

    pub const fn fft_size(self) -> usize {
        self.fft_size
    }
    pub const fn floor_dbfs(self) -> f32 {
        self.floor_dbfs
    }
}

/// Informational harmonic measurement around an estimated fundamental frequency.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarmonicPeak {
    pub multiple: u32,
    pub expected_hz: f64,
    pub measured_hz: f64,
    pub magnitude_dbfs: f32,
}

/// Compact log-frequency binned spectrum for low-bandwidth IPC and Canvas 2D rendering.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogSpectrum {
    pub min_hz: f64,
    pub max_hz: f64,
    pub floor_dbfs: f32,
    pub bins_dbfs: Vec<f32>,
}

/// One-sided Hann-windowed magnitude spectrum normalized to peak full-scale dBFS.
/// `peak_frequency_hz` is strictly informational and must never replace pitch detection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectrumFrame {
    sample_rate: u32,
    fft_size: usize,
    bin_hz: f64,
    floor_dbfs: f32,
    magnitudes_dbfs: Vec<f32>,
    peak_frequency_hz: Option<f64>,
    peak_dbfs: Option<f32>,
}

impl SpectrumFrame {
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub const fn fft_size(&self) -> usize {
        self.fft_size
    }
    pub const fn bin_hz(&self) -> f64 {
        self.bin_hz
    }
    pub const fn floor_dbfs(&self) -> f32 {
        self.floor_dbfs
    }
    pub fn magnitudes_dbfs(&self) -> &[f32] {
        &self.magnitudes_dbfs
    }
    pub const fn peak_frequency_hz(&self) -> Option<f64> {
        self.peak_frequency_hz
    }
    pub const fn peak_dbfs(&self) -> Option<f32> {
        self.peak_dbfs
    }

    pub fn frequency_at_bin(&self, bin: usize) -> f64 {
        bin as f64 * self.bin_hz
    }

    /// Linearly interpolated magnitude in dBFS at `frequency_hz`, or `None` if out of bounds.
    pub fn magnitude_at_hz(&self, frequency_hz: f64) -> Option<f32> {
        if !frequency_hz.is_finite() || frequency_hz < 0.0 || self.magnitudes_dbfs.is_empty() {
            return None;
        }
        let max_hz = f64::from(self.sample_rate) / 2.0;
        if frequency_hz > max_hz {
            return None;
        }
        let pos = frequency_hz / self.bin_hz;
        let idx = pos.floor() as usize;
        if idx + 1 >= self.magnitudes_dbfs.len() {
            return self.magnitudes_dbfs.last().copied();
        }
        let frac = (pos - idx as f64) as f32;
        let a = self.magnitudes_dbfs[idx];
        let b = self.magnitudes_dbfs[idx + 1];
        Some(a + (b - a) * frac)
    }

    /// Inspects spectral peaks near integer multiples `1..=max_harmonics` of `fundamental_hz`.
    pub fn harmonic_peaks(
        &self,
        fundamental_hz: f64,
        max_harmonics: usize,
        min_dbfs: f32,
    ) -> Vec<HarmonicPeak> {
        if !fundamental_hz.is_finite()
            || fundamental_hz <= 0.0
            || max_harmonics == 0
            || !min_dbfs.is_finite()
            || self.magnitudes_dbfs.len() < 3
        {
            return Vec::new();
        }
        let nyquist = f64::from(self.sample_rate) / 2.0;
        let max_bin = self.magnitudes_dbfs.len() - 1;
        let mut peaks = Vec::new();
        for multiple in 1..=max_harmonics {
            let expected_hz = fundamental_hz * multiple as f64;
            if expected_hz >= nyquist {
                break;
            }
            let center = (expected_hz / self.bin_hz).round() as usize;
            // Allow ±3% tolerance or ±2 bins around the expected harmonic for slight inharmonicity.
            let tolerance_bins = ((expected_hz * 0.03) / self.bin_hz).ceil().max(2.0) as usize;
            let lo = center.saturating_sub(tolerance_bins).max(1);
            let hi = (center + tolerance_bins).min(max_bin.saturating_sub(1));
            if lo > hi {
                continue;
            }
            let mut best_bin = lo;
            let mut best_db = self.magnitudes_dbfs[lo];
            for (bin, &db) in self.magnitudes_dbfs[lo..=hi].iter().enumerate() {
                if db > best_db {
                    best_db = db;
                    best_bin = lo + bin;
                }
            }
            if best_db >= min_dbfs {
                let measured_hz = parabolic_peak_hz(&self.magnitudes_dbfs, best_bin, self.bin_hz);
                peaks.push(HarmonicPeak {
                    multiple: multiple as u32,
                    expected_hz,
                    measured_hz,
                    magnitude_dbfs: best_db,
                });
            }
        }
        peaks
    }

    /// Compresses the linear FFT bins onto a log-frequency grid using per-bucket max dBFS.
    pub fn log_binned(
        &self,
        min_hz: f64,
        max_hz: f64,
        num_bins: usize,
    ) -> Result<LogSpectrum, CoreError> {
        if !min_hz.is_finite() || !max_hz.is_finite() || min_hz <= 0.0 || min_hz >= max_hz {
            return Err(CoreError::InvalidValue("log spectrum frequency bounds"));
        }
        if !(8..=2048).contains(&num_bins) {
            return Err(CoreError::InvalidValue("log spectrum bin count (8..=2048)"));
        }
        let nyquist = f64::from(self.sample_rate) / 2.0;
        let effective_max = max_hz.min(nyquist);
        if min_hz >= effective_max {
            return Err(CoreError::InvalidValue("log spectrum max_hz vs Nyquist"));
        }
        let log_min = min_hz.ln();
        let log_span = effective_max.ln() - log_min;
        let max_idx = self.magnitudes_dbfs.len().saturating_sub(1);
        let mut bins_dbfs = vec![self.floor_dbfs; num_bins];
        for (i, slot) in bins_dbfs.iter_mut().enumerate() {
            let f0 = (log_min + log_span * (i as f64 / num_bins as f64)).exp();
            let f1 = (log_min + log_span * ((i + 1) as f64 / num_bins as f64)).exp();
            let idx0 = (f0 / self.bin_hz).floor() as usize;
            let idx1 = (f1 / self.bin_hz).ceil() as usize;
            if idx0 >= max_idx {
                *slot = self.magnitudes_dbfs[max_idx];
                continue;
            }
            let start = idx0.min(max_idx);
            let end = idx1.min(max_idx);
            if start == end || end - start <= 1 {
                let center_hz = (f0 * f1).sqrt();
                *slot = self.magnitude_at_hz(center_hz).unwrap_or(self.floor_dbfs);
            } else {
                let mut peak = self.floor_dbfs;
                for &db in &self.magnitudes_dbfs[start..=end] {
                    if db > peak {
                        peak = db;
                    }
                }
                *slot = peak;
            }
        }
        Ok(LogSpectrum {
            min_hz,
            max_hz: effective_max,
            floor_dbfs: self.floor_dbfs,
            bins_dbfs,
        })
    }
}

/// Reusable FFT spectrum engine with zero steady-state scratch allocations.
pub struct SpectrumAnalyzer {
    config: SpectrumConfig,
    fft: Arc<dyn Fft<f64>>,
    window: Vec<f64>,
    window_sum: f64,
    buffer: Vec<Complex<f64>>,
    scratch: Vec<Complex<f64>>,
}

impl Default for SpectrumAnalyzer {
    fn default() -> Self {
        Self::new(SpectrumConfig::default())
    }
}

impl SpectrumAnalyzer {
    pub fn new(config: SpectrumConfig) -> Self {
        let mut planner = FftPlanner::<f64>::new();
        let fft = planner.plan_fft_forward(config.fft_size);
        let scratch_len = fft.get_inplace_scratch_len();
        let n = config.fft_size;
        let mut window = Vec::with_capacity(n);
        let mut window_sum = 0.0;
        for i in 0..n {
            // Periodic Hann window: exact coherent gain of 0.5 (sum = N / 2).
            let w = 0.5 * (1.0 - (TAU * i as f64 / n as f64).cos());
            window.push(w);
            window_sum += w;
        }
        Self {
            config,
            fft,
            window,
            window_sum,
            buffer: vec![Complex::new(0.0, 0.0); n],
            scratch: vec![Complex::new(0.0, 0.0); scratch_len],
        }
    }

    pub const fn config(&self) -> SpectrumConfig {
        self.config
    }

    /// Analyzes mono normalized PCM `[-1, 1]` at `sample_rate`.
    /// Slices longer than `fft_size` use the trailing `fft_size` samples;
    /// shorter non-empty slices are Hann-windowed over their actual length and zero-padded.
    pub fn analyze(&mut self, pcm: &[f32], sample_rate: u32) -> Result<SpectrumFrame, CoreError> {
        if sample_rate == 0 {
            return Err(CoreError::InvalidValue("sample_rate"));
        }
        if pcm.iter().any(|x| !x.is_finite() || x.abs() > 1.0) {
            return Err(CoreError::InvalidValue("normalized finite PCM (-1..=1)"));
        }
        let n = self.config.fft_size;
        let bins = n / 2 + 1;
        let bin_hz = f64::from(sample_rate) / n as f64;
        if pcm.is_empty() {
            return Ok(SpectrumFrame {
                sample_rate,
                fft_size: n,
                bin_hz,
                floor_dbfs: self.config.floor_dbfs,
                magnitudes_dbfs: vec![self.config.floor_dbfs; bins],
                peak_frequency_hz: None,
                peak_dbfs: None,
            });
        }

        let slice = if pcm.len() > n {
            &pcm[pcm.len() - n..]
        } else {
            pcm
        };
        let len = slice.len();

        let norm_sum = if len == n {
            // Window-weighted mean guarantees X[0] == 0 (zero DC leakage in the windowed frame).
            let weighted_mean = slice
                .iter()
                .zip(&self.window)
                .map(|(&sample, &w)| f64::from(sample) * w)
                .sum::<f64>()
                / self.window_sum;
            for (slot, (&sample, &w)) in self.buffer.iter_mut().zip(slice.iter().zip(&self.window))
            {
                *slot = Complex::new((f64::from(sample) - weighted_mean) * w, 0.0);
            }
            self.window_sum
        } else {
            let mut sum = 0.0;
            let mut weighted_acc = 0.0;
            for (i, &sample) in slice.iter().enumerate() {
                let w = if len > 1 {
                    0.5 * (1.0 - (TAU * i as f64 / len as f64).cos())
                } else {
                    1.0
                };
                sum += w;
                weighted_acc += f64::from(sample) * w;
            }
            let weighted_mean = if sum > 0.0 { weighted_acc / sum } else { 0.0 };
            for (i, &sample) in slice.iter().enumerate() {
                let w = if len > 1 {
                    0.5 * (1.0 - (TAU * i as f64 / len as f64).cos())
                } else {
                    1.0
                };
                self.buffer[i] = Complex::new((f64::from(sample) - weighted_mean) * w, 0.0);
            }
            for slot in &mut self.buffer[len..n] {
                *slot = Complex::new(0.0, 0.0);
            }
            sum
        };

        self.fft
            .process_with_scratch(&mut self.buffer, &mut self.scratch);

        let floor = f64::from(self.config.floor_dbfs);
        let mut magnitudes_dbfs = Vec::with_capacity(bins);
        let mut best_bin: Option<usize> = None;
        let mut best_db = floor;

        for (k, bin_val) in self.buffer.iter().enumerate().take(bins) {
            let scale = if k == 0 || k == bins - 1 { 1.0 } else { 2.0 };
            let amp = if norm_sum > 0.0 {
                scale * bin_val.norm() / norm_sum
            } else {
                0.0
            };
            let db = if amp > 0.0 {
                (20.0 * amp.log10()).clamp(floor, 0.0)
            } else {
                floor
            };
            magnitudes_dbfs.push(db as f32);
            if k > 0 && k < bins - 1 && db > best_db {
                best_db = db;
                best_bin = Some(k);
            }
        }

        let (peak_frequency_hz, peak_dbfs) = match best_bin {
            Some(bin) if best_db > floor + 3.0 => (
                Some(parabolic_peak_hz(&magnitudes_dbfs, bin, bin_hz)),
                Some(best_db as f32),
            ),
            _ => (None, None),
        };

        Ok(SpectrumFrame {
            sample_rate,
            fft_size: n,
            bin_hz,
            floor_dbfs: self.config.floor_dbfs,
            magnitudes_dbfs,
            peak_frequency_hz,
            peak_dbfs,
        })
    }
}

fn parabolic_peak_hz(magnitudes_dbfs: &[f32], bin: usize, bin_hz: f64) -> f64 {
    if bin == 0 || bin + 1 >= magnitudes_dbfs.len() {
        return bin as f64 * bin_hz;
    }
    let left = f64::from(magnitudes_dbfs[bin - 1]);
    let center = f64::from(magnitudes_dbfs[bin]);
    let right = f64::from(magnitudes_dbfs[bin + 1]);
    let denom = left - 2.0 * center + right;
    if denom < 0.0 && denom.is_finite() {
        let delta = 0.5 * (left - right) / denom;
        if delta.is_finite() && delta.abs() <= 0.5 {
            return (bin as f64 + delta) * bin_hz;
        }
    }
    bin as f64 * bin_hz
}
