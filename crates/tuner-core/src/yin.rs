//! DSP 2.0 pitch detection: FFT-accelerated Fast YIN (O(N log N) worst-case) and
//! multi-scale fast/bass analysis with harmonic-aware octave verification.
use crate::{CoreError, FrequencyRange, PitchEstimate, PitchStatus};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use serde::{Deserialize, Serialize};
use std::f64::consts::{PI, TAU};
use std::sync::Arc;

/// Bounds scratch memory; this is not a real-time execution budget.
pub const MAX_YIN_WINDOW_SAMPLES: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    /// Low-latency configuration for guitar, violin, ukulele, voice and mid/high notes (55-5000 Hz).
    pub fn fast() -> Self {
        Self {
            frequency_range: FrequencyRange::new(55.0, 5000.0).expect("valid fast frequency range"),
            window_ms: 85,
            threshold: 0.1,
            min_rms_dbfs: -60.0,
        }
    }

    /// Deep-reach configuration for bass and low piano notes (20-600 Hz).
    pub fn bass() -> Self {
        Self {
            frequency_range: FrequencyRange::new(20.0, 600.0).expect("valid bass frequency range"),
            window_ms: 250,
            threshold: 0.1,
            min_rms_dbfs: -60.0,
        }
    }

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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchDetection {
    pitch: Option<PitchEstimate>,
    status: PitchStatus,
    rms_dbfs: Option<f32>,
}

impl PitchDetection {
    pub fn new_detected(pitch: PitchEstimate, rms_dbfs: Option<f32>) -> Self {
        Self {
            pitch: Some(pitch),
            status: PitchStatus::Detected,
            rms_dbfs,
        }
    }

    pub fn without_pitch(status: PitchStatus, rms_dbfs: Option<f32>) -> Self {
        Self {
            pitch: None,
            status,
            rms_dbfs,
        }
    }

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
}

struct CachedFftPair {
    size: usize,
    forward: Arc<dyn Fft<f64>>,
    inverse: Arc<dyn Fft<f64>>,
    scratch: Vec<Complex<f64>>,
}

/// FFT-accelerated YIN pitch detector (O(N log N) constant worst-case runtime).
/// Reuses all scratch buffers and FFT plans across calls; not for audio callbacks.
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
pub struct YinDetector {
    config: YinConfig,
    planner: FftPlanner<f64>,
    fft_cache: Option<CachedFftPair>,
    centered: Vec<f64>,
    prefix_sq: Vec<f64>,
    fft_buf: Vec<Complex<f64>>,
    difference: Vec<f64>,
    normalized: Vec<f64>,
}

impl Default for YinDetector {
    fn default() -> Self {
        Self::new(YinConfig::default())
    }
}

impl YinDetector {
    pub fn new(config: YinConfig) -> Self {
        Self {
            config,
            planner: FftPlanner::new(),
            fft_cache: None,
            centered: Vec::new(),
            prefix_sq: Vec::new(),
            fft_buf: Vec::new(),
            difference: Vec::new(),
            normalized: Vec::new(),
        }
    }

    pub const fn config(&self) -> YinConfig {
        self.config
    }

    /// Accepts normalized mono PCM `[-1, 1]` at its actual sample rate.
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

        let n = layout.samples;
        resize_f64(&mut self.centered, n)?;
        resize_f64(&mut self.prefix_sq, n + 1)?;

        let mean = pcm.iter().map(|&x| f64::from(x)).sum::<f64>() / n as f64;
        let mut energy = 0.0;
        self.prefix_sq[0] = 0.0;
        for (i, &input) in pcm.iter().enumerate() {
            let v = f64::from(input) - mean;
            self.centered[i] = v;
            energy += v * v;
            self.prefix_sq[i + 1] = energy;
        }

