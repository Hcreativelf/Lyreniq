use tuner_core::{
    test_signal::{harmonic_tone, sine, Harmonic},
    SpectrumAnalyzer, SpectrumConfig,
};

#[test]
fn exact_bin_sine_reports_calibrated_dbfs_and_frequency() {
    let rate = 48_000_u32;
    let fft_size = 4096;
    let bin = 40; // 40 * 48000 / 4096 = 468.75 Hz (exact integer bin)
    let hz = f64::from(rate) * bin as f64 / fft_size as f64;
    let mut analyzer = SpectrumAnalyzer::new(SpectrumConfig::new(fft_size, -100.0).unwrap());

    // Amplitude 0.5 -> -6.0206 dBFS coherent peak gain.
    let pcm = sine(rate, hz, 0.5, fft_size).unwrap();
    let frame = analyzer.analyze(&pcm, rate).unwrap();
    assert_eq!(frame.magnitudes_dbfs().len(), fft_size / 2 + 1);
    assert!((frame.frequency_at_bin(bin) - hz).abs() < 1e-9);
    assert!((frame.peak_frequency_hz().unwrap() - hz).abs() < 0.05);
    let expected_db = 20.0 * 0.5_f64.log10();
    assert!((f64::from(frame.magnitudes_dbfs()[bin]) - expected_db).abs() < 0.05);
    assert!(frame
        .magnitudes_dbfs()
        .iter()
        .all(|v| v.is_finite() && *v <= 0.0));
}

#[test]
fn off_bin_sine_and_two_tones_resolve_cleanly() {
    let rate = 44_100_u32;
    let mut analyzer = SpectrumAnalyzer::new(SpectrumConfig::new(4096, -100.0).unwrap());

    // Off-bin A4 = 440 Hz (bin ~40.867).
    let a4 = sine(rate, 440.0, 0.5, 4096).unwrap();
    let frame = analyzer.analyze(&a4, rate).unwrap();
    let peak = frame.peak_frequency_hz().unwrap();
    assert!(
        (peak - 440.0).abs() < 1.5,
        "off-bin peak {peak} Hz too far from 440 Hz"
    );

    // Two distinct tones: 440 Hz and 1000 Hz.
    let t1 = sine(rate, 440.0, 0.35, 4096).unwrap();
    let t2 = sine(rate, 1000.0, 0.35, 4096).unwrap();
    let mix: Vec<f32> = t1.iter().zip(t2).map(|(&a, b)| a + b).collect();
    let mix_frame = analyzer.analyze(&mix, rate).unwrap();
    let db_440 = mix_frame.magnitude_at_hz(440.0).unwrap();
    let db_1000 = mix_frame.magnitude_at_hz(1000.0).unwrap();
    let db_700 = mix_frame.magnitude_at_hz(700.0).unwrap();
    assert!(db_440 > -15.0 && db_1000 > -15.0);
    assert!(db_700 < db_440 - 30.0);
}

#[test]
fn harmonic_peaks_and_log_binned_spectrum_work_with_dc_removal() {
    let rate = 48_000_u32;
    let mut analyzer = SpectrumAnalyzer::default();
    let harmonics = [
        Harmonic::new(1, 0.3).unwrap(),
        Harmonic::new(2, 0.2).unwrap(),
        Harmonic::new(3, 0.1).unwrap(),
    ];
    let mut pcm = harmonic_tone(rate, 220.0, &harmonics, 4096).unwrap();
    for x in &mut pcm {
        *x += 0.35; // DC bias must not leak into low bins.
    }
    let frame = analyzer.analyze(&pcm, rate).unwrap();
    assert!(frame.magnitudes_dbfs()[0] < -60.0);

    let peaks = frame.harmonic_peaks(220.0, 4, -40.0);
    assert_eq!(peaks.len(), 3);
    assert_eq!(peaks[0].multiple, 1);
    assert_eq!(peaks[1].multiple, 2);
    assert_eq!(peaks[2].multiple, 3);
    assert!((peaks[0].measured_hz - 220.0).abs() < 2.0);
    assert!((peaks[1].measured_hz - 440.0).abs() < 2.0);
    assert!((peaks[2].measured_hz - 660.0).abs() < 2.0);

    let log_spec = frame.log_binned(20.0, 8000.0, 128).unwrap();
    assert_eq!(log_spec.bins_dbfs.len(), 128);
    assert!(log_spec
        .bins_dbfs
        .iter()
        .all(|v| v.is_finite() && *v >= -100.0 && *v <= 0.0));
    let max_log = log_spec
        .bins_dbfs
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(max_log > -15.0);
}

#[test]
fn silence_short_slices_and_invalid_inputs_are_handled_safely() {
    assert!(SpectrumConfig::new(3000, -100.0).is_err());
    assert!(SpectrumConfig::new(256, -100.0).is_err());
    assert!(SpectrumConfig::new(4096, -10.0).is_err());

    let mut analyzer = SpectrumAnalyzer::default();
    let silent = analyzer.analyze(&vec![0.0; 4096], 48_000).unwrap();
    assert_eq!(silent.peak_frequency_hz(), None);
    assert_eq!(silent.peak_dbfs(), None);
    assert!(silent.magnitudes_dbfs().iter().all(|&v| v == -100.0));

    let empty = analyzer.analyze(&[], 48_000).unwrap();
    assert_eq!(empty.peak_frequency_hz(), None);

    // Zero-padded short slice still keeps calibrated amplitude.
    let short_tone = sine(48_000, 468.75, 0.5, 2048).unwrap();
    let short_frame = analyzer.analyze(&short_tone, 48_000).unwrap();
    assert!((short_frame.peak_frequency_hz().unwrap() - 468.75).abs() < 1.5);
    assert!((short_frame.peak_dbfs().unwrap() - (-6.02)).abs() < 1.0);

    // Invalid inputs rejected.
    assert!(analyzer.analyze(&[0.0], 0).is_err());
    assert!(analyzer.analyze(&[f32::NAN], 48_000).is_err());
    assert!(analyzer.analyze(&[1.2], 48_000).is_err());
}
