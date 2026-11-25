use tuner_core::{
    cents_difference, test_signal::sine, FrequencyRange, PitchStatus, YinConfig, YinDetector,
};

fn tone(config: YinConfig, rate: u32, hz: f64, amplitude: f64) -> Vec<f32> {
    sine(rate, hz, amplitude, config.sample_count(rate).unwrap()).unwrap()
}

#[test]
fn stable_pure_tones_meet_one_cent_at_both_actual_sample_rates() {
    let mut detector = YinDetector::default();
    // Required T06 notes, the plan's interior matrix, and off-note frequencies.
    let frequencies = [
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
    ];
    for rate in [44_100, 48_000] {
        for hz in frequencies {
            let pcm = tone(detector.config(), rate, hz, 0.5);
            let result = detector.detect(&pcm, rate).unwrap();
            assert_eq!(
                result.status(),
                PitchStatus::Detected,
                "{rate} Hz / {hz} Hz"
            );
            assert_eq!(result.pitches().len(), 1);
            let pitch = result.pitches()[0];
            let error = cents_difference(pitch.frequency_hz(), hz).unwrap();
            println!(
                "YIN,{rate},{hz:.9},{:.9},{error:.9},{:.6},required",
                pitch.frequency_hz(),
                pitch.confidence()
            );
            assert!(error.abs() <= 1.0, "{rate} Hz / {hz} Hz: {error} cents");
            assert!(pitch.confidence() >= 0.99);
            assert!(result.rms_dbfs().unwrap() <= 0.0);
        }
    }
}

#[test]
fn silence_dc_and_incomplete_windows_clear_previous_estimates() {
    let mut detector = YinDetector::default();
    let config = detector.config();
    let rate = 48_000;
    let n = config.sample_count(rate).unwrap();
    let pcm = tone(config, rate, 440.0, 0.5);
    for empty in [vec![], vec![0.0; n - 1], vec![0.0; n], vec![0.75; n]] {
        assert_eq!(detector.detect(&pcm, rate).unwrap().pitches().len(), 1);
        let result = detector.detect(&empty, rate).unwrap();
        assert_eq!(result.status(), PitchStatus::NoPitch);
        assert!(result.pitches().is_empty());
        assert_eq!(result.rms_dbfs(), None);
    }
}

#[test]
fn energy_gate_uses_dc_removed_rms_and_configured_dbfs() {
    let config =
        YinConfig::new(FrequencyRange::new(100.0, 1000.0).unwrap(), 100, 0.1, -30.0).unwrap();
    let mut detector = YinDetector::new(config);
    let rate = 48_000;
    let quiet = tone(config, rate, 440.0, 0.01);
    let result = detector.detect(&quiet, rate).unwrap();
    assert_eq!(result.status(), PitchStatus::NoPitch);
    assert!(result.pitches().is_empty());
    assert!((result.rms_dbfs().unwrap() + 43.0103).abs() < 0.001);
    let loud = tone(config, rate, 440.0, 0.1);
    assert_eq!(
        detector.detect(&loud, rate).unwrap().status(),
        PitchStatus::Detected
    );
    // A DC level must not make a sub-threshold alternating component audible.
    let biased: Vec<_> = quiet.iter().map(|&x| x + 0.5).collect();
    assert_eq!(
        detector.detect(&biased, rate).unwrap().status(),
        PitchStatus::NoPitch
    );
}

#[test]
fn rate_is_never_assumed_and_buffers_can_change_size() {
    let mut detector = YinDetector::default();
    let config = detector.config();
    let pcm = tone(config, 48_000, 440.0, 0.5);
    let shorter = &pcm[..config.sample_count(44_100).unwrap()];
    // Deliberately supply a different clock: measured physical Hz must scale.
    let wrong_clock = detector.detect(shorter, 44_100).unwrap().pitches()[0].frequency_hz();
    assert!(
        cents_difference(wrong_clock, 440.0 * 44_100.0 / 48_000.0)
            .unwrap()
            .abs()
            < 1.0
    );
    let right_clock = detector.detect(&pcm, 48_000).unwrap().pitches()[0].frequency_hz();
    assert!(cents_difference(right_clock, 440.0).unwrap().abs() < 1.0);
}

