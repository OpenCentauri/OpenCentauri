//! Scoring and calibration engine.
//!
//! This module implements the core shaper fitting logic from Kalico's
//! `ShaperCalibrate` class, replicating:
//! - `_estimate_shaper`: frequency response of a shaper
//! - `_estimate_remaining_vibrations`: vibration reduction score
//! - `_get_shaper_smoothing`: position smoothing due to shaper delay
//! - `fit_shaper`: find best frequency for a given shaper type
//! - `find_best_shaper`: select best shaper across all types

pub use crate::constants::MIN_FREQ;
use crate::constants::{
    DEFAULT_DAMPING_RATIO, MAX_FREQ, MAX_SHAPER_FREQ, SHAPER_VIBRATION_REDUCTION, TARGET_SMOOTHING,
    TEST_DAMPING_RATIOS,
};
use crate::input::PsdInput;
use crate::models::ShaperModel;
use crate::types::{
    CalibrationOutput, DampingRatio, Frequency, PsdBin, ShaperCoefficients, ShaperConfig,
    ShaperFitResult, Smoothing, Vibration,
};
use crate::{Result, ShaperError};
use std::f64::consts::PI;

/// Shaper calibrator configuration.
pub struct ShaperCalibrator {
    shapers: Vec<Box<dyn ShaperModel>>,
    damping_ratio: DampingRatio,
    test_damping_ratios: Vec<f64>,
    scv: f64,
    max_smoothing: Option<f64>,
    shaper_freqs: Option<(Frequency, Frequency, Frequency)>, // start, end, step
    max_freq: Frequency,
}

impl Default for ShaperCalibrator {
    fn default() -> Self {
        Self {
            shapers: Vec::new(),
            damping_ratio: DEFAULT_DAMPING_RATIO,
            test_damping_ratios: TEST_DAMPING_RATIOS.to_vec(),
            scv: 5.0,
            max_smoothing: None,
            shaper_freqs: None,
            max_freq: MAX_FREQ,
        }
    }
}

impl ShaperCalibrator {
    /// Create a new calibrator with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a shaper model to test.
    pub fn with_shaper(mut self, shaper: Box<dyn ShaperModel>) -> Self {
        self.shapers.push(shaper);
        self
    }

    /// Set the damping ratio.
    pub fn with_damping_ratio(mut self, dr: DampingRatio) -> Self {
        self.damping_ratio = dr;
        self
    }

    /// Set test damping ratios (for pessimization).
    pub fn with_test_damping_ratios(mut self, ratios: Vec<f64>) -> Self {
        self.test_damping_ratios = ratios;
        self
    }

    /// Set square corner velocity (for smoothing calculation).
    pub fn with_scv(mut self, scv: f64) -> Self {
        self.scv = scv;
        self
    }

    /// Set maximum allowed smoothing.
    pub fn with_max_smoothing(mut self, max_sm: f64) -> Self {
        self.max_smoothing = Some(max_sm);
        self
    }

    /// Set frequency range to test.
    pub fn with_freq_range(mut self, start: Frequency, end: Frequency, step: Frequency) -> Self {
        self.shaper_freqs = Some((start, end, step));
        self
    }

    /// Set maximum frequency for PSD analysis.
    pub fn with_max_freq(mut self, max_f: Frequency) -> Self {
        self.max_freq = max_f;
        self
    }