        let mean_square = energy / n as f64;
        let rms_dbfs = (mean_square > 0.0).then(|| (10.0 * mean_square.log10()).min(0.0));
        let rms_metric = rms_dbfs.map(|rms| rms as f32);
        if rms_dbfs.is_none_or(|rms| rms <= self.config.min_rms_dbfs) {
            return Ok(PitchDetection::without_pitch(
                PitchStatus::NoPitch,
                rms_metric,
            ));
        }

        let last_lag = layout.max_lag + 1;
        let comparison_samples = n - last_lag;
        resize_f64(&mut self.difference, last_lag + 1)?;
        resize_f64(&mut self.normalized, last_lag + 1)?;

        self.compute_fft_difference(n, comparison_samples, last_lag)?;
        normalize_difference(&self.difference, &mut self.normalized);

        let mut chosen_lag: Option<(usize, f64)> = None;
        for lag in layout.min_lag..=layout.max_lag {
            if let Some(depth) = qualifying_valley(&self.normalized, lag, self.config.threshold) {
                chosen_lag = Some((lag, depth));
                break;
            }
        }

        let Some((mut lag, mut depth)) = chosen_lag else {
            return Ok(PitchDetection::without_pitch(
                PitchStatus::LowConfidence,
                rms_metric,
            ));
        };

