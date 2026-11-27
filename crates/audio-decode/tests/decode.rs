use audio_decode::{AudioDecoder, AudioFormat, ChannelMode, DecodeError};
use std::f64::consts::TAU;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn make_wav_i16(interleaved: &[i16], sample_rate: u32, channels: u16) -> Vec<u8> {
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

fn drain_all(decoder: &mut AudioDecoder) -> Vec<f32> {
    let mut all = Vec::new();
    while let Some(chunk) = decoder.next_chunk().unwrap() {
        assert!(chunk
            .mono_pcm
            .iter()
            .all(|s| s.is_finite() && s.abs() <= 1.0));
        all.extend(chunk.mono_pcm);
    }
    all
}

#[test]
fn decodes_wav_flac_and_mp3_fixtures_with_accurate_metadata() {
    // WAV 48 kHz
    let mut wav =
        AudioDecoder::open_path(fixture_path("tone_440_48k.wav"), ChannelMode::MixMono).unwrap();
    assert_eq!(wav.metadata().format, AudioFormat::Wav);
    assert_eq!(wav.metadata().sample_rate, 48_000);
    assert_eq!(wav.metadata().channels, 1);
    assert!(!wav.metadata().is_lossy);
    let wav_pcm = drain_all(&mut wav);
    assert_eq!(wav_pcm.len(), 48_000);
    assert!((wav.metadata().duration_s.unwrap() - 1.0).abs() < 1e-6);

    // FLAC 44.1 kHz
    let mut flac =
        AudioDecoder::open_path(fixture_path("tone_440_44k1.flac"), ChannelMode::MixMono).unwrap();
    assert_eq!(flac.metadata().format, AudioFormat::Flac);
    assert_eq!(flac.metadata().sample_rate, 44_100);
    assert_eq!(flac.metadata().channels, 1);
    assert!(!flac.metadata().is_lossy);
    let flac_pcm = drain_all(&mut flac);
    assert_eq!(flac_pcm.len(), 44_100);

    // MP3 48 kHz (lossy)
    let mut mp3 =
        AudioDecoder::open_path(fixture_path("tone_440_48k.mp3"), ChannelMode::MixMono).unwrap();
    assert_eq!(mp3.metadata().format, AudioFormat::Mp3);
    assert_eq!(mp3.metadata().sample_rate, 48_000);
    assert!(mp3.metadata().is_lossy);
    let mp3_pcm = drain_all(&mut mp3);
    // Allow encoder delay/padding around 1.0 s (~48,000 samples).
    assert!((40_000..=56_000).contains(&mp3_pcm.len()));
}

#[test]
fn stereo_channel_selection_and_phase_cancellation_detection_work() {
    let rate = 48_000_u32;
    let frames = 4_800_usize;
    let mut stereo_inverted = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let s = (0.5 * (TAU * 440.0 * i as f64 / f64::from(rate)).sin() * 32_767.0) as i16;
        stereo_inverted.push(s);
        stereo_inverted.push(-s);
    }
    let wav = make_wav_i16(&stereo_inverted, rate, 2);

    // MixMono must flag anti-phase cancellation.
    let mut mix_dec =
        AudioDecoder::open_bytes(wav.clone(), Some("wav"), ChannelMode::MixMono).unwrap();
    let mixed = drain_all(&mut mix_dec);
    assert_eq!(mixed.len(), frames);
    assert!(mix_dec.phase_cancellation_warning());

    // Selecting channel 0 cleanly extracts the full-amplitude left channel without warning.
    let mut left_dec =
        AudioDecoder::open_bytes(wav.clone(), Some("wav"), ChannelMode::Channel(0)).unwrap();
    let left = drain_all(&mut left_dec);
    assert!(!left_dec.phase_cancellation_warning());
    let peak = left.iter().fold(0.0_f32, |acc, &x| acc.max(x.abs()));
    assert!(peak > 0.45);

    // Out-of-range channel index rejected.
    assert!(matches!(
        AudioDecoder::open_bytes(wav, Some("wav"), ChannelMode::Channel(2)),
        Err(DecodeError::InvalidChannel {
            requested: 2,
            available: 2
        })
    ));
}

#[test]
fn wrong_extension_malformed_truncated_and_missing_files_handled_safely() {
    // WAV bytes probed with misleading ".mp3" extension still decode via content fallback!
    let wav_bytes = std::fs::read(fixture_path("tone_440_48k.wav")).unwrap();
    let mut wrong_ext =
        AudioDecoder::open_bytes(wav_bytes.clone(), Some("mp3"), ChannelMode::MixMono).unwrap();
    assert_eq!(wrong_ext.metadata().format, AudioFormat::Wav);
    assert_eq!(drain_all(&mut wrong_ext).len(), 48_000);

    // Truncated WAV (cut in half) still decodes the available valid prefix without panicking.
    let truncated = wav_bytes[..wav_bytes.len() / 2].to_vec();
    let mut trunc_dec =
        AudioDecoder::open_bytes(truncated, Some("wav"), ChannelMode::MixMono).unwrap();
    let partial = drain_all(&mut trunc_dec);
    assert!(!partial.is_empty() && partial.len() < 48_000);

    // Garbage bytes rejected cleanly.
    assert!(AudioDecoder::open_bytes(
        vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x11],
        Some("mp3"),
        ChannelMode::MixMono
    )
    .is_err());
    assert!(AudioDecoder::open_bytes(Vec::new(), Some("wav"), ChannelMode::MixMono).is_err());

    // Nonexistent path returns Io error.
    assert!(matches!(
        AudioDecoder::open_path(fixture_path("does_not_exist.wav"), ChannelMode::MixMono),
        Err(DecodeError::Io(_))
    ));
}
