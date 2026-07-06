use amar_core::CoreError;
use amar_data::{
    DataError, load_official_predictions, load_pack_from_path, load_pack_from_str, percentile,
    prediction_error_meters,
};
use std::path::{Path, PathBuf};

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    }
}

#[test]
fn noaa_golden_p95_is_measured_without_threshold() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let data = must(load_pack_from_path(root.join("data/packs/noaa_m0.json")));

    for station in data.stations() {
        let station_id = &station.pack().provider_station_id;
        let mut errors = Vec::new();
        for prediction_path in prediction_files(&root, station_id) {
            let predictions = must(load_official_predictions(prediction_path));
            for official in predictions {
                errors.push(prediction_error_meters(station.model(), official));
            }
        }
        errors.sort_by(|left, right| left.total_cmp(right));
        let p95 = must_some(percentile(&errors, 0.95));
        println!(
            "{} {} method={} samples={} p95_m={:.3}",
            station.pack().station_id,
            station.pack().name,
            station.model().method().as_str(),
            errors.len(),
            p95
        );
        assert!(!errors.is_empty());
    }
}

#[test]
fn load_pack_rejects_unknown_constituents() {
    let pack = r#"{
        "schema_version": "amar-pack-v0",
        "generated_at": "2026-07-06",
        "stations": [{
            "station_id": "noaa:8443970",
            "provider_station_id": "8443970",
            "name": "Boston",
            "latitude_deg": 42.3539,
            "longitude_deg": -71.0503,
            "datum": "MLLW",
            "z0_m": 1.0,
            "method": "station_harmonics_v0",
            "constituents": [{
                "name": "ZZ9",
                "amplitude_m": 0.5,
                "phase_gmt_deg": 10.0,
                "speed_deg_per_hour": 28.984104
            }],
            "source": {
                "provider": "NOAA CO-OPS",
                "license": "United States public domain",
                "extracted_at": "2026-07-06",
                "station_url": "https://example.test/station",
                "datums_url": "https://example.test/datums",
                "harcon_url": "https://example.test/harcon",
                "checksum_sha256": "abc"
            }
        }]
    }"#;
    let result = load_pack_from_str(pack);
    assert!(matches!(
        result,
        Err(DataError::Core(CoreError::UnknownConstituent(name))) if name == "ZZ9"
    ));
}

fn prediction_files(root: &Path, station_id: &str) -> [PathBuf; 3] {
    let station_dir = root.join("fixtures/noaa").join(station_id);
    [
        station_dir.join("predictions_2026-08-15_2026-08-21.json"),
        station_dir.join("predictions_2031-08-15_2031-08-21.json"),
        station_dir.join("predictions_2036-08-15_2036-08-21.json"),
    ]
}

fn must_some<T>(option: Option<T>) -> T {
    match option {
        Some(value) => value,
        None => panic!("missing value"),
    }
}
