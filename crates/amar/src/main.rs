use amar::server::{self, ServerError, SourceResponse};
use amar_core::{
    ConstituentId, CoreError, DatumId, Degrees, DegreesPerHour, HarmonicConstituent, Meters,
    PredictionMethod, TideModel, TideThresholdDirection, UtcDateTime, extrema_between,
    next_extrema_after, predict_height, predict_series, tide_windows,
};
use amar_data::{
    DataError, OfficialExtremum, build_noaa_pack, load_official_hilo_predictions,
    load_official_predictions, load_pack_from_path, load_packs_from_paths, percentile,
    prediction_signed_error_meters,
};
use clap::{Args, Parser, Subcommand};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use thiserror::Error;

const DEFAULT_NOAA_PACK: &str = "data/packs/noaa_m0.json";
const DEFAULT_BREST_PACK: &str = "data/packs/amar-data-brest-experimental.json";
const DEFAULT_BREST_BENCHMARK: &str = "fixtures/refmar/benchmark_brest_v1.json";
const DEFAULT_FIXTURES: &str = "fixtures/noaa";
const DEFAULT_MAX_DISTANCE_KM: f64 = 20.0;
const M0_P95_LIMIT_M: f64 = 0.02;
const HILO_P95_TIME_LIMIT_MIN: f64 = 10.0;
const HILO_P95_HEIGHT_LIMIT_M: f64 = 0.03;
const NEXT_EXTREMA_HORIZON_H: u32 = 72;
const MAX_SERIES_DURATION_H: u32 = 72;
const MIN_SERIES_STEP_MIN: u32 = 6;
const DEFAULT_SERIES_STEP_MIN: u32 = 60;
const MAX_WINDOWS_DURATION_SECONDS: i64 = 31 * 24 * 60 * 60;
const DEFAULT_NOAA_STATIONS: &str = include_str!("../../../data/stations.txt");

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
    #[error("{0}")]
    Server(#[from] ServerError),
    #[error("validation p95 exceeded {limit_cm:.1} cm:\n{failures}")]
    ValidationThreshold { limit_cm: f64, failures: String },
    #[error("validation missing samples:\n{failures}")]
    ValidationSamples { failures: String },
    #[error("station {0} not found in loaded packs")]
    MissingStation(String),
    #[error("benchmark has no usable samples")]
    EmptyBenchmark,
    #[error("station {station_id} has no supported confidence metadata")]
    UnsupportedStationConfidence { station_id: String },
    #[error("station {station_id} has no M2 constituent")]
    MissingM2Constituent { station_id: String },
    #[error("benchmark p95 exceeded {limit_cm:.1} cm: {model} p95_cm={p95_cm:.1}")]
    BenchmarkThreshold {
        model: &'static str,
        limit_cm: f64,
        p95_cm: f64,
    },
    #[error("hilo validation p95 exceeded:\n{failures}")]
    HiloThreshold { failures: String },
    #[error("{0}")]
    InvalidArgument(String),
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
    Window(WindowArgs),
    Serve(ServeArgs),
    Validate(ValidateArgs),
    ValidateHilo(ValidateArgs),
    BenchmarkBrest(BenchmarkBrestArgs),
    PackNoaa(PackNoaaArgs),
}

#[derive(Debug, Args)]
struct TideArgs {
    #[arg(long, allow_hyphen_values = true, value_parser = parse_latitude)]
    lat: f64,
    #[arg(long, allow_hyphen_values = true, value_parser = parse_longitude)]
    lon: f64,
    #[arg(long)]
    at: String,
    #[arg(long = "pack")]
    pack: Vec<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_MAX_DISTANCE_KM)]
    max_distance_km: f64,
    #[arg(long = "duration-h")]
    duration_h: Option<u32>,
    #[arg(long = "step-min")]
    step_min: Option<u32>,
}

