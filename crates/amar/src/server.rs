use amar_core::{
    Meters, TideThresholdDirection, UtcDateTime, next_extrema_after, predict_height,
    predict_series, tide_windows,
};
use amar_data::{DataError, DataSet, StationMatch, load_packs_from_paths};
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;

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

/// Distance confidence scale shared by the CLI and HTTP API.
pub const CONFIDENCE_GRADES: [ConfidenceGrade; 3] = [
    ConfidenceGrade::new(2.0, "A", 8),
    ConfidenceGrade::new(10.0, "B", 15),
    ConfidenceGrade::new(MAX_CONFIDENCE_DISTANCE_KM, "C", 30),
];
const NEXT_EXTREMA_HORIZON_H: u32 = 72;
const MAX_SERIES_DURATION_H: u32 = 72;
const MIN_SERIES_STEP_MIN: u32 = 6;
const MAX_WINDOWS_DURATION_SECONDS: i64 = 31 * 24 * 60 * 60;

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

/// Errors returned while loading data or running the HTTP server.
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("{0}")]
    Data(#[from] DataError),
    #[error("failed to bind {addr}: {source}")]
    Bind {
        addr: String,
        source: std::io::Error,
    },
    #[error("failed to read listener address: {0}")]
    LocalAddr(std::io::Error),
    #[error("server error: {0}")]
    Serve(std::io::Error),
}

#[derive(Clone)]
struct AppState {
    data: Arc<DataSet>,
    max_distance_km: f64,
}

/// Build the M1 API router from an already loaded station pack.
pub fn app(data: DataSet, max_distance_km: f64) -> Router {
    let max_distance_km = max_distance_km.min(MAX_CONFIDENCE_DISTANCE_KM);
    Router::new()
        .route("/tide", post(post_tide))
        .route("/tide/series", post(post_tide_series))
        .route("/tide/windows", post(post_tide_windows))
        .route("/health", get(get_health))
        .route("/coverage", get(get_coverage))
        .with_state(AppState {
            data: Arc::new(data),
            max_distance_km,
        })
}

/// Load a station pack and serve the M1 HTTP API on the requested address.
pub async fn serve(
    addr: &str,
    pack_paths: &[std::path::PathBuf],
    max_distance_km: f64,
) -> Result<(), ServerError> {
    let data = load_packs_from_paths(pack_paths)?;
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|source| ServerError::Bind {
            addr: addr.to_string(),
            source,
        })?;
    let local_addr = listener.local_addr().map_err(ServerError::LocalAddr)?;
    eprintln!("amar serve listening on http://{local_addr}");
    axum::serve(listener, app(data, max_distance_km))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServerError::Serve)
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("failed to listen for shutdown signal: {error}");
    }
}

async fn post_tide(
    State(state): State<AppState>,
    payload: Result<Json<TideRequest>, JsonRejection>,
) -> Result<Json<TideResponse>, ApiError> {
    let Json(request) = payload.map_err(|rejection| {
        ApiError::invalid_request(format!(
            "invalid JSON request body: {}",
            rejection.body_text()
        ))
    })?;
    validate_coordinate("latitude", request.lat, -90.0, 90.0)?;
    validate_coordinate("longitude", request.lon, -180.0, 180.0)?;
    let at = UtcDateTime::parse_rfc3339(&request.datetime)
        .map_err(|_| ApiError::invalid_request("datetime must be a readable RFC 3339 timestamp"))?;
    let station_match = supported_station(&state, request.lat, request.lon)?;
    let prediction = predict_height(station_match.station.model(), at);
    let station = station_match.station.pack();
    let Some(confidence) = confidence_for_station(&station_match) else {
        return Err(ApiError::unsupported_station_confidence(station));
    };
    let (next_high, next_low) =
        next_extrema_after(station_match.station.model(), at, NEXT_EXTREMA_HORIZON_H);

    Ok(Json(TideResponse {
        height_m: round3(prediction.height().as_meters()),
        next_high: next_high.map(TideExtremumResponse::from),
        next_low: next_low.map(TideExtremumResponse::from),
        datum: station.datum.clone(),
        source: SourceResponse::from(&station_match),
        confidence,
        warnings: warnings_for_station(station),
    }))
}

async fn post_tide_series(
    State(state): State<AppState>,
    payload: Result<Json<SeriesRequest>, JsonRejection>,
) -> Result<Json<SeriesResponse>, ApiError> {
    let Json(request) = payload.map_err(|rejection| {
        ApiError::invalid_request(format!(
            "invalid JSON request body: {}",
            rejection.body_text()
        ))
    })?;
    validate_coordinate("latitude", request.lat, -90.0, 90.0)?;
    validate_coordinate("longitude", request.lon, -180.0, 180.0)?;
    if request.duration_h == 0 || request.duration_h > MAX_SERIES_DURATION_H {
        return Err(ApiError::invalid_request(format!(
            "duration_h must be between 1 and {MAX_SERIES_DURATION_H}"
        )));
    }
    if request.step_min < MIN_SERIES_STEP_MIN {
        return Err(ApiError::invalid_request(format!(
            "step_min must be at least {MIN_SERIES_STEP_MIN}"
        )));
    }
    let from = parse_time_field("from", &request.from)?;
    let station_match = supported_station(&state, request.lat, request.lon)?;
    let station = station_match.station.pack();
    let Some(confidence) = confidence_for_station(&station_match) else {
        return Err(ApiError::unsupported_station_confidence(station));
    };
    let series = predict_series(
        station_match.station.model(),
        from,
        request.duration_h,
        request.step_min,
    )
    .into_iter()
    .map(TidePointResponse::from)
    .collect();

    Ok(Json(SeriesResponse {
        series,
        datum: station.datum.clone(),
        source: SourceResponse::from(&station_match),
        confidence,
        warnings: warnings_for_station(station),
    }))
}

