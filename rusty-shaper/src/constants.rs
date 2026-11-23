//! Shared constants from Kalico's shaper calibration implementation.
//!
//! Keeping these in one place prevents drift between the scoring logic,
//! shaper coefficient generation, and CLI defaults.

/// Minimum frequency of interest for PSD analysis (Hz).
pub const MIN_FREQ: f64 = 5.0;

/// Maximum frequency for PSD analysis (Hz).
pub const MAX_FREQ: f64 = 200.0;

/// Maximum shaper frequency to evaluate during fitting (Hz).
pub const MAX_SHAPER_FREQ: f64 = 150.0;

/// Default damping ratio used when none is specified.
pub const DEFAULT_DAMPING_RATIO: f64 = 0.1;

/// Damping ratios used to pessimise remaining vibrations.
pub const TEST_DAMPING_RATIOS: [f64; 3] = [0.075, 0.1, 0.15];

/// Vibration reduction target (20x = 26 dB).
pub const SHAPER_VIBRATION_REDUCTION: f64 = 20.0;

/// Smoothing target for max_accel bisection (mm).
pub const TARGET_SMOOTHING: f64 = 0.12;
