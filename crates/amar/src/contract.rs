use amar_core::{Meters, TideExtremum, TidePoint, TideThresholdDirection, TideWindow, UtcDateTime};
use serde::Serialize;

pub const NEXT_EXTREMA_HORIZON_H: u32 = 72;
pub const MAX_SERIES_DURATION_H: u32 = 72;
pub const MIN_SERIES_STEP_MIN: u32 = 6;
pub const DEFAULT_SERIES_STEP_MIN: u32 = 60;
pub const MAX_WINDOWS_DURATION_SECONDS: i64 = 31 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractViolation {
    message: String,
}

impl ContractViolation {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn into_message(self) -> String {
        self.message
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThresholdField {
    Above,
    Below,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThresholdOptionsError {
    MutuallyExclusive,
    Missing,
    NonFinite { field: ThresholdField, value: f64 },
}

pub fn validate_series_bounds(duration_h: u32, step_min: u32) -> Result<(), ContractViolation> {
    if duration_h == 0 || duration_h > MAX_SERIES_DURATION_H {
        return Err(ContractViolation::new(format!(
            "duration_h must be between 1 and {MAX_SERIES_DURATION_H}"
        )));
    }
    if step_min < MIN_SERIES_STEP_MIN {
        return Err(ContractViolation::new(format!(
            "step_min must be at least {MIN_SERIES_STEP_MIN}"
        )));
    }
    Ok(())
}

pub fn validate_window_range(from: UtcDateTime, to: UtcDateTime) -> Result<(), ContractViolation> {
    if to <= from {
        return Err(ContractViolation::new("to must be after from"));
    }
    if to.seconds_since(from) > MAX_WINDOWS_DURATION_SECONDS {
        return Err(ContractViolation::new(
            "window range must be at most 31 days",
        ));
    }
    Ok(())
}

pub fn threshold_options(
    above_m: Option<f64>,
    below_m: Option<f64>,
) -> Result<(Meters, TideThresholdDirection), ThresholdOptionsError> {
    match (above_m, below_m) {
        (Some(_), Some(_)) => Err(ThresholdOptionsError::MutuallyExclusive),
        (None, None) => Err(ThresholdOptionsError::Missing),
        (Some(value), None) => {
            let threshold = Meters::new(value).map_err(|_| ThresholdOptionsError::NonFinite {
                field: ThresholdField::Above,
                value,
            })?;
            Ok((threshold, TideThresholdDirection::Above))
        }
        (None, Some(value)) => {
            let threshold = Meters::new(value).map_err(|_| ThresholdOptionsError::NonFinite {
                field: ThresholdField::Below,
                value,
            })?;
            Ok((threshold, TideThresholdDirection::Below))
        }
    }
}

pub fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

pub fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

pub fn format_utc(at: UtcDateTime) -> String {
    at.as_chrono().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[derive(Debug, Serialize)]
pub struct TideExtremumResponse {
    t: String,
    height_m: f64,
}

impl From<TideExtremum> for TideExtremumResponse {
    fn from(extremum: TideExtremum) -> Self {
        Self {
            t: format_utc(extremum.at()),
            height_m: round3(extremum.height().as_meters()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TidePointResponse {
    t: String,
    height_m: f64,
}

impl From<TidePoint> for TidePointResponse {
    fn from(point: TidePoint) -> Self {
        Self {
            t: format_utc(point.at()),
            height_m: round3(point.height().as_meters()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TideWindowResponse {
    start: String,
    end: String,
}

impl From<TideWindow> for TideWindowResponse {
    fn from(window: TideWindow) -> Self {
        Self {
            start: format_utc(window.start()),
            end: format_utc(window.end()),
        }
    }
}
