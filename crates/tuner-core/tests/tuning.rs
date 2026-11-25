use tuner_core::{
    cents_difference, frequency_to_midi, midi_to_frequency, CoreError, MidiNote, NoteSpelling,
    TargetSelection, TuningConfig, TuningDirection,
};

fn note(n: i32) -> MidiNote {
    MidiNote::new(n).unwrap()
}
fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected} (tolerance {tolerance})"
    );
}

#[test]
fn standard_pitch_references_and_names() {
    for (midi, hz, name) in [
        (21, 27.5, "A0"),
        (60, 261.6255653005986, "C4"),
        (69, 440.0, "A4"),
        (0, 8.175798915643707, "C-1"),
        (127, 12543.853951415975, "G9"),
    ] {
        close(midi_to_frequency(f64::from(midi), 440.0).unwrap(), hz, 1e-9);
        let result = TuningConfig::default()
            .tune(hz, TargetSelection::Chromatic)
            .unwrap();
        assert_eq!(result.midi_note(), note(midi));
        assert_eq!(result.note_name(), name);
        close(result.cents(), 0.0, 1e-8);
    }
    assert_eq!(note(61).name(NoteSpelling::Sharps), "C#4");
    assert_eq!(note(61).name(NoteSpelling::Flats), "Db4");
    assert_eq!(note(70).name(NoteSpelling::Flats), "Bb4");
    assert_eq!(note(59).name(NoteSpelling::Sharps), "B3");
    assert_eq!(note(72).octave(), 5);
}

#[test]
fn all_midi_notes_round_trip_at_four_a4_references() {
    for a4 in [400.0, 440.0, 442.0, 480.0] {
        let config = TuningConfig::new(a4, 3.0, NoteSpelling::Sharps).unwrap();
        for n in 0..=127 {
            let hz = midi_to_frequency(f64::from(n), a4).unwrap();
            close(frequency_to_midi(hz, a4).unwrap(), f64::from(n), 1e-10);
            let result = config.tune(hz, TargetSelection::Chromatic).unwrap();
            assert_eq!(result.midi_note(), note(n));
            assert_eq!(result.direction(), TuningDirection::InTune);
        }
    }
}

#[test]
fn a4_calibration_changes_target_frequency_and_cents() {
    let measured = 442.0;
    let normal = TuningConfig::default()
        .tune(measured, TargetSelection::Locked(note(69)))
        .unwrap();
    close(normal.cents(), 7.851415040126504, 1e-8);
    assert_eq!(normal.target_hz(), 440.0);
    let calibrated = TuningConfig::new(442.0, 3.0, NoteSpelling::Flats)
        .unwrap()
        .tune(measured, TargetSelection::Chromatic)
        .unwrap();
    assert_eq!(calibrated.target_hz(), 442.0);
    close(calibrated.cents(), 0.0, 1e-8);
    assert_eq!(calibrated.note_name(), "A4");
}

#[test]
fn signed_cents_tolerance_and_unclamped_locked_target() {
    let cases = [
        (-75.0, TuningDirection::Flat),
        (-25.0, TuningDirection::Flat),
        (-5.0, TuningDirection::Flat),
        (-3.0, TuningDirection::InTune),
        (-1.0, TuningDirection::InTune),
        (0.0, TuningDirection::InTune),
        (1.0, TuningDirection::InTune),
        (3.0, TuningDirection::InTune),
        (5.0, TuningDirection::Sharp),
        (25.0, TuningDirection::Sharp),
        (75.0, TuningDirection::Sharp),
    ];
    for (cents, direction) in cases {
        let hz = 440.0 * (cents / 1200.0_f64).exp2();
        let result = TuningConfig::default()
            .tune(hz, TargetSelection::Locked(note(69)))
            .unwrap();
        close(result.cents(), cents, 1e-8);
        assert_eq!(result.direction(), direction);
        assert_eq!(result.midi_note(), note(69));
    }
    let hz = 440.0 * (75.0 / 1200.0_f64).exp2();
    let auto = TuningConfig::default()
        .tune(hz, TargetSelection::Chromatic)
        .unwrap();
    assert_eq!(auto.midi_note(), note(70));
    close(auto.cents(), -25.0, 1e-8);
}

