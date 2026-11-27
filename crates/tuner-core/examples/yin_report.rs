//! Deterministic DSP 2.0 robustness CSV plus warm per-frame wall-clock benchmarks.
//! cargo run -p tuner-core --example yin_report --release --locked
#[path = "../tests/support/cases.rs"]
mod cases;
use std::{hint::black_box, time::Instant};
use tuner_core::{
    cents_difference,
    test_signal::{harmonic_tone, sine, white_noise, Harmonic},
    FrequencyRange, MultiScalePitchDetector, SpectrumAnalyzer, YinConfig, YinDetector,
};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let benchmark_only = args.iter().any(|arg| arg == "--benchmark-only");

    if !benchmark_only {
        println!(
            "FRAME,engine,rate_hz,case,frame,expected_hz,status,measured_hz,error_cents,confidence"
        );
        for rate in [44_100, 48_000] {
            for case in cases::cases(rate) {
                let mut detector = YinDetector::new(case.config);
                let n = case.config.sample_count(rate).unwrap();
                for frame in 0..cases::FRAMES {
                    let start = frame * cases::hop(rate);
                    let input = &case.pcm[start..start + n];
                    let result = detector.detect(input, rate).unwrap();
                    let expected = case
                        .expected_hz
                        .map_or(String::new(), |hz| format!("{hz:.9}"));
                    let (hz, error, confidence) = result.pitches().first().map_or(
                        (String::new(), String::new(), String::new()),
                        |pitch| {
                            (
                                format!("{:.9}", pitch.frequency_hz()),
                                case.expected_hz.map_or(String::new(), |target| {
                                    format!(
                                        "{:.9}",
                                        cents_difference(pitch.frequency_hz(), target).unwrap()
                                    )
                                }),
                                format!("{:.6}", pitch.confidence()),
                            )
                        },
                    );
                    println!(
                        "FRAME,DSP2_FastYIN,{rate},{},{frame},{expected},{:?},{hz},{error},{confidence}",
                        case.name,
                        result.status()
                    );
                }
            }
        }
    }

    println!("BENCH,engine,rate_hz,case,iterations,p50_ms,p95_ms,max_ms");
    for rate in [44_100, 48_000] {
        let config = YinConfig::default();
        let n = config.sample_count(rate).unwrap();
        let scenarios = [
            ("low_20", sine(rate, 20.0, 0.5, n).unwrap()),
            ("low_27.5", sine(rate, 27.5, 0.5, n).unwrap()),
            ("mid_440", sine(rate, 440.0, 0.5, n).unwrap()),
            (
                "dominant_second",
                harmonic_tone(
                    rate,
                    110.0,
                    &[
                        Harmonic::new(1, 0.05).unwrap(),
                        Harmonic::new(2, 0.8).unwrap(),
                    ],
                    n,
                )
                .unwrap(),
            ),
            (
                "noise_only",
                white_noise(cases::NOISE_SEED, 0.2, n).unwrap(),
            ),
        ];

        for (name, pcm) in &scenarios {
            let mut detector = YinDetector::new(config);
            bench_line("DSP2_FastYIN_300ms", rate, name, || {
                black_box(detector.detect(black_box(pcm), rate).unwrap());
            });

            let mut multi = MultiScalePitchDetector::new(
                FrequencyRange::new(19.5, 5000.0).unwrap(),
                0.1,
                -60.0,
            )
            .unwrap();
            let check = multi.detect(pcm, rate).unwrap();
            if *name != "noise_only" {
                assert_eq!(
                    check.pitches().len(),
                    1,
                    "MultiScale must detect {name} @ {rate}"
                );
            }
            bench_line("DSP2_MultiScale", rate, name, || {
                black_box(multi.detect(black_box(pcm), rate).unwrap());
            });
        }

        let mut spectrum = SpectrumAnalyzer::default();
        let spec_pcm = sine(rate, 440.0, 0.5, 4096).unwrap();
        bench_line("DSP2_Spectrum_4096", rate, "mid_440", || {
            black_box(spectrum.analyze(black_box(&spec_pcm), rate).unwrap());
        });
    }
}

fn bench_line(engine: &str, rate: u32, name: &str, mut f: impl FnMut()) {
    let mut times = Vec::with_capacity(64);
    for iteration in 0..67 {
        let start = Instant::now();
        f();
        if iteration >= 3 {
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    times.sort_by(f64::total_cmp);
    let quantile = |q: f64| times[((times.len() as f64 * q).ceil() as usize) - 1];
    println!(
        "BENCH,{engine},{rate},{name},{},{:.6},{:.6},{:.6}",
        times.len(),
        quantile(0.5),
        quantile(0.95),
        times[times.len() - 1]
    );
}
