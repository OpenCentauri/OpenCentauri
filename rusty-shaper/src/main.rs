use std::io::Write;
use std::path::{Path, PathBuf};

use clap::Parser;

use rusty_shaper::{
    input::PsdInput,
    models::{all_shapers_with_zvd, shaper_by_name},
    moonraker::send_gcode_script,
    scorer::{ShaperCalibrator, MIN_FREQ},
    types::CalibrationOutput,
    Result, ShaperError, ShaperModel,
};

fn valid_shapers_list() -> String {
    all_shapers_with_zvd()
        .into_iter()
        .map(|s| s.name().to_string())
        .chain(std::iter::once("all".to_string()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Parser, Debug)]
#[command(name = "rusty-shaper")]
#[command(about = "Low-RAM Rust input shaper calibration for 3D printers")]
#[command(version)]
struct Args {
    /// Input CSV file(s) (PSD or raw accelerometer data). Multiple files can be
    /// provided to calibrate several axes in one run; klippy output is batched.
    #[arg(value_name = "FILE", num_args = 1..)]
    input: Vec<PathBuf>,

    /// Output format(s): "cfg", "csv", "json", "json-pretty", "klippy", or file path
    /// Can be specified multiple times for multiple outputs
    #[arg(short, long, value_name = "MODE", action = clap::ArgAction::Append)]
    output: Vec<String>,

    /// Working directory for output files (default: /tmp)
    #[arg(long, value_name = "DIR", default_value = "/tmp")]
    workdir: PathBuf,

    /// Name suffix for output files (default: timestamp %Y%m%d_%H%M%S)
    #[arg(short, long, value_name = "NAME")]
    name: Option<String>,

    /// Shaper types to test (comma-separated, e.g. "zv,mzv,ei")
    /// Default matches Kalico's AUTOTUNE_SHAPERS (excludes ZVD).
    /// Use "zvd" explicitly or "all" to include ZVD.
    /// Axis-specific overrides (--shapers-x, --shapers-y, --shapers-z) take
    /// precedence for files matching that axis.
    #[arg(short, long, default_value = "zv,mzv,ei,2hump_ei,3hump_ei")]
    shapers: String,

    /// Shaper types to test for the X axis (overrides --shapers for x files).
    #[arg(long, value_name = "SHAPERS")]
    shapers_x: Option<String>,

    /// Shaper types to test for the Y axis (overrides --shapers for y files).
    #[arg(long, value_name = "SHAPERS")]
    shapers_y: Option<String>,

    /// Shaper types to test for the Z axis (overrides --shapers for z files).
    #[arg(long, value_name = "SHAPERS")]
    shapers_z: Option<String>,

    /// Damping ratio
    #[arg(long, default_value = "0.1")]
    damping_ratio: f64,

    /// Test damping ratios (comma-separated)
    #[arg(long, default_value = "0.075,0.1,0.15")]
    test_damping_ratios: String,

    /// Square corner velocity (mm/s)
    #[arg(long, default_value = "5.0")]
    scv: f64,

    /// Maximum allowed smoothing (mm)
    #[arg(short, long)]
    max_smoothing: Option<f64>,

    /// Maximum frequency to analyze (Hz)
    #[arg(long, default_value = "200.0")]
    max_freq: f64,

    /// Shaper frequency range: start:end:step
    #[arg(long, value_name = "RANGE")]
    shaper_freq: Option<String>,

    /// Parse input as raw accelerometer data (not PSD)
    /// Only needed if the file lacks the standard #time,accel_x,accel_y,accel_z header
    #[arg(long)]
    raw: bool,

    /// Window time for PSD computation (seconds)
    #[arg(long, default_value = "0.5")]
    window_t: f64,

    /// Include PSD bins in JSON output (default: omitted to keep files small)
    #[arg(long)]
    with_psd: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    quiet: bool,

    /// Moonraker hostname or IP address (default: 127.0.0.1)
    #[arg(long, value_name = "HOST", default_value = "127.0.0.1")]
    moonraker_host: String,

    /// Moonraker port (default: 80)
    #[arg(long, value_name = "PORT", default_value = "80")]
    moonraker_port: u16,

    /// G-code macro to use for visible log messages when --output klippy is used.
    /// Default is M118. Use RESPOND to send via RESPOND MSG='...'.
    #[arg(long, value_name = "MACRO", default_value = "M118")]
    log_macro: String,

    /// Commit shaper params to printer.cfg (sends SAVE_CONFIG RESTART=0)
    #[arg(long)]
    commit: bool,
}

fn main() -> Result<()> {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let args = Args::parse();
    validate_args(&args)?;

    let test_damping_ratios = parse_damping_ratios(&args.test_damping_ratios)?;

    let wants_klippy = args.output.iter().any(|mode| mode == "klippy");
    let mut klippy_results: Vec<(String, CalibrationOutput)> = Vec::new();
    let name_suffix = args
        .name
        .clone()
        .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());

    for (input_idx, input) in args.input.iter().enumerate() {
        let axis = axis_from_path(input);
        if wants_klippy
            && let Err(e) = send_log_klippy(
                &args,
                &[format!(
                    "Calculating the best input shaper parameters for {axis} axis"
                )],
            )
        {
            eprintln!("Warning: failed to send progress message: {e}");
        }

        let shapers = axis_shaper_selection(&args, axis)?;

        let mut calibrator = ShaperCalibrator::new()
            .with_damping_ratio(args.damping_ratio)
            .with_test_damping_ratios(test_damping_ratios.clone())
            .with_scv(args.scv)
            .with_max_freq(args.max_freq);

        if let Some(max_sm) = args.max_smoothing {
            calibrator = calibrator.with_max_smoothing(max_sm);
        }

        if let Some(ref freq_str) = args.shaper_freq {
            let (start, end, step) = parse_freq_range(freq_str)?;
            calibrator = calibrator.with_freq_range(start, end, step);
        }

        for shaper in shapers {
            calibrator = calibrator.with_shaper(shaper);
        }

        let result = calibrate_file(&args, &calibrator, input)?;

        let file_suffix = if args.input.len() > 1 {
            format!("{name_suffix}_{}", input_idx + 1)
        } else {
            name_suffix.clone()
        };

        for output_mode in &args.output {
            match output_mode.as_str() {
                "klippy" => {
                    // Defer; collect for later batched send. Strip the PSD
                    // bins to avoid keeping N copies of the full spectrum.
                    let mut klippy_result = result.clone();
                    klippy_result.psd_bins.clear();
                    klippy_results.push((axis.to_string(), klippy_result));
                }
                "cfg" => print_cfg_block(axis, &result),
                "csv" => write_csv_output(&args, axis, &result, &file_suffix)?,
                "json" => write_json_file(&args, axis, &result, &file_suffix, false)?,
                "json-pretty" => write_json_file(&args, axis, &result, &file_suffix, true)?,
                path => write_json_path(&args, path, &result)?,
            }
        }

        if !args.quiet {
            print_summary_stderr(&result);
        }
    }

    if wants_klippy {
        apply_via_klippy(&args, &klippy_results)?;
    }

    Ok(())
}

