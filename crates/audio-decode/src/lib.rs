//! Streaming, bounded-memory local audio decoder for WAV, FLAC and MP3 using Symphonia.
//! Never loads arbitrary files into memory at once and never blindly trusts file extensions.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{Cursor, ErrorKind};
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{
    Decoder, DecoderOptions, CODEC_TYPE_FLAC, CODEC_TYPE_MP3, CODEC_TYPE_NULL,
};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    Io(String),
    UnsupportedFormat(String),
    NoAudioTrack,
    InvalidMetadata(&'static str),
    DecodeFailed(String),
    InvalidChannel { requested: usize, available: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "audio I/O error: {msg}"),
            Self::UnsupportedFormat(msg) => write!(f, "unsupported or unrecognized audio: {msg}"),
            Self::NoAudioTrack => f.write_str("file contains no supported audio track"),
            Self::InvalidMetadata(field) => write!(f, "invalid audio metadata: {field}"),
            Self::DecodeFailed(msg) => write!(f, "audio decoding failed: {msg}"),
            Self::InvalidChannel {
                requested,
                available,
            } => write!(
                f,
                "requested channel {requested} out of range (available: {available})"
            ),
        }
    }
}

impl std::error::Error for DecodeError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioFormat {
    Wav,
    Flac,
    Mp3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "mode", content = "index")]
pub enum ChannelMode {
    /// Average all channels into mono, while monitoring for stereo phase cancellation.
    #[default]
    MixMono,
    /// Extract a single zero-indexed channel (e.g. 0 = left, 1 = right).
    Channel(usize),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioMetadata {
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub channels: usize,
    pub total_frames: Option<u64>,
    pub duration_s: Option<f64>,
    pub is_lossy: bool,
}

#[derive(Clone, Debug)]
pub struct DecodedChunk {
    /// Normalized, finite mono PCM in `[-1.0, 1.0]`.
    pub mono_pcm: Vec<f32>,
    /// Number of decoded audio frames in this chunk.
    pub frames_decoded: usize,
    /// True if averaging multiple channels suffered > 12 dB cancellation while channels were active.
    pub phase_cancellation_detected: bool,
}

/// Streaming packet-by-packet audio decoder.
pub struct AudioDecoder {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    metadata: AudioMetadata,
    channel_mode: ChannelMode,
    sample_buf: Option<SampleBuffer<f32>>,
    decoded_frames_total: u64,
    phase_cancellation_warning: bool,
    corrupt_packets_skipped: u64,
    finished: bool,
}

impl AudioDecoder {
    /// Opens a local audio file for streaming decoding.
    /// Automatically falls back to content-based probing if the file extension is misleading.
    pub fn open_path(
        path: impl AsRef<Path>,
        channel_mode: ChannelMode,
    ) -> Result<Self, DecodeError> {
        let path = path.as_ref();
        let ext = path.extension().and_then(|s| s.to_str());
        let file = File::open(path).map_err(|e| DecodeError::Io(e.to_string()))?;
        match Self::from_source(Box::new(file), ext, channel_mode) {
            Ok(decoder) => Ok(decoder),
            Err(DecodeError::UnsupportedFormat(_)) if ext.is_some() => {
                // Wrong extension (e.g. WAV named .mp3): reopen and probe strictly by content.
                let retry_file = File::open(path).map_err(|e| DecodeError::Io(e.to_string()))?;
                Self::from_source(Box::new(retry_file), None, channel_mode)
            }
            Err(err) => Err(err),
        }
    }

    /// Opens an in-memory byte buffer for streaming decoding (useful for tests and WASM).
    pub fn open_bytes(
        bytes: Vec<u8>,
        extension_hint: Option<&str>,
        channel_mode: ChannelMode,
    ) -> Result<Self, DecodeError> {
        if extension_hint.is_some() {
            match Self::from_source(
                Box::new(Cursor::new(bytes.clone())),
                extension_hint,
                channel_mode,
            ) {
                Ok(decoder) => return Ok(decoder),
                Err(DecodeError::UnsupportedFormat(_)) => {}
                Err(err) => return Err(err),
            }
        }
        Self::from_source(Box::new(Cursor::new(bytes)), None, channel_mode)
    }

    fn from_source(
        source: Box<dyn MediaSource>,
        extension_hint: Option<&str>,
        channel_mode: ChannelMode,
    ) -> Result<Self, DecodeError> {
        let mss = MediaSourceStream::new(source, Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = extension_hint {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions {
                    enable_gapless: true,
                    ..FormatOptions::default()
                },
                &MetadataOptions::default(),
            )
            .map_err(|e| DecodeError::UnsupportedFormat(e.to_string()))?;

        let format_reader = probed.format;
        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or(DecodeError::NoAudioTrack)?;

        let track_id = track.id;
        let codec = track.codec_params.codec;
        let (format, is_lossy) = match codec {
            CODEC_TYPE_MP3 => (AudioFormat::Mp3, true),
            CODEC_TYPE_FLAC => (AudioFormat::Flac, false),
            _ => (AudioFormat::Wav, false),
        };

        let sample_rate = track
            .codec_params
            .sample_rate
            .filter(|&r| r > 0)
            .ok_or(DecodeError::InvalidMetadata("missing or zero sample_rate"))?;

        let channels = track
            .codec_params
            .channels
            .map(|c| c.count())
            .filter(|&c| c > 0)
            .ok_or(DecodeError::InvalidMetadata("missing or zero channels"))?;

        if let ChannelMode::Channel(requested) = channel_mode {
            if requested >= channels {
                return Err(DecodeError::InvalidChannel {
                    requested,
                    available: channels,
                });
            }
        }

        let total_frames = track.codec_params.n_frames;
        let duration_s = total_frames.map(|n| n as f64 / f64::from(sample_rate));

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| DecodeError::UnsupportedFormat(e.to_string()))?;

        Ok(Self {
            format_reader,
            decoder,
            track_id,
            metadata: AudioMetadata {
                format,
                sample_rate,
                channels,
                total_frames,
                duration_s,
                is_lossy,
            },
            channel_mode,
            sample_buf: None,
            decoded_frames_total: 0,
            phase_cancellation_warning: false,
            corrupt_packets_skipped: 0,
            finished: false,
        })
    }

