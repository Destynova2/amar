use amar_core::{
    Meters, TideExtremum, TidePoint, TideThresholdDirection, TideWindow, UtcDateTime,
    next_extrema_after, predict_height,
};
use amar_data::{DataSet, StationMatch};
use amar_pack::StationPack;
use serde::Serialize;

pub const NEXT_EXTREMA_HORIZON_H: u32 = 72;
pub const MAX_SERIES_DURATION_H: u32 = 72;
pub const MIN_SERIES_STEP_MIN: u32 = 6;
pub const DEFAULT_SERIES_STEP_MIN: u32 = 60;
pub const MAX_WINDOWS_DURATION_SECONDS: i64 = 31 * 24 * 60 * 60;

/// Confidence heuristic identifier returned by M1 tide responses.
pub const CONFIDENCE_METHOD: &str = "station_harmonics_v0_distance_heuristic";

/// Maximum distance covered by the documented M1 confidence scale.
pub const MAX_CONFIDENCE_DISTANCE_KM: f64 = 20.0;

/// Safety warnings attached to every successful M1 tide response.
pub const DEFAULT_WARNINGS: [&str; 3] = [
    "astronomical_tide_only",
    "not_for_navigation",
    "no_weather_surge",
];

pub const OUTSIDE_VALIDITY_PERIOD_WARNING: &str = "outside_validity_period";
pub const DATUM_REFERENCE_INCOMPLETE_WARNING: &str = "datum_reference_incomplete";

/// Distance confidence scale shared by the CLI and HTTP API.
pub const CONFIDENCE_GRADES: [ConfidenceGrade; 3] = [
    ConfidenceGrade::new(2.0, "A", 8),
    ConfidenceGrade::new(10.0, "B", 15),
    ConfidenceGrade::new(MAX_CONFIDENCE_DISTANCE_KM, "C", 30),
];

#[derive(Clone, Copy, Debug)]
pub struct ConfidenceGrade {
    pub max_distance_km: f64,
    pub grade: &'static str,
    pub sigma_cm: u16,
}

impl ConfidenceGrade {
    pub const fn new(max_distance_km: f64, grade: &'static str, sigma_cm: u16) -> Self {
        Self {
            max_distance_km,
            grade,
            sigma_cm,
        }
    }
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputDatum {
    Default,
    Ign69,
    Recent,
}

#[derive(Clone, Debug)]
pub struct DatumAdjustment {
    pub datum: String,
    pub height_offset_m: f64,
    pub reference_incomplete: bool,
}

impl OutputDatum {
    pub fn parse(value: &str) -> Result<Self, ContractViolation> {
        match value {
            "zero_hydrographique" | "zero-hydrographique" | "zh" | "default" => Ok(Self::Default),
            "ign69" | "IGN69" => Ok(Self::Ign69),
            "recent" | "internal" | "zero_hydrographique_recent" | "zero-hydrographique-recent" => {
                Ok(Self::Recent)
            }
            _ => Err(ContractViolation::new(
                "datum must be one of zero_hydrographique, ign69, recent",
            )),
        }
    }
}

pub fn datum_adjustment(
    station: &StationPack,
    requested: OutputDatum,
) -> Result<DatumAdjustment, ContractViolation> {
    match requested {
        OutputDatum::Default => default_datum_adjustment(station),
        OutputDatum::Ign69 => ign69_datum_adjustment(station),
        OutputDatum::Recent => Ok(DatumAdjustment {
            datum: recent_datum_name(station),
            height_offset_m: 0.0,
            reference_incomplete: false,
        }),
    }
}

fn default_datum_adjustment(station: &StationPack) -> Result<DatumAdjustment, ContractViolation> {
    let Some(reference) = station.datum_reference.as_ref() else {
        return Ok(DatumAdjustment {
            datum: station.datum.clone(),
            height_offset_m: 0.0,
            reference_incomplete: station.experimental == Some(true),
        });
    };
    let Some(offset) = reference.offset_zh_officiel_m else {
        return Ok(DatumAdjustment {
            datum: station.datum.clone(),
            height_offset_m: 0.0,
            reference_incomplete: station.experimental == Some(true),
        });
    };
    Ok(DatumAdjustment {
        datum: format!("{}_officiel", station.datum),
        height_offset_m: offset.get(),
        reference_incomplete: false,
    })
}

fn ign69_datum_adjustment(station: &StationPack) -> Result<DatumAdjustment, ContractViolation> {
    let offset = station
        .datum_reference
        .as_ref()
        .and_then(|reference| reference.offset_ign69_m)
        .ok_or_else(|| {
            ContractViolation::new(format!(
                "datum ign69 is unavailable for {}",
                station.station_id
            ))
        })?;
    Ok(DatumAdjustment {
        datum: "IGN69".to_string(),
        height_offset_m: offset.get(),
        reference_incomplete: false,
    })
}

fn recent_datum_name(station: &StationPack) -> String {
    if station.experimental == Some(true) && station.datum.starts_with("zero_hydrographique") {
        format!("{}_recent", station.datum)
    } else {
        station.datum.clone()
    }
}

#[derive(Debug, Serialize)]
pub struct TideExtremumResponse {
    t: String,
    height_m: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    coefficient: Option<u8>,
}

impl From<TideExtremum> for TideExtremumResponse {
    fn from(extremum: TideExtremum) -> Self {
        Self::from_extremum(extremum, None)
    }
}

impl TideExtremumResponse {
    pub fn from_extremum(extremum: TideExtremum, coefficient: Option<u8>) -> Self {
        Self::from_extremum_with_offset(extremum, 0.0, coefficient)
    }