    fn validate(&self) -> Result<()> {
        validate_damping_ratio(self.damping_ratio, "damping ratio")?;
        validate_positive_finite(self.max_freq, "maximum PSD frequency")?;
        validate_non_negative_finite(self.scv, "square corner velocity")?;

        if self.test_damping_ratios.is_empty() {
            return Err(ShaperError::InvalidInput(
                "At least one test damping ratio is required".to_string(),
            ));
        }
        for ratio in &self.test_damping_ratios {
            validate_damping_ratio(*ratio, "test damping ratio")?;
        }

        if let Some(max_smoothing) = self.max_smoothing {
            validate_non_negative_finite(max_smoothing, "maximum smoothing")?;
        }

        if let Some((start, end, step)) = self.shaper_freqs {
            validate_positive_finite(start, "shaper frequency start")?;
            validate_positive_finite(end, "shaper frequency end")?;
            validate_positive_finite(step, "shaper frequency step")?;
            if start >= end {
                return Err(ShaperError::InvalidInput(
                    "Shaper frequency start must be less than end".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Fit all configured shapers and return the best one.
    pub fn fit(&self, psd: &PsdInput) -> Result<CalibrationOutput> {
        self.validate()?;

        if self.shapers.is_empty() {
            return Err(ShaperError::InvalidInput(
                "No shapers configured".to_string(),
            ));
        }

        let mut all_results = Vec::new();
        let mut best_shaper: Option<ShaperConfig> = None;

        for shaper in &self.shapers {
            let result = self.fit_shaper(shaper.as_ref(), psd)?;

            // Update best shaper using Kalico's selection logic
            if let Some(ref best) = best_shaper {
                if result.best.score * 1.2 < best.score
                    || (result.best.score * 1.05 < best.score
                        && result.best.smoothing * 1.1 < best.smoothing)
                {
                    best_shaper = Some(result.best.clone());
                }
            } else {
                best_shaper = Some(result.best.clone());
            }

            all_results.push(result);
        }

        let best = best_shaper
            .ok_or_else(|| ShaperError::InvalidInput("No valid shaper found".to_string()))?;

        Ok(CalibrationOutput {
            recommended_shaper: best.name.clone(),
            recommended_freq: best.freq,
            recommended_max_accel: best.max_accel,
            all_results,
            psd_bins: psd.bins.clone(),
        })
    }

    /// Fit a single shaper type across its frequency range.
    fn fit_shaper(&self, shaper: &dyn ShaperModel, psd: &PsdInput) -> Result<ShaperFitResult> {
        let freq_range = self
            .shaper_freqs
            .unwrap_or((shaper.min_freq(), MAX_SHAPER_FREQ, 0.2));
        // Respect each shaper's Kalico-defined minimum frequency even when the
        // user supplies a custom range.
        let freq_range = (
            freq_range.0.max(shaper.min_freq()),
            freq_range.1,
            freq_range.2,
        );

        // Index-based grid avoids float accumulation drift vs Kalico's np.arange.
        let count = ((freq_range.1 - freq_range.0) / freq_range.2).ceil() as usize;
        let test_freqs: Vec<f64> = (0..count)
            .map(|i| freq_range.0 + i as f64 * freq_range.2)
            .filter(|&f| f < freq_range.1)
            .collect();

        // Filter PSD to max_freq
        let filtered_bins: Vec<&PsdBin> = psd
            .bins
            .iter()
            .filter(|b| b.freq <= self.max_freq)
            .collect();

        if filtered_bins.is_empty() {
            return Err(ShaperError::InsufficientData(
                "No PSD bins in frequency range".to_string(),
            ));
        }

        let freq_bins: Vec<f64> = filtered_bins.iter().map(|b| b.freq).collect();
        let psd_values: Vec<f64> = filtered_bins.iter().map(|b| b.psd_sum).collect();
        let max_psd = psd_values.iter().copied().fold(0.0, f64::max);

        let mut best_res: Option<ShaperConfig> = None;
        let mut results = Vec::new();

        // Test frequencies in reverse (highest first) for early termination
        for &test_freq in test_freqs.iter().rev() {
            let Some(shaper_coeffs) = shaper.coefficients(test_freq, self.damping_ratio) else {
                // Skip frequencies that do not produce valid coefficients rather
                // than aborting the whole shaper.
                continue;
            };
            let smoothing = get_shaper_smoothing(&shaper_coeffs, self.scv);

            // Early termination if smoothing exceeds max (Kalico semantics:
            // only abort after at least one valid configuration was found).
            if self.max_smoothing.is_some_and(|max_sm| smoothing > max_sm) && best_res.is_some() {
                break;
            }

            // Pessimize over damping ratios
            let mut shaper_vibrations = 0.0;

            for &dr in &self.test_damping_ratios {
                let vibrations = estimate_remaining_vibration_ratio(
                    &shaper_coeffs,
                    dr,
                    &freq_bins,
                    &psd_values,
                    max_psd,
                );

                if vibrations > shaper_vibrations {
                    shaper_vibrations = vibrations;
                }
            }

            let max_accel = find_shaper_max_accel(&shaper_coeffs, self.scv);

            // Score formula from Kalico
            let score = smoothing * (shaper_vibrations.powf(1.5) + shaper_vibrations * 0.2 + 0.01);

            let config = ShaperConfig {
                name: shaper.name().to_string(),
                freq: test_freq,
                vibrs: shaper_vibrations,
                smoothing,
                score,
                max_accel,
            };

            results.push(config.clone());

            // Update best (lowest vibrations)
            let is_better = match best_res.as_ref() {
                Some(best) => best.vibrs > config.vibrs,
                None => true,
            };
            if is_better {
                best_res = Some(config);
            }
        }

        let best = best_res.ok_or_else(|| {
            ShaperError::InvalidInput(format!(
                "No valid configuration for shaper {}",
                shaper.name()
            ))
        })?;

        // Find "optimal" config: not much worse than best, but less smoothing
        let mut selected = best.clone();
        for res in results.iter().rev() {
            if res.vibrs < best.vibrs * 1.1 && res.score < selected.score {
                selected = res.clone();
            }
        }

        Ok(ShaperFitResult {
            shaper_name: shaper.name().to_string(),
            best: selected,
            all_configs: results,
        })
    }
}

fn validate_damping_ratio(value: f64, name: &str) -> Result<()> {
    if value.is_finite() && (0.0..1.0).contains(&value) {
        Ok(())
    } else {
        Err(ShaperError::InvalidInput(format!(
            "Invalid {name}: expected finite value in [0.0, 1.0), got {value}"
        )))
    }
}

fn validate_positive_finite(value: f64, name: &str) -> Result<()> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(ShaperError::InvalidInput(format!(
            "Invalid {name}: expected finite positive value, got {value}"
        )))
    }
}

fn validate_non_negative_finite(value: f64, name: &str) -> Result<()> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(ShaperError::InvalidInput(format!(
            "Invalid {name}: expected finite non-negative value, got {value}"
        )))
    }
}

/// Estimate the frequency response of a shaper.
///
/// Returns the vibration reduction factor (0.0 = perfect, 1.0 = no effect)
/// for each frequency bin.
pub fn estimate_shaper(
    shaper: &ShaperCoefficients,
    damping_ratio: DampingRatio,
    test_freqs: &[f64],
) -> Vec<f64> {
    test_freqs
        .iter()
        .map(|freq| estimate_shaper_response(shaper, damping_ratio, *freq))
        .collect()
}

fn estimate_shaper_response(
    shaper: &ShaperCoefficients,
    damping_ratio: DampingRatio,
    freq: Frequency,
) -> f64 {
    debug_assert!(!shaper.is_empty());

    let inv_d = shaper.inv_sum();
    let last_time = shaper.last_time();
    let omega = 2.0 * PI * freq;
    let damping = damping_ratio * omega;
    let omega_d = omega * (1.0 - damping_ratio * damping_ratio).sqrt();

    let mut s_sum = 0.0;
    let mut c_sum = 0.0;

    for (amplitude, time) in shaper.iter() {
        let weight = amplitude * (-damping * (last_time - time)).exp();
        let angle = omega_d * time;
        s_sum += weight * angle.sin();
        c_sum += weight * angle.cos();
    }

    (s_sum * s_sum + c_sum * c_sum).sqrt() * inv_d
}

/// Estimate remaining vibration ratio after applying a shaper (scalar only).
///
/// Same math as estimate_shaper + threshold scoring but evaluates one frequency
/// at a time to avoid allocating a response vector in the scoring hot path.
fn estimate_remaining_vibration_ratio(
    shaper: &ShaperCoefficients,
    damping_ratio: DampingRatio,
    freq_bins: &[f64],
    psd: &[f64],
    max_psd: f64,
) -> Vibration {
    let vibr_threshold = max_psd / SHAPER_VIBRATION_REDUCTION;

    let mut remaining = 0.0;
    let mut all = 0.0;

    for (&freq, &psd_value) in freq_bins.iter().zip(psd.iter()) {
        let response = estimate_shaper_response(shaper, damping_ratio, freq);
        remaining += (response * psd_value - vibr_threshold).max(0.0);
        all += (psd_value - vibr_threshold).max(0.0);
    }

    if all > 0.0 { remaining / all } else { 0.0 }
}

/// Calculate shaper smoothing (position offset in mm).
fn get_shaper_smoothing(shaper: &ShaperCoefficients, scv: f64) -> Smoothing {
    get_shaper_smoothing_with_accel(shaper, 5000.0, scv)
}

/// Calculate shaper smoothing with a specific acceleration.
fn get_shaper_smoothing_with_accel(shaper: &ShaperCoefficients, accel: f64, scv: f64) -> Smoothing {
    let half_accel = accel * 0.5;
    let inv_d = shaper.inv_sum();
    let ts = shaper.time_shift();

    let mut offset_90 = 0.0;
    let mut offset_180 = 0.0;

    for (amplitude, time) in shaper.iter() {
        let delta = time - ts;
        if delta >= 0.0 {
            offset_90 += amplitude * (scv + half_accel * delta) * delta;
        }
        offset_180 += amplitude * half_accel * delta.powi(2);
    }

    offset_90 *= inv_d * 2.0_f64.sqrt();
    offset_180 *= inv_d;

    offset_90.max(offset_180)
}

/// Find maximum acceleration that keeps smoothing below target.
fn find_shaper_max_accel(shaper: &ShaperCoefficients, scv: f64) -> f64 {
    bisect(|test_accel| {
        get_shaper_smoothing_with_accel(shaper, test_accel, scv) <= TARGET_SMOOTHING
    })
}

/// Binary search helper.
fn bisect<F>(func: F) -> f64
where
    F: Fn(f64) -> bool,
{
    let mut left = 1.0;
    let mut right = 1.0;

    if !func(1e-9) {
        return 0.0;
    }

    while !func(left) {
        right = left;
        left *= 0.5;
    }

    if right == left {
        while func(right) {
            right *= 2.0;
        }
    }

    while right - left > 1e-8 {
        let middle = (left + right) * 0.5;
        if func(middle) {
            left = middle;
        } else {
            right = middle;
        }
    }

    left
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MzvShaper, ShaperModel};
    use approx::assert_relative_eq;