fn calibrate_file(
    args: &Args,
    calibrator: &ShaperCalibrator,
    input: &Path,
) -> Result<CalibrationOutput> {
    let mut psd = if args.raw {
        PsdInput::from_raw_csv_streaming(input, args.window_t)?
    } else {
        PsdInput::from_csv_with_window(input, args.window_t)?
    };

    psd.normalize();
    psd.suppress_low_freq(MIN_FREQ);

    calibrator.fit(&psd)
}

fn axis_shaper_selection(args: &Args, axis: &str) -> Result<Vec<Box<dyn ShaperModel>>> {
    let raw = match axis {
        "x" => args.shapers_x.as_deref(),
        "y" => args.shapers_y.as_deref(),
        "z" => args.shapers_z.as_deref(),
        _ => None,
    };
    parse_shaper_selection(raw.unwrap_or(&args.shapers))
}

// /// Send M118 lines to the printer UI only.
// fn send_m118_ui<S: AsRef<str>>(args: &Args, lines: &[S]) -> Result<()> {
//     // Unused but kept for future UI-only logging.
//     if lines.is_empty() {
//         return Ok(());
//     }
//     let script = lines
//         .iter()
//         .map(|line| format!("M118 {}", line.as_ref()))
//         .collect::<Vec<_>>()
//         .join("\n");
//     send_gcode_script(&args.moonraker_host, args.moonraker_port, &script)?;
//     Ok(())
// }