        // Harmonic-aware multiple-period verification:
        // Weak or missing fundamentals can produce a shallow dip at T/2 or T/3 while
        // the true fundamental at 2*lag or 3*lag exhibits a much deeper periodicity valley.
        if depth > 0.002 {
            let initial_period = interpolate(&self.difference, lag).0;
            'multiples: for multiple in [2.0, 3.0] {
                let center = (initial_period * multiple).round() as usize;
                let lo = center.saturating_sub(2).max(layout.min_lag);
                let hi = center.saturating_add(2).min(layout.max_lag);
                if lo > hi {
                    continue;
                }
                for candidate in lo..=hi {
                    if let Some(better) =
                        qualifying_valley(&self.normalized, candidate, depth * 0.25)
                    {
                        lag = candidate;
                        depth = better;
                        break 'multiples;
                    }
                }
            }
        }

        let period = if self.difference[lag] <= mean_square * comparison_samples as f64 * 1e-11 {
            // Near numerical zero (including FFT f64 roundoff), keep exact integer repetition.
            lag as f64
        } else if lag >= 2 && lag + 2 <= last_lag {
            let stride = (lag / 32).max(1).min(lag / 2).min((last_lag - lag) / 2);
            refine_period(&self.difference, lag, stride)
        } else {
            interpolate(&self.difference, lag).0
        };

        self.finish(period, depth, sample_rate, rms_metric)
    }

    /// Computes YIN difference d(tau) = E0 + E(tau) - 2*R(tau) for all tau in 0..=last_lag
    /// using a single complex-packed forward FFT + one inverse FFT of size M = next_pow2(N).
    fn compute_fft_difference(
        &mut self,
        n: usize,
        comparison_samples: usize,
        last_lag: usize,
    ) -> Result<(), CoreError> {
        let fft_len = n.next_power_of_two();
        self.ensure_fft_plan(fft_len)?;

        if self.fft_buf.len() < fft_len {
            self.fft_buf
                .try_reserve_exact(fft_len - self.fft_buf.len())
                .map_err(|_| CoreError::AllocationFailed)?;
        }
        self.fft_buf.resize(fft_len, Complex::new(0.0, 0.0));

        // Pack A[j] = x[j] (j < W) into real part and B[j] = x[j] (j < N) into imag part.
        for j in 0..comparison_samples {
            let v = self.centered[j];
            self.fft_buf[j] = Complex::new(v, v);
        }
        for j in comparison_samples..n {
            self.fft_buf[j] = Complex::new(0.0, self.centered[j]);
        }
        for slot in &mut self.fft_buf[n..fft_len] {
            *slot = Complex::new(0.0, 0.0);
        }

        let cache = self
            .fft_cache
            .as_mut()
            .expect("FFT cache must exist after ensure_fft_plan");
        cache
            .forward
            .process_with_scratch(&mut self.fft_buf, &mut cache.scratch);

        // Unpack A_hat and B_hat via conjugate symmetry and form C[k] = conj(A_hat[k]) * B_hat[k].
        let z0 = self.fft_buf[0];
        self.fft_buf[0] = Complex::new(z0.re * z0.im, 0.0);
        let half = fft_len / 2;
        if half > 0 {
            let zh = self.fft_buf[half];
            self.fft_buf[half] = Complex::new(zh.re * zh.im, 0.0);
        }
        for k in 1..half {
            let zk = self.fft_buf[k];
            let zm_conj = self.fft_buf[fft_len - k].conj();
            let a_hat = (zk + zm_conj) * 0.5;
            let diff = zk - zm_conj;
            let b_hat = Complex::new(diff.im * 0.5, -diff.re * 0.5);
            let ck = a_hat.conj() * b_hat;
            self.fft_buf[k] = ck;
            self.fft_buf[fft_len - k] = ck.conj();
        }

        cache
            .inverse
            .process_with_scratch(&mut self.fft_buf, &mut cache.scratch);

        let inv_m = 1.0 / fft_len as f64;
        let e0 = self.prefix_sq[comparison_samples];
        self.difference[0] = 0.0;
        for lag in 1..=last_lag {
            let e_tau = self.prefix_sq[lag + comparison_samples] - self.prefix_sq[lag];
            let corr = self.fft_buf[lag].re * inv_m;
            self.difference[lag] = (e0 + e_tau - 2.0 * corr).max(0.0);
        }
        Ok(())
    }

    fn ensure_fft_plan(&mut self, size: usize) -> Result<(), CoreError> {
        if self.fft_cache.as_ref().is_some_and(|c| c.size == size) {
            return Ok(());
        }
        let forward = self.planner.plan_fft_forward(size);
        let inverse = self.planner.plan_fft_inverse(size);
        let scratch_len = forward
            .get_inplace_scratch_len()
            .max(inverse.get_inplace_scratch_len());
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(scratch_len)
            .map_err(|_| CoreError::AllocationFailed)?;
        scratch.resize(scratch_len, Complex::new(0.0, 0.0));
        self.fft_cache = Some(CachedFftPair {
            size,
            forward,
            inverse,
            scratch,
        });
        Ok(())
    }

    fn finish(
        &self,
        period: f64,
        depth: f64,
        sample_rate: u32,
        rms_metric: Option<f32>,
    ) -> Result<PitchDetection, CoreError> {
        let frequency = f64::from(sample_rate) / period;
        if frequency < self.config.frequency_range.min_hz()
            || frequency > self.config.frequency_range.max_hz()
        {
            return Ok(PitchDetection::without_pitch(
                PitchStatus::NoPitch,
                rms_metric,
            ));
        }
        Ok(PitchDetection::new_detected(
            PitchEstimate::new(frequency, (1.0 - depth).clamp(0.0, 1.0) as f32)?,
            rms_metric,
        ))
    }
}

const DECIMATION_FACTOR: usize = 4;
const FIR_TAPS: usize = 31;

/// Multi-scale real-time pitch detector combining:
/// 1. Fast path (~85 ms window) for snappy mid/high-frequency lock (>= 55 Hz).
/// 2. 4x anti-aliased decimated bass path (up to 300 ms effective window) for low bass/piano (20-160 Hz).
pub struct MultiScalePitchDetector {
    frequency_range: FrequencyRange,
    fast_detector: Option<YinDetector>,
    bass_detector: Option<YinDetector>,
    fir_kernel: [f64; FIR_TAPS],
    decimated: Vec<f32>,
}

