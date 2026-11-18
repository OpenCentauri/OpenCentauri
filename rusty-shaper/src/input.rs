//! Input parsing: raw accelerometer CSV and pre-computed PSD CSV.

use crate::types::PsdBin;
use crate::{Result, ShaperError};
use csv::{ReaderBuilder, StringRecord};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Pre-computed PSD data from a CSV file.
///
/// Expected CSV format (Kalico `save_calibration_data` output):
/// ```csv
/// freq,psd_x,psd_y,psd_z,psd_xyz,accel_per_hz
/// 0.0,1.23e-4,2.34e-4,3.45e-4,6.02e-4,100.0
/// ```
#[derive(Debug, Clone)]
pub struct PsdInput {
    pub bins: Vec<PsdBin>,
    pub accel_per_hz: Option<f64>,
    pub(crate) normalized: bool,
}

impl PsdInput {
    /// Load PSD data from a Kalico-format CSV file.
    ///
    /// Auto-detects the input format:
    /// - PSD CSV: `freq,psd_x,psd_y,psd_z,psd_xyz,accel_per_hz`
    /// - Raw accelerometer: `#time,accel_x,accel_y,accel_z` (parsed and converted to PSD)
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_csv_with_window(path, 0.5)
    }

    /// Load PSD or raw accelerometer data, using `window_t_sec` for raw CSV input.
    pub fn from_csv_with_window<P: AsRef<Path>>(path: P, window_t_sec: f64) -> Result<Self> {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);

        // Peek at the first line to detect format (including comment headers)
        let mut first_line = String::new();
        let n = reader.read_line(&mut first_line)?;
        if n == 0 {
            return Err(ShaperError::InvalidInput("Empty file".to_string()));
        }

        let trimmed_first = first_line.trim();

        // Check if this is a raw accelerometer file (auto-detect and convert)
        // Raw files start with "#time,accel_x,accel_y,accel_z" comment header
        let is_raw = trimmed_first.starts_with("#time,accel_x,accel_y,accel_z")
            || trimmed_first.starts_with("time,accel_x,accel_y,accel_z");
        if is_raw {
            return Self::from_raw_csv_streaming(&path, window_t_sec);
        }

        // Check if this is a PSD file
        // Accept both the standard Kalico header and the variant with shaper columns
        // e.g. "freq,psd_x,psd_y,psd_z,psd_xyz,accel_per_hz" or "freq,psd_x,psd_y,psd_z,psd_xyz,mzv(46.0)"
        let is_psd = trimmed_first.starts_with("freq,psd_x,psd_y,psd_z,psd_xyz");

        if !is_psd {
            return Err(ShaperError::InvalidInput(format!(
                "Expected PSD CSV header (freq,psd_x,...) or raw accelerometer header (#time,accel_x,...), got: {}",
                trimmed_first.chars().take(60).collect::<String>()
            )));
        }

        // Parse the CSV data
        let file = File::open(&path)?;
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .comment(Some(b'#'))
            .from_reader(file);

        let mut bins = Vec::new();
        let mut accel_per_hz = None;

        for (idx, result) in csv_reader.records().enumerate() {
            let record = result?;
            if idx == 0 {
                // Skip header row - check if first field is the literal header text
                let first = record.get(0).unwrap_or("");
                if first == "freq" || first.starts_with("freq,psd") {
                    continue;
                }
            }

            let row = idx + 1;
            let freq = parse_non_negative_field(&record, 0, row, "frequency")?;
            let psd_x = parse_non_negative_field(&record, 1, row, "psd_x")?;
            let psd_y = parse_non_negative_field(&record, 2, row, "psd_y")?;
            let psd_z = parse_non_negative_field(&record, 3, row, "psd_z")?;

            // Column 4 is psd_sum (redundant, we compute it)
            // Column 5 is accel_per_hz; keep the first parseable finite value.
            if accel_per_hz.is_none() {
                accel_per_hz = parse_optional_non_negative_field(&record, 5, row, "accel_per_hz")?;
            }

            bins.push(PsdBin::new(freq, psd_x, psd_y, psd_z));
        }

        if bins.is_empty() {
            return Err(ShaperError::InsufficientData(
                "No PSD bins found in file".to_string(),
            ));
        }

        // Sort by frequency (just in case)
        bins.sort_by(|a, b| a.freq.total_cmp(&b.freq));

        Ok(Self {
            bins,
            accel_per_hz,
            normalized: false,
        })
    }

    /// Load raw accelerometer CSV and convert to PSD using streaming Welch.
    ///
    /// Two-pass approach:
    /// 1. Scan metadata (sample count, first/last timestamp → fs, nfft).
    /// 2. Stream windows, accumulate PSD for X/Y/Z without storing full sample set.
    pub fn from_raw_csv_streaming<P: AsRef<Path>>(path: P, window_t_sec: f64) -> Result<Self> {
        let meta = scan_raw_csv_metadata(&path)?;

        let m = welch_window_size(meta.sampling_freq, window_t_sec)?;

        if meta.samples <= m {
            return Err(ShaperError::InsufficientData(format!(
                "Need more than {} samples for window size {}",
                meta.samples, m
            )));
        }

        // Build reusable Welch accumulator
        let overlap = m / 2;
        let step = m - overlap;

        // Kaiser window
        let window: Vec<f64> = (0..m)
            .map(|i| {
                let x = 2.0 * (i as f64) / ((m - 1) as f64) - 1.0;
                bessel_i0(6.0 * (1.0 - x * x).sqrt()) / bessel_i0(6.0)
            })
            .collect();

        let window_sum_sq: f64 = window.iter().map(|w| w * w).sum();
        let scale = 1.0 / window_sum_sq;

        let df = meta.sampling_freq / m as f64;

        // Accumulators for PSD bins
        let n_bins = m / 2 + 1;
        let mut psd_x = vec![0.0; n_bins];
        let mut psd_y = vec![0.0; n_bins];
        let mut psd_z = vec![0.0; n_bins];
        let mut count = 0usize;

        // FFT planner and scratch buffer
        use rustfft::FftPlanner;
        use rustfft::num_complex::Complex;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(m);
        let mut fft_input: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); m];

        // Second pass: stream windows
        let file = File::open(&path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .comment(Some(b'#'))
            .from_reader(file);

        // Rolling buffer for one window of samples per axis
        let mut x_buf: Vec<f64> = Vec::with_capacity(m + step);
        let mut y_buf: Vec<f64> = Vec::with_capacity(m + step);
        let mut z_buf: Vec<f64> = Vec::with_capacity(m + step);

        for (total_parsed, result) in reader.records().enumerate() {
            let record = result?;
            if total_parsed == 0 && is_raw_header(&record) {
                continue;
            }

            let row = total_parsed + 1;
            let x = parse_finite_field(&record, 1, row, "accel_x")?;
            let y = parse_finite_field(&record, 2, row, "accel_y")?;
            let z = parse_finite_field(&record, 3, row, "accel_z")?;

            x_buf.push(x);
            y_buf.push(y);
            z_buf.push(z);

            // When we have enough samples for a full window, process it
            if x_buf.len() >= m {
                process_window(
                    WindowData {
                        x: &x_buf[..m],
                        y: &y_buf[..m],
                        z: &z_buf[..m],
                    },
                    &window,
                    &fft,
                    &mut fft_input,
                    PsdAccum {
                        x: &mut psd_x[..],
                        y: &mut psd_y[..],
                        z: &mut psd_z[..],
                    },
                );
                count += 1;

                // Slide: shift remaining samples to the front in place.
                let new_len = x_buf.len() - step;
                x_buf.copy_within(step.., 0);
                x_buf.truncate(new_len);
                y_buf.copy_within(step.., 0);
                y_buf.truncate(new_len);
                z_buf.copy_within(step.., 0);
                z_buf.truncate(new_len);
            }
        }

        if count == 0 {
            return Err(ShaperError::Math("No valid windows".to_string()));
        }

        // Build PSD bins from accumulators
        let mut bins = Vec::with_capacity(n_bins);
        for i in 0..n_bins {
            let mut px = psd_x[i] / count as f64;
            let mut py = psd_y[i] / count as f64;
            let mut pz = psd_z[i] / count as f64;
            px *= scale / meta.sampling_freq;
            py *= scale / meta.sampling_freq;
            pz *= scale / meta.sampling_freq;

            // Double non-DC, non-Nyquist terms for one-sided PSD
            if i > 0 && i < m / 2 {
                px *= 2.0;
                py *= 2.0;
                pz *= 2.0;
            }

            bins.push(PsdBin::new(i as f64 * df, px, py, pz));
        }

        Ok(Self {
            bins,
            accel_per_hz: None,
            normalized: false,
        })
    }

    /// Normalize PSD values to frequencies (Kalico-style).
    pub fn normalize(&mut self) {
        for bin in &mut self.bins {
            bin.normalize();
        }
        self.normalized = true;
    }

    /// Suppress low-frequency noise. Automatically normalizes first if needed
    /// because the suppression factor is calibrated against normalized PSD.
    pub fn suppress_low_freq(&mut self, min_freq: f64) {
        if !self.normalized {
            self.normalize();
        }
        for bin in &mut self.bins {
            bin.suppress_low_freq(min_freq);
        }
    }

    /// Get the maximum PSD value (used for vibration threshold).
    pub fn max_psd(&self) -> f64 {
        self.bins.iter().map(|b| b.psd_sum).fold(0.0, f64::max)
    }

    /// Filter bins to a maximum frequency.
    pub fn truncate_to_max_freq(&mut self, max_freq: f64) {
        self.bins.retain(|b| b.freq <= max_freq);
    }
}

