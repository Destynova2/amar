use amar_data::load_pack_from_path;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("amar-{name}-{}-{nanos}", std::process::id()))
}

#[test]
fn validate_rejects_stations_without_prediction_samples() {
    let root = workspace_root();
    let fixtures = unique_temp_dir("empty-predictions");
    let data = must(load_pack_from_path(root.join("data/packs/noaa_m0.json")));
    let mut expected_missing_sample = None;
    for station in data.stations() {
        let station = station.pack();
        if expected_missing_sample.is_none() {
            expected_missing_sample = Some(format!("{} samples=0", station.station_id));
        }
        must(fs::create_dir_all(
            fixtures.join(&station.provider_station_id),
        ));
    }

    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("validate")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--fixtures")
            .arg(&fixtures)
            .output(),
    );
    let _ = fs::remove_dir_all(&fixtures);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("validation missing samples"));
    let Some(expected_missing_sample) = expected_missing_sample else {
        panic!("expected at least one station in pack");
    };
    assert!(stderr.contains(&expected_missing_sample));
}

#[test]
fn tide_rejects_latitude_out_of_range() {
    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("tide")
            .arg("--lat")
            .arg("91")
            .arg("--lon")
            .arg("0")
            .arg("--at")
            .arg("2026-08-15T00:00:00Z")
            .output(),
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("latitude must be between -90 and 90 degrees"));
}

#[test]
fn tide_returns_brest_experimental_confidence() {
    let root = workspace_root();
    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("tide")
            .arg("--lat")
            .arg("48.383")
            .arg("--lon")
            .arg("-4.495")
            .arg("--at")
            .arg("2026-08-15T12:00:00Z")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--pack")
            .arg(root.join("data/packs/amar-data-brest-experimental.json"))
            .output(),
    );

    assert!(output.status.success());
    let body = must(serde_json::from_slice::<Value>(&output.stdout));
    assert_eq!(body["datum"], "zero_hydrographique_brest");
    assert_eq!(body["source"]["id"], "refmar:3");
    assert_eq!(
        body["confidence"]["method"],
        "calibrated_station_experimental"
    );
    assert_eq!(body["confidence"]["residual_benchmark_cm"], 15.8);
    assert!(body["confidence"]["grade"].is_null());
    assert_eq!(body["next_high"]["coefficient"], 101);
    assert!(body["warnings"].to_string().contains("experimental"));
    assert!(body["warnings"].to_string().contains("not_shom"));
    assert!(
        body["warnings"]
            .to_string()
            .contains("coefficient_experimental")
    );
}

#[test]
fn coef_returns_brest_derived_coefficient() {
    let root = workspace_root();
    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("coef")
            .arg("--at")
            .arg("2026-08-15T12:00:00Z")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--pack")
            .arg(root.join("data/packs/amar-data-brest-experimental.json"))
            .output(),
    );

    assert!(output.status.success());
    let body = must(serde_json::from_slice::<Value>(&output.stdout));
    assert_eq!(body["coefficient"], 101);
    assert_eq!(body["unit_m"], 3.05);
    assert_eq!(body["brest_high"]["coefficient"], 101);
    assert_eq!(body["brest_high"]["t"], "2026-08-15T17:47:37Z");
}

#[test]
fn window_returns_brest_kayak_windows() {
    let root = workspace_root();
    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("window")
            .arg("--lat")
            .arg("48.383")
            .arg("--lon")
            .arg("-4.495")
            .arg("--from")
            .arg("2026-08-15T00:00:00Z")
            .arg("--to")
            .arg("2026-08-16T12:00:00Z")
            .arg("--above")
            .arg("4.5")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--pack")
            .arg(root.join("data/packs/amar-data-brest-experimental.json"))
            .output(),
    );

    assert!(output.status.success());
    let body = must(serde_json::from_slice::<Value>(&output.stdout));
    assert_eq!(body["datum"], "zero_hydrographique_brest");
    assert_eq!(body["source"]["id"], "refmar:3");
    assert!(body["warnings"].to_string().contains("experimental"));
    assert!(
        body["windows"]
            .as_array()
            .is_some_and(|windows| !windows.is_empty())
    );
}

#[test]
fn window_above_negative_threshold_returns_full_range() {
    let root = workspace_root();
    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("window")
            .arg("--lat")
            .arg("48.383")
            .arg("--lon")
            .arg("-4.495")
            .arg("--from")
            .arg("2026-08-15T00:00:00Z")
            .arg("--to")
            .arg("2026-08-16T12:00:00Z")
            .arg("--above")
            .arg("-5")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--pack")
            .arg(root.join("data/packs/amar-data-brest-experimental.json"))
            .output(),
    );

    assert!(output.status.success());
    let body = must(serde_json::from_slice::<Value>(&output.stdout));
    let Some(windows) = body["windows"].as_array() else {
        panic!("windows array");
    };
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0]["start"], "2026-08-15T00:00:00Z");
    assert_eq!(windows[0]["end"], "2026-08-16T12:00:00Z");
}

#[test]
fn window_below_negative_threshold_returns_no_windows() {
    let root = workspace_root();
    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("window")
            .arg("--lat")
            .arg("48.383")
            .arg("--lon")
            .arg("-4.495")
            .arg("--from")
            .arg("2026-08-15T00:00:00Z")
            .arg("--to")
            .arg("2026-08-16T12:00:00Z")
            .arg("--below")
            .arg("-5")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--pack")
            .arg(root.join("data/packs/amar-data-brest-experimental.json"))
            .output(),
    );

    assert!(output.status.success());
    let body = must(serde_json::from_slice::<Value>(&output.stdout));
    assert_eq!(body["windows"].as_array().map(Vec::len), Some(0));
}