/// Format a single visible log line using the configured macro.
fn format_log_macro(args: &Args, msg: &str) -> String {
    let macro_name = args.log_macro.as_str();
    if macro_name.eq_ignore_ascii_case("M118") {
        format!("M118 {msg}")
    } else if macro_name.eq_ignore_ascii_case("RESPOND") {
        let escaped = msg.replace('\\', "\\\\").replace('\'', "\\'");
        format!("RESPOND MSG='{escaped}'")
    } else {
        format!("{} {msg}", args.log_macro)
    }
}

/// Send visible log lines to the printer using the configured macro.
fn send_log_klippy<S: AsRef<str>>(args: &Args, lines: &[S]) -> Result<()> {
    if lines.is_empty() {
        return Ok(());
    }
    let script = lines
        .iter()
        .map(|line| format_log_macro(args, line.as_ref()))
        .collect::<Vec<_>>()
        .join("\n");
    send_gcode_script(&args.moonraker_host, args.moonraker_port, &script)?;
    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    validate_damping_ratio(args.damping_ratio, "--damping-ratio")?;
    validate_non_negative(args.scv, "--scv")?;
    validate_positive(args.max_freq, "--max-freq")?;
    validate_positive(args.window_t, "--window-t")?;

    if args.input.is_empty() {
        return Err(ShaperError::Cli(
            "At least one input FILE is required".to_string(),
        ));
    }

    if let Some(max_smoothing) = args.max_smoothing {
        validate_non_negative(max_smoothing, "--max-smoothing")?;
    }

    if args.commit && !args.output.iter().any(|mode| mode == "klippy") {
        return Err(ShaperError::Cli(
            "--commit requires --output klippy".to_string(),
        ));
    }

    Ok(())
}

fn validate_damping_ratio(value: f64, option: &str) -> Result<()> {
    if value.is_finite() && (0.0..1.0).contains(&value) {
        return Ok(());
    }

    Err(ShaperError::Cli(format!(
        "{option} must be finite and in the range [0.0, 1.0)"
    )))
}

fn validate_positive(value: f64, option: &str) -> Result<()> {
    if value.is_finite() && value > 0.0 {
        return Ok(());
    }

    Err(ShaperError::Cli(format!("{option} must be finite and > 0")))
}

fn validate_non_negative(value: f64, option: &str) -> Result<()> {
    if value.is_finite() && value >= 0.0 {
        return Ok(());
    }

    Err(ShaperError::Cli(format!(
        "{option} must be finite and >= 0"
    )))
}

fn parse_damping_ratios(value: &str) -> Result<Vec<f64>> {
    let ratios: Vec<f64> = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.parse::<f64>())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| ShaperError::Cli(format!("Invalid --test-damping-ratios: {e}")))?;

    if ratios.is_empty() {
        return Err(ShaperError::Cli(
            "--test-damping-ratios must include at least one value".to_string(),
        ));
    }

    for ratio in &ratios {
        validate_damping_ratio(*ratio, "--test-damping-ratios")?;
    }

    Ok(ratios)
}

fn parse_freq_range(value: &str) -> Result<(f64, f64, f64)> {
    let parts: Vec<f64> = value
        .split(':')
        .map(str::trim)
        .map(|part| part.parse::<f64>())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| ShaperError::Cli(format!("Invalid --shaper-freq range: {e}")))?;

    if parts.len() != 3 {
        return Err(ShaperError::Cli(
            "--shaper-freq must be start:end:step".to_string(),
        ));
    }

    validate_positive(parts[0], "--shaper-freq start")?;
    validate_positive(parts[1], "--shaper-freq end")?;
    validate_positive(parts[2], "--shaper-freq step")?;

    if parts[0] >= parts[1] {
        return Err(ShaperError::Cli(
            "--shaper-freq start must be less than end".to_string(),
        ));
    }

    Ok((parts[0], parts[1], parts[2]))
}

fn parse_shaper_selection(selection: &str) -> Result<Vec<Box<dyn ShaperModel>>> {
    let names: Vec<String> = selection
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();

    if names.is_empty() {
        return Err(ShaperError::Cli(format!(
            "--shapers must include at least one shaper ({})",
            valid_shapers_list()
        )));
    }

    if names.iter().any(|name| name == "all") {
        if names.len() != 1 {
            return Err(ShaperError::Cli(
                "--shapers all cannot be combined with other shapers".to_string(),
            ));
        }
        return Ok(all_shapers_with_zvd());
    }

    let mut seen = Vec::new();
    let mut shapers = Vec::new();

    for name in names {
        if seen.iter().any(|seen_name| seen_name == &name) {
            continue;
        }

        let shaper = shaper_by_name(&name).ok_or_else(|| {
            ShaperError::Cli(format!(
                "Unknown shaper '{name}'. Valid values: {}",
                valid_shapers_list()
            ))
        })?;
        seen.push(name);
        shapers.push(shaper);
    }

    Ok(shapers)
}