impl MultiScalePitchDetector {
    pub fn new(
        frequency_range: FrequencyRange,
        threshold: f64,
        min_rms_dbfs: f64,
    ) -> Result<Self, CoreError> {
        let fast_min = frequency_range.min_hz().max(55.0);
        let fast_detector = if fast_min < frequency_range.max_hz() {
            let fast_range = FrequencyRange::new(fast_min, frequency_range.max_hz())?;
            Some(YinDetector::new(YinConfig::new(
                fast_range,
                85,
                threshold,
                min_rms_dbfs,
            )?))
        } else {
            None
        };

        let bass_detector = if frequency_range.min_hz() < 55.0 {
            let bass_min = (frequency_range.min_hz() * 0.95).max(10.0);
            let bass_max = frequency_range.max_hz().min(160.0);
            let bass_range = FrequencyRange::new(bass_min, bass_max)?;
            Some(YinDetector::new(YinConfig::new(
                bass_range,
                280,
                threshold,
                min_rms_dbfs,
            )?))
        } else {
            None
        };

        Ok(Self {
            frequency_range,
            fast_detector,
            bass_detector,
            fir_kernel: build_lowpass_fir(),
            decimated: Vec::new(),
        })
    }

    pub const fn frequency_range(&self) -> FrequencyRange {
        self.frequency_range
    }

    /// Number of samples needed for fast-path detection.
    pub fn fast_window_samples(&self, sample_rate: u32) -> Result<usize, CoreError> {
        if let Some(fast) = &self.fast_detector {
            fast.config().sample_count(sample_rate)
        } else {
            self.full_window_samples(sample_rate)
        }
    }

    /// Number of samples needed for full low-frequency reach.
    pub fn full_window_samples(&self, sample_rate: u32) -> Result<usize, CoreError> {
        if let Some(bass) = &self.bass_detector {
            let dec_rate = sample_rate / DECIMATION_FACTOR as u32;
            let dec_samples = bass.config().sample_count(dec_rate)?;
            Ok(dec_samples * DECIMATION_FACTOR + FIR_TAPS)
        } else if let Some(fast) = &self.fast_detector {
            fast.config().sample_count(sample_rate)
        } else {
            Err(CoreError::InvalidValue("no active detector path"))
        }
    }

    /// Analyzes the trailing portion of `pcm` using the fast path first and falling back
    /// to the 4x-decimated bass path when configured for frequencies below 55 Hz.
    pub fn detect(&mut self, pcm: &[f32], sample_rate: u32) -> Result<PitchDetection, CoreError> {
        self.frequency_range.validate_for_sample_rate(sample_rate)?;
        if pcm.iter().any(|x| !x.is_finite() || x.abs() > 1.0) {
            return Err(CoreError::InvalidValue("normalized finite PCM (-1..=1)"));
        }

        let mut fast_result: Option<PitchDetection> = None;
        if let Some(fast) = &mut self.fast_detector {
            let fast_n = fast.config().sample_count(sample_rate)?;
            if pcm.len() >= fast_n {
                let tail = &pcm[pcm.len() - fast_n..];
                let res = fast.detect(tail, sample_rate)?;
                let accept_immediately = match res.pitches().first() {
                    Some(p) => {
                        self.bass_detector.is_none()
                            || p.frequency_hz() >= 110.0
                            || p.confidence() >= 0.94
                    }
                    None => false,
                };
                if accept_immediately {
                    return Ok(res);
                }
                fast_result = Some(res);
            }
        }

        if let Some(bass) = &mut self.bass_detector {
            let dec_rate = sample_rate / DECIMATION_FACTOR as u32;
            if dec_rate >= 2_000 {
                let dec_n = bass.config().sample_count(dec_rate)?;
                let required = (dec_n - 1) * DECIMATION_FACTOR + FIR_TAPS;
                if pcm.len() >= required {
                    let start = pcm.len() - required;
                    let src = &pcm[start..];
                    self.decimated.clear();
                    if self.decimated.capacity() < dec_n {
                        self.decimated
                            .try_reserve_exact(dec_n - self.decimated.capacity())
                            .map_err(|_| CoreError::AllocationFailed)?;
                    }
                    for m in 0..dec_n {
                        let base = m * DECIMATION_FACTOR;
                        let mut acc = 0.0;
                        for (k, &coeff) in self.fir_kernel.iter().enumerate() {
                            acc += f64::from(src[base + k]) * coeff;
                        }
                        self.decimated.push(acc.clamp(-1.0, 1.0) as f32);
                    }
                    let mut bass_res = bass.detect(&self.decimated, dec_rate)?;
                    // Compensate if sample_rate is not an exact multiple of DECIMATION_FACTOR.
                    let rate_scale =
                        (f64::from(sample_rate) / DECIMATION_FACTOR as f64) / f64::from(dec_rate);
                    if let Some(pitch) = bass_res.pitches().first().copied() {
                        let corrected_hz = pitch.frequency_hz() * rate_scale;
                        if (self.frequency_range.min_hz()..=self.frequency_range.max_hz())
                            .contains(&corrected_hz)
                        {
                            bass_res = PitchDetection::new_detected(
                                PitchEstimate::new(corrected_hz, pitch.confidence())?,
                                bass_res.rms_dbfs(),
                            );
                        } else {
                            bass_res = PitchDetection::without_pitch(
                                PitchStatus::NoPitch,
                                bass_res.rms_dbfs(),
                            );
                        }
                    }

                    if bass_res.status() == PitchStatus::Detected {
                        let bass_conf = bass_res.pitches()[0].confidence();
                        let fast_conf = fast_result
                            .and_then(|r| r.pitches().first().map(|p| p.confidence()))
                            .unwrap_or(-1.0);
                        if bass_conf >= fast_conf {
                            return Ok(bass_res);
                        }
                    } else if fast_result.is_none() {
                        return Ok(bass_res);
                    }
                }
            }
        }

        Ok(
            fast_result
                .unwrap_or_else(|| PitchDetection::without_pitch(PitchStatus::NoPitch, None)),
        )
    }
}

