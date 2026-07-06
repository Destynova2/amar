use amar_core::{CoreError, UtcDateTime, predict_height};
use amar_data::{
    DataError, build_noaa_pack, load_official_predictions, load_pack_from_path,
    prediction_error_meters,
};
use clap::{Args, Parser, Subcommand};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use thiserror::Error;

const DEFAULT_PACK: &str = "data/packs/noaa_m0.json";
const DEFAULT_FIXTURES: &str = "fixtures/noaa";
const DEFAULT_MAX_DISTANCE_KM: f64 = 20.0;
const M0_P95_LIMIT_M: f64 = 0.05;

#[derive(Debug, Error)]
enum CliError {
    #[error("{0}")]
    Data(#[from] DataError),
    #[error("{0}")]
    Core(#[from] CoreError),
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation p95 exceeded {limit_cm:.1} cm:\n{failures}")]
    ValidationThreshold { limit_cm: f64, failures: String },
}

#[derive(Debug, Parser)]
#[command(name = "amar")]
#[command(about = "Offline astronomical tide from versioned station packs")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Tide(TideArgs),
    Validate(ValidateArgs),
    PackNoaa(PackNoaaArgs),
}

#[derive(Debug, Args)]
struct TideArgs {
    #[arg(long, allow_hyphen_values = true)]
    lat: f64,
    #[arg(long, allow_hyphen_values = true)]
    lon: f64,
    #[arg(long)]
    at: String,
    #[arg(long, default_value = DEFAULT_PACK)]
    pack: PathBuf,
    #[arg(long, default_value_t = DEFAULT_MAX_DISTANCE_KM)]
    max_distance_km: f64,
}

#[derive(Debug, Args)]
struct ValidateArgs {
    #[arg(long, default_value = DEFAULT_PACK)]
    pack: PathBuf,
    #[arg(long, default_value = DEFAULT_FIXTURES)]
    fixtures: PathBuf,
}

#[derive(Debug, Args)]
struct PackNoaaArgs {
    #[arg(long, default_value = DEFAULT_FIXTURES)]
    fixtures: PathBuf,
    #[arg(long, default_value = DEFAULT_PACK)]
    out: PathBuf,
    #[arg(long, default_value = "2026-07-06")]
    extracted_at: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    match cli.command {
        Command::Tide(args) => tide(args),
        Command::Validate(args) => validate(args),
        Command::PackNoaa(args) => pack_noaa(args),
    }
}

fn tide(args: TideArgs) -> Result<(), CliError> {
    let data = load_pack_from_path(&args.pack)?;
    let at = UtcDateTime::parse_rfc3339(&args.at)?;
    let station_match = data.nearest_station(args.lat, args.lon, args.max_distance_km)?;
    let prediction = predict_height(station_match.station.model(), at);
    let station = station_match.station.pack();
    let output = json!({
        "height_m": round3(prediction.height().as_meters()),
        "datum": station.datum,
        "source": {
            "kind": "station",
            "id": station.station_id,
            "name": station.name,
            "distance_km": round3(station_match.distance_km),
            "data_version": station.source.extracted_at,
        },
        "method": prediction.method().as_str(),
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn validate(args: ValidateArgs) -> Result<(), CliError> {
    let data = load_pack_from_path(&args.pack)?;
    let mut failures = Vec::new();

    for station in data.stations() {
        let station_id = &station.pack().provider_station_id;
        let station_dir = args.fixtures.join(station_id);
        let mut errors = Vec::new();
        let mut window_summaries = BTreeMap::new();
        for prediction_path in prediction_files(&station_dir)? {
            let predictions = load_official_predictions(&prediction_path)?;
            let mut window_errors = Vec::new();
            for official in predictions {
                let error = prediction_error_meters(station.model(), official);
                errors.push(error);
                window_errors.push(error);
            }
            window_errors.sort_by(|left, right| left.total_cmp(right));
            let window_p95 = percentile(&window_errors, 0.95).unwrap_or(0.0);
            window_summaries.insert(
                prediction_window_label(&prediction_path),
                (window_errors.len(), window_p95),
            );
        }
        errors.sort_by(|left, right| left.total_cmp(right));
        let p95 = percentile(&errors, 0.95).unwrap_or(0.0);
        println!(
            "{} {} method={} samples={} p95_m={:.3} p95_cm={:.1}",
            station.pack().station_id,
            station.pack().name,
            station.model().method().as_str(),
            errors.len(),
            p95,
            p95 * 100.0
        );
        if p95 > M0_P95_LIMIT_M {
            failures.push(format!(
                "{} all p95_cm={:.1}",
                station.pack().station_id,
                p95 * 100.0
            ));
        }
        for (window, (samples, window_p95)) in window_summaries {
            println!(
                "{} {} window={} samples={} p95_m={:.3} p95_cm={:.1}",
                station.pack().station_id,
                station.pack().name,
                window,
                samples,
                window_p95,
                window_p95 * 100.0
            );
            if window_p95 > M0_P95_LIMIT_M {
                failures.push(format!(
                    "{} {} p95_cm={:.1}",
                    station.pack().station_id,
                    window,
                    window_p95 * 100.0
                ));
            }
        }
    }

    if data.stations().is_empty() {
        println!("no stations validated");
    }
    if !failures.is_empty() {
        return Err(CliError::ValidationThreshold {
            limit_cm: M0_P95_LIMIT_M * 100.0,
            failures: failures.join("\n"),
        });
    }
    Ok(())
}

fn pack_noaa(args: PackNoaaArgs) -> Result<(), CliError> {
    let pack = build_noaa_pack(&args.fixtures, &args.extracted_at)?;
    let output = serde_json::to_string_pretty(&pack)?;
    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent).map_err(|source| CliError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(&args.out, format!("{output}\n")).map_err(|source| CliError::Io {
        path: args.out.clone(),
        source,
    })?;
    Ok(())
}

fn prediction_files(station_dir: &Path) -> Result<Vec<PathBuf>, CliError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(station_dir).map_err(|source| CliError::Io {
        path: station_dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| CliError::Io {
            path: station_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with("predictions_") && name.ends_with(".json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn prediction_window_label(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("predictions_"))
        .unwrap_or("unknown")
        .to_string()
}

fn percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }
    let index = ((sorted_values.len() - 1) as f64 * percentile).ceil() as usize;
    sorted_values.get(index).copied()
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}
