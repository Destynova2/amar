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
    for station_id in ["8443970", "9414290", "8729840", "9447130"] {
        must(fs::create_dir_all(fixtures.join(station_id)));
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
    assert!(stderr.contains("noaa:8443970 samples=0"));
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