    pub fn from_extremum_with_offset(
        extremum: TideExtremum,
        height_offset_m: f64,
        coefficient: Option<u8>,
    ) -> Self {
        Self {
            t: format_utc(extremum.at()),
            height_m: round3(extremum.height().as_meters() + height_offset_m),
            coefficient,
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
        Self::from_point_with_offset(point, 0.0)
    }
}

impl TidePointResponse {
    pub fn from_point_with_offset(point: TidePoint, height_offset_m: f64) -> Self {
        Self {
            t: format_utc(point.at()),
            height_m: round3(point.height().as_meters() + height_offset_m),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TidePayload {
    height_m: f64,
    next_high: Option<TideExtremumResponse>,
    next_low: Option<TideExtremumResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    series: Option<Vec<TidePointResponse>>,
    datum: String,
    source: SourceResponse,
    confidence: ConfidenceResponse,
    warnings: Vec<&'static str>,
}

pub type TideResponse = TidePayload;
pub type SeriesResponse = TidePayload;

pub fn build_tide_payload(
    station_match: &StationMatch<'_>,
    at: UtcDateTime,
    datum_adjustment: &DatumAdjustment,
    data: &DataSet,
    series: Option<Vec<TidePoint>>,
) -> Option<TidePayload> {
    let station = station_match.station.pack();
    let prediction = predict_height(station_match.station.model(), at);
    let confidence = confidence_for_station(station_match)?;
    let (next_high, next_low) =
        next_extrema_after(station_match.station.model(), at, NEXT_EXTREMA_HORIZON_H);
    let next_high_coefficient = next_high.and_then(|high| {
        crate::coefficient::coefficient_for_station_high(data, station, high.at())
            .map(|coefficient| coefficient.coefficient)
    });
    Some(TidePayload {
        height_m: round3(prediction.height().as_meters() + datum_adjustment.height_offset_m),
        next_high: next_high.map(|high| {
            TideExtremumResponse::from_extremum_with_offset(
                high,
                datum_adjustment.height_offset_m,
                next_high_coefficient,
            )
        }),
        next_low: next_low.map(|low| {
            TideExtremumResponse::from_extremum_with_offset(
                low,
                datum_adjustment.height_offset_m,
                None,
            )
        }),
        series: series.map(|series| {
            series
                .into_iter()
                .map(|point| {
                    TidePointResponse::from_point_with_offset(
                        point,
                        datum_adjustment.height_offset_m,
                    )
                })
                .collect()
        }),
        datum: datum_adjustment.datum.clone(),
        source: SourceResponse::from(station_match),
        confidence,
        warnings: warnings_for_station_at_with_options(
            station,
            Some(at),
            next_high_coefficient.is_some(),
            datum_adjustment.reference_incomplete,
        ),
    })
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

/// Serialized station source metadata shared by CLI and HTTP responses.
#[derive(Debug, Serialize)]
pub struct SourceResponse {
    pub(crate) kind: &'static str,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) distance_km: f64,
    pub(crate) data_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) valid_until: Option<String>,
}

impl From<&StationMatch<'_>> for SourceResponse {
    fn from(station_match: &StationMatch<'_>) -> Self {
        let station = station_match.station.pack();
        Self {
            kind: "station",
            id: station.station_id.clone(),
            name: station.name.clone(),
            distance_km: round3(station_match.distance_km),
            data_version: station
                .data_version
                .clone()
                .unwrap_or_else(|| station.source.extracted_at.clone()),
            valid_until: station.valid_until.clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidityPeriodViolation {
    pub valid_from: String,
    pub valid_until: String,
}

pub fn validity_period_violation(
    station: &StationPack,
    at: UtcDateTime,
) -> Option<ValidityPeriodViolation> {
    let valid_from = station.valid_from.as_ref()?;
    let valid_until = station.valid_until.as_ref()?;
    let from = UtcDateTime::parse_rfc3339(valid_from).ok()?;
    let until = UtcDateTime::parse_rfc3339(valid_until).ok()?;
    if at < from || at > until {
        Some(ValidityPeriodViolation {
            valid_from: valid_from.clone(),
            valid_until: valid_until.clone(),
        })
    } else {
        None
    }
}

pub fn strict_validity_period_violation(
    station: &StationPack,
    at: UtcDateTime,
    strict_validity: bool,
) -> Option<ValidityPeriodViolation> {
    if strict_validity {
        validity_period_violation(station, at)
    } else {
        None
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ConfidenceResponse {
    Distance {
        grade: &'static str,
        sigma_cm: u16,
        method: &'static str,
    },
    Experimental {
        method: &'static str,
        residual_benchmark_cm: f64,
        validation_period: String,
    },
}

/// Confidence metadata for a matched station.
///
/// NOAA-style stations use the M1 distance heuristic. Experimental stations
/// must carry empirical benchmark metadata in their pack.
pub fn confidence_for_station(station_match: &StationMatch<'_>) -> Option<ConfidenceResponse> {
    let station = station_match.station.pack();
    if station.experimental == Some(true) {
        let validation_period = station.validation_period.as_ref()?;
        return Some(ConfidenceResponse::Experimental {
            method: "calibrated_station_experimental",
            residual_benchmark_cm: round1(station.residual_benchmark_cm?),
            validation_period: format!("{}/{}", validation_period.start, validation_period.end),
        });
    }
    confidence_for_distance_km(station_match.distance_km)
}

/// M1 confidence is deliberately distance-only.
///
/// Later milestones replace this with empirical validation, not a wider radius.
pub fn confidence_for_distance_km(distance_km: f64) -> Option<ConfidenceResponse> {
    let confidence = CONFIDENCE_GRADES
        .iter()
        .find(|confidence| distance_km <= confidence.max_distance_km)?;
    Some(ConfidenceResponse::Distance {
        grade: confidence.grade,
        sigma_cm: confidence.sigma_cm,
        method: CONFIDENCE_METHOD,
    })
}

/// Warning set shared by CLI and HTTP responses.
pub fn warnings_for_station(station: &StationPack) -> Vec<&'static str> {
    warnings_for_station_at_with_options(station, None, false, false)
}

pub fn warnings_for_station_at_with_options(
    station: &StationPack,
    at: Option<UtcDateTime>,
    has_coefficient: bool,
    datum_reference_incomplete: bool,
) -> Vec<&'static str> {
    let mut warnings = DEFAULT_WARNINGS.to_vec();
    if station.experimental == Some(true) {
        warnings.push("experimental");
    }
    if station.not_shom == Some(true) {
        warnings.push("not_shom");
    }
    if at.is_some_and(|at| validity_period_violation(station, at).is_some()) {
        warnings.push(OUTSIDE_VALIDITY_PERIOD_WARNING);
    }
    if has_coefficient {
        warnings.push(crate::coefficient::COEFFICIENT_EXPERIMENTAL_WARNING);
    }
    if datum_reference_incomplete {
        warnings.push(DATUM_REFERENCE_INCOMPLETE_WARNING);
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confidence(distance_km: f64) -> ConfidenceResponse {
        match confidence_for_distance_km(distance_km) {
            Some(confidence) => confidence,
            None => panic!("expected confidence for {distance_km} km"),
        }
    }

    #[test]
    fn confidence_grade_b_is_bounded_at_ten_km() {
        let confidence = confidence(10.0);

        match confidence {
            ConfidenceResponse::Distance {
                grade,
                sigma_cm,
                method,
            } => {
                assert_eq!(grade, "B");
                assert_eq!(sigma_cm, 15);
                assert_eq!(method, CONFIDENCE_METHOD);
            }
            ConfidenceResponse::Experimental { .. } => panic!("expected distance confidence"),
        }
    }

    #[test]
    fn confidence_grade_c_is_bounded_at_twenty_km() {
        let confidence = confidence(MAX_CONFIDENCE_DISTANCE_KM);

        match confidence {
            ConfidenceResponse::Distance {
                grade, sigma_cm, ..
            } => {
                assert_eq!(grade, "C");
                assert_eq!(sigma_cm, 30);
            }
            ConfidenceResponse::Experimental { .. } => panic!("expected distance confidence"),
        }
    }

    #[test]
    fn confidence_is_undefined_beyond_documented_domain() {
        assert!(confidence_for_distance_km(MAX_CONFIDENCE_DISTANCE_KM + 0.001).is_none());
    }
}