    /// Mock shaper that returns None for coefficients at a specific frequency.
    /// Used to verify fit_shaper skips individual frequencies gracefully.
    struct SkippingShaper {
        skip_freq: f64,
    }

    impl ShaperModel for SkippingShaper {
        fn name(&self) -> &'static str {
            "skipper"
        }

        fn min_freq(&self) -> Frequency {
            20.0
        }

        fn coefficients(
            &self,
            freq: Frequency,
            _damping_ratio: DampingRatio,
        ) -> Option<ShaperCoefficients> {
            if (freq - self.skip_freq).abs() < 1e-6 {
                return None;
            }
            // Return a valid ZV-like two-pulse shaper.
            ShaperCoefficients::try_new(&[1.0, (-0.1_f64 * PI).exp()], &[0.0, 0.5 / freq])
        }
    }

    #[test]
    fn test_estimate_shaper_dc() {
        let mzv = MzvShaper;
        let Some(coeffs) = mzv.coefficients(40.0, 0.1) else {
            panic!("MZV should always produce coefficients");
        };

        // At DC (0 Hz), shaper should have no effect (response = 1.0)
        let vals = estimate_shaper(&coeffs, 0.1, &[0.0]);
        assert_relative_eq!(vals[0], 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_estimate_shaper_at_freq() {
        let mzv = MzvShaper;
        let Some(coeffs) = mzv.coefficients(40.0, 0.1) else {
            panic!("MZV should always produce coefficients");
        };

        // At the shaper frequency, response should be near 0
        let vals = estimate_shaper(&coeffs, 0.1, &[40.0]);
        assert!(
            vals[0] < 0.1,
            "Expected near-zero at shaper freq, got {}",
            vals[0]
        );
    }

    #[test]
    fn test_shaper_smoothing() {
        let mzv = MzvShaper;
        let Some(coeffs) = mzv.coefficients(40.0, 0.1) else {
            panic!("MZV should always produce coefficients");
        };

        let smoothing = get_shaper_smoothing(&coeffs, 5.0);
        assert!(smoothing > 0.0);
        assert!(smoothing < 1.0); // Should be in mm range
    }

    #[test]
    fn test_bisect() {
        let result = bisect(|x| x <= 0.5);
        assert_relative_eq!(result, 0.5, epsilon = 1e-7);
    }

    #[test]
    fn test_find_max_accel() {
        let mzv = MzvShaper;
        let Some(coeffs) = mzv.coefficients(40.0, 0.1) else {
            panic!("MZV should always produce coefficients");
        };

        let max_accel = find_shaper_max_accel(&coeffs, 5.0);
        assert!(max_accel > 0.0);

        // Verify it's actually at the boundary
        let smoothing_at_max = get_shaper_smoothing_with_accel(&coeffs, max_accel, 5.0);
        assert_relative_eq!(smoothing_at_max, TARGET_SMOOTHING, epsilon = 1e-6);
    }

    #[test]
    fn max_smoothing_allows_kalico_compatible_early_exit() {
        // Kalico only aborts a shaper once a valid configuration has been
        // found and a subsequent frequency exceeds max_smoothing. With a very
        // tight limit the first tested frequency still produces a result, so
        // the fit must succeed (not return "No valid configuration").
        let psd = PsdInput {
            bins: (0..80)
                .map(|i| {
                    let freq = i as f64 * 2.5;
                    let power = if (freq - 50.0).abs() < 1.0 { 10.0 } else { 1.0 };
                    PsdBin::new(freq, power, 0.0, 0.0)
                })
                .collect(),
            accel_per_hz: None,
            normalized: false,
        };

        let result = ShaperCalibrator::new()
            .with_shaper(Box::new(MzvShaper))
            .with_max_smoothing(0.0)
            .fit(&psd)
            .expect("Kalico-compatible max_smoothing should return a valid fit");

        let mzv = result
            .all_results
            .iter()
            .find(|r| r.shaper_name == "mzv")
            .expect("mzv result should exist");
        assert!(mzv.best.smoothing > 0.0);
    }

    #[test]
    fn invalid_frequency_step_is_rejected() {
        let psd = PsdInput {
            bins: vec![PsdBin::new(10.0, 1.0, 0.0, 0.0)],
            accel_per_hz: None,
            normalized: false,
        };

        let err = ShaperCalibrator::new()
            .with_shaper(Box::new(MzvShaper))
            .with_freq_range(20.0, 60.0, 0.0)
            .fit(&psd)
            .err()
            .unwrap();
        assert!(err.to_string().contains("step"));
    }

    #[test]
    fn frequency_grid_matches_np_arange() {
        // 20.0:60.0:0.2 should produce the same count as Kalico's np.arange.
        let start: f64 = 20.0;
        let end: f64 = 60.0;
        let step: f64 = 0.2;
        let count = ((end - start) / step).ceil() as usize;
        let freqs: Vec<f64> = (0..count)
            .map(|i| start + i as f64 * step)
            .filter(|&f| f < end)
            .collect();

        assert!(!freqs.is_empty());
        assert!(freqs.last().unwrap() < &end);
        // Last value should be close to end - step (within float noise).
        assert_relative_eq!(freqs.last().unwrap(), &(end - step), epsilon = 1e-9);
    }

    #[test]
    fn shaper_min_freq_clips_custom_range() {
        // MZV min_freq is 23.0; a user range starting at 15 Hz should be
        // clipped to the shaper's minimum.
        let psd = PsdInput {
            bins: (0..80)
                .map(|i| {
                    let freq = i as f64 * 2.5;
                    PsdBin::new(freq, 1.0, 0.0, 0.0)
                })
                .collect(),
            accel_per_hz: None,
            normalized: false,
        };

        let result = ShaperCalibrator::new()
            .with_shaper(Box::new(MzvShaper))
            .with_freq_range(15.0, 80.0, 1.0)
            .fit(&psd)
            .expect("fit should succeed");

        let mzv = result
            .all_results
            .iter()
            .find(|r| r.shaper_name == "mzv")
            .expect("mzv result should exist");
        assert!(
            mzv.best.freq >= 23.0,
            "MZV frequency {} should be clipped to min_freq 23.0",
            mzv.best.freq
        );
    }

    #[test]
    fn missing_coefficients_are_skipped_not_fatal() {
        // A shaper that cannot produce coefficients at exactly 50 Hz should
        // still produce a valid fit for surrounding frequencies.
        let psd = PsdInput {
            bins: (0..80)
                .map(|i| {
                    let freq = i as f64 * 2.5;
                    PsdBin::new(freq, 1.0, 0.0, 0.0)
                })
                .collect(),
            accel_per_hz: None,
            normalized: false,
        };

        let result = ShaperCalibrator::new()
            .with_shaper(Box::new(SkippingShaper { skip_freq: 50.0 }))
            .with_freq_range(20.0, 80.0, 1.0)
            .fit(&psd)
            .expect("fit should succeed even when one frequency skips");

        let skipper = result
            .all_results
            .iter()
            .find(|r| r.shaper_name == "skipper")
            .expect("skipper result should exist");
        assert!(skipper.best.freq >= 20.0);
        // 50 Hz should not appear in the evaluated configs.
        assert!(
            !skipper
                .all_configs
                .iter()
                .any(|c| (c.freq - 50.0).abs() < 1e-6),
            "skipped frequency should not be in results"
        );
    }

    /// Real-world regression: pin the recommendation against goldens generated by
    /// Kalico's `scripts/calibrate_shaper.py` (see `test/data/real/generate_kalico_goldens.py`).
    /// Tolerances are chosen to be tight enough to catch regressions but loose enough
    /// to absorb floating-point and windowing differences between the implementations.
    mod real_world_regression {
        use super::*;
        use flate2::read::GzDecoder;
        use std::fs::File;
        use std::io::copy;

        const DEFAULT_SHAPER_NAMES: &[&str] = &["zv", "mzv", "ei", "2hump_ei", "3hump_ei"];
        const FREQ_TOL_HZ: f64 = 0.5;
        const VIBRS_TOL: f64 = 0.005; // 0.5 percentage points
        const SMOOTHING_TOL: f64 = 0.005;
        const MAX_ACCEL_TOL: f64 = 100.0;

        struct Capture {
            author: &'static str,
            axis: &'static str,
            csv: &'static str,
        }

        const CAPTURES: &[Capture] = &[
            Capture {
                author: "krishlulla",
                axis: "x",
                csv: "raw_data_x_lis2dw_20260628_050622.csv",
            },
            Capture {
                author: "krishlulla",
                axis: "y",
                csv: "raw_data_y_lis2dw_20260628_050734.csv",
            },
            Capture {
                author: "peterb0288",
                axis: "x",
                csv: "raw_data_x_lis2dw_20260628_223144.csv",
            },
            Capture {
                author: "peterb0288",
                axis: "y",
                csv: "raw_data_y_lis2dw_20260628_223328.csv",
            },
            Capture {
                author: "atomique13",
                axis: "x",
                csv: "raw_data_x_lis2dw_20260619_232243.csv",
            },
            Capture {
                author: "atomique13",
                axis: "y",
                csv: "raw_data_y_lis2dw_20260619_232401.csv",
            },
            Capture {
                author: "jaimbo",
                axis: "x",
                csv: "raw_data_x_lis2dw_20260629_125021.csv",
            },
            Capture {
                author: "jaimbo",
                axis: "y",
                csv: "raw_data_y_lis2dw_20260629_124904.csv",
            },
            Capture {
                author: "harrym",
                axis: "x",
                csv: "raw_data_x_lis2dw_20260629_170415.csv",
            },
            Capture {
                author: "harrym",
                axis: "y",
                csv: "raw_data_y_lis2dw_20260629_173511.csv",
            },
            Capture {
                author: "lizard_0619",
                axis: "x",
                csv: "raw_data_x_lis2dw_20260619_223752.csv",
            },
            Capture {
                author: "lizard_0619",
                axis: "y",
                csv: "raw_data_y_lis2dw_20260619_223839.csv",
            },
            Capture {
                author: "lizard_0629",
                axis: "x",
                csv: "raw_data_x_lis2dw_20260629_174255.csv",
            },
            Capture {
                author: "lizard_0629",
                axis: "y",
                csv: "raw_data_y_lis2dw_20260629_174451.csv",
            },
        ];

        fn data_dir() -> std::path::PathBuf {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("test")
                .join("data")
                .join("real")
        }

        fn load_golden(author: &str, axis: &str) -> serde_json::Value {
            let path = data_dir().join(format!("{author}_{axis}.json"));
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
        }

        fn fit_real_capture(csv_name: &str) -> CalibrationOutput {
            let path = data_dir().join(format!("{csv_name}.gz"));
            let mut capture = tempfile::NamedTempFile::new()
                .expect("temporary capture file should be created");
            let mut decoder = GzDecoder::new(
                File::open(&path)
                    .unwrap_or_else(|e| panic!("failed to open {}: {e}", path.display())),
            );
            copy(&mut decoder, capture.as_file_mut())
                .unwrap_or_else(|e| panic!("failed to decompress {}: {e}", path.display()));

            let mut psd = crate::input::PsdInput::from_raw_csv_streaming(capture.path(), 0.5)
                .expect("real capture should parse");
            // Match the CLI's calibration pipeline exactly.
            psd.normalize();
            psd.suppress_low_freq(crate::scorer::MIN_FREQ);

            let mut calibrator = ShaperCalibrator::new();
            for name in DEFAULT_SHAPER_NAMES {
                calibrator = calibrator.with_shaper(
                    crate::models::shaper_by_name(name).expect("default shaper should exist"),
                );
            }
            calibrator.fit(&psd).expect("real capture should fit")
        }

        fn assert_close_f64(actual: f64, expected: f64, tol: f64, label: &str) {
            assert!(
                (actual - expected).abs() <= tol,
                "{label}: actual {actual:.4}, expected {expected:.4}, tolerance ±{tol:.4}"
            );
        }

        #[test]
        fn all_captures_match_kalico_recommendation() {
            for cap in CAPTURES {
                let golden = load_golden(cap.author, cap.axis);
                let expected_name = golden["recommended"]["name"].as_str().unwrap();
                let expected_freq = golden["recommended"]["freq"].as_f64().unwrap();

                let output = fit_real_capture(cap.csv);
                let label = format!("{}_{}", cap.author, cap.axis);

                assert_eq!(
                    output.recommended_shaper, expected_name,
                    "{label}: recommended shaper changed"
                );
                assert_close_f64(
                    output.recommended_freq,
                    expected_freq,
                    FREQ_TOL_HZ,
                    &format!("{label}: recommended freq"),
                );
            }
        }

        #[test]
        fn all_captures_match_kalico_per_shaper_metrics() {
            for cap in CAPTURES {
                let golden = load_golden(cap.author, cap.axis);
                let output = fit_real_capture(cap.csv);
                let label = format!("{}_{}", cap.author, cap.axis);

                for golden_shaper in golden["all_shapers"].as_array().unwrap() {
                    let name = golden_shaper["name"].as_str().unwrap();
                    let fit = output
                        .all_results
                        .iter()
                        .find(|fit| fit.shaper_name == name)
                        .unwrap_or_else(|| panic!("{label}: shaper {name} missing from results"));

                    assert_close_f64(
                        fit.best.freq,
                        golden_shaper["freq"].as_f64().unwrap(),
                        FREQ_TOL_HZ,
                        &format!("{label}: {name} freq"),
                    );
                    assert_close_f64(
                        fit.best.vibrs,
                        golden_shaper["vibrs"].as_f64().unwrap(),
                        VIBRS_TOL,
                        &format!("{label}: {name} vibrs"),
                    );
                    assert_close_f64(
                        fit.best.smoothing,
                        golden_shaper["smoothing"].as_f64().unwrap(),
                        SMOOTHING_TOL,
                        &format!("{label}: {name} smoothing"),
                    );
                    assert_close_f64(
                        fit.best.max_accel,
                        golden_shaper["max_accel"].as_f64().unwrap(),
                        MAX_ACCEL_TOL,
                        &format!("{label}: {name} max_accel"),
                    );
                }
            }
        }
    }
}