async fn post_tide_windows(
    State(state): State<AppState>,
    payload: Result<Json<WindowsRequest>, JsonRejection>,
) -> Result<Json<WindowsResponse>, ApiError> {
    let Json(request) = payload.map_err(|rejection| {
        ApiError::invalid_request(format!(
            "invalid JSON request body: {}",
            rejection.body_text()
        ))
    })?;
    validate_coordinate("latitude", request.lat, -90.0, 90.0)?;
    validate_coordinate("longitude", request.lon, -180.0, 180.0)?;
    let from = parse_time_field("from", &request.from)?;
    let to = parse_time_field("to", &request.to)?;
    if to <= from {
        return Err(ApiError::invalid_request("to must be after from"));
    }
    if to.seconds_since(from) > MAX_WINDOWS_DURATION_SECONDS {
        return Err(ApiError::invalid_request(
            "window range must be at most 31 days",
        ));
    }
    let (threshold, direction) = threshold_request(request.above_m, request.below_m)?;
    let station_match = supported_station(&state, request.lat, request.lon)?;
    let station = station_match.station.pack();
    let Some(confidence) = confidence_for_station(&station_match) else {
        return Err(ApiError::unsupported_station_confidence(station));
    };
    let windows = tide_windows(
        station_match.station.model(),
        from,
        to,
        threshold,
        direction,
    )
    .into_iter()
    .map(TideWindowResponse::from)
    .collect();

    Ok(Json(WindowsResponse {
        windows,
        datum: station.datum.clone(),
        source: SourceResponse::from(&station_match),
        confidence,
        warnings: warnings_for_station(station),
    }))
}

async fn get_health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        version: env!("CARGO_PKG_VERSION"),
        station_count: state.data.stations().len(),
        data_version: state.data.pack().generated_at.clone(),
    })
}

async fn get_coverage(State(state): State<AppState>) -> Json<CoverageResponse> {
    Json(CoverageResponse {
        data_version: state.data.pack().generated_at.clone(),
        stations: state
            .data
            .stations()
            .iter()
            .map(|station| {
                let station = station.pack();
                CoverageStation {
                    id: station.station_id.clone(),
                    name: station.name.clone(),
                    lat: station.latitude_deg.get(),
                    lon: station.longitude_deg.get(),
                    datum: station.datum.clone(),
                    max_distance_km: state.max_distance_km,
                }
            })
            .collect(),
    })
}

fn supported_station(state: &AppState, lat: f64, lon: f64) -> Result<StationMatch<'_>, ApiError> {
    let Some(station_match) = state.data.closest_station(lat, lon) else {
        return Err(ApiError::no_supported_source(None, state.max_distance_km));
    };
    if station_match.distance_km <= state.max_distance_km {
        Ok(station_match)
    } else {
        Err(ApiError::no_supported_source(
            Some(station_match),
            state.max_distance_km,
        ))
    }
}

fn validate_coordinate(name: &'static str, value: f64, min: f64, max: f64) -> Result<(), ApiError> {
    if !value.is_finite() {
        return Err(ApiError::invalid_request(format!("{name} must be finite")));
    }
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(ApiError::invalid_request(format!(
            "{name} must be between {min:.0} and {max:.0} degrees"
        )))
    }
}

fn parse_time_field(name: &'static str, value: &str) -> Result<UtcDateTime, ApiError> {
    UtcDateTime::parse_rfc3339(value).map_err(|_| {
        ApiError::invalid_request(format!("{name} must be a readable RFC 3339 timestamp"))
    })
}