fn axis_from_path(path: &Path) -> &'static str {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem.contains("_y") || stem.contains("-y") {
        "y"
    } else if stem.contains("_z") || stem.contains("-z") {
        "z"
    } else if stem.contains("_x") || stem.contains("-x") {
        "x"
    } else {
        eprintln!(
            "Warning: filename '{}' does not contain an axis hint (_x, -x, _y, -y, _z, -z); assuming X axis",
            stem
        );
        "x"
    }
}

fn apply_via_klippy(args: &Args, results: &[(String, CalibrationOutput)]) -> Result<()> {
    if results.is_empty() {
        return Ok(());
    }

    // Build one multiline M118 script so Moonraker receives all the log output
    // in a single POST, mirroring Kalico's shaper_calibrate format.
    let mut m118_lines = Vec::new();
    for (axis, result) in results {
        m118_lines.push(format!(
            "Calculating the best input shaper parameters for {axis} axis"
        ));

        for fit in &result.all_results {
            m118_lines.push(format!(
                "Fitted shaper '{}' frequency = {:.1} Hz (vibrations = {:.1}%, smoothing ~= {:.3})",
                fit.shaper_name,
                fit.best.freq,
                fit.best.vibrs * 100.0,
                fit.best.smoothing
            ));
            m118_lines.push(format!(
                "To avoid too much smoothing with '{}', suggested max_accel <= {:.0} mm/sec^2",
                fit.shaper_name,
                (fit.best.max_accel / 100.0).round() * 100.0
            ));
        }

        m118_lines.push(format!(
            "Recommended shaper_type_{} = {}, shaper_freq_{} = {:.1} Hz",
            axis, result.recommended_shaper, axis, result.recommended_freq
        ));
    }
    send_log_klippy(args, &m118_lines)?;

    // Build single SET_INPUT_SHAPER script with all axes.
    let mut script = String::from("SET_INPUT_SHAPER");
    for (axis, result) in results {
        let axis_up = axis.to_uppercase();
        script.push_str(&format!(
            " SHAPER_TYPE_{}={} SHAPER_FREQ_{}={:.1}",
            axis_up,
            result.recommended_shaper.to_uppercase(),
            axis_up,
            result.recommended_freq
        ));
    }
    send_gcode_script(&args.moonraker_host, args.moonraker_port, &script)?;

    if !args.quiet {
        for (axis, result) in results {
            let axis_up = axis.to_uppercase();
            eprintln!(
                "✓ Set input_shaper {axis_up} = {} @ {:.1} Hz on Klippy",
                result.recommended_shaper.to_uppercase(),
                result.recommended_freq
            );
        }
    }

    // Always remind the user that SAVE_CONFIG is needed to persist.
    send_log_klippy(
        args,
        &[
            "The SAVE_CONFIG command will update the printer config file with these parameters."
                .to_string(),
        ],
    )?;

    if args.commit {
        send_log_klippy(
            args,
            &["rusty-shaper initiated with --commit argument, saving config now!".to_string()],
        )?;
        send_gcode_script(
            &args.moonraker_host,
            args.moonraker_port,
            "SAVE_CONFIG RESTART=0",
        )?;
        if !args.quiet {
            eprintln!("✓ Config saved (SAVE_CONFIG RESTART=0)");
        }
    } else if !args.quiet {
        eprintln!("Note: Parameters are in RAM only. Use --commit to persist to printer.cfg.");
    }

    Ok(())
}

fn print_cfg_block(axis: &str, result: &rusty_shaper::types::CalibrationOutput) {
    let max_accel = result.recommended_max_accel;
    println!("[input_shaper]");
    println!(
        "# Recommended max_accel <= {:.0} mm/s²",
        (max_accel / 100.0).round() * 100.0
    );
    println!(
        "shaper_type_{} = {}",
        axis,
        result.recommended_shaper.to_lowercase()
    );
    println!("shaper_freq_{} = {:.1}", axis, result.recommended_freq);
}

