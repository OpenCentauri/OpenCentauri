//! Input shaper model definitions.
//!
//! Each shaper type implements the [`ShaperModel`] trait, providing:
//! - Coefficient generation for a given frequency and damping ratio
//! - A human-readable name
//! - Minimum valid frequency

use crate::constants::SHAPER_VIBRATION_REDUCTION;
use crate::types::{DampingRatio, Frequency, ShaperCoefficients};
use std::f64::consts::PI;

/// Trait for input shaper models.
pub trait ShaperModel: Send + Sync {
    /// Shaper name (e.g., "mzv", "ei", "zv").
    fn name(&self) -> &'static str;

    /// Minimum valid frequency for this shaper.
    fn min_freq(&self) -> Frequency;

    /// Generate shaper coefficients for the given frequency and damping ratio.
    fn coefficients(
        &self,
        freq: Frequency,
        damping_ratio: DampingRatio,
    ) -> Option<ShaperCoefficients>;
}

/// Zero Vibration (ZV) shaper.
pub struct ZvShaper;

impl ShaperModel for ZvShaper {
    fn name(&self) -> &'static str {
        "zv"
    }
    fn min_freq(&self) -> Frequency {
        21.0
    }

    fn coefficients(
        &self,
        freq: Frequency,
        damping_ratio: DampingRatio,
    ) -> Option<ShaperCoefficients> {
        let df = (1.0 - damping_ratio * damping_ratio).sqrt();
        let k = (-damping_ratio * PI / df).exp();
        let t_d = 1.0 / (freq * df);

        ShaperCoefficients::try_new(&[1.0, k], &[0.0, 0.5 * t_d])
    }
}

/// Zero Vibration Derivative (ZVD) shaper.
pub struct ZvdShaper;

impl ShaperModel for ZvdShaper {
    fn name(&self) -> &'static str {
        "zvd"
    }
    fn min_freq(&self) -> Frequency {
        29.0
    }

    fn coefficients(
        &self,
        freq: Frequency,
        damping_ratio: DampingRatio,
    ) -> Option<ShaperCoefficients> {
        let df = (1.0 - damping_ratio * damping_ratio).sqrt();
        let k = (-damping_ratio * PI / df).exp();
        let t_d = 1.0 / (freq * df);

        ShaperCoefficients::try_new(&[1.0, 2.0 * k, k * k], &[0.0, 0.5 * t_d, t_d])
    }
}

/// Modified Zero Vibration (MZV) shaper.
pub struct MzvShaper;

impl ShaperModel for MzvShaper {
    fn name(&self) -> &'static str {
        "mzv"
    }
    fn min_freq(&self) -> Frequency {
        23.0
    }

    fn coefficients(
        &self,
        freq: Frequency,
        damping_ratio: DampingRatio,
    ) -> Option<ShaperCoefficients> {
        let df = (1.0 - damping_ratio * damping_ratio).sqrt();
        let k = (-0.75 * damping_ratio * PI / df).exp();
        let t_d = 1.0 / (freq * df);

        let a1 = 1.0 - 1.0 / 2.0_f64.sqrt();
        let a2 = (2.0_f64.sqrt() - 1.0) * k;
        let a3 = a1 * k * k;

        ShaperCoefficients::try_new(&[a1, a2, a3], &[0.0, 0.375 * t_d, 0.75 * t_d])
    }
}

/// Extra-Insensitive (EI) shaper.
pub struct EiShaper;

impl ShaperModel for EiShaper {
    fn name(&self) -> &'static str {
        "ei"
    }
    fn min_freq(&self) -> Frequency {
        29.0
    }

    fn coefficients(
        &self,
        freq: Frequency,
        damping_ratio: DampingRatio,
    ) -> Option<ShaperCoefficients> {
        let v_tol = 1.0 / SHAPER_VIBRATION_REDUCTION;
        let df = (1.0 - damping_ratio * damping_ratio).sqrt();
        let k = (-damping_ratio * PI / df).exp();
        let t_d = 1.0 / (freq * df);

        let a1 = 0.25 * (1.0 + v_tol);
        let a2 = 0.5 * (1.0 - v_tol) * k;
        let a3 = a1 * k * k;

        ShaperCoefficients::try_new(&[a1, a2, a3], &[0.0, 0.5 * t_d, t_d])
    }
}

/// 2-Hump Extra-Insensitive shaper.
pub struct TwoHumpEiShaper;

impl ShaperModel for TwoHumpEiShaper {
    fn name(&self) -> &'static str {
        "2hump_ei"
    }
    fn min_freq(&self) -> Frequency {
        39.0
    }

    fn coefficients(
        &self,
        freq: Frequency,
        damping_ratio: DampingRatio,
    ) -> Option<ShaperCoefficients> {
        let v_tol = 1.0 / SHAPER_VIBRATION_REDUCTION;
        let df = (1.0 - damping_ratio * damping_ratio).sqrt();
        let k = (-damping_ratio * PI / df).exp();
        let t_d = 1.0 / (freq * df);

        let v2 = v_tol * v_tol;
        let x = (v2 * ((1.0 - v2).sqrt() + 1.0)).powf(1.0 / 3.0);
        let a1 = (3.0 * x * x + 2.0 * x + 3.0 * v2) / (16.0 * x);
        let a2 = (0.5 - a1) * k;
        let a3 = a2 * k;
        let a4 = a1 * k * k * k;

        ShaperCoefficients::try_new(&[a1, a2, a3, a4], &[0.0, 0.5 * t_d, t_d, 1.5 * t_d])
    }
}

