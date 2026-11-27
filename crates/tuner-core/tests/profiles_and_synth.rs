use tuner_core::{
    cents_difference, DetectionMode, FrequencyRange, InstrumentProfile, InstrumentProfileId,
    MidiNote, PitchEstimate, PitchStatus, ToneGenerator, ToneWaveform, TunerSnapshot,
    TunerSnapshotInit, TuningConfig, YinConfig, YinDetector, DEFAULT_SAFE_TONE_GAIN,
};

#[test]
fn builtin_instrument_profiles_cover_prd_targets_and_locking() {
    let all = InstrumentProfile::builtin_all();
    assert_eq!(all.len(), 10);

    let guitar = InstrumentProfile::builtin(InstrumentProfileId::GuitarStandard).unwrap();
    assert_eq!(guitar.targets().len(), 6);
    assert_eq!(guitar.detection_mode(), DetectionMode::Fast);

    let tuning = TuningConfig::default();
    // 82.41 Hz (E2) automatically matches lowest guitar string.
    let auto = tuning
        .tune(82.406889, guitar.target_selection(None))
        .unwrap();
    assert_eq!(auto.note_name(), "E2");
    assert_eq!(auto.pitch_class(), "E");
    assert_eq!(auto.octave(), 2);

    // Locking A2 (MIDI 45) measures unclamped cents even when playing E2.
    let locked = tuning
        .tune(
            82.406889,
            guitar.target_selection(Some(MidiNote::new(45).unwrap())),
        )
        .unwrap();
    assert_eq!(locked.note_name(), "A2");
    assert!((locked.cents() + 500.0).abs() < 0.05);

    // Custom profile validation.
    let custom = InstrumentProfile::custom(
        "DADGAD",
        [38, 45, 50, 55, 57, 62]
            .into_iter()
            .map(|n| MidiNote::new(n).unwrap())
            .collect(),
        FrequencyRange::new(60.0, 1200.0).unwrap(),
    )
    .unwrap();
    assert_eq!(custom.id(), InstrumentProfileId::Custom);
    assert!(InstrumentProfile::custom(
        " ",
        vec![MidiNote::new(69).unwrap()],
        FrequencyRange::default()
    )
    .is_err());
    assert!(InstrumentProfile::custom("Empty", vec![], FrequencyRange::default()).is_err());
}

#[test]
fn tone_generator_is_click_free_accurate_and_fades_to_silence() {
    let rate = 48_000_u32;
    let mut gen = ToneGenerator::new(rate, 440.0, DEFAULT_SAFE_TONE_GAIN).unwrap();
    assert!(gen.is_silent());

    // Start playing: fade-in must not have a sudden jump at sample 0.
    gen.start();
    let mut buf = vec![0.0_f32; 9_600]; // 200 ms
    gen.fill_buffer(&mut buf);
    assert!(buf[0].abs() < 0.01, "start must fade in smoothly");
    assert!(buf.iter().all(|s| s.is_finite() && s.abs() <= 1.0));

    // Steady tail must measure as 440.0 Hz within ±0.5 cent through YinDetector!
    let config = YinConfig::fast();
    let n = config.sample_count(rate).unwrap();
    let tail = &buf[buf.len() - n..];
    let det = YinDetector::new(config).detect(tail, rate).unwrap();
    assert_eq!(det.status(), PitchStatus::Detected);
    assert!(
        cents_difference(det.pitches()[0].frequency_hz(), 440.0)
            .unwrap()
            .abs()
            < 0.5
    );

    // Frequency change and soft-harmonic mode remain bounded and continuous.
    gen.set_waveform(ToneWaveform::SoftHarmonic);
    gen.set_frequency_hz(329.6275569).unwrap(); // E4
    let mut glide = vec![0.0_f32; 9_600];
    gen.fill_buffer(&mut glide);
    assert!(glide.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
    let det_e4 = YinDetector::new(config)
        .detect(&glide[glide.len() - n..], rate)
        .unwrap();
    assert_eq!(det_e4.status(), PitchStatus::Detected);
    assert!(
        cents_difference(det_e4.pitches()[0].frequency_hz(), 329.6275569)
            .unwrap()
            .abs()
            < 0.5
    );

    // Stop: fade-out reaches exact silence within 25 ms.
    gen.stop();
    let mut tail_out = vec![0.0_f32; 1_200];
    gen.fill_buffer(&mut tail_out);
    assert!(gen.is_silent());
    assert_eq!(*tail_out.last().unwrap(), 0.0);
}

#[test]
fn serde_contracts_round_trip_and_reject_invalid_midi() {
    let tuning = TuningConfig::default()
        .tune(442.0, tuner_core::TargetSelection::Chromatic)
        .unwrap();
    let snapshot = TunerSnapshot::new(TunerSnapshotInit {
        session_id: 7,
        sequence: 42,
        timestamp_s: 1.25,
        actual_sample_rate: 48_000,
        status: PitchStatus::Detected,
        raw_pitch: Some(PitchEstimate::new(442.0, 0.99).unwrap()),
        display_frequency_hz: Some(442.0),
        is_held: false,
        tuning: Some(tuning),
        rms_dbfs: Some(-14.2),
        dropped_frames: 0,
        reference_a4_hz: 440.0,
        tolerance_cents: 3.0,
    })
    .unwrap();

    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(json.contains("\"sessionId\":7"));
    assert!(json.contains("\"referenceA4Hz\":440.0"));
    assert!(json.contains("\"pitchClass\":\"A\""));
    let decoded: TunerSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, snapshot);

    // Out-of-range MIDI must fail deserialization.
    assert!(serde_json::from_str::<MidiNote>("128").is_err());
    assert!(serde_json::from_str::<MidiNote>("69").is_ok());

    // Inactive snapshot cannot carry stale pitch or display frequency.
    assert!(TunerSnapshot::new(TunerSnapshotInit {
        session_id: 1,
        sequence: 1,
        timestamp_s: 0.0,
        actual_sample_rate: 48_000,
        status: PitchStatus::Idle,
        raw_pitch: None,
        display_frequency_hz: Some(440.0),
        is_held: false,
        tuning: None,
        rms_dbfs: None,
        dropped_frames: 0,
        reference_a4_hz: 440.0,
        tolerance_cents: 3.0,
    })
    .is_err());
}