    pub const fn metadata(&self) -> &AudioMetadata {
        &self.metadata
    }
    pub const fn channel_mode(&self) -> ChannelMode {
        self.channel_mode
    }
    pub const fn decoded_frames_total(&self) -> u64 {
        self.decoded_frames_total
    }
    pub const fn phase_cancellation_warning(&self) -> bool {
        self.phase_cancellation_warning
    }
    pub const fn corrupt_packets_skipped(&self) -> u64 {
        self.corrupt_packets_skipped
    }

    /// Decodes and returns the next mono PCM chunk, or `Ok(None)` at clean end-of-stream.
    pub fn next_chunk(&mut self) -> Result<Option<DecodedChunk>, DecodeError> {
        if self.finished {
            return Ok(None);
        }

        loop {
            let packet = match self.format_reader.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(io_err))
                    if io_err.kind() == ErrorKind::UnexpectedEof =>
                {
                    self.finished = true;
                    if self.decoded_frames_total == 0 && self.corrupt_packets_skipped > 0 {
                        return Err(DecodeError::DecodeFailed(
                            "stream ended without any valid audio frames".into(),
                        ));
                    }
                    return Ok(None);
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(err) => {
                    self.finished = true;
                    if self.decoded_frames_total > 0 {
                        // Truncated tail after valid decoded audio: end cleanly without panicking.
                        return Ok(None);
                    }
                    return Err(DecodeError::DecodeFailed(err.to_string()));
                }
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(msg)) => {
                    self.corrupt_packets_skipped += 1;
                    if self.corrupt_packets_skipped > 32 && self.decoded_frames_total == 0 {
                        self.finished = true;
                        return Err(DecodeError::DecodeFailed(msg.to_string()));
                    }
                    continue;
                }
                Err(SymphoniaError::IoError(io_err))
                    if io_err.kind() == ErrorKind::UnexpectedEof =>
                {
                    self.finished = true;
                    return Ok(None);
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(err) => {
                    self.finished = true;
                    return Err(DecodeError::DecodeFailed(err.to_string()));
                }
            };

            let spec = *decoded.spec();
            let channels = spec.channels.count();
            if channels == 0 {
                continue;
            }
            if let ChannelMode::Channel(requested) = self.channel_mode {
                if requested >= channels {
                    return Err(DecodeError::InvalidChannel {
                        requested,
                        available: channels,
                    });
                }
            }

            let capacity = decoded.capacity() as u64;
            let needs_new_buf = self
                .sample_buf
                .as_ref()
                .is_none_or(|buf| buf.capacity() < decoded.capacity() * channels);
            if needs_new_buf {
                self.sample_buf = Some(SampleBuffer::<f32>::new(capacity, spec));
            }
            let sample_buf = self
                .sample_buf
                .as_mut()
                .expect("sample_buf must exist after initialization");
            sample_buf.copy_interleaved_ref(decoded);
            let interleaved = sample_buf.samples();
            let frames = interleaved.len() / channels;
            if frames == 0 {
                continue;
            }

            let (mono_pcm, phase_cancelled) =
                downmix_to_mono(interleaved, channels, self.channel_mode);
            if phase_cancelled {
                self.phase_cancellation_warning = true;
            }
            self.decoded_frames_total += frames as u64;

            return Ok(Some(DecodedChunk {
                mono_pcm,
                frames_decoded: frames,
                phase_cancellation_detected: phase_cancelled,
            }));
        }
    }
}

fn sanitize_sample(sample: f32) -> f32 {
    if sample.is_finite() {
        sample.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

fn downmix_to_mono(interleaved: &[f32], channels: usize, mode: ChannelMode) -> (Vec<f32>, bool) {
    let frames = interleaved.len() / channels;
    let mut mono = Vec::with_capacity(frames);

    match mode {
        ChannelMode::Channel(ch) => {
            for frame in interleaved.chunks_exact(channels) {
                mono.push(sanitize_sample(frame[ch]));
            }
            (mono, false)
        }
        ChannelMode::MixMono if channels == 1 => {
            for &s in interleaved {
                mono.push(sanitize_sample(s));
            }
            (mono, false)
        }
        ChannelMode::MixMono => {
            let inv_ch = 1.0 / channels as f64;
            let mut ch_energy = 0.0_f64;
            let mut mix_energy = 0.0_f64;
            for frame in interleaved.chunks_exact(channels) {
                let mut sum = 0.0_f64;
                for &raw in frame {
                    let s = f64::from(sanitize_sample(raw));
                    ch_energy += s * s;
                    sum += s;
                }
                let mixed = (sum * inv_ch).clamp(-1.0, 1.0);
                mix_energy += mixed * mixed;
                mono.push(mixed as f32);
            }
            let avg_ch_ms = ch_energy / (frames * channels) as f64;
            let mix_ms = mix_energy / frames as f64;
            // Signal present (> -50 dBFS) yet mixed mono lost > 12 dB (1/16 energy).
            let phase_cancelled = avg_ch_ms > 1e-5 && mix_ms < avg_ch_ms * 0.0625;
            (mono, phase_cancelled)
        }
    }
}