#[derive(Debug, Args)]
struct WindowArgs {
    #[arg(long, allow_hyphen_values = true, value_parser = parse_latitude)]
    lat: f64,
    #[arg(long, allow_hyphen_values = true, value_parser = parse_longitude)]
    lon: f64,
    #[arg(long)]
    from: String,
    #[arg(long)]
    to: String,
    #[arg(long = "above", value_parser = parse_finite_f64)]
    above_m: Option<f64>,
    #[arg(long = "below", value_parser = parse_finite_f64)]
    below_m: Option<f64>,
    #[arg(long = "pack")]
    pack: Vec<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_MAX_DISTANCE_KM)]
    max_distance_km: f64,
}

#[derive(Debug, Args)]
struct ServeArgs {
    #[arg(long, default_value = "127.0.0.1:3000")]
    addr: String,
    #[arg(long = "pack")]
    pack: Vec<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_MAX_DISTANCE_KM)]
    max_distance_km: f64,
}

#[derive(Debug, Args)]
struct ValidateArgs {
    #[arg(long, default_value = DEFAULT_NOAA_PACK)]
    pack: PathBuf,
    #[arg(long, default_value = DEFAULT_FIXTURES)]
    fixtures: PathBuf,
}

#[derive(Debug, Args)]
struct PackNoaaArgs {
    #[arg(long, default_value = DEFAULT_FIXTURES)]
    fixtures: PathBuf,
    #[arg(long, default_value = DEFAULT_NOAA_PACK)]
    out: PathBuf,
    #[arg(long, default_value = "2026-07-06")]
    extracted_at: String,
    #[arg(long = "station")]
    stations: Vec<String>,
}

#[derive(Debug, Args)]
struct BenchmarkBrestArgs {
    #[arg(long = "pack")]
    pack: Vec<PathBuf>,
    #[arg(long, default_value = DEFAULT_BREST_BENCHMARK)]
    benchmark: PathBuf,
    #[arg(long, default_value = "refmar:3")]
    station_id: String,
    #[arg(long = "p95-limit-cm", value_parser = parse_non_negative_f64)]
    p95_limit_cm: Option<f64>,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

async fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    match cli.command {
        Command::Tide(args) => tide(args),
        Command::Window(args) => window(args),
        Command::Serve(args) => serve(args).await,
        Command::Validate(args) => validate(args),
        Command::ValidateHilo(args) => validate_hilo(args),
        Command::BenchmarkBrest(args) => benchmark_brest(args),
        Command::PackNoaa(args) => pack_noaa(args),
    }
}

async fn serve(args: ServeArgs) -> Result<(), CliError> {
    server::serve(
        &args.addr,
        &effective_pack_paths(&args.pack),
        args.max_distance_km,
    )
    .await?;
    Ok(())
}

