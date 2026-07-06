use amar_core::{
    Meters, TideExtremumKind, TideThresholdDirection, UtcDateTime, extrema_between, predict_height,
    tide_windows,
};
use amar_data::{LoadedStation, load_packs_from_paths};
use std::path::{Path, PathBuf};

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    }
}

fn must_some<T>(option: Option<T>) -> T {
    match option {
        Some(value) => value,
        None => panic!("missing value"),
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn brest_station() -> LoadedStation {
    let root = workspace_root();
    let data = must(load_packs_from_paths(&[
        root.join("data/packs/noaa_m0.json"),
        root.join("data/packs/amar-data-brest-experimental.json"),
    ]));
    data.stations()
        .iter()
        .find(|station| station.pack().station_id == "refmar:3")
        .cloned()
        .unwrap_or_else(|| panic!("missing Brest station"))
}

#[test]
fn brest_window_boundaries_are_threshold_crossings() {
    let station = brest_station();
    let from = must(UtcDateTime::parse_rfc3339("2026-08-15T00:00:00Z"));
    let to = must(UtcDateTime::parse_rfc3339("2026-08-16T12:00:00Z"));
    let threshold = must(Meters::new(4.5));
    let windows = tide_windows(
        station.model(),
        from,
        to,
        threshold,
        TideThresholdDirection::Above,
    );

    assert!(!windows.is_empty());
    for window in windows {
        let start_height = predict_height(station.model(), window.start())
            .height()
            .as_meters();
        let end_height = predict_height(station.model(), window.end())
            .height()
            .as_meters();
        assert!((start_height - threshold.as_meters()).abs() < 0.005);
        assert!((end_height - threshold.as_meters()).abs() < 0.005);
    }
}

#[test]
fn brest_highs_and_lows_alternate() {
    let station = brest_station();
    let from = must(UtcDateTime::parse_rfc3339("2026-08-15T00:00:00Z"));
    let to = must(UtcDateTime::parse_rfc3339("2026-08-18T00:00:00Z"));
    let extrema = extrema_between(station.model(), from, to);

    assert!(extrema.len() >= 8);
    for pair in extrema.windows(2) {
        assert_ne!(pair[0].kind(), pair[1].kind());
    }
}

#[test]
fn brest_height_is_monotonic_between_adjacent_extrema() {
    let station = brest_station();
    let from = must(UtcDateTime::parse_rfc3339("2026-08-15T00:00:00Z"));
    let to = must(UtcDateTime::parse_rfc3339("2026-08-17T00:00:00Z"));
    let extrema = extrema_between(station.model(), from, to);

    for pair in extrema.windows(2) {
        let rising = match (pair[0].kind(), pair[1].kind()) {
            (TideExtremumKind::Low, TideExtremumKind::High) => true,
            (TideExtremumKind::High, TideExtremumKind::Low) => false,
            _ => panic!("extrema must alternate"),
        };
        let mut at = pair[0].at().add_seconds(6 * 60);
        let mut previous = predict_height(station.model(), pair[0].at())
            .height()
            .as_meters();
        while at < pair[1].at() {
            let height = predict_height(station.model(), at).height().as_meters();
            if rising {
                assert!(
                    height + 0.005 >= previous,
                    "expected rising segment at {:?}",
                    at
                );
            } else {
                assert!(
                    height <= previous + 0.005,
                    "expected falling segment at {:?}",
                    at
                );
            }
            previous = height;
            at = at.add_seconds(6 * 60);
        }
    }
}

#[test]
fn noaa_hilo_fixture_matches_predicted_extremum() {
    let root = workspace_root();
    let data = must(load_packs_from_paths(&[
        root.join("data/packs/noaa_m0.json")
    ]));
    let station = must_some(
        data.stations()
            .iter()
            .find(|station| station.pack().station_id == "noaa:9414290"),
    );
    let official = must(amar_data::load_official_hilo_predictions(
        root.join("fixtures/noaa/9414290/hilo_2026-08-15_2026-08-21.json"),
    ));
    let first = must_some(official.first()).at;
    let last = must_some(official.last()).at;
    let predicted = extrema_between(
        station.model(),
        first.add_seconds(-12 * 60 * 60),
        last.add_seconds(12 * 60 * 60),
    );

    for official_extremum in official {
        let closest = must_some(
            predicted
                .iter()
                .filter(|extremum| extremum.kind() == official_extremum.kind)
                .min_by_key(|extremum| extremum.at().seconds_since(official_extremum.at).abs()),
        );
        assert!(closest.at().seconds_since(official_extremum.at).abs() <= 60);
        assert!((closest.height().as_meters() - official_extremum.height.as_meters()).abs() < 0.01);
    }
}
