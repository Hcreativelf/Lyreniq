use tuner_core::{FrequencyRange, PitchEstimate, PitchFrame, PitchStatus};

#[test]
fn detected_frames_preserve_source_metadata_and_valid_estimate() {
    let estimate = PitchEstimate::new(440.0, 0.95).unwrap();
    let frame = PitchFrame::new(
        42,
        0.125,
        vec![estimate],
        PitchStatus::Detected,
        Some(-18.0),
    )
    .unwrap();
    assert_eq!(frame.session_id(), 42);
    assert_eq!(frame.timestamp_s(), 0.125);
    assert_eq!(frame.status(), PitchStatus::Detected);
    assert_eq!(frame.pitches(), &[estimate]);
    assert_eq!(frame.pitches()[0].frequency_hz(), 440.0);
    assert_eq!(frame.pitches()[0].confidence(), 0.95);
    assert_eq!(frame.rms_dbfs(), Some(-18.0));
}

#[test]
fn inactive_states_cannot_retain_pitch_or_inactive_signal() {
    let pitch = PitchEstimate::new(440.0, 0.9).unwrap();
    for status in [
        PitchStatus::Idle,
        PitchStatus::Listening,
        PitchStatus::NoPitch,
        PitchStatus::LowConfidence,
        PitchStatus::Error,
    ] {
        assert!(PitchFrame::new(1, 0.0, vec![pitch], status, None).is_err());
        assert!(PitchFrame::new(1, 0.0, vec![], status, None).is_ok());
    }
    for status in [PitchStatus::Idle, PitchStatus::Error] {
        assert!(PitchFrame::new(1, 0.0, vec![], status, Some(-18.0)).is_err());
    }
    for status in [
        PitchStatus::Listening,
        PitchStatus::NoPitch,
        PitchStatus::LowConfidence,
    ] {
        assert!(PitchFrame::new(1, 0.0, vec![], status, Some(-90.0)).is_ok());
    }
    assert!(PitchFrame::new(1, 0.0, vec![], PitchStatus::Detected, None).is_err());
    assert!(PitchFrame::new(1, 0.0, vec![pitch, pitch], PitchStatus::Detected, None).is_err());
}

#[test]
fn non_finite_frame_values_and_confidence_are_rejected() {
    for f in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(PitchEstimate::new(f, 0.9).is_err());
    }
    for confidence in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
        assert!(PitchEstimate::new(440.0, confidence).is_err());
    }
    for confidence in [0.0, 1.0] {
        assert!(PitchEstimate::new(440.0, confidence).is_ok());
    }
    for timestamp in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(PitchFrame::new(0, timestamp, vec![], PitchStatus::NoPitch, None).is_err());
    }
    for rms in [1.0, f32::NAN, f32::NEG_INFINITY] {
        assert!(PitchFrame::new(0, 0.0, vec![], PitchStatus::Listening, Some(rms)).is_err());
    }
}

#[test]
fn frequency_ranges_require_order_and_actual_nyquist_compatibility() {
    let range = FrequencyRange::default();
    assert_eq!((range.min_hz(), range.max_hz()), (20.0, 5000.0));
    for rate in [44_100, 48_000] {
        assert!(range.validate_for_sample_rate(rate).is_ok());
    }
    for rate in [0, 8000, 10_000] {
        assert!(range.validate_for_sample_rate(rate).is_err());
    }
    for (min, max) in [
        (0.0, 5000.0),
        (100.0, 100.0),
        (200.0, 100.0),
        (f64::NAN, 5000.0),
        (20.0, f64::INFINITY),
    ] {
        assert!(FrequencyRange::new(min, max).is_err());
    }
}
