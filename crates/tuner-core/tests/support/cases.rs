use tuner_core::{
    test_signal::{harmonic_tone, sine, white_noise, Harmonic},
    YinConfig,
};

pub const FRAMES: usize = 5;
pub const HOP_MS: u32 = 20;
pub const NOISE_SEED: u64 = 0x4c59_5245_4e49_5107;

pub struct Case {
    pub name: String,
    pub expected_hz: Option<f64>,
    pub pcm: Vec<f32>,
    pub config: YinConfig,
}

pub fn cases(rate: u32) -> Vec<Case> {
    let config = YinConfig::default();
    let length = config.sample_count(rate).unwrap() + (FRAMES - 1) * hop(rate);
    let mut output = Vec::new();
    for hz in [
        20.0,
        27.5,
        55.0,
        110.0,
        123.45,
        220.0,
        261.6255653005986,
        432.123,
        440.0,
        442.0,
        880.0,
        1760.0,
        5000.0,
    ] {
        output.push(Case {
            name: format!("pure_{hz}"),
            expected_hz: Some(hz),
            pcm: sine(rate, hz, 0.5, length).unwrap(),
            config,
        });
    }
    for (name, components) in [
        ("weak_fundamental", vec![(1, 0.02), (2, 0.46), (3, 0.4)]),
        ("dominant_second", vec![(1, 0.05), (2, 0.8)]),
        ("dominant_third", vec![(1, 0.05), (3, 0.8)]),
        ("missing_fundamental", vec![(2, 0.45), (3, 0.45)]),
    ] {
        let harmonics: Vec<_> = components
            .into_iter()
            .map(|(n, a)| Harmonic::new(n, a).unwrap())
            .collect();
        output.push(Case {
            name: name.into(),
            expected_hz: Some(110.0),
            pcm: harmonic_tone(rate, 110.0, &harmonics, length).unwrap(),
            config,
        });
    }
    for hz in [55.0, 110.0, 440.0] {
        for snr in [20.0, 10.0, 0.0] {
            let clean = sine(rate, hz, 0.1, length).unwrap();
            output.push(Case {
                name: format!("noise_{snr}db_{hz}"),
                expected_hz: Some(hz),
                pcm: with_noise(&clean, snr, NOISE_SEED ^ hz.to_bits()),
                config,
            });
        }
    }
    let mut dc = sine(rate, 110.0, 0.2, length).unwrap();
    for sample in &mut dc {
        *sample += 0.6;
    }
    output.push(Case {
        name: "dc_offset".into(),
        expected_hz: Some(110.0),
        pcm: dc,
        config,
    });
    let mut decay = sine(rate, 110.0, 0.5, length).unwrap();
    for (n, sample) in decay.iter_mut().enumerate() {
        *sample *= (0.01_f64.powf(n as f64 / (length - 1) as f64)) as f32;
    }
    output.push(Case {
        name: "decay".into(),
        expected_hz: Some(110.0),
        pcm: decay,
        config,
    });
    for (name, pcm) in [
        ("silence", vec![0.0; length]),
        ("constant_dc", vec![0.8; length]),
        ("sub_gate", sine(rate, 440.0, 0.00001, length).unwrap()),
        ("noise_only", white_noise(NOISE_SEED, 0.2, length).unwrap()),
    ] {
        output.push(Case {
            name: name.into(),
            expected_hz: None,
            pcm,
            config,
        });
    }
    output
}

pub fn hop(rate: u32) -> usize {
    (rate * HOP_MS / 1000) as usize
}

/// Scale centered noise to the requested whole-fixture SNR. No clipping or
/// automatic gain change; chosen clean amplitudes leave adequate headroom.
pub fn with_noise(clean: &[f32], snr_db: f64, seed: u64) -> Vec<f32> {
    let noise = white_noise(seed, 1.0, clean.len()).unwrap();
    let mean = |data: &[f32]| data.iter().map(|&x| f64::from(x)).sum::<f64>() / data.len() as f64;
    let signal_mean = mean(clean);
    let noise_mean = mean(&noise);
    let energy = |data: &[f32], m: f64| {
        data.iter()
            .map(|&x| (f64::from(x) - m).powi(2))
            .sum::<f64>()
    };
    let gain =
        (energy(clean, signal_mean) / energy(&noise, noise_mean) / 10_f64.powf(snr_db / 10.0))
            .sqrt();
    clean
        .iter()
        .zip(noise)
        .map(|(&x, n)| {
            let value = f64::from(x) + (f64::from(n) - noise_mean) * gain;
            assert!(value.abs() <= 1.0);
            value as f32
        })
        .collect()
}
