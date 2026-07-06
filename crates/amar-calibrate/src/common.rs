use amar_core::CoreError;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use thiserror::Error;

pub(crate) const REFMAR_BASE: &str = "https://services.data.shom.fr/maregraphie";
pub(crate) const BREST_SHOM_ID: &str = "3";
pub(crate) const VALIDATED_HOURLY_SOURCE: u8 = 4;

#[derive(Debug, Error)]
pub(crate) enum CalError {
    #[error("{0}")]
    Core(#[from] CoreError),
    #[error("{0}")]
    Pack(#[from] amar_pack::PackError),
    #[error("{0}")]
    Data(#[from] amar_data::DataError),
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid timestamp {0}")]
    InvalidTimestamp(String),
    #[error(
        "open-meteo hourly time length {time_len} does not match surface_pressure length {pressure_len}"
    )]
    OpenMeteoHourlyLength {
        time_len: usize,
        pressure_len: usize,
    },
    #[error(
        "open-meteo timezone must be GMT with utc_offset_seconds=0, got {timezone} with offset {utc_offset_seconds}"
    )]
    OpenMeteoTimezone {
        timezone: String,
        utc_offset_seconds: i32,
    },
    #[error("invalid observation CSV line {line}: {reason}")]
    InvalidCsvLine { line: String, reason: String },
    #[error("no observations available for {0}")]
    EmptyObservations(String),
    #[error("least-squares solve failed")]
    SolveFailed,
    #[error("missing {field} for station {station_id}")]
    MissingStationPeriod {
        station_id: String,
        field: &'static str,
    },
    #[error("station {0} not found in pack")]
    MissingStation(String),
    #[error("quality gate failed: {0}")]
    QualityGate(String),
    #[error(
        "calibration window is too short for annual constituents SA/SSA: got {days} days, need at least {required_days} days; extend calibration_start or remove annual terms"
    )]
    UnresolvableAnnualConstituents { days: i64, required_days: i64 },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Observation {
    pub(crate) at: DateTime<Utc>,
    pub(crate) value_m: f64,
    pub(crate) source: u8,
}

pub(crate) fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>, CalError> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(|_| CalError::InvalidTimestamp(value.to_string()))
}

pub(crate) fn format_rfc3339(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
