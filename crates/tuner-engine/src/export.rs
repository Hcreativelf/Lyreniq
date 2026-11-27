//! CSV export for offline pitch timelines with explicit configuration metadata headers.
//! Never writes private absolute file paths or raw audio bytes.
use crate::{EngineError, FileAnalysisResult};
use audio_decode::ChannelMode;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use tuner_core::PitchStatus;

/// Exports pitch timeline frames and analysis metadata to a CSV writer.
/// Missing values (silent / low-confidence frames) are emitted as empty cells, never zeros or NaN.
pub fn export_timeline_csv(
    result: &FileAnalysisResult,
    writer: &mut impl Write,
) -> Result<(), EngineError> {
    let channel_desc = match result.config.channel_mode {
        ChannelMode::MixMono => "mix_mono".to_string(),
        ChannelMode::Channel(idx) => format!("channel_{idx}"),
    };
    let locked_desc = result.config.locked_note.map_or_else(
        || "auto".into(),
        |n| n.name(result.config.tuning.spelling()),
    );

    writeln!(writer, "# generator=Lyreniq {}", env!("CARGO_PKG_VERSION"))
        .map_err(|e| EngineError::Io(e.to_string()))?;
    writeln!(
        writer,
        "# format={:?},is_lossy={},sample_rate_hz={},channels={},channel_mode={}",
        result.metadata.format,
        result.metadata.is_lossy,
        result.metadata.sample_rate,
        result.metadata.channels,
        channel_desc
    )
    .map_err(|e| EngineError::Io(e.to_string()))?;
    writeln!(
        writer,
        "# reference_a4_hz={:.4},tolerance_cents={:.2},profile={},target={},hop_ms={},valid_coverage={:.6}",
        result.config.tuning.reference_a4_hz(),
        result.config.tuning.tolerance_cents(),
        result.config.profile.label(),
        locked_desc,
        result.config.hop_ms,
        result.summary.valid_coverage_ratio
    )
    .map_err(|e| EngineError::Io(e.to_string()))?;

    writeln!(
        writer,
        "time_s,frequency_hz,note,target_hz,cents,confidence,rms_dbfs,status"
    )
    .map_err(|e| EngineError::Io(e.to_string()))?;

    for frame in &result.frames {
        let status_str = match frame.status {
            PitchStatus::Idle => "idle",
            PitchStatus::Listening => "listening",
            PitchStatus::NoPitch => "no_pitch",
            PitchStatus::LowConfidence => "low_confidence",
            PitchStatus::Detected => "detected",
            PitchStatus::Error => "error",
        };
        let freq = frame
            .frequency_hz
            .map_or_else(String::new, |v| format!("{v:.6}"));
        let (note, target, cents) = match &frame.tuning {
            Some(t) => (
                t.note_name().to_string(),
                format!("{:.6}", t.target_hz()),
                format!("{:.4}", t.cents()),
            ),
            None => (String::new(), String::new(), String::new()),
        };
        let conf = frame
            .confidence
            .map_or_else(String::new, |v| format!("{v:.4}"));
        let rms = frame
            .rms_dbfs
            .map_or_else(String::new, |v| format!("{v:.2}"));

        writeln!(
            writer,
            "{:.6},{},{},{},{},{},{},{}",
            frame.time_s, freq, note, target, cents, conf, rms, status_str
        )
        .map_err(|e| EngineError::Io(e.to_string()))?;
    }
    writer.flush().map_err(|e| EngineError::Io(e.to_string()))?;
    Ok(())
}

/// Writes the pitch timeline CSV to `path` (supports Unicode / Chinese file paths).
pub fn export_timeline_csv_to_path(
    result: &FileAnalysisResult,
    path: impl AsRef<Path>,
) -> Result<(), EngineError> {
    let file = File::create(path.as_ref()).map_err(|e| EngineError::Io(e.to_string()))?;
    let mut writer = BufWriter::new(file);
    export_timeline_csv(result, &mut writer)
}