fn parse_finite_field(record: &StringRecord, index: usize, row: usize, name: &str) -> Result<f64> {
    let raw = record
        .get(index)
        .ok_or_else(|| ShaperError::InvalidInput(format!("Missing {name} at row {row}")))?;
    let value: f64 = raw
        .trim()
        .parse()
        .map_err(|_| ShaperError::InvalidInput(format!("Invalid {name} at row {row}: {raw}")))?;

    if value.is_finite() {
        Ok(value)
    } else {
        Err(ShaperError::InvalidInput(format!(
            "Invalid {name} at row {row}: non-finite value {raw}"
        )))
    }
}

fn parse_non_negative_field(
    record: &StringRecord,
    index: usize,
    row: usize,
    name: &str,
) -> Result<f64> {
    let value = parse_finite_field(record, index, row, name)?;
    if value >= 0.0 {
        Ok(value)
    } else {
        Err(ShaperError::InvalidInput(format!(
            "Invalid {name} at row {row}: expected non-negative value, got {value}"
        )))
    }
}

fn parse_optional_non_negative_field(
    record: &StringRecord,
    index: usize,
    row: usize,
    name: &str,
) -> Result<Option<f64>> {
    let Some(raw) = record.get(index) else {
        return Ok(None);
    };
    if raw.trim().is_empty() {
        return Ok(None);
    }

    parse_non_negative_field(record, index, row, name).map(Some)
}

