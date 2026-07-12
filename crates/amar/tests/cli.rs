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
fn tide_rejects_unknown_datum() {
    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("tide")
            .arg("--lat")
            .arg("48.383")
            .arg("--lon")
            .arg("-4.495")
            .arg("--at")
            .arg("2026-08-15T12:00:00Z")
            .arg("--datum")
            .arg("wgs84")
            .output(),
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("datum must be one of zero_hydrographique, ign69, recent"));
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
    assert_eq!(body["datum"], "zero_hydrographique_brest_officiel");
    assert_eq!(body["height_m"], 0.958);
    assert_eq!(body["source"]["id"], "refmar:3");
    assert_eq!(
        body["confidence"]["method"],
        "calibrated_station_experimental"
    );
    assert_eq!(body["confidence"]["residual_benchmark_cm"], 15.8);
    assert!(body["confidence"]["grade"].is_null());
    assert_eq!(body["source"]["valid_until"], "2031-04-01T00:00:00Z");
    assert_eq!(body["next_high"]["coefficient"], 101);
    assert!(body["warnings"].to_string().contains("experimental"));
    assert!(body["warnings"].to_string().contains("not_shom"));
    assert!(
        !body["warnings"]
            .to_string()
            .contains("outside_validity_period")
    );
    assert!(
        body["warnings"]
            .to_string()
            .contains("coefficient_experimental")
    );
}

#[test]
fn tide_returns_brest_ign69_when_requested() {
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
            .arg("--datum")
            .arg("ign69")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--pack")
            .arg(root.join("data/packs/amar-data-brest-experimental.json"))
            .output(),
    );

    assert!(output.status.success());
    let body = must(serde_json::from_slice::<Value>(&output.stdout));
    assert_eq!(body["datum"], "IGN69");
    assert_eq!(body["height_m"], -2.554);
    assert_eq!(body["next_high"]["height_m"], 3.712);
}

#[test]
fn tide_series_keeps_instant_height_and_extrema() {
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
            .arg("--datum")
            .arg("recent")
            .arg("--duration-h")
            .arg("2")
            .arg("--step-min")
            .arg("60")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--pack")
            .arg(root.join("data/packs/amar-data-brest-experimental.json"))
            .output(),
    );

    assert!(output.status.success());
    let body = must(serde_json::from_slice::<Value>(&output.stdout));
    assert_eq!(body["datum"], "zero_hydrographique_brest_recent");
    assert_eq!(body["height_m"], 1.081);
    assert_eq!(body["next_high"]["height_m"], 7.347);
    assert_eq!(body["series"].as_array().map(Vec::len), Some(3));
    assert_eq!(body["series"][0]["height_m"], body["height_m"]);
}

#[test]
fn tide_warns_after_station_validity_period() {
    let root = workspace_root();
    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("tide")
            .arg("--lat")
            .arg("48.383")
            .arg("--lon")
            .arg("-4.495")
            .arg("--at")
            .arg("2032-04-02T00:00:00Z")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--pack")
            .arg(root.join("data/packs/amar-data-brest-experimental.json"))
            .output(),
    );

    assert!(output.status.success());
    let body = must(serde_json::from_slice::<Value>(&output.stdout));
    assert_eq!(body["source"]["id"], "refmar:3");
    assert_eq!(body["source"]["valid_until"], "2031-04-01T00:00:00Z");
    assert!(
        body["warnings"]
            .to_string()
            .contains("outside_validity_period")
    );
}

#[test]
fn tide_strict_validity_rejects_after_station_validity_period() {
    let root = workspace_root();
    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .arg("tide")
            .arg("--lat")
            .arg("48.383")
            .arg("--lon")
            .arg("-4.495")
            .arg("--at")
            .arg("2032-04-02T00:00:00Z")
            .arg("--strict-validity")
            .arg("--pack")
            .arg(root.join("data/packs/noaa_m0.json"))
            .arg("--pack")
            .arg(root.join("data/packs/amar-data-brest-experimental.json"))
            .output(),
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("outside_validity_period"));
    assert!(stderr.contains("valid_until=2031-04-01T00:00:00Z"));
}

#[test]
fn tide_loads_default_packs_from_extracted_archive() {
    let root = workspace_root();
    let archive = unique_temp_dir("archive-default-packs");
    let packs = archive.join("packs");
    must(fs::create_dir_all(&packs));
    for file in [
        "noaa_m0.json",
        "amar-data-brest-experimental.json",
        "amar-data-france-experimental.json",
    ] {
        must(fs::copy(
            root.join("data/packs").join(file),
            packs.join(file),
        ));
    }

    let output = must(
        Command::new(env!("CARGO_BIN_EXE_amar"))
            .current_dir(&archive)
            .arg("tide")
            .arg("--lat")
            .arg("37.806")
            .arg("--lon")
            .arg("-122.465")
            .arg("--at")
            .arg("2026-08-15T12:00:00Z")
            .output(),
    );
    let _ = fs::remove_dir_all(&archive);

    assert!(output.status.success());
    let body = must(serde_json::from_slice::<Value>(&output.stdout));
    assert_eq!(body["source"]["id"], "noaa:9414290");
    assert!(body["height_m"].is_number());
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
