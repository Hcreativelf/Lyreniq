use std::f64::consts::TAU;
use tuner_core::{
    test_signal::{harmonic_tone, sine, Harmonic, MAX_FIXTURE_SAMPLES, MAX_HARMONICS},
    CoreError,
};

#[test]
fn sine_has_known_samples_and_repeats_deterministically() {
    let pcm = sine(16, 4.0, 0.5, 8).unwrap();
    for (sample, expected) in pcm.iter().zip([0.0, 0.5, 0.0, -0.5, 0.0, 0.5, 0.0, -0.5]) {
        assert!((sample - expected).abs() < 1e-6);
    }
    assert_eq!(pcm, sine(16, 4.0, 0.5, 8).unwrap());
    assert!(sine(48_000, 440.0, 0.0, 100)
        .unwrap()
        .iter()
        .all(|&x| x == 0.0));
    assert!(sine(48_000, 440.0, 0.5, 0).unwrap().is_empty());
}

#[test]
fn requested_sample_rates_have_the_expected_cycle_count() {
    for rate in [44_100, 48_000] {
        let pcm = sine(rate, 440.0, 0.8, (rate / 10) as usize).unwrap();
        assert_eq!(pcm.len(), (rate / 10) as usize);
        let positive_crossings = pcm
            .windows(2)
            .filter(|pair| pair[0] <= 0.0 && pair[1] > 0.0)
            .count();
        assert_eq!(positive_crossings, 44);
        assert!(pcm.iter().all(|&x| x.is_finite() && x.abs() <= 0.800001));
    }
}

#[test]
fn weak_fundamental_and_strong_harmonics_keep_the_requested_weights() {
    let harmonics = [
        Harmonic::new(1, 0.08).unwrap(),
        Harmonic::new(2, 0.6).unwrap(),
        Harmonic::new(3, 0.25).unwrap(),
    ];
    let pcm = harmonic_tone(48_000, 125.0, &harmonics, 3072).unwrap();
    assert_eq!(pcm, harmonic_tone(48_000, 125.0, &harmonics, 3072).unwrap());
    for h in harmonics {
        let projection = 2.0 / pcm.len() as f64
            * pcm
                .iter()
                .enumerate()
                .map(|(n, &x)| {
                    f64::from(x)
                        * (TAU * 125.0 * f64::from(h.multiple()) * n as f64 / 48_000.0).sin()
                })
                .sum::<f64>();
        assert!((projection - h.amplitude()).abs() < 1e-6);
    }
    let energy = pcm.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>() / pcm.len() as f64;
    let expected = (0.08_f64.powi(2) + 0.6_f64.powi(2) + 0.25_f64.powi(2)) / 2.0;
    assert!((energy - expected).abs() < 1e-7);
}

#[test]
fn generator_rejects_invalid_and_aliased_components() {
    for f in [0.0, -1.0, f64::NAN, f64::INFINITY, 24_000.0] {
        assert!(sine(48_000, f, 0.5, 16).is_err());
    }
    assert!(sine(0, 440.0, 0.5, 16).is_err());
    assert!(harmonic_tone(48_000, 9000.0, &[Harmonic::new(3, 0.5).unwrap()], 16).is_err());
    assert!(harmonic_tone(48_000, 440.0, &[], 16).is_err());
    for amplitude in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
        assert!(Harmonic::new(1, amplitude).is_err());
    }
    assert!(Harmonic::new(0, 0.5).is_err());
    assert!(harmonic_tone(
        48_000,
        440.0,
        &[
            Harmonic::new(1, 0.7).unwrap(),
            Harmonic::new(2, 0.7).unwrap()
        ],
        16
    )
    .is_err());
    assert!(harmonic_tone(
        48_000,
        440.0,
        &vec![Harmonic::new(1, 0.0).unwrap(); MAX_HARMONICS + 1],
        16
    )
    .is_err());
}

#[test]
fn fixture_size_is_bounded_before_allocation() {
    for count in [MAX_FIXTURE_SAMPLES + 1, usize::MAX] {
        assert_eq!(
            sine(48_000, 440.0, 0.5, count),
            Err(CoreError::FixtureTooLarge)
        );
    }
}