fn threshold_request(
    above_m: Option<f64>,
    below_m: Option<f64>,
) -> Result<(Meters, TideThresholdDirection), ApiError> {
    match (above_m, below_m) {
        (Some(_), Some(_)) => Err(ApiError::invalid_request(
            "above_m and below_m are mutually exclusive",
        )),
        (None, None) => Err(ApiError::invalid_request(
            "one of above_m or below_m is required",
        )),
        (Some(value), None) => {
            let threshold = Meters::new(value)
                .map_err(|_| ApiError::invalid_request("above_m must be finite"))?;
            Ok((threshold, TideThresholdDirection::Above))
        }
        (None, Some(value)) => {
            let threshold = Meters::new(value)
                .map_err(|_| ApiError::invalid_request("below_m must be finite"))?;
            Ok((threshold, TideThresholdDirection::Below))
        }
    }
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
pub fn warnings_for_station(station: &amar_pack::StationPack) -> Vec<&'static str> {
    let mut warnings = DEFAULT_WARNINGS.to_vec();
    if station.experimental == Some(true) {
        warnings.push("experimental");
    }
    if station.not_shom == Some(true) {
        warnings.push("not_shom");
    }
    warnings
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn format_utc(at: UtcDateTime) -> String {
    at.as_chrono().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[derive(Debug, Deserialize)]
struct TideRequest {
    lat: f64,
    lon: f64,
    datetime: String,
}

#[derive(Debug, Serialize)]
struct TideResponse {
    height_m: f64,
    next_high: Option<TideExtremumResponse>,
    next_low: Option<TideExtremumResponse>,
    datum: String,
    source: SourceResponse,
    confidence: ConfidenceResponse,
    warnings: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct SeriesRequest {
    lat: f64,
    lon: f64,
    from: String,
    duration_h: u32,
    step_min: u32,
}

#[derive(Debug, Serialize)]
struct SeriesResponse {
    series: Vec<TidePointResponse>,
    datum: String,
    source: SourceResponse,
    confidence: ConfidenceResponse,
    warnings: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct WindowsRequest {
    lat: f64,
    lon: f64,
    from: String,
    to: String,
    above_m: Option<f64>,
    below_m: Option<f64>,
}

#[derive(Debug, Serialize)]
struct WindowsResponse {
    windows: Vec<TideWindowResponse>,
    datum: String,
    source: SourceResponse,
    confidence: ConfidenceResponse,
    warnings: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct TideExtremumResponse {
    t: String,
    height_m: f64,
}

impl From<amar_core::TideExtremum> for TideExtremumResponse {
    fn from(extremum: amar_core::TideExtremum) -> Self {
        Self {
            t: format_utc(extremum.at()),
            height_m: round3(extremum.height().as_meters()),
        }
    }
}

#[derive(Debug, Serialize)]
struct TidePointResponse {
    t: String,
    height_m: f64,
}

impl From<amar_core::TidePoint> for TidePointResponse {
    fn from(point: amar_core::TidePoint) -> Self {
        Self {
            t: format_utc(point.at()),
            height_m: round3(point.height().as_meters()),
        }
    }
}

#[derive(Debug, Serialize)]
struct TideWindowResponse {
    start: String,
    end: String,
}

impl From<amar_core::TideWindow> for TideWindowResponse {
    fn from(window: amar_core::TideWindow) -> Self {
        Self {
            start: format_utc(window.start()),
            end: format_utc(window.end()),
        }
    }
}

/// Serialized station source metadata shared by CLI and HTTP responses.
#[derive(Debug, Serialize)]
pub struct SourceResponse {
    kind: &'static str,
    id: String,
    name: String,
    distance_km: f64,
    data_version: String,
}

impl From<&StationMatch<'_>> for SourceResponse {
    fn from(station_match: &StationMatch<'_>) -> Self {
        let station = station_match.station.pack();
        Self {
            kind: "station",
            id: station.station_id.clone(),
            name: station.name.clone(),
            distance_km: round3(station_match.distance_km),
            data_version: station.source.extracted_at.clone(),
        }
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

#[derive(Debug, Serialize)]
struct HealthResponse {
    version: &'static str,
    station_count: usize,
    data_version: String,
}

#[derive(Debug, Serialize)]
struct CoverageResponse {
    data_version: String,
    stations: Vec<CoverageStation>,
}

#[derive(Debug, Serialize)]
struct CoverageStation {
    id: String,
    name: String,
    lat: f64,
    lon: f64,
    datum: String,
    max_distance_km: f64,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    body: Box<ErrorResponse>,
}

impl ApiError {
    fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: Box::new(ErrorResponse {
                error: "invalid_request",
                message: message.into(),
                max_distance_km: None,
                nearest_source: None,
            }),
        }
    }

    fn no_supported_source(nearest_source: Option<StationMatch<'_>>, max_distance_km: f64) -> Self {
        let nearest_source =
            nearest_source.map(|station_match| SourceResponse::from(&station_match));
        let message = match &nearest_source {
            Some(source) => format!(
                "no supported source within {max_distance_km:.1} km; nearest station is {} {} at {:.3} km",
                source.id, source.name, source.distance_km
            ),
            None => format!("no supported source within {max_distance_km:.1} km"),
        };
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: Box::new(ErrorResponse {
                error: "no_supported_source",
                message,
                max_distance_km: Some(max_distance_km),
                nearest_source,
            }),
        }
    }

    fn unsupported_station_confidence(station: &amar_pack::StationPack) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: Box::new(ErrorResponse {
                error: "no_supported_source",
                message: format!(
                    "station {} has no supported confidence metadata",
                    station.station_id
                ),
                max_distance_km: None,
                nearest_source: None,
            }),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(*self.body)).into_response()
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_distance_km: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nearest_source: Option<SourceResponse>,
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
