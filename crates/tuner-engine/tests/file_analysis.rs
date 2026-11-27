use audio_decode::{AudioDecoder, AudioFormat, ChannelMode};
use std::f64::consts::TAU;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tuner_core::{
    cents_difference, InstrumentProfile, InstrumentProfileId, MidiNote, NoteSpelling, PitchStatus,
    TuningConfig,
};
use tuner_engine::{
    export_timeline_csv, export_timeline_csv_to_path, EngineError, FileAnalysisConfig, FileAnalyzer,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn encode_wav_i16(interleaved: &[i16], sample_rate: u32, channels: u16) -> Vec<u8> {
    let data_bytes = (interleaved.len() * 2) as u32;
    let byte_rate = sample_rate * u32::from(channels) * 2;
    let block_align = channels * 2;
    let mut out = Vec::with_capacity(44 + data_bytes as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16_u32.to_le_bytes());
    out.extend_from_slice(&1_u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&16_u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_bytes.to_le_bytes());
    for &s in interleaved {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

#[test]
fn analyzes_all_three_fixture_formats_wav_flac_and_mp3() {
    let cancel = AtomicBool::new(false);
    for (file, expected_fmt, expected_lossy, max_cents) in [
        ("tone_440_48k.wav", AudioFormat::Wav, false, 1.0),
        ("tone_440_44k1.flac", AudioFormat::Flac, false, 1.0),
        ("tone_440_48k.mp3", AudioFormat::Mp3, true, 2.0),
    ] {
        let mut last_frac = 0.0_f32;
        let result = FileAnalyzer::analyze_path(
            fixture_path(file),
            FileAnalysisConfig::default(),
            &cancel,
            |progress| {
                if let Some(f) = progress.fraction {
                    assert!(f >= last_frac && f <= 1.0);
                    last_frac = f;
                }
            },
        )
        .unwrap();

        assert_eq!(result.metadata.format, expected_fmt);
        assert_eq!(result.metadata.is_lossy, expected_lossy);
        assert!(!result.frames.is_empty());
        assert!(result.summary.valid_coverage_ratio > 0.8);
        assert_eq!(result.summary.dominant_note.as_deref(), Some("A4"));
        assert!(
            result.summary.median_cents.unwrap().abs() <= max_cents,
            "{file}: median_cents = {:?}",
            result.summary.median_cents
        );

        // Timestamps strictly monotonic.
        for pair in result.frames.windows(2) {
            assert!(pair[1].time_s > pair[0].time_s);
        }
    }
}

#[test]
fn segmented_melody_preserves_silence_gaps_and_statistics_honesty() {
    // 0.6s A4 (440 Hz) -> 0.5s silence -> 0.6s C4 (261.6256 Hz)
    let rate = 48_000_u32;
    let mut pcm = Vec::new();
    let push_tone = |buf: &mut Vec<i16>, hz: f64, dur_s: f64| {
        let count = (f64::from(rate) * dur_s) as usize;
        for i in 0..count {
            let s = 0.5 * (TAU * hz * i as f64 / f64::from(rate)).sin();
            buf.push((s * 32_767.0) as i16);
        }
    };
    push_tone(&mut pcm, 440.0, 0.6);
    pcm.extend(std::iter::repeat_n(0_i16, (f64::from(rate) * 0.5) as usize));
    push_tone(&mut pcm, 261.6255653005986, 0.6);

    let wav = encode_wav_i16(&pcm, rate, 1);
    let decoder = AudioDecoder::open_bytes(wav, Some("wav"), ChannelMode::MixMono).unwrap();
    let cancel = AtomicBool::new(false);
    let result =
        FileAnalyzer::analyze_decoder(decoder, FileAnalysisConfig::default(), &cancel, |_| {})
            .unwrap();

    let silent_frames = result
        .frames
        .iter()
        .filter(|f| f.status == PitchStatus::NoPitch)
        .count();
    assert!(
        silent_frames > 5,
        "middle silence must produce NoPitch gaps"
    );
    assert!(result.summary.detected_frames < result.summary.total_analysis_frames);
    assert!(result.summary.in_tune_frames <= result.summary.detected_frames);

    // Verify both A4 and C4 appear accurately in their respective segments.
    let has_a4 = result.frames.iter().any(|f| {
        f.tuning.as_ref().is_some_and(|t| t.note_name() == "A4")
            && cents_difference(f.frequency_hz.unwrap(), 440.0)
                .unwrap()
                .abs()
                < 1.0
    });
    let has_c4 = result.frames.iter().any(|f| {
        f.tuning.as_ref().is_some_and(|t| t.note_name() == "C4")
            && cents_difference(f.frequency_hz.unwrap(), 261.6255653005986)
                .unwrap()
                .abs()
                < 1.0
    });
    assert!(has_a4 && has_c4);
}

#[test]
fn cancellation_stops_analysis_and_a4_calibration_preserves_physical_hz() {
    let cancel = AtomicBool::new(true);
    let err = FileAnalyzer::analyze_path(
        fixture_path("tone_440_48k.wav"),
        FileAnalysisConfig::default(),
        &cancel,
        |_| {},
    )
    .unwrap_err();
    assert_eq!(err, EngineError::Cancelled);

    // Mid-stream cancellation on a 3-second file.
    let rate = 48_000_u32;
    let samples: Vec<i16> = (0..rate as usize * 3)
        .map(|i| (0.4 * (TAU * 440.0 * i as f64 / f64::from(rate)).sin() * 32_767.0) as i16)
        .collect();
    let wav = encode_wav_i16(&samples, rate, 1);
    let decoder = AudioDecoder::open_bytes(wav, Some("wav"), ChannelMode::MixMono).unwrap();
    let mid_cancel = AtomicBool::new(false);
    let mid_err =
        FileAnalyzer::analyze_decoder(decoder, FileAnalysisConfig::default(), &mid_cancel, |p| {
            if p.emitted_points >= 5 {
                mid_cancel.store(true, Ordering::Relaxed);
            }
        })
        .unwrap_err();
    assert_eq!(mid_err, EngineError::Cancelled);

    // A4 = 442 Hz shifts cents of 440 Hz tone to ~ -7.85 cents without altering physical Hz.
    let active = AtomicBool::new(false);
    let cfg = FileAnalysisConfig {
        tuning: TuningConfig::new(442.0, 3.0, NoteSpelling::Sharps).unwrap(),
        profile: InstrumentProfile::builtin(InstrumentProfileId::Chromatic).unwrap(),
        locked_note: Some(MidiNote::new(69).unwrap()),
        ..FileAnalysisConfig::default()
    };
    let res =
        FileAnalyzer::analyze_path(fixture_path("tone_440_48k.wav"), cfg, &active, |_| {}).unwrap();
    let first_det = res
        .frames
        .iter()
        .find(|f| f.frequency_hz.is_some())
        .unwrap();
    assert!((first_det.frequency_hz.unwrap() - 440.0).abs() < 0.1);
    assert!((first_det.tuning.as_ref().unwrap().cents() + 7.8514).abs() < 0.2);
}

#[test]
fn csv_export_matches_frames_omits_private_paths_and_supports_unicode_filenames() {
    let cancel = AtomicBool::new(false);
    let res = FileAnalyzer::analyze_path(
        fixture_path("tone_440_48k.wav"),
        FileAnalysisConfig::default(),
        &cancel,
        |_| {},
    )
    .unwrap();

    let mut bytes = Vec::new();
    export_timeline_csv(&res, &mut bytes).unwrap();
    let csv = String::from_utf8(bytes).unwrap();
    assert!(csv.contains("# generator=Lyreniq"));
    assert!(csv.contains("reference_a4_hz=440.0000"));
    assert!(
        !csv.contains("tone_440_48k.wav"),
        "must not leak local paths"
    );
    let data_lines: Vec<_> = csv.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(
        data_lines[0],
        "time_s,frequency_hz,note,target_hz,cents,confidence,rms_dbfs,status"
    );
    assert_eq!(data_lines.len() - 1, res.frames.len());
    assert!(!csv.contains("NaN") && !csv.contains("inf"));

    // Export to a Unicode (Chinese) path in temp dir and verify round-trip.
    let tmp_path = std::env::temp_dir().join("lyreniq_音准分析测试_export.csv");
    export_timeline_csv_to_path(&res, &tmp_path).unwrap();
    let on_disk = std::fs::read_to_string(&tmp_path).unwrap();
    assert_eq!(on_disk, csv);
    let _ = std::fs::remove_file(&tmp_path);
}

#[test]
fn long_sixty_second_stream_completes_with_bounded_streaming_window() {
    // 60 seconds at 44.1 kHz (~2.65M samples) analyzed at 50 ms hop -> ~1200 timeline frames.
    let rate = 44_100_u32;
    let total_samples = rate as usize * 60;
    let samples: Vec<i16> = (0..total_samples)
        .map(|i| (0.4 * (TAU * 110.0 * i as f64 / f64::from(rate)).sin() * 32_767.0) as i16)
        .collect();
    let wav = encode_wav_i16(&samples, rate, 1);
    let decoder = AudioDecoder::open_bytes(wav, Some("wav"), ChannelMode::MixMono).unwrap();
    let cancel = AtomicBool::new(false);
    let cfg = FileAnalysisConfig {
        hop_ms: 50,
        profile: InstrumentProfile::builtin(InstrumentProfileId::GuitarStandard).unwrap(),
        ..FileAnalysisConfig::default()
    };
    let mut progress_ticks = 0_usize;
    let res = FileAnalyzer::analyze_decoder(decoder, cfg, &cancel, |_| {
        progress_ticks += 1;
    })
    .unwrap();
    assert!(progress_ticks > 10);
    assert!((1150..=1210).contains(&res.frames.len()));
    assert_eq!(res.summary.dominant_note.as_deref(), Some("A2"));
    assert!(res.summary.median_cents.unwrap().abs() < 1.0);
}
