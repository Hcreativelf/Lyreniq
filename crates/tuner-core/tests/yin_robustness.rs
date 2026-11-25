#[path = "support/cases.rs"]
mod cases;
use tuner_core::{
    cents_difference,
    test_signal::{sine, white_noise},
    FrequencyRange, PitchStatus, YinConfig, YinDetector,
};

#[test]
fn fixed_matrix_handles_harmonics_noise_decay_dc_and_silence() {
    for rate in [44_100, 48_000] {
        for case in cases::cases(rate) {
            let mut detector = YinDetector::new(case.config);
            let n = case.config.sample_count(rate).unwrap();
            let must_detect = case.expected_hz.is_some()
                && !case.name.starts_with("noise_0db")
                && !case.name.starts_with("noise_10db");
            for frame in 0..cases::FRAMES {
                let start = frame * cases::hop(rate);
                let result = detector.detect(&case.pcm[start..start + n], rate).unwrap();
                if must_detect {
                    assert_eq!(
                        result.status(),
                        PitchStatus::Detected,
                        "{} / {rate} / {frame}",
                        case.name
                    );
                }
                if case.expected_hz.is_none() || case.name.starts_with("noise_0db") {
                    assert!(
                        result.pitches().is_empty(),
                        "{} / {rate} / {frame}",
                        case.name
                    );
                }
                if let (Some(expected), Some(pitch)) = (case.expected_hz, result.pitches().first())
                {
                    let error = cents_difference(pitch.frequency_hz(), expected)
                        .unwrap()
                        .abs();
                    let limit = if case.name.starts_with("noise_20db") {
                        5.0
                    } else if case.name.starts_with("noise_10db") {
                        10.0
                    } else if case.name == "decay" {
                        5.0
                    } else {
                        1.0
                    };
                    assert!(
                        error <= limit,
                        "{} / {rate} / {frame}: {error} cents > {limit}",
                        case.name
                    );
                }
                assert!(result.pitches().len() <= 1);
                assert!(result
                    .rms_dbfs()
                    .is_none_or(|rms| rms.is_finite() && rms <= 0.0));
            }
        }
    }
}

#[test]
fn reference_reproduces_recorded_t06_frequency_values() {
    let csv = include_str!("../../../docs/evidence/T06/pure-tone-errors.csv");
    for row in csv.lines().skip(1) {
        let fields: Vec<_> = row.split(',').collect();
        let rate = fields[0].parse::<u32>().unwrap();
        let hz = fields[1].parse::<f64>().unwrap();
        let recorded = fields[2].parse::<f64>().unwrap();
        let config = if fields[5] == "required" {
            YinConfig::default()
        } else {
            YinConfig::new(FrequencyRange::new(19.9, 5100.0).unwrap(), 300, 0.1, -60.0).unwrap()
        };
        let pcm = sine(rate, hz, 0.5, config.sample_count(rate).unwrap()).unwrap();
        let result = YinDetector::new(config)
            .detect_reference(&pcm, rate)
            .unwrap();
        assert!((result.pitches()[0].frequency_hz() - recorded).abs() < 2e-9);
    }
}

#[test]
fn low_tones_work_with_explicit_100_200_300_ms_windows() {
    for rate in [44_100, 48_000] {
        for ms in [100, 200, 300] {
            let config =
                YinConfig::new(FrequencyRange::new(25.0, 5000.0).unwrap(), ms, 0.1, -60.0).unwrap();
            for hz in [27.5, 55.0] {
                let pcm = sine(rate, hz, 0.5, config.sample_count(rate).unwrap()).unwrap();
                let result = YinDetector::new(config).detect(&pcm, rate).unwrap();
                assert_eq!(result.status(), PitchStatus::Detected);
                assert!(
                    cents_difference(result.pitches()[0].frequency_hz(), hz)
                        .unwrap()
                        .abs()
                        <= 1.0
                );
            }
        }
    }
}

#[test]
fn seeded_noise_is_bounded_repeatable_and_has_expected_rms() {
    let pcm = white_noise(cases::NOISE_SEED, 0.2, 48_000).unwrap();
    assert_eq!(pcm, white_noise(cases::NOISE_SEED, 0.2, 48_000).unwrap());
    assert_ne!(
        pcm,
        white_noise(cases::NOISE_SEED + 1, 0.2, 48_000).unwrap()
    );
    assert!(pcm.iter().all(|x| x.is_finite() && x.abs() <= 0.2));
    let mean = pcm.iter().map(|&x| f64::from(x)).sum::<f64>() / pcm.len() as f64;
    let rms = (pcm.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>() / pcm.len() as f64).sqrt();
    assert!(mean.abs() < 0.002);
    assert!((rms - 0.2 / 3_f64.sqrt()).abs() < 0.002);
    assert!(white_noise(0, f64::NAN, 10).is_err());
    assert!(white_noise(0, 1.01, 10).is_err());
    assert!(white_noise(0, 0.5, 480_001).is_err());
    assert_eq!(white_noise(0, 0.0, 3).unwrap(), vec![0.0; 3]);
}

#[test]
fn fixture_noise_has_measured_snr_without_clipping() {
    let clean = sine(48_000, 110.0, 0.1, 14_400).unwrap();
    let mean = clean.iter().map(|&x| f64::from(x)).sum::<f64>() / clean.len() as f64;
    let signal = clean
        .iter()
        .map(|&x| (f64::from(x) - mean).powi(2))
        .sum::<f64>();
    for snr in [20.0, 10.0, 0.0] {
        let mixed = cases::with_noise(&clean, snr, cases::NOISE_SEED);
        let noise = mixed
            .iter()
            .zip(&clean)
            .map(|(&a, &b)| (f64::from(a) - f64::from(b)).powi(2))
            .sum::<f64>();
        assert!((10.0 * (signal / noise).log10() - snr).abs() < 0.001);
        assert!(mixed.iter().all(|x| x.abs() <= 1.0));
    }
}
