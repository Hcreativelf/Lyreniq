use tuner_core::{
    cents_difference,
    test_signal::{sine, white_noise},
    CoreError, FrequencyRange, PitchDetection, PitchStatus, PitchTracker, PitchTrackerConfig,
    YinConfig, YinDetector,
};

fn detection(hz: f64) -> PitchDetection {
    let config =
        YinConfig::new(FrequencyRange::new(100.0, 1000.0).unwrap(), 60, 0.1, -60.0).unwrap();
    let pcm = sine(48_000, hz, 0.5, config.sample_count(48_000).unwrap()).unwrap();
    YinDetector::new(config).detect(&pcm, 48_000).unwrap()
}

#[test]
fn strict_median_reduces_jitter_without_modifying_raw_measurements() {
    let mut tracker = PitchTracker::new(PitchTrackerConfig::strict_median()).unwrap();
    for (hz, expected) in [
        (440.0, 440.0),
        (441.0, 441.0),
        (439.0, 440.0),
        (440.0, 440.0),
    ] {
        let raw = detection(hz);
        let result = tracker.process(Ok(raw)).unwrap();
        assert_eq!(result.raw(), raw);
        assert!(!result.is_held());
        assert!(
            cents_difference(result.display_frequency_hz().unwrap(), expected)
                .unwrap()
                .abs()
                < 0.01
        );
    }
}

#[test]
fn note_jumps_silence_and_errors_reset_immediately() {
    let mut tracker = PitchTracker::default();
    for hz in [440.0, 440.2, 439.8, 261.6255653] {
        let raw = detection(hz);
        let result = tracker.process(Ok(raw)).unwrap();
        if hz < 300.0 {
            assert_eq!(
                result.display_frequency_hz(),
                Some(raw.pitches()[0].frequency_hz())
            );
            assert!(!result.is_held());
        }
    }

    // Silence (NoPitch) must clear immediately even when hold_frames > 0.
    let silence = PitchDetection::without_pitch(PitchStatus::NoPitch, None);
    let after_silence = tracker.process(Ok(silence)).unwrap();
    assert_eq!(after_silence.display_frequency_hz(), None);
    assert!(!after_silence.is_held());

    // Errors always reset.
    let raw = detection(442.0);
    assert!(tracker
        .process(Ok(raw))
        .unwrap()
        .display_frequency_hz()
        .is_some());
    assert!(tracker
        .process(Err(CoreError::InvalidValue("test error")))
        .is_err());
    let fresh = detection(440.0);
    assert_eq!(
        tracker.process(Ok(fresh)).unwrap().display_frequency_hz(),
        Some(fresh.pitches()[0].frequency_hz())
    );

    tracker.reset();
    let after_reset = detection(439.0);
    assert_eq!(
        tracker
            .process(Ok(after_reset))
            .unwrap()
            .display_frequency_hz(),
        Some(after_reset.pitches()[0].frequency_hz())
    );
}

#[test]
fn transient_low_confidence_holds_briefly_then_clears() {
    let mut tracker = PitchTracker::default(); // hold_frames = 2
    let locked = detection(440.0);
    let first = tracker.process(Ok(locked)).unwrap();
    let expected_hz = first.display_frequency_hz().unwrap();

    let uncertain = PitchDetection::without_pitch(PitchStatus::LowConfidence, Some(-25.0));
    // Hold frames 1 and 2 keep display pitch while exposing true LowConfidence raw status.
    for _ in 0..2 {
        let held = tracker.process(Ok(uncertain)).unwrap();
        assert_eq!(held.raw().status(), PitchStatus::LowConfidence);
        assert!(held.is_held());
        assert!((held.display_frequency_hz().unwrap() - expected_hz).abs() < 1e-9);
    }
    // Third consecutive LowConfidence clears display.
    let cleared = tracker.process(Ok(uncertain)).unwrap();
    assert_eq!(cleared.display_frequency_hz(), None);
    assert!(!cleared.is_held());

    // Noise window also produces LowConfidence and clears after hold expires.
    let config = YinConfig::default();
    let n = config.sample_count(48_000).unwrap();
    let noise = white_noise(7, 0.2, n).unwrap();
    let noise_det = YinDetector::new(config).detect(&noise, 48_000).unwrap();
    assert_eq!(
        tracker
            .process(Ok(noise_det))
            .unwrap()
            .display_frequency_hz(),
        None
    );
}

#[test]
fn adaptive_ema_damps_tiny_jitter_and_tracks_vibrato() {
    let mut tracker = PitchTracker::default();
    // Seed at 440 Hz, then alternate ±1 cent jitter: display deviation must stay smaller than raw.
    let base = 440.0;
    tracker.process(Ok(detection(base))).unwrap();
    tracker.process(Ok(detection(base))).unwrap();
    let mut max_display_dev = 0.0_f64;
    for step in 0..8 {
        let offset = if step % 2 == 0 { 1.5 } else { -1.5 };
        let hz = base * (offset / 1200.0_f64).exp2();
        let tracked = tracker.process(Ok(detection(hz))).unwrap();
        let dev = cents_difference(tracked.display_frequency_hz().unwrap(), base)
            .unwrap()
            .abs();
        max_display_dev = max_display_dev.max(dev);
    }
    assert!(max_display_dev < 1.5);

    // Gentle vibrato stays within the source pitch band.
    let rate = 48_000;
    let config = YinConfig::fast();
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
    tracker.reset();
    for frame in 0..10 {
        let result = tracker
            .process(detector.detect(&pcm[frame * hop..frame * hop + n], rate))
            .unwrap();
        let hz = result.display_frequency_hz().unwrap();
        assert!(cents_difference(hz, 440.0).unwrap().abs() <= 10.5);
    }
}

#[test]
fn invalid_tracker_configs_are_rejected() {
    assert!(PitchTracker::new(PitchTrackerConfig {
        jump_reset_cents: 5.0,
        ..PitchTrackerConfig::default()
    })
    .is_err());
    assert!(PitchTracker::new(PitchTrackerConfig {
        alpha_stable: 0.8,
        alpha_fast: 0.4,
        ..PitchTrackerConfig::default()
    })
    .is_err());
    assert!(PitchTracker::new(PitchTrackerConfig {
        hold_frames: 11,
        ..PitchTrackerConfig::default()
    })
    .is_err());
}