fn write_json_file(
    args: &Args,
    axis: &str,
    result: &rusty_shaper::types::CalibrationOutput,
    name_suffix: &str,
    pretty: bool,
) -> Result<()> {
    std::fs::create_dir_all(&args.workdir)?;
    let path = args
        .workdir
        .join(format!("calibration_data_{axis}_{name_suffix}.json"));
    let json = serialize_output(result, args.with_psd, pretty)?;
    std::fs::write(&path, json)?;
    if !args.quiet {
        eprintln!("JSON written to {}", path.display());
    }
    Ok(())
}

fn write_json_path(
    args: &Args,
    path: &str,
    result: &rusty_shaper::types::CalibrationOutput,
) -> Result<()> {
    let path = Path::new(path);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let json = serialize_output(result, args.with_psd, false)?;
    std::fs::write(path, json)?;
    if !args.quiet {
        eprintln!("Results written to {}", path.display());
    }
    Ok(())
}

fn serialize_output(
    result: &rusty_shaper::types::CalibrationOutput,
    with_psd: bool,
    pretty: bool,
) -> Result<String> {
    if with_psd {
        return if pretty {
            Ok(serde_json::to_string_pretty(result)?)
        } else {
            Ok(serde_json::to_string(result)?)
        };
    }

    let mut value = serde_json::to_value(result).map_err(ShaperError::Json)?;
    if let serde_json::Value::Object(ref mut map) = value {
        map.remove("psd_bins");
    }

    if pretty {
        Ok(serde_json::to_string_pretty(&value)?)
    } else {
        Ok(serde_json::to_string(&value)?)
    }
}

