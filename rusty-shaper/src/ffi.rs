//! C-ABI entrypoints for Kalico's Python ctypes integration.
//!
//! Kalico calls these from `shaper_calibrate.py` via `ctypes.CDLL` when
//! `calibration_backend = rusty` is configured.  The interface is
//! deliberately narrow: all inputs arrive as null-terminated C strings or
//! plain `f64` values, and results are returned as a heap-allocated JSON
//! string that the caller must free with `rusty_shaper_free_string`.
//!
//! # Safety
//! All `*const libc::c_char` parameters are checked for null before
//! conversion.  The returned `*mut libc::c_char` is always either a valid
//! `CString` allocation or null on error, never an interior pointer.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

use crate::{
    input::PsdInput,
    models::shaper_by_name,
    scorer::{ShaperCalibrator, MIN_FREQ},
    types::CalibrationOutput,
};

// ──────────────────────────────────────────────────────────────────────────
// Internal helpers
// ──────────────────────────────────────────────────────────────────────────

/// Convert a nullable `*const c_char` to an owned `String`, returning `None`
/// for null pointers and invalid UTF-8.
unsafe fn maybe_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

/// Parse a comma-separated shaper list (e.g. `"zv,mzv,ei"`) into the
/// `ShaperModel` boxes expected by `ShaperCalibrator`.
fn parse_shapers(s: &str) -> Option<Vec<Box<dyn crate::ShaperModel>>> {
    let names: Vec<&str> = s.split(',').map(str::trim).filter(|n| !n.is_empty()).collect();
    if names.is_empty() {
        return None;
    }
    if names.iter().any(|n| *n == "all") {
        return Some(crate::models::all_shapers_with_zvd());
    }
    let mut out = Vec::new();
    for name in names {
        out.push(shaper_by_name(name)?);
    }
    Some(out)
}

/// Build a `ShaperCalibrator` from the common FFI parameters.
///
/// # Parameters (all nullable / sentinel-valued)
/// * `shapers_csv`    – comma-separated shaper names, e.g. `"zv,mzv,ei"`
///                      (null → Kalico default set)
/// * `damping_ratio`  – e.g. `0.1`  (≤ 0 → default 0.1)
/// * `test_dr_csv`    – comma-separated test damping ratios
///                      (null → `"0.075,0.1,0.15"`)
/// * `scv`            – square corner velocity mm/s  (≤ 0 → 5.0)
/// * `max_smoothing`  – mm  (≤ 0 → unconstrained)
/// * `max_freq`       – Hz  (≤ 0 → 200.0)
/// * `freq_range_csv` – `"start:end:step"` Hz  (null → shaper default)
fn build_calibrator(
    shapers_csv: Option<String>,
    damping_ratio: f64,
    test_dr_csv: Option<String>,
    scv: f64,
    max_smoothing: f64,
    max_freq: f64,
    freq_range_csv: Option<String>,
) -> Result<ShaperCalibrator, String> {
    const DEFAULT_SHAPERS: &str = "zv,mzv,ei,2hump_ei,3hump_ei";
    const DEFAULT_TEST_DR: &str = "0.075,0.1,0.15";

    let shapers_str = shapers_csv.as_deref().unwrap_or(DEFAULT_SHAPERS);
    let shapers =
        parse_shapers(shapers_str).ok_or_else(|| format!("Invalid shapers: {shapers_str}"))?;

    let dr = if damping_ratio > 0.0 { damping_ratio } else { 0.1 };

    let test_dr_str = test_dr_csv.as_deref().unwrap_or(DEFAULT_TEST_DR);
    let test_drs: Vec<f64> = test_dr_str
        .split(',')
        .map(str::trim)
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    if test_drs.is_empty() {
        return Err(format!("Invalid test_damping_ratios: {test_dr_str}"));
    }

    let scv_val = if scv > 0.0 { scv } else { 5.0 };
    let max_freq_val = if max_freq > 0.0 { max_freq } else { 200.0 };

    let mut cal = ShaperCalibrator::new()
        .with_damping_ratio(dr)
        .with_test_damping_ratios(test_drs)
        .with_scv(scv_val)
        .with_max_freq(max_freq_val);

    if max_smoothing > 0.0 {
        cal = cal.with_max_smoothing(max_smoothing);
    }

    if let Some(ref range) = freq_range_csv {
        let parts: Vec<f64> = range
            .split(':')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();
        if parts.len() == 3 && parts[0] > 0.0 && parts[1] > parts[0] && parts[2] > 0.0 {
            cal = cal.with_freq_range(parts[0], parts[1], parts[2]);
        } else {
            return Err(format!("Invalid freq_range '{range}': expected start:end:step"));
        }
    }

    for shaper in shapers {
        cal = cal.with_shaper(shaper);
    }

    Ok(cal)
}

