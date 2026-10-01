//! Core types used throughout the shaper calibration pipeline.

use serde::{Deserialize, Serialize};

/// Frequency in Hz.
pub type Frequency = f64;

/// A single PSD bin value.
pub type PsdValue = f64;

/// Damping ratio (dimensionless, typically 0.05–0.15).
pub type DampingRatio = f64;

/// Vibration ratio (0.0 = perfect, 1.0 = no reduction).
pub type Vibration = f64;

/// Smoothing value in mm (position offset due to shaper delay).
pub type Smoothing = f64;

/// Score for ranking shaper configurations (lower is better).
pub type Score = f64;

/// A single frequency bin with its PSD values.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PsdBin {
    pub freq: Frequency,
    pub psd_x: PsdValue,
    pub psd_y: PsdValue,
    pub psd_z: PsdValue,
    pub psd_sum: PsdValue,
}

impl PsdBin {
    /// Create a new PSD bin.
    pub fn new(freq: Frequency, psd_x: PsdValue, psd_y: PsdValue, psd_z: PsdValue) -> Self {
        Self {
            freq,
            psd_x,
            psd_y,
            psd_z,
            psd_sum: psd_x + psd_y + psd_z,
        }
    }

    /// Normalize PSD values by dividing by frequency (avoids low-freq bias).
    /// This matches Kalico's `normalize_to_frequencies()` behavior.
    pub fn normalize(&mut self) {
        let f = self.freq + 0.1; // Avoid division by zero
        self.psd_x /= f;
        self.psd_y /= f;
        self.psd_z /= f;
        self.psd_sum = self.psd_x + self.psd_y + self.psd_z;
    }

    /// Apply low-frequency noise suppression.
    /// Matches Kalico's exponential suppression for freq < 2 * MIN_FREQ.
    pub fn suppress_low_freq(&mut self, min_freq: Frequency) {
        let threshold = 2.0 * min_freq;
        if self.freq < threshold {
            let factor = (-((threshold / (self.freq + 0.1)).powi(2)) + 1.0).exp();
            self.psd_x *= factor;
            self.psd_y *= factor;
            self.psd_z *= factor;
            self.psd_sum = self.psd_x + self.psd_y + self.psd_z;
        }
    }
}

/// Shaper coefficients: (amplitudes, times).
///
/// Amplitudes intentionally preserve Kalico's unnormalized values. Scoring
/// normalizes by the amplitude sum when evaluating the response.
/// Times are in seconds, starting from 0.0.
/// Fixed-size storage for up to 5 coefficients (largest shaper is 3HUMP_EI).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaperCoefficients {
    len: usize,
    amplitudes: [f64; 5],
    times: [f64; 5],
}

impl ShaperCoefficients {
    /// Create coefficients from externally supplied slices.
    pub fn try_new(amplitudes: &[f64], times: &[f64]) -> Option<Self> {
        if amplitudes.is_empty() || amplitudes.len() > 5 || amplitudes.len() != times.len() {
            return None;
        }
        if amplitudes
            .iter()
            .chain(times.iter())
            .any(|value| !value.is_finite())
        {
            return None;
        }
        if amplitudes.iter().sum::<f64>() <= 0.0 {
            return None;
        }

        let mut a = [0.0; 5];
        let mut t = [0.0; 5];
        a[..amplitudes.len()].copy_from_slice(amplitudes);
        t[..times.len()].copy_from_slice(times);
        Some(Self {
            len: amplitudes.len(),
            amplitudes: a,
            times: t,
        })
    }

    /// Number of active impulses.
    pub fn len(&self) -> usize {
        self.len
    }

    /// True when no impulses are present.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Active impulse amplitudes.
    pub fn amplitudes(&self) -> &[f64] {
        &self.amplitudes[..self.len]
    }

    /// Active impulse times in seconds.
    pub fn times(&self) -> &[f64] {
        &self.times[..self.len]
    }

    /// Time of the final impulse.
    pub fn last_time(&self) -> f64 {
        self.times[self.len - 1]
    }

    /// Iterate over (amplitude, time) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.amplitudes()
            .iter()
            .zip(self.times())
            .map(|(a, t)| (*a, *t))
    }

    /// Compute the normalized inverse of the sum of amplitudes.
    pub fn inv_sum(&self) -> f64 {
        1.0 / self.amplitudes().iter().sum::<f64>()
    }

    /// Compute the shaper's time shift (weighted average of pulse times).
    pub fn time_shift(&self) -> f64 {
        let inv_d = self.inv_sum();
        self.amplitudes()
            .iter()
            .zip(self.times())
            .map(|(a, t)| a * t)
            .sum::<f64>()
            * inv_d
    }
}

/// A single shaper configuration result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaperConfig {
    pub name: String,
    pub freq: Frequency,
    pub vibrs: Vibration,
    pub smoothing: Smoothing,
    pub score: Score,
    pub max_accel: f64,
}

/// Complete calibration result for a single shaper type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaperFitResult {
    pub shaper_name: String,
    pub best: ShaperConfig,
    pub all_configs: Vec<ShaperConfig>,
}

/// Overall calibration output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationOutput {
    pub recommended_shaper: String,
    pub recommended_freq: Frequency,
    pub recommended_max_accel: f64,
    pub all_results: Vec<ShaperFitResult>,
    /// PSD bins used for calibration (for CSV output).
    pub psd_bins: Vec<PsdBin>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shaper_coefficients_reject_invalid_inputs() {
        assert!(ShaperCoefficients::try_new(&[], &[]).is_none());
        assert!(ShaperCoefficients::try_new(&[1.0; 6], &[0.0; 6]).is_none());
        assert!(ShaperCoefficients::try_new(&[1.0], &[f64::NAN]).is_none());
        assert!(ShaperCoefficients::try_new(&[0.0], &[0.0]).is_none());
    }
}