fn tide(args: TideArgs) -> Result<(), CliError> {
    let data = load_packs_from_paths(&effective_pack_paths(&args.pack))?;
    let at = UtcDateTime::parse_rfc3339(&args.at)?;
    let station_match = data.nearest_station(
        args.lat,
        args.lon,
        effective_max_distance_km(args.max_distance_km),
    )?;
    let prediction = predict_height(station_match.station.model(), at);
    let station = station_match.station.pack();
    let confidence = server::confidence_for_station(&station_match).ok_or_else(|| {
        CliError::UnsupportedStationConfidence {
            station_id: station.station_id.clone(),
        }
    })?;
    let output = if let Some(duration_h) = args.duration_h {
        let step_min = args.step_min.unwrap_or(DEFAULT_SERIES_STEP_MIN);
        validate_series_args(duration_h, step_min)?;
        let series = predict_series(station_match.station.model(), at, duration_h, step_min)
            .into_iter()
            .map(point_json)
            .collect::<Vec<_>>();
        json!({
            "series": series,
            "datum": station.datum,
            "source": SourceResponse::from(&station_match),
            "confidence": confidence,
            "warnings": server::warnings_for_station(station),
        })
    } else {
        if args.step_min.is_some() {
            return Err(CliError::InvalidArgument(
                "--step-min requires --duration-h".to_string(),
            ));
        }
        let (next_high, next_low) =
            next_extrema_after(station_match.station.model(), at, NEXT_EXTREMA_HORIZON_H);
        json!({
            "height_m": round3(prediction.height().as_meters()),
            "next_high": next_high.map(extremum_json),
            "next_low": next_low.map(extremum_json),
            "datum": station.datum,
            "source": SourceResponse::from(&station_match),
            "confidence": confidence,
            "warnings": server::warnings_for_station(station),
        })
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn window(args: WindowArgs) -> Result<(), CliError> {
    let data = load_packs_from_paths(&effective_pack_paths(&args.pack))?;
    let from = UtcDateTime::parse_rfc3339(&args.from)?;
    let to = UtcDateTime::parse_rfc3339(&args.to)?;
    validate_window_range(from, to)?;
    let (threshold, direction) = threshold_args(args.above_m, args.below_m)?;
    let station_match = data.nearest_station(
        args.lat,
        args.lon,
        effective_max_distance_km(args.max_distance_km),
    )?;
    let station = station_match.station.pack();
    let confidence = server::confidence_for_station(&station_match).ok_or_else(|| {
        CliError::UnsupportedStationConfidence {
            station_id: station.station_id.clone(),
        }
    })?;
    let windows = tide_windows(
        station_match.station.model(),
        from,
        to,
        threshold,
        direction,
    )
    .into_iter()
    .map(window_json)
    .collect::<Vec<_>>();
    let output = json!({
        "windows": windows,
        "datum": station.datum,
        "source": SourceResponse::from(&station_match),
        "confidence": confidence,
        "warnings": server::warnings_for_station(station),
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn validate(args: ValidateArgs) -> Result<(), CliError> {
    let data = load_pack_from_path(&args.pack)?;
    let mut failures = Vec::new();
    let mut sample_failures = Vec::new();

    for station in data.stations() {
        let station_id = &station.pack().provider_station_id;
        let station_dir = args.fixtures.join(station_id);
        let mut errors = Vec::new();
        let mut window_summaries = BTreeMap::new();
        for prediction_path in prediction_files(&station_dir)? {
            let predictions = load_official_predictions(&prediction_path)?;
            let mut window_errors = Vec::new();
            for official in predictions {
                let error = prediction_signed_error_meters(station.model(), official);
                errors.push(error);
                window_errors.push(error);
            }
            let Some(window_stats) = error_stats(&window_errors) else {
                sample_failures.push(format!(
                    "{} {} samples=0",
                    station.pack().station_id,
                    prediction_window_label(&prediction_path)
                ));
                continue;
            };
            window_summaries.insert(prediction_window_label(&prediction_path), window_stats);
        }
        let Some(stats) = error_stats(&errors) else {
            println!(
                "{} {} method={} samples=0 bias_cm=NA std_cm=NA p95_m=NA p95_cm=NA",
                station.pack().station_id,
                station.pack().name,
                station.model().method().as_str(),
            );
            sample_failures.push(format!("{} samples=0", station.pack().station_id));
            continue;
        };
        println!(
            "{} {} method={} samples={} bias_cm={:.1} std_cm={:.1} p95_m={:.3} p95_cm={:.1}",
            station.pack().station_id,
            station.pack().name,
            station.model().method().as_str(),
            stats.samples,
            stats.bias * 100.0,
            stats.std_dev * 100.0,
            stats.p95_abs,
            stats.p95_abs * 100.0
        );
        if stats.p95_abs > M0_P95_LIMIT_M {
            failures.push(format!(
                "{} all p95_cm={:.1}",
                station.pack().station_id,
                stats.p95_abs * 100.0
            ));
        }
        for (window, window_stats) in window_summaries {
            println!(
                "{} {} window={} samples={} bias_cm={:.1} std_cm={:.1} p95_m={:.3} p95_cm={:.1}",
                station.pack().station_id,
                station.pack().name,
                window,
                window_stats.samples,
                window_stats.bias * 100.0,
                window_stats.std_dev * 100.0,
                window_stats.p95_abs,
                window_stats.p95_abs * 100.0
            );
            if window_stats.p95_abs > M0_P95_LIMIT_M {
                failures.push(format!(
                    "{} {} p95_cm={:.1}",
                    station.pack().station_id,
                    window,
                    window_stats.p95_abs * 100.0
                ));
            }
        }
    }

    if data.stations().is_empty() {
        println!("no stations validated");
    }
    if !sample_failures.is_empty() {
        return Err(CliError::ValidationSamples {
            failures: sample_failures.join("\n"),
        });
    }
    if !failures.is_empty() {
        return Err(CliError::ValidationThreshold {
            limit_cm: M0_P95_LIMIT_M * 100.0,
            failures: failures.join("\n"),
        });
    }
    Ok(())
}

fn validate_hilo(args: ValidateArgs) -> Result<(), CliError> {
    let data = load_pack_from_path(&args.pack)?;
    let mut failures = Vec::new();
    let mut sample_failures = Vec::new();

    for station in data.stations() {
        let station_id = &station.pack().provider_station_id;
        let station_dir = args.fixtures.join(station_id);
        let mut time_errors_min = Vec::new();
        let mut height_errors_m = Vec::new();
        let mut window_summaries = BTreeMap::new();
        for hilo_path in hilo_files(&station_dir)? {
            let official = load_official_hilo_predictions(&hilo_path)?;
            let Some((from, to)) = official_time_bounds(&official) else {
                sample_failures.push(format!(
                    "{} {} samples=0",
                    station.pack().station_id,
                    hilo_window_label(&hilo_path)
                ));
                continue;
            };
            let predicted = extrema_between(
                station.model(),
                from.add_seconds(-12 * 60 * 60),
                to.add_seconds(12 * 60 * 60),
            );
            let mut window_time_errors_min = Vec::new();
            let mut window_height_errors_m = Vec::new();
            for official_extremum in official {
                let Some(predicted_extremum) = closest_extremum(&predicted, official_extremum)
                else {
                    sample_failures.push(format!(
                        "{} {} missing predicted {:?}",
                        station.pack().station_id,
                        hilo_window_label(&hilo_path),
                        official_extremum.kind
                    ));
                    continue;
                };
                let dt_min = predicted_extremum
                    .at()
                    .seconds_since(official_extremum.at)
                    .abs() as f64
                    / 60.0;
                let dh_m = (predicted_extremum.height().as_meters()
                    - official_extremum.height.as_meters())
                .abs();
                time_errors_min.push(dt_min);
                height_errors_m.push(dh_m);
                window_time_errors_min.push(dt_min);
                window_height_errors_m.push(dh_m);
            }
            if let Some(window_stats) = hilo_stats(&window_time_errors_min, &window_height_errors_m)
            {
                window_summaries.insert(hilo_window_label(&hilo_path), window_stats);
            }
        }

        let Some(stats) = hilo_stats(&time_errors_min, &height_errors_m) else {
            println!(
                "{} {} method={} hilo_samples=0 p50_dt_min=NA p95_dt_min=NA max_dt_min=NA p50_dh_cm=NA p95_dh_cm=NA max_dh_cm=NA",
                station.pack().station_id,
                station.pack().name,
                station.model().method().as_str(),
            );
            sample_failures.push(format!("{} hilo_samples=0", station.pack().station_id));
            continue;
        };
        println!(
            "{} {} method={} hilo_samples={} p50_dt_min={:.2} p95_dt_min={:.2} max_dt_min={:.2} p50_dh_cm={:.1} p95_dh_cm={:.1} max_dh_cm={:.1}",
            station.pack().station_id,
            station.pack().name,
            station.model().method().as_str(),
            stats.samples,
            stats.p50_dt_min,
            stats.p95_dt_min,
            stats.max_dt_min,
            stats.p50_dh_m * 100.0,
            stats.p95_dh_m * 100.0,
            stats.max_dh_m * 100.0
        );
        if stats.p95_dt_min > HILO_P95_TIME_LIMIT_MIN || stats.p95_dh_m > HILO_P95_HEIGHT_LIMIT_M {
            failures.push(format!(
                "{} all p95_dt_min={:.2} p95_dh_cm={:.1}",
                station.pack().station_id,
                stats.p95_dt_min,
                stats.p95_dh_m * 100.0
            ));
        }
        for (window, window_stats) in window_summaries {
            println!(
                "{} {} window={} hilo_samples={} p50_dt_min={:.2} p95_dt_min={:.2} max_dt_min={:.2} p50_dh_cm={:.1} p95_dh_cm={:.1} max_dh_cm={:.1}",
                station.pack().station_id,
                station.pack().name,
                window,
                window_stats.samples,
                window_stats.p50_dt_min,
                window_stats.p95_dt_min,
                window_stats.max_dt_min,
                window_stats.p50_dh_m * 100.0,
                window_stats.p95_dh_m * 100.0,
                window_stats.max_dh_m * 100.0
            );
            if window_stats.p95_dt_min > HILO_P95_TIME_LIMIT_MIN
                || window_stats.p95_dh_m > HILO_P95_HEIGHT_LIMIT_M
            {
                failures.push(format!(
                    "{} {} p95_dt_min={:.2} p95_dh_cm={:.1}",
                    station.pack().station_id,
                    window,
                    window_stats.p95_dt_min,
                    window_stats.p95_dh_m * 100.0
                ));
            }
        }
    }

    if data.stations().is_empty() {
        println!("no stations validated");
    }
    if !sample_failures.is_empty() {
        return Err(CliError::ValidationSamples {
            failures: sample_failures.join("\n"),
        });
    }
    if !failures.is_empty() {
        return Err(CliError::HiloThreshold {
            failures: failures.join("\n"),
        });
    }
    Ok(())
}

fn benchmark_brest(args: BenchmarkBrestArgs) -> Result<(), CliError> {
    let data = load_packs_from_paths(&effective_pack_paths(&args.pack))?;
    let station = data
        .stations()
        .iter()
        .find(|station| station.pack().station_id == args.station_id)
        .ok_or_else(|| CliError::MissingStation(args.station_id.clone()))?;
    let benchmark = load_brest_benchmark(&args.benchmark)?;
    let z0_m = station.pack().z0_m.get();
    let m2_model = m2_only_model(station)?;

    let mut calibrated_residuals = Vec::new();
    let mut z0_residuals = Vec::new();
    let mut m2_residuals = Vec::new();
    let mut missing = 0_usize;

    for sample in &benchmark.samples {
        let Some(observed_m) = sample.observed_m else {
            missing += 1;
            continue;
        };
        let at = UtcDateTime::parse_rfc3339(&sample.timestamp)?;
        let calibrated = predict_height(station.model(), at).height().as_meters();
        let m2 = predict_height(&m2_model, at).height().as_meters();
        calibrated_residuals.push(observed_m - calibrated);
        z0_residuals.push(observed_m - z0_m);
        m2_residuals.push(observed_m - m2);
    }

    let calibrated = benchmark_stats(&calibrated_residuals).ok_or(CliError::EmptyBenchmark)?;
    let z0 = benchmark_stats(&z0_residuals).ok_or(CliError::EmptyBenchmark)?;
    let m2 = benchmark_stats(&m2_residuals).ok_or(CliError::EmptyBenchmark)?;

    println!(
        "benchmark_brest_v1 station={} datum={} validation_period={}/{} samples={} missing={} checksum={}",
        benchmark.station_id,
        benchmark.datum,
        benchmark.validation_period.start,
        benchmark.validation_period.end,
        calibrated_residuals.len(),
        missing,
        benchmark.checksum_sha256.as_deref().unwrap_or("NA"),
    );
    println!("résidu = niveau d'eau observé − marée astronomique prédite (météo incluse)");
    println!("model,rms_cm,bias_cm,mae_cm,p95_cm,max_cm");
    print_benchmark_stats("calibrated_station_experimental", calibrated);
    print_benchmark_stats("z0_constant", z0);
    print_benchmark_stats("m2_only", m2);
    if let Some(limit_cm) = args.p95_limit_cm
        && calibrated.p95_cm > limit_cm
    {
        return Err(CliError::BenchmarkThreshold {
            model: "calibrated_station_experimental",
            limit_cm,
            p95_cm: calibrated.p95_cm,
        });
    }
    Ok(())
}

fn pack_noaa(args: PackNoaaArgs) -> Result<(), CliError> {
    let pack = if args.stations.is_empty() {
        build_noaa_pack(&args.fixtures, &args.extracted_at, default_noaa_stations())?
    } else {
        build_noaa_pack(
            &args.fixtures,
            &args.extracted_at,
            args.stations.iter().map(String::as_str),
        )?
    };
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

fn effective_pack_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    if !paths.is_empty() {
        return paths.to_vec();
    }
    let mut defaults = vec![PathBuf::from(DEFAULT_NOAA_PACK)];
    let brest = PathBuf::from(DEFAULT_BREST_PACK);
    if brest.exists() {
        defaults.push(brest);
    }
    defaults
}

fn default_noaa_stations() -> impl Iterator<Item = &'static str> {
    DEFAULT_NOAA_STATIONS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

fn effective_max_distance_km(max_distance_km: f64) -> f64 {
    max_distance_km.min(server::MAX_CONFIDENCE_DISTANCE_KM)
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

fn hilo_files(station_dir: &Path) -> Result<Vec<PathBuf>, CliError> {
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
        if name.starts_with("hilo_") && name.ends_with(".json") {
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

fn hilo_window_label(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("hilo_"))
        .unwrap_or("unknown")
        .to_string()
}

#[derive(Clone, Copy)]
struct ErrorStats {
    samples: usize,
    bias: f64,
    std_dev: f64,
    p95_abs: f64,
}

#[derive(Clone, Copy)]
struct HiloStats {
    samples: usize,
    p50_dt_min: f64,
    p95_dt_min: f64,
    max_dt_min: f64,
    p50_dh_m: f64,
    p95_dh_m: f64,
    max_dh_m: f64,
}

fn error_stats(errors: &[f64]) -> Option<ErrorStats> {
    if errors.is_empty() {
        return None;
    }
    let samples = errors.len();
    let bias = errors.iter().sum::<f64>() / samples as f64;
    let variance = errors
        .iter()
        .map(|error| {
            let centered = error - bias;
            centered * centered
        })
        .sum::<f64>()
        / samples as f64;
    let mut absolute_errors = errors.iter().map(|error| error.abs()).collect::<Vec<_>>();
    absolute_errors.sort_by(|left, right| left.total_cmp(right));
    let p95_abs = percentile(&absolute_errors, 0.95)?;
    Some(ErrorStats {
        samples,
        bias,
        std_dev: variance.sqrt(),
        p95_abs,
    })
}

fn official_time_bounds(official: &[OfficialExtremum]) -> Option<(UtcDateTime, UtcDateTime)> {
    let first = official.first()?.at;
    let mut from = first;
    let mut to = first;
    for extremum in official {
        from = from.min(extremum.at);
        to = to.max(extremum.at);
    }
    Some((from, to))
}

fn closest_extremum(
    predicted: &[amar_core::TideExtremum],
    official: OfficialExtremum,
) -> Option<amar_core::TideExtremum> {
    predicted
        .iter()
        .copied()
        .filter(|extremum| extremum.kind() == official.kind)
        .min_by_key(|extremum| extremum.at().seconds_since(official.at).abs())
}

fn hilo_stats(time_errors_min: &[f64], height_errors_m: &[f64]) -> Option<HiloStats> {
    if time_errors_min.is_empty() || time_errors_min.len() != height_errors_m.len() {
        return None;
    }
    let mut sorted_time = time_errors_min.to_vec();
    sorted_time.sort_by(|left, right| left.total_cmp(right));
    let mut sorted_height = height_errors_m.to_vec();
    sorted_height.sort_by(|left, right| left.total_cmp(right));
    Some(HiloStats {
        samples: sorted_time.len(),
        p50_dt_min: percentile(&sorted_time, 0.50)?,
        p95_dt_min: percentile(&sorted_time, 0.95)?,
        max_dt_min: sorted_time.last().copied().unwrap_or(0.0),
        p50_dh_m: percentile(&sorted_height, 0.50)?,
        p95_dh_m: percentile(&sorted_height, 0.95)?,
        max_dh_m: sorted_height.last().copied().unwrap_or(0.0),
    })
}

#[derive(Debug, Deserialize)]
struct BrestBenchmark {
    station_id: String,
    datum: String,
    validation_period: BenchmarkPeriod,
    #[serde(default)]
    checksum_sha256: Option<String>,
    samples: Vec<BrestBenchmarkSample>,
}

#[derive(Debug, Deserialize)]
struct BenchmarkPeriod {
    start: String,
    end: String,
}

#[derive(Debug, Deserialize)]
struct BrestBenchmarkSample {
    timestamp: String,
    observed_m: Option<f64>,
}

#[derive(Clone, Copy)]
struct BenchmarkStats {
    rms_cm: f64,
    bias_cm: f64,
    mae_cm: f64,
    p95_cm: f64,
    max_cm: f64,
}

fn load_brest_benchmark(path: &Path) -> Result<BrestBenchmark, CliError> {
    let data = fs::read_to_string(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&data).map_err(CliError::from)
}

fn m2_only_model(station: &amar_data::LoadedStation) -> Result<TideModel, CliError> {
    let m2 = station
        .pack()
        .constituents
        .iter()
        .find(|constituent| constituent.name == "M2")
        .ok_or_else(|| CliError::MissingM2Constituent {
            station_id: station.pack().station_id.clone(),
        })?;
    let constituent = HarmonicConstituent::new(
        ConstituentId::new(&m2.name)?,
        Meters::new(m2.amplitude_m.get())?,
        Degrees::new(m2.phase_gmt_deg.get())?,
        DegreesPerHour::new(m2.speed_deg_per_hour.get())?,
    );
    TideModel::new(
        DatumId::new(&station.pack().datum)?,
        Meters::new(station.pack().z0_m.get())?,
        vec![constituent],
        PredictionMethod::StationHarmonicsV0,
    )
    .map_err(CliError::from)
}

fn benchmark_stats(residuals_m: &[f64]) -> Option<BenchmarkStats> {
    if residuals_m.is_empty() {
        return None;
    }
    let samples = residuals_m.len() as f64;
    let bias_m = residuals_m.iter().sum::<f64>() / samples;
    let rms_m = (residuals_m
        .iter()
        .map(|residual| residual * residual)
        .sum::<f64>()
        / samples)
        .sqrt();
    let mae_m = residuals_m
        .iter()
        .map(|residual| residual.abs())
        .sum::<f64>()
        / samples;
    let mut absolute = residuals_m
        .iter()
        .map(|residual| residual.abs())
        .collect::<Vec<_>>();
    absolute.sort_by(|left, right| left.total_cmp(right));
    let p95_m = percentile(&absolute, 0.95)?;
    let max_m = absolute.last().copied().unwrap_or(0.0);
    Some(BenchmarkStats {
        rms_cm: rms_m * 100.0,
        bias_cm: bias_m * 100.0,
        mae_cm: mae_m * 100.0,
        p95_cm: p95_m * 100.0,
        max_cm: max_m * 100.0,
    })
}

fn print_benchmark_stats(name: &str, stats: BenchmarkStats) {
    println!(
        "{name},{:.1},{:.1},{:.1},{:.1},{:.1}",
        stats.rms_cm, stats.bias_cm, stats.mae_cm, stats.p95_cm, stats.max_cm
    );
}

fn validate_series_args(duration_h: u32, step_min: u32) -> Result<(), CliError> {
    if duration_h == 0 || duration_h > MAX_SERIES_DURATION_H {
        return Err(CliError::InvalidArgument(format!(
            "duration_h must be between 1 and {MAX_SERIES_DURATION_H}"
        )));
    }
    if step_min < MIN_SERIES_STEP_MIN {
        return Err(CliError::InvalidArgument(format!(
            "step_min must be at least {MIN_SERIES_STEP_MIN}"
        )));
    }
    Ok(())
}

fn validate_window_range(from: UtcDateTime, to: UtcDateTime) -> Result<(), CliError> {
    if to <= from {
        return Err(CliError::InvalidArgument(
            "to must be after from".to_string(),
        ));
    }
    if to.seconds_since(from) > MAX_WINDOWS_DURATION_SECONDS {
        return Err(CliError::InvalidArgument(
            "window range must be at most 31 days".to_string(),
        ));
    }
    Ok(())
}

fn threshold_args(
    above_m: Option<f64>,
    below_m: Option<f64>,
) -> Result<(Meters, TideThresholdDirection), CliError> {
    match (above_m, below_m) {
        (Some(_), Some(_)) => Err(CliError::InvalidArgument(
            "--above and --below are mutually exclusive".to_string(),
        )),
        (None, None) => Err(CliError::InvalidArgument(
            "one of --above or --below is required".to_string(),
        )),
        (Some(value), None) => Ok((Meters::new(value)?, TideThresholdDirection::Above)),
        (None, Some(value)) => Ok((Meters::new(value)?, TideThresholdDirection::Below)),
    }
}

fn extremum_json(extremum: amar_core::TideExtremum) -> serde_json::Value {
    json!({
        "t": format_utc(extremum.at()),
        "height_m": round3(extremum.height().as_meters()),
    })
}

fn point_json(point: amar_core::TidePoint) -> serde_json::Value {
    json!({
        "t": format_utc(point.at()),
        "height_m": round3(point.height().as_meters()),
    })
}

fn window_json(window: amar_core::TideWindow) -> serde_json::Value {
    json!({
        "start": format_utc(window.start()),
        "end": format_utc(window.end()),
    })
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn format_utc(at: UtcDateTime) -> String {
    at.as_chrono().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn parse_latitude(value: &str) -> Result<f64, String> {
    parse_coordinate(value, "latitude", -90.0, 90.0)
}

fn parse_longitude(value: &str) -> Result<f64, String> {
    parse_coordinate(value, "longitude", -180.0, 180.0)
}

fn parse_coordinate(value: &str, name: &str, min: f64, max: f64) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|error| format!("invalid {name} {value}: {error}"))?;
    if (min..=max).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(format!(
            "{name} must be between {min:.0} and {max:.0} degrees"
        ))
    }
}

fn parse_non_negative_f64(value: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|error| format!("invalid non-negative number {value}: {error}"))?;
    if parsed.is_finite() && parsed >= 0.0 {
        Ok(parsed)
    } else {
        Err("value must be a finite non-negative number".to_string())
    }
}

fn parse_finite_f64(value: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|error| format!("invalid number {value}: {error}"))?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err("value must be finite".to_string())
    }
}