/// 31-tap Hamming-windowed sinc anti-aliasing FIR low-pass filter (cutoff = 0.09 * fs).
fn build_lowpass_fir() -> [f64; FIR_TAPS] {
    let mut kernel = [0.0; FIR_TAPS];
    let half = (FIR_TAPS - 1) as f64 / 2.0;
    let fc = 0.09;
    let mut sum = 0.0;
    for (i, slot) in kernel.iter_mut().enumerate() {
        let m = i as f64 - half;
        let sinc = if m == 0.0 {
            2.0 * fc
        } else {
            (TAU * fc * m).sin() / (PI * m)
        };
        let hamming = 0.54 - 0.46 * (TAU * i as f64 / (FIR_TAPS - 1) as f64).cos();
        let w = sinc * hamming;
        *slot = w;
        sum += w;
    }
    for slot in &mut kernel {
        *slot /= sum;
    }
    kernel
}

fn qualifying_valley(values: &[f64], lag: usize, threshold: f64) -> Option<f64> {
    if lag == 0 || lag + 1 >= values.len() {
        return None;
    }
    if values[lag] >= values[lag - 1] || values[lag] > values[lag + 1] {
        return None;
    }
    let depth = interpolate(values, lag).1;
    (depth < threshold).then_some(depth)
}

