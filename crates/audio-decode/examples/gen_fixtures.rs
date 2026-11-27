//! Deterministic synthetic WAV and RFC-9639-compliant FLAC fixture generator.
//! Usage: cargo run -p audio-decode --example gen_fixtures --locked
use std::f64::consts::TAU;
use std::fs;
use std::path::Path;

fn main() {
    let dir = Path::new("tests/fixtures");
    fs::create_dir_all(dir).expect("create tests/fixtures");

    let wav_samples = sine_i16(48_000, 440.0, 0.5, 48_000);
    let wav_bytes = encode_wav_i16(&wav_samples, 48_000, 1);
    fs::write(dir.join("tone_440_48k.wav"), &wav_bytes).expect("write wav fixture");

    let flac_samples = sine_i16(44_100, 440.0, 0.5, 44_100);
    let flac_bytes = encode_flac_mono_i16(&flac_samples, 44_100);
    fs::write(dir.join("tone_440_44k1.flac"), &flac_bytes).expect("write flac fixture");

    println!(
        "Generated tone_440_48k.wav ({} bytes) and tone_440_44k1.flac ({} bytes)",
        wav_bytes.len(),
        flac_bytes.len()
    );
}

fn sine_i16(sample_rate: u32, hz: f64, amplitude: f64, count: usize) -> Vec<i16> {
    (0..count)
        .map(|i| {
            let s = amplitude * (TAU * hz * i as f64 / f64::from(sample_rate)).sin();
            (s * 32_767.0).round().clamp(-32_768.0, 32_767.0) as i16
        })
        .collect()
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
    out.extend_from_slice(&1_u16.to_le_bytes()); // PCM
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

/// Minimal RFC 9639 compliant 16-bit mono FLAC encoder using verbatim subframes.
fn encode_flac_mono_i16(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    const BLOCK_SIZE: usize = 4096;
    let mut out = Vec::new();
    out.extend_from_slice(b"fLaC");

    // STREAMINFO metadata block (last block = 1, type = 0, length = 34).
    out.push(0x80);
    out.extend_from_slice(&[0x00, 0x00, 34]);
    let min_block = (BLOCK_SIZE as u16).min(samples.len().max(16) as u16);
    let max_block = BLOCK_SIZE as u16;
    out.extend_from_slice(&min_block.to_be_bytes());
    out.extend_from_slice(&max_block.to_be_bytes());
    out.extend_from_slice(&[0, 0, 0]); // min frame size unknown
    out.extend_from_slice(&[0, 0, 0]); // max frame size unknown

    // 20-bit sample_rate | 3-bit (channels-1=0) | 5-bit (bps-1=15) | 36-bit total_samples
    let total = samples.len() as u64;
    let packed: u64 = (u64::from(sample_rate) << 44) | (15_u64 << 36) | (total & 0x000F_FFFF_FFFF);
    out.extend_from_slice(&packed.to_be_bytes());
    out.extend_from_slice(&[0_u8; 16]); // MD5 = 0 (unset, valid per RFC 9639)

    let sr_code: u8 = match sample_rate {
        44_100 => 0b1001,
        48_000 => 0b1010,
        _ => panic!("unsupported fixture rate {sample_rate}"),
    };

    for (frame_idx, chunk) in samples.chunks(BLOCK_SIZE).enumerate() {
        let mut frame = Vec::with_capacity(16 + chunk.len() * 2);
        let n = chunk.len();
        let (bs_code, extra_bs): (u8, Option<u16>) = if n == 4096 {
            (0b1100, None)
        } else {
            (0b0111, Some((n - 1) as u16))
        };
        // Sync code (14 bits: 0x3FFE), reserved (0), blocking strategy (0 = fixed)
        frame.push(0xFF);
        frame.push(0xF8);
        frame.push((bs_code << 4) | sr_code);
        // Channel assignment = 0 (mono), sample size = 0b100 (16-bit), reserved = 0
        frame.push(0x08);
        // UTF-8 coded frame number
        encode_utf8_u64(frame_idx as u64, &mut frame);
        if let Some(bs_minus_1) = extra_bs {
            frame.extend_from_slice(&bs_minus_1.to_be_bytes());
        }
        let crc8 = flac_crc8(&frame);
        frame.push(crc8);

        // Subframe header: 0 (pad) | 000001 (verbatim) | 0 (no wasted bits) => 0x02
        frame.push(0x02);
        for &s in chunk {
            frame.extend_from_slice(&s.to_be_bytes());
        }
        let crc16 = flac_crc16(&frame);
        frame.extend_from_slice(&crc16.to_be_bytes());
        out.extend_from_slice(&frame);
    }

    out
}

fn encode_utf8_u64(val: u64, out: &mut Vec<u8>) {
    if val < 0x80 {
        out.push(val as u8);
    } else if val < 0x800 {
        out.push(0xC0 | ((val >> 6) as u8));
        out.push(0x80 | ((val & 0x3F) as u8));
    } else {
        out.push(0xE0 | ((val >> 12) as u8));
        out.push(0x80 | (((val >> 6) & 0x3F) as u8));
        out.push(0x80 | ((val & 0x3F) as u8));
    }
}

fn flac_crc8(data: &[u8]) -> u8 {
    let mut crc = 0_u8;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            crc = if (crc & 0x80) != 0 {
                (crc << 1) ^ 0x07
            } else {
                crc << 1
            };
        }
    }
    crc
}

fn flac_crc16(data: &[u8]) -> u16 {
    let mut crc = 0_u16;
    for &byte in data {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            crc = if (crc & 0x8000) != 0 {
                (crc << 1) ^ 0x8005
            } else {
                crc << 1
            };
        }
    }
    crc
}