fn is_raw_header(record: &StringRecord) -> bool {
    record
        .get(0)
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("time"))
        && record
            .get(1)
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("accel_x"))
        && record
            .get(2)
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("accel_y"))
        && record
            .get(3)
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("accel_z"))
}

fn welch_window_size(fs: f64, window_t_sec: f64) -> Result<usize> {
    if !fs.is_finite() || fs <= 0.0 {
        return Err(ShaperError::InvalidInput(format!(
            "Invalid sampling frequency: {fs}"
        )));
    }
    if !window_t_sec.is_finite() || window_t_sec <= 0.0 {
        return Err(ShaperError::InvalidInput(format!(
            "Invalid window time: {window_t_sec}"
        )));
    }

    let window_samples = (fs * window_t_sec).ceil();
    if !window_samples.is_finite() || window_samples < 2.0 {
        return Err(ShaperError::InvalidInput(format!(
            "Window time {window_t_sec} is too short for sampling frequency {fs}"
        )));
    }
    if window_samples > (usize::MAX / 2) as f64 {
        return Err(ShaperError::InvalidInput(
            "Window size is too large for this platform".to_string(),
        ));
    }

    Ok((window_samples as usize).next_power_of_two())
}

/// Metadata from a raw CSV scan (first pass).
struct RawCsvMetadata {
    samples: usize,
    sampling_freq: f64,
}