/// Five-point quartic interpolation reduces three-point high-frequency bias and
/// smooths low-frequency adjacent-lag noise at O(1) cost from precomputed difference[].
fn refine_period(values: &[f64], lag: usize, stride: usize) -> f64 {
    let fallback = interpolate(values, lag).0;
    let (m2, m1, c, p1, p2) = (
        values[lag - 2 * stride],
        values[lag - stride],
        values[lag],
        values[lag + stride],
        values[lag + 2 * stride],
    );
    let a1 = (m2 - 8.0 * m1 + 8.0 * p1 - p2) / 12.0;
    let a2 = (-m2 + 16.0 * m1 - 30.0 * c + 16.0 * p1 - p2) / 24.0;
    let a3 = (-m2 + 2.0 * m1 - 2.0 * p1 + p2) / 12.0;
    let a4 = (m2 - 4.0 * m1 + 6.0 * c - 4.0 * p1 + p2) / 24.0;
    let mut offset = (fallback - lag as f64) / stride as f64;
    for _ in 0..5 {
        let slope = a1 + offset * (2.0 * a2 + offset * (3.0 * a3 + offset * 4.0 * a4));
        let curvature = 2.0 * a2 + offset * (6.0 * a3 + offset * 12.0 * a4);
        if curvature <= 0.0 || !curvature.is_finite() {
            return fallback;
        }
        offset -= slope / curvature;
        if !offset.is_finite() || offset.abs() > if stride == 1 { 1.0 } else { 2.0 } {
            return fallback;
        }
    }
    lag as f64 + offset * stride as f64
}

fn resize_f64(buffer: &mut Vec<f64>, len: usize) -> Result<(), CoreError> {
    if buffer.len() < len {
        buffer
            .try_reserve_exact(len - buffer.len())
            .map_err(|_| CoreError::AllocationFailed)?;
    }
    buffer.resize(len, 0.0);
    Ok(())
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
    use super::{build_lowpass_fir, interpolate, normalize_difference, refine_period, YinDetector};

    #[test]
    fn fft_difference_matches_direct_definition_exactly() {
        let pcm = [0.15_f64, -0.4, 0.85, 0.2, -0.7, 0.5, -0.1, 0.6, -0.3, 0.25];
        let n = pcm.len();
        let last_lag = 4;
        let comparison_samples = n - last_lag;

        let mut prefix_sq = vec![0.0; n + 1];
        for i in 0..n {
            prefix_sq[i + 1] = prefix_sq[i] + pcm[i] * pcm[i];
        }
        let mut detector = YinDetector {
            centered: pcm.to_vec(),
            prefix_sq,
            difference: vec![0.0; last_lag + 1],
            ..YinDetector::default()
        };
        detector
            .compute_fft_difference(n, comparison_samples, last_lag)
            .unwrap();

        for lag in 1..=last_lag {
            let direct: f64 = pcm[..comparison_samples]
                .iter()
                .zip(&pcm[lag..lag + comparison_samples])
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            assert!(
                (detector.difference[lag] - direct).abs() < 1e-12,
                "lag {lag}: fft={} vs direct={direct}",
                detector.difference[lag]
            );
        }
    }

    #[test]
    fn quartic_refinement_finds_known_vertex_at_multiple_strides() {
        for stride in [1, 2] {
            let center = 3 * stride;
            let target = center as f64 + 0.25 * stride as f64;
            let values: Vec<_> = (0..=6 * stride)
                .map(|n| {
                    let x = n as f64 - target;
                    x * x + 0.001 * x.powi(4)
                })
                .collect();
            assert!((refine_period(&values, center, stride) - target).abs() < 1e-10);
        }
    }

    #[test]
    fn cmndf_and_interpolation_handle_hand_computed_and_flat_inputs() {
        let mut normalized = [0.0; 5];
        normalize_difference(&[0.0, 3.0, 4.0, 3.0, 0.0], &mut normalized);
        assert_eq!(normalized, [1.0, 1.0, 8.0 / 7.0, 0.9, 0.0]);
        normalize_difference(&[0.0; 5], &mut normalized);
        assert_eq!(normalized, [1.0; 5]);

        let (lag, depth) = interpolate(&[1.5625, 0.0625, 0.5625], 1);
        assert_eq!((lag, depth), (1.25, 0.0));
        assert_eq!(interpolate(&[1.0; 3], 1), (1.0, 1.0));
    }

    #[test]
    fn decimation_lowpass_fir_has_unity_dc_gain_and_symmetry() {
        let k = build_lowpass_fir();
        let sum: f64 = k.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
        for i in 0..k.len() / 2 {
            assert!((k[i] - k[k.len() - 1 - i]).abs() < 1e-12);
        }
    }
}
