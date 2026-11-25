use tuner_core::{
    cents_difference,
    test_signal::{sine, white_noise},
    CoreError, FrequencyRange, PitchDetection, PitchSmoother, YinConfig, YinDetector,
};

fn detection(hz: f64) -> PitchDetection {
    let config =
        YinConfig::new(FrequencyRange::new(100.0, 1000.0).unwrap(), 60, 0.1, -60.0).unwrap();
    let pcm = sine(48_000, hz, 0.5, config.sample_count(48_000).unwrap()).unwrap();
    YinDetector::new(config).detect(&pcm, 48_000).unwrap()
}

#[test]
fn median_reduces_jitter_without_modifying_raw_measurements() {
    let mut smoother = PitchSmoother::default();
    for (hz, expected) in [
        (440.0, 440.0),
        (441.0, 441.0),
        (439.0, 440.0),
        (440.0, 440.0),
    ] {
        let raw = detection(hz);
        let result = smoother.process(Ok(raw)).unwrap();
        assert_eq!(result.raw(), raw);
        assert!(
            cents_difference(result.display_frequency_hz().unwrap(), expected)
                .unwrap()
                .abs()
                < 0.01
        );
    }
}

#[test]
fn note_jumps_and_invalid_frames_reset_immediately() {
    let mut smoother = PitchSmoother::default();
    for hz in [440.0, 441.0, 439.0, 261.6255653] {
        let raw = detection(hz);
        let result = smoother.process(Ok(raw)).unwrap();
        if hz < 300.0 {
            assert_eq!(
                result.display_frequency_hz(),
                Some(raw.pitches()[0].frequency_hz())
            );
        }
    }
    let config = YinConfig::default();
    let n = config.sample_count(48_000).unwrap();
    for pcm in [
        vec![0.0; n],
        white_noise(7, 0.2, n).unwrap(),
        vec![f32::NAN; n],
    ] {
        let result = smoother.process(YinDetector::new(config).detect(&pcm, 48_000));
        if let Ok(result) = result {
            assert_eq!(result.display_frequency_hz(), None);
        }
        let raw = detection(442.0);
        assert_eq!(
            smoother.process(Ok(raw)).unwrap().display_frequency_hz(),
            Some(raw.pitches()[0].frequency_hz())
        );
    }
    assert!(smoother
        .process(Err(CoreError::InvalidValue("test error")))
        .is_err());
    let raw = detection(440.0);
    assert_eq!(
        smoother.process(Ok(raw)).unwrap().display_frequency_hz(),
        Some(raw.pitches()[0].frequency_hz())
    );
    smoother.reset();
    let raw = detection(439.0);
    assert_eq!(
        smoother.process(Ok(raw)).unwrap().display_frequency_hz(),
        Some(raw.pitches()[0].frequency_hz())
    );
}

#[test]
fn gentle_vibrato_stays_within_source_pitch_band() {
    let rate = 48_000;
    let config = YinConfig::default();
    let n = config.sample_count(rate).unwrap();
    let hop = 960;
    let mut phase = 0.0_f64;
    let pcm: Vec<f32> = (0..n + 9 * hop)
        .map(|sample| {
            let t = sample as f64 / f64::from(rate);
            let hz = 440.0 * 2_f64.powf(10.0 * (std::f64::consts::TAU * 2.0 * t).sin() / 1200.0);
            phase += std::f64::consts::TAU * hz / f64::from(rate);
            (phase.sin() * 0.5) as f32
        })
        .collect();
    let mut detector = YinDetector::new(config);
    let mut smoother = PitchSmoother::default();
    for frame in 0..10 {
        let result = smoother
            .process(detector.detect(&pcm[frame * hop..frame * hop + n], rate))
            .unwrap();
        let hz = result.display_frequency_hz().unwrap();
        assert!(cents_difference(hz, 440.0).unwrap().abs() <= 10.0);
    }
}
