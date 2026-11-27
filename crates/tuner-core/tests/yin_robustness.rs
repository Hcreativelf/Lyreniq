#[path = "support/cases.rs"]
mod cases;
use std::f64::consts::TAU;
use tuner_core::{
    cents_difference,
    test_signal::{sine, white_noise},
    FrequencyRange, MultiScalePitchDetector, PitchStatus, YinConfig, YinDetector,
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
fn inharmonic_string_with_weak_fundamental_avoids_octave_error() {
    // Stiff-string guitar/piano model: f_n = n * f0 * sqrt(1 + B * n^2) with weak fundamental.
    let f0 = 82.4068892282175_f64; // E2
    let stiffness = 0.00025_f64;
    let harmonics = [(1_u32, 0.08_f64), (2, 0.50), (3, 0.25), (4, 0.12)];
    for rate in [44_100_u32, 48_000] {
        let config = YinConfig::default();
        let n = config.sample_count(rate).unwrap();
        let pcm: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f64 / f64::from(rate);
                let sample: f64 = harmonics
                    .iter()
                    .map(|&(k, amp)| {
                        let fk = f64::from(k) * f0 * (1.0 + stiffness * f64::from(k * k)).sqrt();
                        amp * (TAU * fk * t).sin()
                    })
                    .sum();
                sample as f32
            })
            .collect();
        let result = YinDetector::new(config).detect(&pcm, rate).unwrap();
        assert_eq!(result.status(), PitchStatus::Detected);
        let err = cents_difference(result.pitches()[0].frequency_hz(), f0)
            .unwrap()
            .abs();
        assert!(
            err < 5.0,
            "inharmonic E2 @ {rate}: detected {:.3} Hz ({err:.2} cents)",
            result.pitches()[0].frequency_hz()
        );
    }
}

#[test]
fn multi_scale_detector_locks_fast_mid_high_and_decimated_bass() {
    for rate in [44_100_u32, 48_000] {
        let mut detector =
            MultiScalePitchDetector::new(FrequencyRange::new(19.5, 5000.0).unwrap(), 0.1, -60.0)
                .unwrap();
        let fast_n = detector.fast_window_samples(rate).unwrap();
        let full_n = detector.full_window_samples(rate).unwrap();
        assert!(fast_n < full_n / 2);

        // Mid/high tones must lock accurately using ONLY the fast ~85 ms window.
        for hz in [82.406889, 110.0, 261.625565, 440.0, 1760.0] {
            let pcm = sine(rate, hz, 0.5, fast_n).unwrap();
            let res = detector.detect(&pcm, rate).unwrap();
            assert_eq!(res.status(), PitchStatus::Detected, "fast {hz} @ {rate}");
            let err = cents_difference(res.pitches()[0].frequency_hz(), hz)
                .unwrap()
                .abs();
            assert!(err <= 1.0, "fast {hz} @ {rate}: {err:.4} cents");
        }

        // Low bass tones below 55 Hz must lock accurately via 4x decimated bass path.
        for hz in [20.0, 27.5, 30.867706, 41.203445] {
            let pcm = sine(rate, hz, 0.5, full_n).unwrap();
            let res = detector.detect(&pcm, rate).unwrap();
            assert_eq!(res.status(), PitchStatus::Detected, "bass {hz} @ {rate}");
            let err = cents_difference(res.pitches()[0].frequency_hz(), hz)
                .unwrap()
                .abs();
            assert!(err <= 1.0, "bass {hz} @ {rate}: {err:.4} cents");
        }
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