/// Scan a raw CSV to compute sample count and sampling frequency without storing samples.
fn scan_raw_csv_metadata<P: AsRef<Path>>(path: P) -> Result<RawCsvMetadata> {
    let file = File::open(&path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(file);

    let mut first_time: Option<f64> = None;
    let mut previous_time: Option<f64> = None;
    let mut last_time: f64 = 0.0;
    let mut count = 0usize;

    for (idx, result) in reader.records().enumerate() {
        let record = result?;
        if idx == 0 && is_raw_header(&record) {
            continue;
        }

        let row = idx + 1;
        let time = parse_finite_field(&record, 0, row, "time")?;
        if let Some(previous) = previous_time.filter(|previous| time <= *previous) {
            return Err(ShaperError::InvalidInput(format!(
                "Timestamps must be strictly increasing at row {row}: {time} <= {previous}"
            )));
        }

        if first_time.is_none() {
            first_time = Some(time);
        }
        previous_time = Some(time);
        last_time = time;
        count += 1;
    }

    let first = first_time
        .ok_or_else(|| ShaperError::InsufficientData("No data rows in raw CSV".to_string()))?;

    if count < 2 {
        return Err(ShaperError::InsufficientData(
            "Need at least 2 raw samples".to_string(),
        ));
    }

    let duration = last_time - first;
    if !duration.is_finite() || duration <= 0.0 {
        return Err(ShaperError::InvalidInput(
            "Raw CSV timestamps must span a positive duration".to_string(),
        ));
    }

    let fs = (count - 1) as f64 / duration;
    if !fs.is_finite() || fs <= 0.0 {
        return Err(ShaperError::InvalidInput(format!(
            "Invalid sampling frequency calculated from raw CSV: {fs}"
        )));
    }

    Ok(RawCsvMetadata {
        samples: count,
        sampling_freq: fs,
    })
}

struct WindowData<'a> {
    x: &'a [f64],
    y: &'a [f64],
    z: &'a [f64],
}

struct PsdAccum<'a> {
    x: &'a mut [f64],
    y: &'a mut [f64],
    z: &'a mut [f64],
}

/// Process one window of X/Y/Z data: detrend, window, FFT, accumulate PSD.
fn process_window(
    data: WindowData<'_>,
    window: &[f64],
    fft: &std::sync::Arc<dyn rustfft::Fft<f64>>,
    fft_input: &mut [rustfft::num_complex::Complex<f64>],
    psd: PsdAccum<'_>,
) {
    process_axis(data.x, window, fft, fft_input, psd.x);
    process_axis(data.y, window, fft, fft_input, psd.y);
    process_axis(data.z, window, fft, fft_input, psd.z);
}

fn process_axis(
    data: &[f64],
    window: &[f64],
    fft: &std::sync::Arc<dyn rustfft::Fft<f64>>,
    fft_input: &mut [rustfft::num_complex::Complex<f64>],
    psd_acc: &mut [f64],
) {
    debug_assert_eq!(data.len(), window.len());
    debug_assert_eq!(fft_input.len(), window.len());

    let n_bins = window.len() / 2 + 1;
    debug_assert_eq!(psd_acc.len(), n_bins);

    let mean: f64 = data.iter().sum::<f64>() / data.len() as f64;
    for ((slot, sample), weight) in fft_input.iter_mut().zip(data.iter()).zip(window.iter()) {
        *slot = rustfft::num_complex::Complex::new((*sample - mean) * *weight, 0.0);
    }

    fft.process(fft_input);

    for (acc, bin) in psd_acc.iter_mut().zip(fft_input.iter()).take(n_bins) {
        *acc += bin.norm_sqr();
    }
}

/// Raw accelerometer data (time, accel_x, accel_y, accel_z).
///
/// This is the input format from `TEST_RESONANCES OUTPUT=raw_data`.
#[derive(Debug, Clone)]
pub struct RawInput {
    samples: Vec<(f64, f64, f64, f64)>, // time, x, y, z
}