/// 3-Hump Extra-Insensitive shaper.
pub struct ThreeHumpEiShaper;

impl ShaperModel for ThreeHumpEiShaper {
    fn name(&self) -> &'static str {
        "3hump_ei"
    }
    fn min_freq(&self) -> Frequency {
        48.0
    }

    fn coefficients(
        &self,
        freq: Frequency,
        damping_ratio: DampingRatio,
    ) -> Option<ShaperCoefficients> {
        let v_tol = 1.0 / SHAPER_VIBRATION_REDUCTION;
        let df = (1.0 - damping_ratio * damping_ratio).sqrt();
        let k = (-damping_ratio * PI / df).exp();
        let t_d = 1.0 / (freq * df);

        let k2 = k * k;
        let a1 = 0.0625 * (1.0 + 3.0 * v_tol + 2.0 * (2.0 * (v_tol + 1.0) * v_tol).sqrt());
        let a2 = 0.25 * (1.0 - v_tol) * k;
        let a3 = (0.5 * (1.0 + v_tol) - 2.0 * a1) * k2;
        let a4 = a2 * k2;
        let a5 = a1 * k2 * k2;

        ShaperCoefficients::try_new(
            &[a1, a2, a3, a4, a5],
            &[0.0, 0.5 * t_d, t_d, 1.5 * t_d, 2.0 * t_d],
        )
    }
}

/// Get all built-in shaper models.
pub fn all_shapers() -> Vec<Box<dyn ShaperModel>> {
    vec![
        Box::new(ZvShaper),
        Box::new(MzvShaper),
        Box::new(EiShaper),
        Box::new(TwoHumpEiShaper),
        Box::new(ThreeHumpEiShaper),
    ]
}

/// Extended shaper set including ZVD (not in Kalico's default AUTOTUNE_SHAPERS).
pub fn all_shapers_with_zvd() -> Vec<Box<dyn ShaperModel>> {
    vec![
        Box::new(ZvShaper),
        Box::new(MzvShaper),
        Box::new(ZvdShaper),
        Box::new(EiShaper),
        Box::new(TwoHumpEiShaper),
        Box::new(ThreeHumpEiShaper),
    ]
}

/// Get a built-in shaper by name.
pub fn shaper_by_name(name: &str) -> Option<Box<dyn ShaperModel>> {
    match name.trim() {
        "zv" => Some(Box::new(ZvShaper)),
        "mzv" => Some(Box::new(MzvShaper)),
        "zvd" => Some(Box::new(ZvdShaper)),
        "ei" => Some(Box::new(EiShaper)),
        "2hump_ei" => Some(Box::new(TwoHumpEiShaper)),
        "3hump_ei" => Some(Box::new(ThreeHumpEiShaper)),
        _ => None,
    }
}

/// Get shaper coefficients by name (for CSV output generation).
pub fn get_shaper_coefficients(
    name: &str,
    freq: Frequency,
    damping_ratio: DampingRatio,
) -> Option<ShaperCoefficients> {
    shaper_by_name(name).and_then(|shaper| shaper.coefficients(freq, damping_ratio))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_mzv_coefficients() {
        let mzv = MzvShaper;
        let coeffs = mzv.coefficients(40.0, 0.1).unwrap();

        // Kalico amplitudes are NOT normalized to 1.0; normalization happens in scoring
        // Just verify they're positive and times are non-decreasing
        assert!(coeffs.amplitudes().iter().all(|a| *a > 0.0));

        // Times should be non-decreasing
        for pair in coeffs.times().windows(2) {
            assert!(pair[1] >= pair[0]);
        }
    }

    #[test]
    fn test_ei_coefficients() {
        let ei = EiShaper;
        let coeffs = ei.coefficients(40.0, 0.1).unwrap();

        // Kalico amplitudes are NOT normalized to 1.0; normalization happens in scoring
        assert!(coeffs.amplitudes().iter().all(|a| *a > 0.0));
    }

    #[test]
    fn test_zv_vs_kalico() {
        let zv = ZvShaper;
        let coeffs = zv.coefficients(40.0, 0.1).unwrap();

        // Compare with known Kalico values for ZV @ 40Hz, damping=0.1
        let df = (1.0_f64 - 0.01_f64).sqrt();
        let k = (-0.1 * PI / df).exp();
        let t_d = 1.0 / (40.0 * df);

        assert_relative_eq!(coeffs.amplitudes()[0], 1.0, epsilon = 1e-10);
        assert_relative_eq!(coeffs.amplitudes()[1], k, epsilon = 1e-10);
        assert_relative_eq!(coeffs.times()[0], 0.0, epsilon = 1e-10);
        assert_relative_eq!(coeffs.times()[1], 0.5 * t_d, epsilon = 1e-10);
    }

    #[test]
    fn test_unknown_shaper_name_is_rejected() {
        assert!(shaper_by_name("not_a_shaper").is_none());
        assert!(get_shaper_coefficients("not_a_shaper", 40.0, 0.1).is_none());
    }
}
