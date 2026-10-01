//! # rusty-shaper
//!
//! Low-RAM Rust input shaper calibration for 3D printers.
//!
//! This crate re-implements the core shaper calibration logic from
//! Kalico (Klipper) in pure Rust, with a focus on:
//! - **Minimal RAM usage** (streaming PSD computation, no full-array materialization)
//! - **Extensibility** (trait-based shaper model system)
//! - **Correctness** (validated against Kalico reference output)
//!
//! ## Architecture
//!
//! ```text
//! Input ──► Parser ──► PSD ──► ShaperModel ──► Scorer ──► Recommendation
//!           (CSV)     (Welch)   (MZV, EI...)            (JSON/stdout)
//! ```
//!
//! ## Key Design Decisions
//!
//! 1. **Streaming-first**: Raw accelerometer data is processed in overlapping
//!    windows without loading the entire sample matrix into memory.
//! 2. **Trait-based shapers**: The `ShaperModel` trait allows adding new shaper
//!    types without modifying the core scoring logic.
//! 3. **PSD-first path**: For already-computed PSD CSV files, we skip FFT
//!    entirely and go straight to scoring.
//! 4. **No numpy dependency**: All math is explicit, making the behavior
//!    deterministic and easy to audit.
//!
//! ## Example
//!
//! ```rust,no_run
//! use rusty_shaper::{CalibrationOutput, MzvShaper, PsdInput, ShaperCalibrator};
//! use rusty_shaper::scorer::MIN_FREQ;
//!
//! fn main() -> rusty_shaper::Result<()> {
//!     let mut psd = PsdInput::from_csv("resonances.csv")?;
//!     psd.normalize();
//!     psd.suppress_low_freq(MIN_FREQ);
//!
//!     let calibrator = ShaperCalibrator::new()
//!         .with_shaper(Box::new(MzvShaper))
//!         .with_damping_ratio(0.1);
//!
//!     let result : CalibrationOutput = calibrator.fit(&psd)?;
//!     println!("Best MZV: {} Hz", result.recommended_freq);
//!     Ok(())
//! }
//! ```

pub mod constants;
pub mod input;
pub mod models;
pub mod moonraker;
pub mod scorer;
pub mod types;

pub use input::PsdInput;
pub use models::{MzvShaper, ShaperModel, all_shapers, all_shapers_with_zvd};
pub use scorer::ShaperCalibrator;
pub use types::CalibrationOutput;
pub use types::{DampingRatio, Frequency, PsdBin, Smoothing, Vibration};

use thiserror::Error;

/// Errors that can occur during shaper calibration.
#[derive(Error, Debug)]
pub enum ShaperError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
    #[error("Math error: {0}")]
    Math(String),
    #[error("CLI argument error: {0}")]
    Cli(String),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Moonraker error: {0}")]
    Moonraker(#[from] moonraker::MoonrakerError),
}

/// Result type alias for this crate.
pub type Result<T> = std::result::Result<T, ShaperError>;