#[test]
fn narrow_range_includes_fractional_lags_and_rejects_unqualified_valleys() {
    let config =
        YinConfig::new(FrequencyRange::new(464.9, 465.1).unwrap(), 60, 0.1, -60.0).unwrap();
    let mut detector = YinDetector::new(config);
    for rate in [44_100, 48_000] {
        let pcm = tone(config, rate, 465.0, 0.5);
        let result = detector.detect(&pcm, rate).unwrap();
        assert_eq!(result.status(), PitchStatus::Detected);
        assert!((result.pitches()[0].frequency_hz() - 465.0).abs() < 0.1);
        let outside = tone(config, rate, 250.0, 0.5);
        let result = detector.detect(&outside, rate).unwrap();
        assert_eq!(result.status(), PitchStatus::LowConfidence);
        assert!(result.pitches().is_empty());
        assert!(result.rms_dbfs().is_some());
    }
}

#[test]
fn invalid_configuration_and_unsupported_geometry_fail_before_analysis() {
    let range = FrequencyRange::default();
    for window in [0, 59, 301, u32::MAX] {
        assert!(YinConfig::new(range, window, 0.1, -60.0).is_err());
    }
    for threshold in [f64::NAN, f64::INFINITY, -0.1, 0.0, 1.0] {
        assert!(YinConfig::new(range, 300, threshold, -60.0).is_err());
    }
    for gate in [f64::NAN, f64::NEG_INFINITY, -160.1, 0.1] {
        assert!(YinConfig::new(range, 300, 0.1, gate).is_err());
    }
    let config = YinConfig::default();
    assert_eq!(config.sample_count(44_100).unwrap(), 13_230);
    assert_eq!(config.sample_count(48_000).unwrap(), 14_400);
    for rate in [0, 8000, 10_000, u32::MAX] {
        assert!(config.sample_count(rate).is_err());
    }
    let short = YinConfig::new(range, 60, 0.1, -60.0).unwrap();
    assert!(short.sample_count(48_000).is_err());
    let tiny_min = YinConfig::new(
        FrequencyRange::new(f64::MIN_POSITIVE, 1000.0).unwrap(),
        300,
        0.1,
        -60.0,
    )
    .unwrap();
    assert!(tiny_min.sample_count(48_000).is_err());
    // Bound is per sample count, not an arbitrary assumption of 44.1/48 kHz.
    assert_eq!(config.sample_count(192_000).unwrap(), 57_600);
}

#[test]
fn malformed_pcm_is_rejected_including_incomplete_windows() {
    let mut detector = YinDetector::default();
    let n = detector.config().sample_count(48_000).unwrap();
    for invalid in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        1.01,
        -1.01,
        f32::MAX,
    ] {
        assert!(detector.detect(&[invalid], 48_000).is_err());
        let mut pcm = vec![0.0; n];
        pcm[n - 1] = invalid;
        assert!(detector.detect(&pcm, 48_000).is_err());
    }
    assert!(detector.detect(&vec![0.0; n + 1], 48_000).is_err());
    assert!(detector.detect(&[], 0).is_err());
    // A failed call cannot poison the following valid window.
    let pcm = tone(detector.config(), 48_000, 442.0, 0.5);
    assert_eq!(
        detector.detect(&pcm, 48_000).unwrap().status(),
        PitchStatus::Detected
    );
}

#[test]
fn boundary_characterization_never_clamps_or_returns_outside_configured_range() {
    // Slightly wider limits expose raw endpoint error rather than hiding it
    // behind range rejection. These rows characterize, not certify, ±1 cent.
    let config =
        YinConfig::new(FrequencyRange::new(19.9, 5100.0).unwrap(), 300, 0.1, -60.0).unwrap();
    let mut detector = YinDetector::new(config);
    for rate in [44_100, 48_000] {
        for hz in [20.0, 5000.0] {
            let pcm = tone(config, rate, hz, 0.5);
            let result = detector.detect(&pcm, rate).unwrap();
            assert_eq!(result.status(), PitchStatus::Detected);
            let pitch = result.pitches()[0];
            let error = cents_difference(pitch.frequency_hz(), hz).unwrap();
            println!(
                "YIN,{rate},{hz:.9},{:.9},{error:.9},{:.6},boundary_characterization",
                pitch.frequency_hz(),
                pitch.confidence()
            );
            assert!((19.9..=5100.0).contains(&pitch.frequency_hz()));
            // The default strict range must never clamp that estimate to a bound.
            let mut strict = YinDetector::default();
            let bounded = strict.detect(&pcm, rate).unwrap();
            println!("YIN_BOUNDARY_DEFAULT,{rate},{hz:.9},{:?}", bounded.status());
            if let Some(estimate) = bounded.pitches().first() {
                assert!((20.0..=5000.0).contains(&estimate.frequency_hz()));
                assert!(
                    cents_difference(estimate.frequency_hz(), pitch.frequency_hz())
                        .unwrap()
                        .abs()
                        < 0.1
                );
            }
        }
    }
}