impl RawInput {
    /// Load raw accelerometer data from a CSV file.
    ///
    /// Expected format:
    /// ```csv
    /// #time,accel_x,accel_y,accel_z
    /// 0.0,0.001,0.002,9.81
    /// 0.0005,0.0011,0.0021,9.81
    /// ```
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(&path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .comment(Some(b'#'))
            .from_reader(file);

        let mut samples = Vec::new();
        let mut previous_time = None;

        for (idx, result) in reader.records().enumerate() {
            let record = result?;
            if idx == 0 && is_raw_header(&record) {
                continue;
            }

            let row = idx + 1;
            let time = parse_finite_field(&record, 0, row, "time")?;
            if let Some(previous) = previous_time.filter(|previous| time <= *previous) {
                return Err(ShaperError::InvalidInput(format!(
                    "Timestamps must be strictly increasing at row {row}: {time} <= {previous}"
                )));
            }

            let x = parse_finite_field(&record, 1, row, "accel_x")?;
            let y = parse_finite_field(&record, 2, row, "accel_y")?;
            let z = parse_finite_field(&record, 3, row, "accel_z")?;

            previous_time = Some(time);
            samples.push((time, x, y, z));
        }

        if samples.len() < 2 {
            return Err(ShaperError::InsufficientData(
                "Need at least 2 samples".to_string(),
            ));
        }

        Ok(Self { samples })
    }

    /// Borrow the raw samples as `(time, accel_x, accel_y, accel_z)` tuples.
    pub fn samples(&self) -> &[(f64, f64, f64, f64)] {
        &self.samples
    }

    /// Compute the sampling frequency from the data.
    pub fn sampling_freq(&self) -> f64 {
        let first = self.samples[0].0;
        let last = self.samples[self.samples.len() - 1].0;
        (self.samples.len() - 1) as f64 / (last - first)
    }

    /// Convert raw data to PSD using Welch's method.
    ///
    /// This is the streaming version that processes data in overlapping windows
    /// without materializing the full window matrix.
    pub fn to_psd(&self, window_t_sec: f64) -> Result<PsdInput> {
        let fs = self.sampling_freq();
        let n = self.samples.len();
        let m = welch_window_size(fs, window_t_sec)?;

        if n <= m {
            return Err(ShaperError::InsufficientData(format!(
                "Need more than {} samples for window size {}",
                n, m
            )));
        }

        // Extract individual axis data
        let x_data: Vec<f64> = self.samples.iter().map(|s| s.1).collect();
        let y_data: Vec<f64> = self.samples.iter().map(|s| s.2).collect();
        let z_data: Vec<f64> = self.samples.iter().map(|s| s.3).collect();

        // Compute PSD for each axis
        let (freqs, psd_x) = welch_psd(&x_data, fs, m)?;
        let (_, psd_y) = welch_psd(&y_data, fs, m)?;
        let (_, psd_z) = welch_psd(&z_data, fs, m)?;

        let bins = freqs
            .into_iter()
            .zip(psd_x)
            .zip(psd_y)
            .zip(psd_z)
            .map(|(((f, px), py), pz)| PsdBin::new(f, px, py, pz))
            .collect();

        Ok(PsdInput {
            bins,
            accel_per_hz: None,
            normalized: false,
        })
    }
}

/// Compute Welch's PSD estimate for a single axis.
///
/// Uses Kaiser window with beta=6.0, 50% overlap.
/// Returns (frequencies, psd_values).
fn welch_psd(data: &[f64], fs: f64, nfft: usize) -> Result<(Vec<f64>, Vec<f64>)> {
    use rustfft::FftPlanner;
    use rustfft::num_complex::Complex;

    let window_size = nfft;
    let overlap = nfft / 2;
    let step = window_size - overlap;

    let n_windows = (data.len() - overlap) / step;
    if n_windows == 0 {
        return Err(ShaperError::InsufficientData(
            "Not enough data for even one window".to_string(),
        ));
    }

    // Kaiser window with beta=6.0
    let window: Vec<f64> = (0..window_size)
        .map(|i| {
            let x = 2.0 * (i as f64) / ((window_size - 1) as f64) - 1.0;
            bessel_i0(6.0 * (1.0 - x * x).sqrt()) / bessel_i0(6.0)
        })
        .collect();

    let window_sum_sq: f64 = window.iter().map(|w| w * w).sum();
    let scale = 1.0 / window_sum_sq;

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(nfft);

    // Reusable scratch buffers
    let mut psd_acc = vec![0.0; nfft / 2 + 1];
    let mut fft_input: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); nfft];
    let mut count = 0usize;

    for w in 0..n_windows {
        let start = w * step;
        let end = start + window_size;
        if end > data.len() {
            break;
        }

        // Detrend and window directly into the FFT input buffer.
        let mean: f64 = data[start..end].iter().sum::<f64>() / window_size as f64;
        for ((slot, sample), weight) in fft_input
            .iter_mut()
            .zip(data[start..end].iter())
            .zip(window.iter())
        {
            *slot = Complex::new((*sample - mean) * *weight, 0.0);
        }

        fft.process(&mut fft_input);

        for (acc, bin) in psd_acc.iter_mut().zip(fft_input.iter()).take(nfft / 2 + 1) {
            *acc += bin.norm_sqr();
        }
        count += 1;
    }

    if count == 0 {
        return Err(ShaperError::Math("No valid windows".to_string()));
    }

    // Average and scale
    let df = fs / nfft as f64;
    let mut freqs = Vec::with_capacity(nfft / 2 + 1);
    let mut psd = Vec::with_capacity(nfft / 2 + 1);

    for (i, acc) in psd_acc.iter().enumerate().take(nfft / 2 + 1) {
        let mut val = acc / count as f64;
        val *= scale / fs;

        // Double non-DC, non-Nyquist terms for one-sided PSD
        if i > 0 && i < nfft / 2 {
            val *= 2.0;
        }

        freqs.push(i as f64 * df);
        psd.push(val);
    }

    Ok((freqs, psd))
}