#[test]
fn chromatic_half_step_ties_choose_the_upper_note_without_clipping_edges() {
    for (midi, expected) in [
        (69.5 - 1e-6, 69),
        (69.5, 70),
        (69.5 + 1e-6, 70),
        (-0.5, 0),
        (127.4999, 127),
    ] {
        let hz = 440.0 * ((midi - 69.0) / 12.0_f64).exp2();
        assert_eq!(
            TuningConfig::default()
                .tune(hz, TargetSelection::Chromatic)
                .unwrap()
                .midi_note(),
            note(expected)
        );
    }
    for midi in [-0.5001, 127.5] {
        let hz = 440.0 * ((midi - 69.0) / 12.0_f64).exp2();
        assert_eq!(
            TuningConfig::default().tune(hz, TargetSelection::Chromatic),
            Err(CoreError::MidiOutOfRange)
        );
    }
}

#[test]
fn target_lists_are_order_independent_and_ties_choose_upper_note() {
    let d4 = midi_to_frequency(62.0, 440.0).unwrap();
    for targets in [
        vec![note(60), note(64)],
        vec![note(64), note(60)],
        vec![note(64), note(60), note(64)],
    ] {
        let result = TuningConfig::default()
            .tune(d4, TargetSelection::Nearest(&targets))
            .unwrap();
        assert_eq!(result.midi_note(), note(64));
        close(result.cents(), -200.0, 1e-8);
        let low = TuningConfig::default()
            .tune(262.0, TargetSelection::Nearest(&targets))
            .unwrap();
        assert_eq!(low.midi_note(), note(60));
    }
    assert_eq!(
        TuningConfig::default().tune(440.0, TargetSelection::Nearest(&[])),
        Err(CoreError::EmptyTargets)
    );
}

#[test]
fn invalid_numbers_and_configuration_are_rejected() {
    for f in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(frequency_to_midi(f, 440.0).is_err());
        assert!(cents_difference(f, 440.0).is_err());
        assert!(cents_difference(440.0, f).is_err());
        assert!(TuningConfig::default()
            .tune(f, TargetSelection::Locked(note(69)))
            .is_err());
    }
    for reference in [0.0, 399.99, 480.01, f64::NAN, f64::INFINITY] {
        assert!(TuningConfig::new(reference, 3.0, NoteSpelling::Sharps).is_err());
        assert!(frequency_to_midi(440.0, reference).is_err());
        assert!(midi_to_frequency(69.0, reference).is_err());
    }
    for tolerance in [-1.0, 50.01, f64::NAN, f64::INFINITY] {
        assert!(TuningConfig::new(440.0, tolerance, NoteSpelling::Sharps).is_err());
    }
    for tolerance in [0.0, 50.0] {
        assert!(TuningConfig::new(440.0, tolerance, NoteSpelling::Sharps).is_ok());
    }
    for midi in [-1.0, 128.0, f64::NAN, f64::INFINITY] {
        assert!(midi_to_frequency(midi, 440.0).is_err());
    }
    for midi in [-1, 128, i32::MIN, i32::MAX] {
        assert!(MidiNote::new(midi).is_err());
    }
}

#[test]
fn extreme_positive_frequencies_do_not_overflow_the_ratio() {
    let smallest = f64::from_bits(1);
    let cents = cents_difference(f64::MAX, smallest).unwrap();
    assert!(cents.is_finite() && cents > 0.0);
    assert_eq!(cents_difference(smallest, f64::MAX).unwrap(), -cents);
    for f in [smallest, f64::MAX] {
        assert!(frequency_to_midi(f, 440.0).unwrap().is_finite());
        assert_eq!(
            TuningConfig::default().tune(f, TargetSelection::Chromatic),
            Err(CoreError::MidiOutOfRange)
        );
    }
}
