use amar_data::{load_official_predictions, load_pack_from_path, prediction_error_meters};
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
        let p95 = percentile(&errors, 0.95);
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

fn prediction_files(root: &Path, station_id: &str) -> [PathBuf; 3] {
    let station_dir = root.join("fixtures/noaa").join(station_id);
    [
        station_dir.join("predictions_2026-08-15_2026-08-21.json"),
        station_dir.join("predictions_2031-08-15_2031-08-21.json"),
        station_dir.join("predictions_2036-08-15_2036-08-21.json"),
    ]
}

fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let index = ((sorted_values.len() - 1) as f64 * percentile).ceil() as usize;
    sorted_values[index]
}