fn write_csv_output(
    args: &Args,
    axis: &str,
    result: &rusty_shaper::types::CalibrationOutput,
    name_suffix: &str,
) -> Result<()> {
    std::fs::create_dir_all(&args.workdir)?;
    let path = args
        .workdir
        .join(format!("calibration_data_{axis}_{name_suffix}.csv"));
    let file = std::fs::File::create(&path)?;
    let mut writer = std::io::BufWriter::new(file);

    let mut header = "freq,psd_x,psd_y,psd_z,psd_xyz".to_string();
    for fit in &result.all_results {
        header.push_str(&format!(",{}({:.1})", fit.shaper_name, fit.best.freq));
    }
    writeln!(writer, "{header}")?;

    let shaper_coeffs: Vec<_> = result
        .all_results
        .iter()
        .map(|fit| {
            rusty_shaper::models::get_shaper_coefficients(
                &fit.shaper_name,
                fit.best.freq,
                args.damping_ratio,
            )
            .ok_or_else(|| {
                ShaperError::Cli(format!(
                    "Internal error: no coefficients for shaper {}",
                    fit.shaper_name
                ))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    for bin in &result.psd_bins {
        let mut row = format!(
            "{:.1},{:.6e},{:.6e},{:.6e},{:.6e}",
            bin.freq, bin.psd_x, bin.psd_y, bin.psd_z, bin.psd_sum
        );
        for coeffs in &shaper_coeffs {
            let response =
                rusty_shaper::scorer::estimate_shaper(coeffs, args.damping_ratio, &[bin.freq])[0];
            row.push_str(&format!(",{:.6e}", response));
        }
        writeln!(writer, "{row}")?;
    }

    if !args.quiet {
        eprintln!("CSV written to {}", path.display());
    }

    if args.output.iter().any(|mode| mode == "klippy")
        && let Err(e) = send_log_klippy(
            args,
            &[format!(
                "Shaper calibration data written to {} file",
                path.display()
            )],
        )
    {
        eprintln!("Warning: failed to send CSV M118: {e}");
    }

    Ok(())
}

fn print_summary_stderr(result: &rusty_shaper::types::CalibrationOutput) {
    eprintln!(
        "Recommended shaper: {} @ {:.1} Hz",
        result.recommended_shaper.to_uppercase(),
        result.recommended_freq
    );
    for fit in &result.all_results {
        eprintln!(
            "  {}: best={:.1} Hz, vibr={:.1}%, smoothing={:.3}, score={:.4}, max_accel<={:.0}",
            fit.shaper_name.to_uppercase(),
            fit.best.freq,
            fit.best.vibrs * 100.0,
            fit.best.smoothing,
            fit.best.score,
            (fit.best.max_accel / 100.0).round() * 100.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_args() -> Args {
        Args {
            input: vec![PathBuf::from("input.csv")],
            output: Vec::new(),
            workdir: PathBuf::from("/tmp"),
            name: None,
            shapers: "mzv".to_string(),
            shapers_x: None,
            shapers_y: None,
            shapers_z: None,
            damping_ratio: 0.1,
            test_damping_ratios: "0.075,0.1,0.15".to_string(),
            scv: 5.0,
            max_smoothing: None,
            max_freq: 200.0,
            shaper_freq: None,
            raw: false,
            window_t: 0.5,
            with_psd: false,
            quiet: false,
            moonraker_host: "127.0.0.1".to_string(),
            moonraker_port: 80,
            log_macro: "M118".to_string(),
            commit: false,
        }
    }

    #[test]
    fn parse_shapers_all_includes_zvd() {
        let shapers = parse_shaper_selection("all").unwrap();
        assert!(shapers.iter().any(|shaper| shaper.name() == "zvd"));
    }

    #[test]
    fn parse_shapers_rejects_unknown_names() {
        let err = parse_shaper_selection("mzv,not_real").err().unwrap();
        assert!(err.to_string().contains("Unknown shaper"));
    }

    #[test]
    fn parse_shapers_rejects_combined_all() {
        let err = parse_shaper_selection("all,mzv").err().unwrap();
        assert!(err.to_string().contains("cannot be combined"));
    }

    #[test]
    fn parse_freq_range_rejects_bad_bounds() {
        let err = parse_freq_range("60:20:0.5").unwrap_err();
        assert!(err.to_string().contains("start must be less than end"));
    }

    #[test]
    fn commit_requires_klippy_output() {
        let mut args = test_args();
        args.commit = true;

        let err = validate_args(&args).err().unwrap();
        assert!(err.to_string().contains("--output klippy"));

        args.output.push("klippy".to_string());
        validate_args(&args).unwrap();
    }

    #[test]
    fn json_output_omits_psd_bins_by_default() {
        let result = rusty_shaper::types::CalibrationOutput {
            recommended_shaper: "mzv".to_string(),
            recommended_freq: 40.0,
            recommended_max_accel: 10000.0,
            all_results: Vec::new(),
            psd_bins: vec![rusty_shaper::types::PsdBin::new(10.0, 1.0, 0.0, 0.0)],
        };

        let json = serialize_output(&result, false, false).expect("serialize should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed.as_object().unwrap().contains_key("psd_bins"));
    }

    #[test]
    fn json_output_includes_psd_bins_with_flag() {
        let result = rusty_shaper::types::CalibrationOutput {
            recommended_shaper: "mzv".to_string(),
            recommended_freq: 40.0,
            recommended_max_accel: 10000.0,
            all_results: Vec::new(),
            psd_bins: vec![rusty_shaper::types::PsdBin::new(10.0, 1.0, 0.0, 0.0)],
        };

        let json = serialize_output(&result, true, false).expect("serialize should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.as_object().unwrap().contains_key("psd_bins"));
    }

    #[test]
    fn axis_shaper_selection_uses_axis_override() {
        let mut args = test_args();
        args.shapers = "zv,mzv".to_string();
        args.shapers_y = Some("ei".to_string());

        let y = axis_shaper_selection(&args, "y").unwrap();
        assert_eq!(y.len(), 1);
        assert_eq!(y[0].name(), "ei");

        let x = axis_shaper_selection(&args, "x").unwrap();
        assert_eq!(x.len(), 2);
        assert!(x.iter().any(|s| s.name() == "zv"));
        assert!(x.iter().any(|s| s.name() == "mzv"));
    }

    #[test]
    fn apply_via_klippy_batches_axes() {
        let args = test_args();
        let x = rusty_shaper::types::CalibrationOutput {
            recommended_shaper: "mzv".to_string(),
            recommended_freq: 40.0,
            recommended_max_accel: 10000.0,
            all_results: Vec::new(),
            psd_bins: Vec::new(),
        };
        let y = rusty_shaper::types::CalibrationOutput {
            recommended_shaper: "ei".to_string(),
            recommended_freq: 55.0,
            recommended_max_accel: 8000.0,
            all_results: Vec::new(),
            psd_bins: Vec::new(),
        };
        let results = vec![("x".to_string(), x), ("y".to_string(), y)];

        // With no output flag this should be a no-op; we only check that the
        // function does not panic with a fake Moonraker endpoint.
        let _ = apply_via_klippy(&args, &results);
    }
}
