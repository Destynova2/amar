use amar::server::ServerError;
use amar_core::CoreError;
use amar_data::DataError;
use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub(crate) const DEFAULT_NOAA_PACK: &str = "data/packs/noaa_m0.json";
pub(crate) const DEFAULT_FIXTURES: &str = "fixtures/noaa";
pub(crate) const HILO_P95_TIME_LIMIT_MIN: f64 = 10.0;
pub(crate) const HILO_P95_HEIGHT_LIMIT_M: f64 = 0.03;

#[derive(Debug, Error)]
pub(crate) enum CliError {
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
    #[error("benchmark gate failed:\n{failures}")]
    BenchmarkThreshold { failures: String },
    #[error("hilo validation p95 exceeded:\n{failures}")]
    HiloThreshold { failures: String },
    #[error(
        "outside_validity_period station={station_id} at={at} valid_from={valid_from} valid_until={valid_until}"
    )]
    OutsideValidityPeriod {
        station_id: String,
        at: String,
        valid_from: String,
        valid_until: String,
    },
    #[error("{0}")]
    InvalidArgument(String),
}

#[derive(Debug, Args)]
pub(crate) struct ValidateArgs {
    #[arg(long, default_value = DEFAULT_NOAA_PACK)]
    pub(crate) pack: PathBuf,
    #[arg(long, default_value = DEFAULT_FIXTURES)]
    pub(crate) fixtures: PathBuf,
}

pub(crate) fn prediction_files(station_dir: &Path) -> Result<Vec<PathBuf>, CliError> {
    fixture_files(station_dir, "predictions_")
}

pub(crate) fn hilo_files(station_dir: &Path) -> Result<Vec<PathBuf>, CliError> {
    fixture_files(station_dir, "hilo_")
}

fn fixture_files(station_dir: &Path, prefix: &str) -> Result<Vec<PathBuf>, CliError> {
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
        if name.starts_with(prefix) && name.ends_with(".json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

pub(crate) fn prediction_window_label(path: &Path) -> String {
    fixture_window_label(path, "predictions_")
}

pub(crate) fn hilo_window_label(path: &Path) -> String {
    fixture_window_label(path, "hilo_")
}

fn fixture_window_label(path: &Path, prefix: &str) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix(prefix))
        .unwrap_or("unknown")
        .to_string()
}
