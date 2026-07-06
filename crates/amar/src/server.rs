use amar_core::{CoreError, UtcDateTime, predict_height};
use amar_data::{DataError, DataSet, StationMatch, load_pack_from_path};
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;

pub const CONFIDENCE_METHOD: &str = "station_harmonics_v0_distance_heuristic";
pub const DEFAULT_WARNINGS: [&str; 3] = [
    "astronomical_tide_only",
    "not_for_navigation",
    "no_weather_surge",
];

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

pub fn app(data: DataSet, max_distance_km: f64) -> Router {
    Router::new()
        .route("/tide", post(post_tide))
        .route("/health", get(get_health))
        .route("/coverage", get(get_coverage))
        .with_state(AppState {
            data: Arc::new(data),
            max_distance_km,
        })
}

pub async fn serve(
    addr: &str,
    pack_path: impl AsRef<Path>,
    max_distance_km: f64,
) -> Result<(), ServerError> {
    let data = load_pack_from_path(pack_path)?;
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|source| ServerError::Bind {
            addr: addr.to_string(),
            source,
        })?;
    let local_addr = listener.local_addr().map_err(ServerError::LocalAddr)?;
    eprintln!("amar serve listening on http://{local_addr}");
    axum::serve(listener, app(data, max_distance_km))
        .await
        .map_err(ServerError::Serve)
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
    let confidence = confidence_for_distance_km(station_match.distance_km);

    Ok(Json(TideResponse {
        height_m: round3(prediction.height().as_meters()),
        datum: station.datum.clone(),
        source: SourceResponse {
            kind: "station",
            id: station.station_id.clone(),
            name: station.name.clone(),
            distance_km: round3(station_match.distance_km),
            data_version: station.source.extracted_at.clone(),
        },
        confidence,
        warnings: DEFAULT_WARNINGS,
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

// M1 v0.1 confidence is deliberately distance-only:
// <= 2 km -> A / 8 cm, <= 10 km -> B / 15 cm, <= 20 km -> C / 30 cm.
// Later milestones replace this with empirical validation, not a wider radius.
fn confidence_for_distance_km(distance_km: f64) -> ConfidenceResponse {
    let (grade, sigma_cm) = if distance_km <= 2.0 {
        ("A", 8)
    } else if distance_km <= 10.0 {
        ("B", 15)
    } else {
        ("C", 30)
    };
    ConfidenceResponse {
        grade,
        sigma_cm,
        method: CONFIDENCE_METHOD,
    }
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
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
    datum: String,
    source: SourceResponse,
    confidence: ConfidenceResponse,
    warnings: [&'static str; 3],
}

#[derive(Debug, Serialize)]
struct SourceResponse {
    kind: &'static str,
    id: String,
    name: String,
    distance_km: f64,
    data_version: String,
}

#[derive(Debug, Serialize)]
struct ConfidenceResponse {
    grade: &'static str,
    sigma_cm: u16,
    method: &'static str,
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
        let nearest_source = nearest_source.map(|station_match| {
            let station = station_match.station.pack();
            SourceResponse {
                kind: "station",
                id: station.station_id.clone(),
                name: station.name.clone(),
                distance_km: round3(station_match.distance_km),
                data_version: station.source.extracted_at.clone(),
            }
        });
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

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        Self::invalid_request(error.to_string())
    }
}
