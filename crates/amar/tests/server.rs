use amar::server::app;
use amar_data::load_packs_from_paths;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tower::ServiceExt;

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn noaa_pack_path() -> PathBuf {
    workspace_root().join("data/packs/noaa_m0.json")
}

fn brest_pack_path() -> PathBuf {
    workspace_root().join("data/packs/amar-data-brest-experimental.json")
}

fn pack_paths() -> Vec<PathBuf> {
    vec![noaa_pack_path(), brest_pack_path()]
}

#[tokio::test]
async fn tide_nominal_response_matches_snapshot() {
    let body = r#"{"lat":37.806,"lon":-122.465,"datetime":"2026-08-15T12:00:00Z"}"#;
    assert_post_snapshot(
        body,
        StatusCode::OK,
        include_str!("snapshots/tide_nominal_sf.json"),
    )
    .await;
}

#[tokio::test]
async fn tide_brest_experimental_response_matches_snapshot() {
    let body = r#"{"lat":48.383,"lon":-4.495,"datetime":"2026-08-15T12:00:00Z"}"#;
    assert_post_snapshot(
        body,
        StatusCode::OK,
        include_str!("snapshots/tide_brest_experimental.json"),
    )
    .await;
}

#[tokio::test]
async fn tide_invalid_input_matches_snapshot() {
    let body = r#"{"lat":91,"lon":0,"datetime":"2026-08-15T12:00:00Z"}"#;
    assert_post_snapshot(
        body,
        StatusCode::BAD_REQUEST,
        include_str!("snapshots/tide_invalid_400.json"),
    )
    .await;
}