/// Serialize a `CalibrationOutput` to a heap-allocated C string (JSON).
/// Returns null on serialisation failure.
fn result_to_cstring(output: &CalibrationOutput) -> *mut c_char {
    match serde_json::to_string(output) {
        Ok(json) => match CString::new(json) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Public FFI surface
// ──────────────────────────────────────────────────────────────────────────

/// Calibrate from a **raw accelerometer CSV** file.
///
/// # Parameters
/// * `csv_path`        – null-terminated path to the raw CSV file
/// * `shapers_csv`     – comma-separated shaper names (null → default set)
/// * `damping_ratio`   – nominal damping ratio (≤ 0 → 0.1)
/// * `test_dr_csv`     – comma-separated test damping ratios (null → default)
/// * `scv`             – square corner velocity mm/s (≤ 0 → 5.0)
/// * `max_smoothing`   – max smoothing mm (≤ 0 → unconstrained)
/// * `max_freq`        – max PSD frequency Hz (≤ 0 → 200.0)
/// * `freq_range_csv`  – `"start:end:step"` Hz (null → shaper default)
/// * `window_t`        – Welch window duration seconds (≤ 0 → 0.5)
///
/// # Returns
/// Pointer to a heap-allocated, null-terminated JSON string, or null on
/// error.  **Must be freed with `rusty_shaper_free_string`.**
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rusty_shaper_calibrate_from_csv(
    csv_path: *const c_char,
    shapers_csv: *const c_char,
    damping_ratio: f64,
    test_dr_csv: *const c_char,
    scv: f64,
    max_smoothing: f64,
    max_freq: f64,
    freq_range_csv: *const c_char,
    window_t: f64,
) -> *mut c_char {
    let path_str = match unsafe { maybe_str(csv_path) } {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let wt = if window_t > 0.0 { window_t } else { 0.5 };

    let calibrator = match build_calibrator(
        unsafe { maybe_str(shapers_csv) },
        damping_ratio,
        unsafe { maybe_str(test_dr_csv) },
        scv,
        max_smoothing,
        max_freq,
        unsafe { maybe_str(freq_range_csv) },
    ) {
        Ok(c) => c,
        Err(_) => return std::ptr::null_mut(),
    };

    let mut psd = match PsdInput::from_raw_csv_streaming(Path::new(&path_str), wt) {
        Ok(p) => p,
        Err(_) => return std::ptr::null_mut(),
    };
    psd.normalize();
    psd.suppress_low_freq(MIN_FREQ);

    match calibrator.fit(&psd) {
        Ok(ref output) => result_to_cstring(output),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Calibrate from a **pre-computed PSD CSV** file (Kalico resonances output).
///
/// Parameters are identical to `rusty_shaper_calibrate_from_csv` except
/// `window_t` is absent (no FFT windowing needed for pre-computed PSDs).
///
/// # Returns
/// Pointer to a heap-allocated, null-terminated JSON string, or null on
/// error.  **Must be freed with `rusty_shaper_free_string`.**
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rusty_shaper_calibrate_from_psd(
    csv_path: *const c_char,
    shapers_csv: *const c_char,
    damping_ratio: f64,
    test_dr_csv: *const c_char,
    scv: f64,
    max_smoothing: f64,
    max_freq: f64,
    freq_range_csv: *const c_char,
) -> *mut c_char {
    let path_str = match unsafe { maybe_str(csv_path) } {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let calibrator = match build_calibrator(
        unsafe { maybe_str(shapers_csv) },
        damping_ratio,
        unsafe { maybe_str(test_dr_csv) },
        scv,
        max_smoothing,
        max_freq,
        unsafe { maybe_str(freq_range_csv) },
    ) {
        Ok(c) => c,
        Err(_) => return std::ptr::null_mut(),
    };

    let mut psd = match PsdInput::from_csv_with_window(Path::new(&path_str), 0.5) {
        Ok(p) => p,
        Err(_) => return std::ptr::null_mut(),
    };
    psd.normalize();
    psd.suppress_low_freq(MIN_FREQ);

    match calibrator.fit(&psd) {
        Ok(ref output) => result_to_cstring(output),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string previously returned by one of the `rusty_shaper_calibrate_*`
/// functions.  Passing null is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rusty_shaper_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}
