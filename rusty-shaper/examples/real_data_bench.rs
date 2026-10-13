//! Benchmark suite for the real-world raw accelerometer captures shipped in
//! `test/data/real/`.
//!
//! Run the binary on the host machine or on a live Centauri Carbon 1 running
//! Cosmos:
//!
//! ```bash
//! # Host (x86_64)
//! cargo run --release --example real_data_bench
//!
//! # ARM target (Centauri Carbon 1) via cross
//! cross run --release --target armv7-unknown-linux-musleabihf \
//!   --example real_data_bench
//! ```
//!
//! On the Centauri Carbon 1 the manifest path baked into the binary does not
//! exist,
//! so set `RUSTY_SHAPER_DATA_DIR` to the directory holding the compressed CSV
//! fixtures and JSON goldens:
//!
//! ```bash
//! RUSTY_SHAPER_DATA_DIR=/user-resource/scratch/rusty-shaper-bench/test/data/real \
//!   ./real_data_bench
//! ```
//!
//! For wall-clock and RSS measurement, wrap with `/usr/bin/time -v`:
//!
//! ```bash
//! /usr/bin/time -v ./target/release/examples/real_data_bench
//! ```
//!
//! The benchmark prints a markdown table summarising each capture's
//! recommended shaper, frequency, score, max_accel, and elapsed wall time.
//! It loads the Kalico-derived goldens from `test/data/real/<label>.json`
//! and exits non-zero if any capture drifts more than 0.5 Hz from Kalico.

use std::fs::File;
use std::io::copy;
use std::path::PathBuf;
use std::time::Instant;

use flate2::read::GzDecoder;
use rusty_shaper::input::PsdInput;
use rusty_shaper::models::shaper_by_name;
use rusty_shaper::scorer::{MIN_FREQ, ShaperCalibrator};

const DEFAULT_SHAPER_NAMES: &[&str] = &["zv", "mzv", "ei", "2hump_ei", "3hump_ei"];
const FREQ_TOLERANCE_HZ: f64 = 0.5;

struct Capture {
    label: &'static str,
    csv: &'static str,
}

const CAPTURES: &[Capture] = &[
    Capture {
        label: "krishlulla_x",
        csv: "raw_data_x_lis2dw_20260628_050622.csv",
    },
    Capture {
        label: "krishlulla_y",
        csv: "raw_data_y_lis2dw_20260628_050734.csv",
    },
    Capture {
        label: "peterb0288_x",
        csv: "raw_data_x_lis2dw_20260628_223144.csv",
    },
    Capture {
        label: "peterb0288_y",
        csv: "raw_data_y_lis2dw_20260628_223328.csv",
    },
    Capture {
        label: "atomique13_x",
        csv: "raw_data_x_lis2dw_20260619_232243.csv",
    },
    Capture {
        label: "atomique13_y",
        csv: "raw_data_y_lis2dw_20260619_232401.csv",
    },
    Capture {
        label: "jaimbo_x",
        csv: "raw_data_x_lis2dw_20260629_125021.csv",
    },
    Capture {
        label: "jaimbo_y",
        csv: "raw_data_y_lis2dw_20260629_124904.csv",
    },
    Capture {
        label: "harrym_x",
        csv: "raw_data_x_lis2dw_20260629_170415.csv",
    },
    Capture {
        label: "harrym_y",
        csv: "raw_data_y_lis2dw_20260629_173511.csv",
    },
    Capture {
        label: "lizard_0619_x",
        csv: "raw_data_x_lis2dw_20260619_223752.csv",
    },
    Capture {
        label: "lizard_0619_y",
        csv: "raw_data_y_lis2dw_20260619_223839.csv",
    },
    Capture {
        label: "lizard_0629_x",
        csv: "raw_data_x_lis2dw_20260629_174255.csv",
    },
    Capture {
        label: "lizard_0629_y",
        csv: "raw_data_y_lis2dw_20260629_174451.csv",
    },
];

fn real_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("RUSTY_SHAPER_DATA_DIR") {
        return PathBuf::from(dir);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test")
        .join("data")
        .join("real")
}

fn load_golden(label: &str) -> serde_json::Value {
    let path = real_data_dir().join(format!("{label}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
}

fn fit_capture(csv: &str) -> (rusty_shaper::types::CalibrationOutput, std::time::Duration) {
    let path = real_data_dir().join(format!("{csv}.gz"));
    let mut capture = tempfile::NamedTempFile::new()
        .expect("temporary capture file should be created");
    let mut decoder = GzDecoder::new(
        File::open(&path).unwrap_or_else(|e| panic!("failed to open {}: {e}", path.display())),
    );
    copy(&mut decoder, capture.as_file_mut())
        .unwrap_or_else(|e| panic!("failed to decompress {}: {e}", path.display()));

    let mut psd = PsdInput::from_raw_csv_streaming(capture.path(), 0.5)
        .unwrap_or_else(|e| panic!("{csv} parse failed: {e}"));
    psd.normalize();
    psd.suppress_low_freq(MIN_FREQ);

    let mut calibrator = ShaperCalibrator::new();
    for name in DEFAULT_SHAPER_NAMES {
        calibrator =
            calibrator.with_shaper(shaper_by_name(name).expect("default shaper should exist"));
    }

    let start = Instant::now();
    let output = calibrator.fit(&psd).expect("calibration should fit");
    let elapsed = start.elapsed();
    (output, elapsed)
}

fn main() {
    println!("# rusty-shaper real-data benchmark");
    println!();
    println!("Captures: {}", CAPTURES.len());
    println!("Shaper set: {}", DEFAULT_SHAPER_NAMES.join(","));
    println!();
    println!("| capture | shaper | freq (Hz) | max_accel | score | wall (ms) | drift (Hz) |");
    println!("|---------|--------|-----------|-----------|-------|-----------|------------|");

    let mut drift_failures = Vec::new();
    for capture in CAPTURES {
        let golden = load_golden(capture.label);
        let expected_name = golden["recommended"]["name"].as_str().unwrap();
        let expected_freq = golden["recommended"]["freq"].as_f64().unwrap();

        let (output, elapsed) = fit_capture(capture.csv);
        let drift = (output.recommended_freq - expected_freq).abs();
        let drift_ok = drift <= FREQ_TOLERANCE_HZ;
        let shaper_ok = output.recommended_shaper == expected_name;
        if !drift_ok || !shaper_ok {
            drift_failures.push((
                capture.label,
                output.recommended_shaper.clone(),
                expected_name.to_string(),
                output.recommended_freq,
                expected_freq,
                drift,
            ));
        }
        println!(
            "| {} | {} | {:.1} | {:.0} | {:.4} | {:.0} | {:.2} |",
            capture.label,
            output.recommended_shaper,
            output.recommended_freq,
            output.recommended_max_accel,
            output
                .all_results
                .iter()
                .find(|r| r.shaper_name == output.recommended_shaper)
                .map(|r| r.best.score)
                .unwrap_or(f64::NAN),
            elapsed.as_secs_f64() * 1000.0,
            drift,
        );
    }

    if !drift_failures.is_empty() {
        eprintln!();
        eprintln!("FAIL: regressions exceeded {FREQ_TOLERANCE_HZ:.1} Hz tolerance:");
        for (label, got, want, freq, want_freq, drift) in drift_failures {
            eprintln!(
                "  {label}: got {got} @ {freq:.2} Hz (want {want} @ {want_freq:.1} Hz, drift {drift:.2})"
            );
        }
        std::process::exit(1);
    }

    println!();
    println!("PASS: all captures within {FREQ_TOLERANCE_HZ:.1} Hz of Kalico golden.");
}