/// Modified Bessel function of the first kind, order 0.
/// Used for Kaiser window computation.
fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;

    // Series expansion: I_0(x) = sum_{k=0}^inf (x^2/4)^k / (k!)^2
    for k in 1..20 {
        term *= (x * x) / (4.0 * (k as f64) * (k as f64));
        sum += term;
        if term.abs() < 1e-15 {
            break;
        }
    }

    sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_bessel_i0() {
        // I_0(0) = 1
        assert_relative_eq!(bessel_i0(0.0), 1.0, epsilon = 1e-10);
        // I_0(6.0) ≈ 67.23 (from scipy.special.i0)
        let expected = 67.234406976476;
        assert_relative_eq!(bessel_i0(6.0), expected, epsilon = 1e-6);
    }

    #[test]
    fn test_kaiser_window() {
        let beta = 6.0;
        let n = 256;
        let window: Vec<f64> = (0..n)
            .map(|i| {
                let x = 2.0 * (i as f64) / ((n - 1) as f64) - 1.0;
                bessel_i0(beta * (1.0 - x * x).sqrt()) / bessel_i0(beta)
            })
            .collect();

        // Window should be symmetric
        for i in 0..n / 2 {
            assert_relative_eq!(window[i], window[n - 1 - i], epsilon = 1e-10);
        }

        // Center should be close to 1.0 (within numerical precision of Bessel ratio)
        assert_relative_eq!(window[n / 2], 1.0, epsilon = 1e-4);

        // Edges should be small but not necessarily near 0 for small beta
        assert!(window[0] < 0.5);
    }

    #[test]
    fn test_welch_psd_sine_wave() {
        // Generate a known sine wave at 50 Hz, sampled at 1000 Hz
        let fs = 1000.0;
        let freq = 50.0;
        let duration = 2.0;
        let n = (fs * duration) as usize;

        let data: Vec<f64> = (0..n)
            .map(|i| {
                let t = i as f64 / fs;
                (2.0 * std::f64::consts::PI * freq * t).sin()
            })
            .collect();

        let nfft = 256;
        let (freqs, psd) = welch_psd(&data, fs, nfft).unwrap();

        // Find peak frequency
        let max_idx = psd
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        let peak_freq = freqs[max_idx];

        // Should be close to 50 Hz
        assert_relative_eq!(peak_freq, 50.0, epsilon = 5.0);
    }

    #[test]
    fn from_csv_accepts_plain_raw_header() {
        let mut csv_data = "time,accel_x,accel_y,accel_z\n".to_string();
        for i in 0..100 {
            let t = i as f64 / 1000.0;
            csv_data.push_str(&format!("{t:.6},0.0,0.0,0.0\n"));
        }

        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmpfile.path(), csv_data).unwrap();

        let psd = PsdInput::from_csv_with_window(tmpfile.path(), 0.01).unwrap();
        assert!(!psd.bins.is_empty());
    }

    #[test]
    fn raw_csv_rejects_non_increasing_timestamps() {
        let csv_data = "time,accel_x,accel_y,accel_z\n0.0,0.0,0.0,0.0\n0.0,0.0,0.0,0.0\n";
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmpfile.path(), csv_data).unwrap();

        let err = PsdInput::from_raw_csv_streaming(tmpfile.path(), 0.01)
            .err()
            .unwrap();
        assert!(err.to_string().contains("strictly increasing"));
    }

    #[test]
    fn psd_csv_rejects_non_finite_values() {
        let csv_data = "freq,psd_x,psd_y,psd_z,psd_xyz,accel_per_hz\nNaN,1.0,1.0,1.0,3.0,100.0\n";
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmpfile.path(), csv_data).unwrap();

        let err = PsdInput::from_csv(tmpfile.path()).err().unwrap();
        assert!(err.to_string().contains("non-finite"));
    }

    #[test]
    fn test_streaming_raw_psd_matches_in_memory_path() {
        // Generate synthetic 50 Hz data with three axes
        let fs = 1000.0;
        let freq = 50.0;
        let duration = 2.0;
        let n = (fs * duration) as usize;

        let mut csv_data = "#time,accel_x,accel_y,accel_z\n".to_string();
        for i in 0..n {
            let t = i as f64 / fs;
            let x = (2.0 * std::f64::consts::PI * freq * t).sin();
            let y = (2.0 * std::f64::consts::PI * freq * t).cos();
            let z = 0.0;
            csv_data.push_str(&format!("{:.6},{:.6},{:.6},{:.6}\n", t, x, y, z));
        }

        // Write to temp file
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmpfile.path(), &csv_data).unwrap();

        // In-memory path
        let raw = RawInput::from_csv(tmpfile.path()).unwrap();
        let psd_mem = raw.to_psd(0.5).unwrap();

        // Streaming path
        let psd_stream = PsdInput::from_raw_csv_streaming(tmpfile.path(), 0.5).unwrap();

        // Compare peak frequencies
        let peak_mem = psd_mem
            .bins
            .iter()
            .max_by(|a, b| a.psd_sum.partial_cmp(&b.psd_sum).unwrap())
            .unwrap()
            .freq;
        let peak_stream = psd_stream
            .bins
            .iter()
            .max_by(|a, b| a.psd_sum.partial_cmp(&b.psd_sum).unwrap())
            .unwrap()
            .freq;

        assert_relative_eq!(peak_mem, peak_stream, epsilon = 1.0);
        assert_eq!(psd_mem.bins.len(), psd_stream.bins.len());

        // Compare first and last bin frequencies
        assert_relative_eq!(
            psd_mem.bins[0].freq,
            psd_stream.bins[0].freq,
            epsilon = 1e-6
        );
        assert_relative_eq!(
            psd_mem.bins.last().unwrap().freq,
            psd_stream.bins.last().unwrap().freq,
            epsilon = 1e-6
        );
    }

    #[test]
    fn test_streaming_raw_psd_rejects_short_input() {
        // Create a tiny CSV that won't pass window requirements
        let csv_data = "#time,accel_x,accel_y,accel_z\n0.0,0.0,0.0,0.0\n0.001,0.0,0.0,0.0\n";
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmpfile.path(), csv_data).unwrap();

        let result = PsdInput::from_raw_csv_streaming(tmpfile.path(), 0.5);
        assert!(result.is_err(), "Expected InsufficientData for short input");
    }

    #[test]
    fn suppress_low_freq_auto_normalizes() {
        let mut psd = PsdInput {
            bins: vec![
                PsdBin::new(1.0, 10.0, 0.0, 0.0),
                PsdBin::new(100.0, 1.0, 0.0, 0.0),
            ],
            accel_per_hz: None,
            normalized: false,
        };

        // Calling suppress_low_freq without an explicit normalize() should
        // still produce the same result as normalize() + suppress_low_freq().
        let mut expected = psd.clone();
        expected.normalize();
        expected.suppress_low_freq(5.0);

        psd.suppress_low_freq(5.0);
        assert!(psd.normalized);
        assert_eq!(psd.bins.len(), expected.bins.len());
        for (actual, expected) in psd.bins.iter().zip(expected.bins.iter()) {
            assert_relative_eq!(actual.psd_sum, expected.psd_sum, epsilon = 1e-12);
        }
    }
}