#[tokio::test]
async fn tide_series_returns_bounded_points_with_metadata() {
    let actual = post_json(
        "/tide/series",
        r#"{"lat":37.806,"lon":-122.465,"from":"2026-08-15T12:00:00Z","duration_h":2,"step_min":60}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::OK);
    assert_eq!(actual.body["datum"], "MLLW");
    assert_eq!(actual.body["source"]["id"], "noaa:9414290");
    assert_eq!(actual.body["series"].as_array().map(Vec::len), Some(3));
    assert_eq!(actual.body["series"][0]["t"], "2026-08-15T12:00:00Z");
    assert_eq!(actual.body["series"][2]["t"], "2026-08-15T14:00:00Z");
}

#[tokio::test]
async fn tide_series_rejects_too_fine_step() {
    let actual = post_json(
        "/tide/series",
        r#"{"lat":37.806,"lon":-122.465,"from":"2026-08-15T12:00:00Z","duration_h":2,"step_min":5}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::BAD_REQUEST);
    assert_eq!(actual.body["error"], "invalid_request");
    assert_eq!(actual.body["message"], "step_min must be at least 6");
}

#[tokio::test]
async fn tide_series_rejects_zero_duration() {
    let actual = post_json(
        "/tide/series",
        r#"{"lat":37.806,"lon":-122.465,"from":"2026-08-15T12:00:00Z","duration_h":0,"step_min":60}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::BAD_REQUEST);
    assert_eq!(actual.body["error"], "invalid_request");
    assert_eq!(
        actual.body["message"],
        "duration_h must be between 1 and 72"
    );
}

#[tokio::test]
async fn tide_series_rejects_duration_above_limit() {
    let actual = post_json(
        "/tide/series",
        r#"{"lat":37.806,"lon":-122.465,"from":"2026-08-15T12:00:00Z","duration_h":73,"step_min":60}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::BAD_REQUEST);
    assert_eq!(actual.body["error"], "invalid_request");
    assert_eq!(
        actual.body["message"],
        "duration_h must be between 1 and 72"
    );
}

#[tokio::test]
async fn tide_windows_returns_threshold_crossing_windows() {
    let actual = post_json(
        "/tide/windows",
        r#"{"lat":37.806,"lon":-122.465,"from":"2026-08-15T00:00:00Z","to":"2026-08-16T00:00:00Z","above_m":1.5}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::OK);
    assert_eq!(actual.body["datum"], "MLLW");
    assert_eq!(actual.body["source"]["id"], "noaa:9414290");
    assert!(
        actual.body["windows"]
            .as_array()
            .is_some_and(|windows| !windows.is_empty())
    );
    assert!(actual.body["windows"][0]["start"].as_str().is_some());
    assert!(actual.body["windows"][0]["end"].as_str().is_some());
}

#[tokio::test]
async fn tide_windows_above_negative_threshold_returns_full_range() {
    let actual = post_json(
        "/tide/windows",
        r#"{"lat":48.383,"lon":-4.495,"from":"2026-08-15T00:00:00Z","to":"2026-08-16T12:00:00Z","above_m":-5}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::OK);
    let Some(windows) = actual.body["windows"].as_array() else {
        panic!("windows array");
    };
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0]["start"], "2026-08-15T00:00:00Z");
    assert_eq!(windows[0]["end"], "2026-08-16T12:00:00Z");
}

#[tokio::test]
async fn tide_windows_below_negative_threshold_returns_no_windows() {
    let actual = post_json(
        "/tide/windows",
        r#"{"lat":48.383,"lon":-4.495,"from":"2026-08-15T00:00:00Z","to":"2026-08-16T12:00:00Z","below_m":-5}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::OK);
    assert_eq!(actual.body["windows"].as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn tide_windows_rejects_range_above_limit() {
    let actual = post_json(
        "/tide/windows",
        r#"{"lat":37.806,"lon":-122.465,"from":"2026-08-15T00:00:00Z","to":"2026-09-16T00:00:00Z","above_m":1.5}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::BAD_REQUEST);
    assert_eq!(actual.body["error"], "invalid_request");
    assert_eq!(
        actual.body["message"],
        "window range must be at most 31 days"
    );
}

#[tokio::test]
async fn tide_windows_rejects_to_not_after_from() {
    let actual = post_json(
        "/tide/windows",
        r#"{"lat":37.806,"lon":-122.465,"from":"2026-08-15T00:00:00Z","to":"2026-08-15T00:00:00Z","above_m":1.5}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::BAD_REQUEST);
    assert_eq!(actual.body["error"], "invalid_request");
    assert_eq!(actual.body["message"], "to must be after from");
}

#[tokio::test]
async fn tide_windows_rejects_ambiguous_threshold() {
    let actual = post_json(
        "/tide/windows",
        r#"{"lat":37.806,"lon":-122.465,"from":"2026-08-15T00:00:00Z","to":"2026-08-16T00:00:00Z","above_m":1.5,"below_m":0.2}"#,
        20.0,
    )
    .await;

    assert_eq!(actual.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        actual.body["message"],
        "above_m and below_m are mutually exclusive"
    );
}

#[tokio::test]
async fn tide_malformed_json_returns_400() {
    let actual = post_tide(r#"{"lat":37.806,"lon":"#, 20.0).await;

    assert_eq!(actual.status, StatusCode::BAD_REQUEST);
    assert_eq!(actual.body["error"], "invalid_request");
    assert!(
        actual.body["message"]
            .as_str()
            .unwrap_or("")
            .contains("JSON")
    );
}

#[tokio::test]
async fn tide_invalid_datetime_returns_400() {
    let body = r#"{"lat":37.806,"lon":-122.465,"datetime":"tomorrow"}"#;
    let actual = post_tide(body, 20.0).await;

    assert_eq!(actual.status, StatusCode::BAD_REQUEST);
    assert_eq!(actual.body["error"], "invalid_request");
    assert_eq!(
        actual.body["message"],
        "datetime must be a readable RFC 3339 timestamp"
    );
}

#[tokio::test]
async fn tide_longitude_out_of_range_returns_400() {
    let body = r#"{"lat":0,"lon":181,"datetime":"2026-08-15T12:00:00Z"}"#;
    let actual = post_tide(body, 20.0).await;

    assert_eq!(actual.status, StatusCode::BAD_REQUEST);
    assert_eq!(actual.body["error"], "invalid_request");
    assert_eq!(
        actual.body["message"],
        "longitude must be between -180 and 180 degrees"
    );
}

#[tokio::test]
async fn tide_accepts_distance_equal_to_max_distance() {
    let data = must(load_packs_from_paths(&pack_paths()));
    let boundary_distance = match data.closest_station(37.806, -122.465) {
        Some(station_match) => station_match.distance_km,
        None => panic!("expected a closest station"),
    };
    let service = app(data, boundary_distance);
    let actual = request_json(
        service,
        must(
            Request::builder()
                .method(Method::POST)
                .uri("/tide")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"lat":37.806,"lon":-122.465,"datetime":"2026-08-15T12:00:00Z"}"#,
                )),
        ),
    )
    .await;

    assert_eq!(actual.status, StatusCode::OK);
}

#[tokio::test]
async fn tide_refuses_matches_beyond_confidence_domain() {
    let data = must(load_packs_from_paths(&pack_paths()));
    let station = match data.stations().first() {
        Some(station) => station.pack(),
        None => panic!("expected at least one station in pack"),
    };
    let body = format!(
        r#"{{"lat":{},"lon":{},"datetime":"2026-08-15T12:00:00Z"}}"#,
        station.latitude_deg.get() + 0.5,
        station.longitude_deg.get()
    );
    let service = app(data, 100.0);
    let actual = request_json(
        service,
        must(
            Request::builder()
                .method(Method::POST)
                .uri("/tide")
                .header("content-type", "application/json")
                .body(Body::from(body)),
        ),
    )
    .await;

    assert_eq!(actual.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(actual.body["max_distance_km"], 20.0);
}

#[tokio::test]
async fn health_and_coverage_expose_loaded_pack() {
    let data = must(load_packs_from_paths(&pack_paths()));
    let expected_station_count = data.stations().len();
    let service = app(data, 20.0);

    let health = request_json(
        service.clone(),
        must(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty()),
        ),
    )
    .await;
    assert_eq!(health.status, StatusCode::OK);
    assert_eq!(
        health.body["station_count"].as_u64(),
        Some(expected_station_count as u64)
    );
    assert_eq!(health.body["data_version"], "2026-07-06+2026-07-06-m2.2");

    let coverage = request_json(
        service,
        must(
            Request::builder()
                .method(Method::GET)
                .uri("/coverage")
                .body(Body::empty()),
        ),
    )
    .await;
    assert_eq!(coverage.status, StatusCode::OK);
    assert_eq!(
        coverage.body["stations"].as_array().map(Vec::len),
        Some(expected_station_count)
    );
    assert!(coverage.body.to_string().contains("noaa:9414290"));
}

async fn assert_post_snapshot(body: &str, expected_status: StatusCode, expected_snapshot: &str) {
    let actual = post_tide(body, 20.0).await;
    assert_eq!(actual.status, expected_status);
    let expected = must(serde_json::from_str::<Value>(expected_snapshot));
    assert_eq!(actual.body, expected);
}

async fn post_tide(body: &str, max_distance_km: f64) -> JsonResponse {
    post_json("/tide", body, max_distance_km).await
}

async fn post_json(uri: &str, body: &str, max_distance_km: f64) -> JsonResponse {
    let service = app(must(load_packs_from_paths(&pack_paths())), max_distance_km);
    request_json(
        service,
        must(
            Request::builder()
                .method(Method::POST)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string())),
        ),
    )
    .await
}

struct JsonResponse {
    status: StatusCode,
    body: Value,
}

async fn request_json(service: axum::Router, request: Request<Body>) -> JsonResponse {
    let response = must(service.oneshot(request).await);
    let status = response.status();
    let bytes = must(response.into_body().collect().await).to_bytes();
    let body = must(serde_json::from_slice::<Value>(&bytes));
    JsonResponse { status, body }
}
