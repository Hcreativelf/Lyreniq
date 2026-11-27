//! Constant-memory streaming offline audio pitch analysis with cancellation,
//! monotonic progress reporting and summary statistics.
use crate::EngineError;
use audio_decode::{AudioDecoder, AudioMetadata, ChannelMode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use tuner_core::{
    InstrumentProfile, InstrumentProfileId, MidiNote, MultiScalePitchDetector, PitchStatus,
    TuningConfig, TuningDirection, TuningResult, YinConfig, YinDetector,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAnalysisConfig {
    pub tuning: TuningConfig,
    pub profile: InstrumentProfile,
    pub locked_note: Option<MidiNote>,
    pub channel_mode: ChannelMode,
    pub hop_ms: u32,
    pub use_multi_scale: bool,
    pub yin_threshold: f64,
    pub min_rms_dbfs: f64,
}

impl Default for FileAnalysisConfig {
    fn default() -> Self {
        Self {
            tuning: TuningConfig::default(),
            profile: InstrumentProfile::builtin(InstrumentProfileId::Chromatic)
                .expect("chromatic profile must be valid"),
            locked_note: None,
            channel_mode: ChannelMode::MixMono,
            hop_ms: 20,
            use_multi_scale: true,
            yin_threshold: 0.1,
            min_rms_dbfs: -60.0,
        }
    }
}

impl FileAnalysisConfig {
    pub fn validate(&self) -> Result<(), EngineError> {
        if !(10..=100).contains(&self.hop_ms) {
            return Err(EngineError::InvalidConfig("hop_ms (10..=100)"));
        }
        if !self.yin_threshold.is_finite() || self.yin_threshold <= 0.0 || self.yin_threshold >= 1.0
        {
            return Err(EngineError::InvalidConfig("yin_threshold (0..1 exclusive)"));
        }
        if !self.min_rms_dbfs.is_finite() || !(-160.0..=0.0).contains(&self.min_rms_dbfs) {
            return Err(EngineError::InvalidConfig("min_rms_dbfs (-160..=0)"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineFrame {
    /// Center timestamp of the analyzed window in seconds.
    pub time_s: f64,
    pub status: PitchStatus,
    /// Raw measured physical frequency in Hz (never smoothed).
    pub frequency_hz: Option<f64>,
    pub confidence: Option<f32>,
    pub rms_dbfs: Option<f32>,
    pub tuning: Option<TuningResult>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisProgress {
    pub decoded_frames: u64,
    pub total_frames: Option<u64>,
    /// Monotonically non-decreasing fraction in `[0.0, 1.0]` when total length is known.
    pub fraction: Option<f32>,
    pub emitted_points: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSummary {
    pub total_analysis_frames: usize,
    pub detected_frames: usize,
    /// Only frames with `PitchStatus::Detected` and `TuningDirection::InTune`.
    /// Silent / undetected frames are NEVER counted as in-tune.
    pub in_tune_frames: usize,
    pub valid_coverage_ratio: f64,
    pub in_tune_ratio_of_detected: Option<f64>,
    pub mean_cents: Option<f64>,
    pub median_cents: Option<f64>,
    pub std_dev_cents: Option<f64>,
    pub min_cents: Option<f64>,
    pub max_cents: Option<f64>,
    pub dominant_note: Option<String>,
    pub phase_cancellation_warning: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAnalysisResult {
    pub metadata: AudioMetadata,
    pub config: FileAnalysisConfig,
    pub summary: AnalysisSummary,
    pub frames: Vec<TimelineFrame>,
}

enum AnalysisDetector {
    MultiScale(Box<MultiScalePitchDetector>),
    Fixed(Box<YinDetector>),
}

impl AnalysisDetector {
    fn new(config: &FileAnalysisConfig, sample_rate: u32) -> Result<(Self, usize), EngineError> {
        let range = config.profile.frequency_range();
        range.validate_for_sample_rate(sample_rate)?;
        if config.use_multi_scale {
            let det =
                MultiScalePitchDetector::new(range, config.yin_threshold, config.min_rms_dbfs)?;
            let window = det.full_window_samples(sample_rate)?;
            Ok((Self::MultiScale(Box::new(det)), window))
        } else {
            let min_window_ms = ((2.2 * 1000.0 / range.min_hz()).ceil() as u32).clamp(60, 300);
            let yin_cfg = YinConfig::new(
                range,
                min_window_ms,
                config.yin_threshold,
                config.min_rms_dbfs,
            )?;
            let window = yin_cfg.sample_count(sample_rate)?;
            Ok((Self::Fixed(Box::new(YinDetector::new(yin_cfg))), window))
        }
    }

    fn detect(
        &mut self,
        pcm: &[f32],
        sample_rate: u32,
    ) -> Result<tuner_core::PitchDetection, EngineError> {
        match self {
            Self::MultiScale(det) => Ok(det.detect(pcm, sample_rate)?),
            Self::Fixed(det) => Ok(det.detect(pcm, sample_rate)?),
        }
    }
}

/// Streaming offline analyzer using a bounded sliding PCM buffer.
pub struct FileAnalyzer;

impl FileAnalyzer {
    pub fn analyze_path(
        path: impl AsRef<Path>,
        config: FileAnalysisConfig,
        cancel: &AtomicBool,
        on_progress: impl FnMut(AnalysisProgress),
    ) -> Result<FileAnalysisResult, EngineError> {
        config.validate()?;
        if cancel.load(Ordering::Relaxed) {
            return Err(EngineError::Cancelled);
        }
        let decoder = AudioDecoder::open_path(path, config.channel_mode)?;
        Self::analyze_decoder(decoder, config, cancel, on_progress)
    }

    pub fn analyze_decoder(
        mut decoder: AudioDecoder,
        config: FileAnalysisConfig,
        cancel: &AtomicBool,
        mut on_progress: impl FnMut(AnalysisProgress),
    ) -> Result<FileAnalysisResult, EngineError> {
        config.validate()?;
        let metadata = decoder.metadata().clone();
        let sample_rate = metadata.sample_rate;
        let hop_samples =
            ((u64::from(sample_rate) * u64::from(config.hop_ms)) / 1000).max(1) as usize;

        let (mut detector, window_samples) = AnalysisDetector::new(&config, sample_rate)?;
        let mut ring: VecDeque<f32> = VecDeque::with_capacity(window_samples + hop_samples * 2);
        let mut scratch: Vec<f32> = Vec::with_capacity(window_samples);
        let mut frames: Vec<TimelineFrame> = Vec::new();

        let mut window_start_sample: u64 = 0;
        let mut last_fraction: f32 = 0.0;

        while let Some(chunk) = decoder.next_chunk()? {
            if cancel.load(Ordering::Relaxed) {
                return Err(EngineError::Cancelled);
            }
            ring.extend(chunk.mono_pcm);

            while ring.len() >= window_samples {
                if cancel.load(Ordering::Relaxed) {
                    return Err(EngineError::Cancelled);
                }
                scratch.clear();
                scratch.extend(ring.iter().take(window_samples).copied());

                let det = detector.detect(&scratch, sample_rate)?;
                let center_time_s = (window_start_sample as f64 + window_samples as f64 * 0.5)
                    / f64::from(sample_rate);
                let target_sel = config.profile.target_selection(config.locked_note);

                let frame = if let Some(pitch) = det.pitches().first().copied() {
                    let tuning = config.tuning.tune(pitch.frequency_hz(), target_sel).ok();
                    TimelineFrame {
                        time_s: center_time_s,
                        status: det.status(),
                        frequency_hz: Some(pitch.frequency_hz()),
                        confidence: Some(pitch.confidence()),
                        rms_dbfs: det.rms_dbfs(),
                        tuning,
                    }
                } else {
                    TimelineFrame {
                        time_s: center_time_s,
                        status: det.status(),
                        frequency_hz: None,
                        confidence: None,
                        rms_dbfs: det.rms_dbfs(),
                        tuning: None,
                    }
                };
                frames.push(frame);

                let drop_count = hop_samples.min(ring.len());
                ring.drain(..drop_count);
                window_start_sample += drop_count as u64;
            }

            let decoded_total = decoder.decoded_frames_total();
            let fraction = metadata.total_frames.filter(|&t| t > 0).map(|total| {
                let raw = (decoded_total as f64 / total as f64).clamp(0.0, 1.0) as f32;
                let monotonic = raw.max(last_fraction).min(1.0);
                last_fraction = monotonic;
                monotonic
            });

            on_progress(AnalysisProgress {
                decoded_frames: decoded_total,
                total_frames: metadata.total_frames,
                fraction,
                emitted_points: frames.len(),
            });
        }

        // If the entire audio clip was shorter than `window_samples` (e.g. short 100 ms sample),
        // analyze whatever is in `ring` once when multi-scale fast window can cover it.
        if frames.is_empty() && !ring.is_empty() {
            if cancel.load(Ordering::Relaxed) {
                return Err(EngineError::Cancelled);
            }
            scratch.clear();
            scratch.extend(ring.iter().copied());
            let det = detector.detect(&scratch, sample_rate)?;
            let center_time_s = (scratch.len() as f64 * 0.5) / f64::from(sample_rate);
            let target_sel = config.profile.target_selection(config.locked_note);
            let frame = if let Some(pitch) = det.pitches().first().copied() {
                let tuning = config.tuning.tune(pitch.frequency_hz(), target_sel).ok();
                TimelineFrame {
                    time_s: center_time_s,
                    status: det.status(),
                    frequency_hz: Some(pitch.frequency_hz()),
                    confidence: Some(pitch.confidence()),
                    rms_dbfs: det.rms_dbfs(),
                    tuning,
                }
            } else {
                TimelineFrame {
                    time_s: center_time_s,
                    status: det.status(),
                    frequency_hz: None,
                    confidence: None,
                    rms_dbfs: det.rms_dbfs(),
                    tuning: None,
                }
            };
            frames.push(frame);
        }

        let final_decoded = decoder.decoded_frames_total();
        on_progress(AnalysisProgress {
            decoded_frames: final_decoded,
            total_frames: metadata.total_frames,
            fraction: Some(1.0),
            emitted_points: frames.len(),
        });

        let summary = compute_summary(&frames, decoder.phase_cancellation_warning());
        Ok(FileAnalysisResult {
            metadata: AudioMetadata {
                total_frames: metadata.total_frames.or(Some(final_decoded)),
                duration_s: metadata
                    .duration_s
                    .or_else(|| Some(final_decoded as f64 / f64::from(sample_rate))),
                ..metadata
            },
            config,
            summary,
            frames,
        })
    }
}

fn compute_summary(frames: &[TimelineFrame], phase_cancellation_warning: bool) -> AnalysisSummary {
    let total = frames.len();
    let mut cents_values: Vec<f64> = Vec::new();
    let mut in_tune_frames = 0_usize;
    let mut note_counts: HashMap<&str, usize> = HashMap::new();

    for frame in frames {
        if frame.status == PitchStatus::Detected {
            if let Some(tuning) = &frame.tuning {
                cents_values.push(tuning.cents());
                if tuning.direction() == TuningDirection::InTune {
                    in_tune_frames += 1;
                }
                *note_counts.entry(tuning.note_name()).or_insert(0) += 1;
            }
        }
    }

    let detected_frames = cents_values.len();
    let valid_coverage_ratio = if total > 0 {
        detected_frames as f64 / total as f64
    } else {
        0.0
    };

    if detected_frames == 0 {
        return AnalysisSummary {
            total_analysis_frames: total,
            detected_frames: 0,
            in_tune_frames: 0,
            valid_coverage_ratio,
            in_tune_ratio_of_detected: None,
            mean_cents: None,
            median_cents: None,
            std_dev_cents: None,
            min_cents: None,
            max_cents: None,
            dominant_note: None,
            phase_cancellation_warning,
        };
    }

    cents_values.sort_by(f64::total_cmp);
    let mean = cents_values.iter().sum::<f64>() / detected_frames as f64;
    let variance = cents_values
        .iter()
        .map(|&c| (c - mean).powi(2))
        .sum::<f64>()
        / detected_frames as f64;
    let median = if detected_frames % 2 == 1 {
        cents_values[detected_frames / 2]
    } else {
        0.5 * (cents_values[detected_frames / 2 - 1] + cents_values[detected_frames / 2])
    };
    let dominant_note = note_counts
        .into_iter()
        .max_by(|(na, ca), (nb, cb)| ca.cmp(cb).then_with(|| nb.cmp(na)))
        .map(|(note, _)| note.to_string());

    AnalysisSummary {
        total_analysis_frames: total,
        detected_frames,
        in_tune_frames,
        valid_coverage_ratio,
        in_tune_ratio_of_detected: Some(in_tune_frames as f64 / detected_frames as f64),
        mean_cents: Some(mean),
        median_cents: Some(median),
        std_dev_cents: Some(variance.sqrt()),
        min_cents: cents_values.first().copied(),
        max_cents: cents_values.last().copied(),
        dominant_note,
        phase_cancellation_warning,
    }
}
