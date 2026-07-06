use amar::server::app;
use amar_data::load_pack_from_path;
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

fn pack_path() -> PathBuf {
    workspace_root().join("data/packs/noaa_m0.json")
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
async fn tide_brest_refusal_matches_snapshot() {
    let body = r#"{"lat":48.383,"lon":-4.495,"datetime":"2026-08-15T12:00:00Z"}"#;
    assert_post_snapshot(
        body,
        StatusCode::UNPROCESSABLE_ENTITY,
        include_str!("snapshots/tide_brest_422.json"),
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
async fn health_and_coverage_expose_loaded_pack() {
    let service = app(must(load_pack_from_path(pack_path())), 20.0);

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
    assert_eq!(health.body["station_count"], 8);
    assert_eq!(health.body["data_version"], "2026-07-06");

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
    assert_eq!(coverage.body["stations"].as_array().map(Vec::len), Some(8));
    assert!(coverage.body.to_string().contains("noaa:9414290"));
}

async fn assert_post_snapshot(body: &str, expected_status: StatusCode, expected_snapshot: &str) {
    let service = app(must(load_pack_from_path(pack_path())), 20.0);
    let actual = request_json(
        service,
        must(
            Request::builder()
                .method(Method::POST)
                .uri("/tide")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string())),
        ),
    )
    .await;
    assert_eq!(actual.status, expected_status);
    let expected = must(serde_json::from_str::<Value>(expected_snapshot));
    assert_eq!(actual.body, expected);
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
