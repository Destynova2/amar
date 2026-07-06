use amar_core::{TideExtremum, TideExtremumKind, UtcDateTime, extrema_between, next_extrema_after};
use amar_data::{DataSet, LoadedStation};
use amar_pack::StationPack;

pub const BREST_STATION_ID: &str = "refmar:3";
pub const BREST_TIDAL_UNIT_M: f64 = 3.05;
pub const COEFFICIENT_MIN: u8 = 20;
pub const COEFFICIENT_MAX: u8 = 120;
pub const COEFFICIENT_EXPERIMENTAL_WARNING: &str = "coefficient_experimental";

const ASSOCIATED_HIGH_SEARCH_H: i64 = 18;
const NEXT_HIGH_SEARCH_H: u32 = 72;
const FRANCE_MIN_LAT: f64 = 41.0;
const FRANCE_MAX_LAT: f64 = 52.0;
const FRANCE_MIN_LON: f64 = -6.0;
const FRANCE_MAX_LON: f64 = 10.0;

#[derive(Clone, Copy, Debug)]
pub struct TideCoefficient {
    pub coefficient: u8,
    pub brest_high: TideExtremum,
}

pub fn coefficient_after(data: &DataSet, at: UtcDateTime) -> Option<TideCoefficient> {
    let brest = brest_station(data)?;
    let (next_high, _) = next_extrema_after(brest.model(), at, NEXT_HIGH_SEARCH_H);
    Some(from_brest_high(brest, next_high?))
}

pub fn coefficient_for_station_high(
    data: &DataSet,
    station: &StationPack,
    station_high_at: UtcDateTime,
) -> Option<TideCoefficient> {
    if !is_french_coefficient_station(station) {
        return None;
    }
    let brest = brest_station(data)?;
    let from = station_high_at.add_seconds(-ASSOCIATED_HIGH_SEARCH_H * 60 * 60);
    let to = station_high_at.add_seconds(ASSOCIATED_HIGH_SEARCH_H * 60 * 60);
    extrema_between(brest.model(), from, to)
        .into_iter()
        .filter(|extremum| extremum.kind() == TideExtremumKind::High)
        .min_by_key(|extremum| extremum.at().seconds_since(station_high_at).abs())
        .map(|high| from_brest_high(brest, high))
}

pub fn is_french_coefficient_station(station: &StationPack) -> bool {
    station.station_id.starts_with("refmar:")
        && (FRANCE_MIN_LAT..=FRANCE_MAX_LAT).contains(&station.latitude_deg.get())
        && (FRANCE_MIN_LON..=FRANCE_MAX_LON).contains(&station.longitude_deg.get())
}

fn brest_station(data: &DataSet) -> Option<&LoadedStation> {
    data.stations()
        .iter()
        .find(|station| station.pack().station_id == BREST_STATION_ID)
}

fn from_brest_high(brest: &LoadedStation, brest_high: TideExtremum) -> TideCoefficient {
    let mean_level_m = brest.pack().z0_m.get();
    let raw = 100.0 * (brest_high.height().as_meters() - mean_level_m) / BREST_TIDAL_UNIT_M;
    let coefficient = raw
        .round()
        .clamp(f64::from(COEFFICIENT_MIN), f64::from(COEFFICIENT_MAX)) as u8;
    TideCoefficient {
        coefficient,
        brest_high,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amar_data::load_packs_from_paths;
    use std::path::{Path, PathBuf};

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        }
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn data() -> DataSet {
        let root = workspace_root();
        must(load_packs_from_paths(&[
            root.join("data/packs/noaa_m0.json"),
            root.join("data/packs/amar-data-brest-experimental.json"),
        ]))
    }

    #[test]
    fn coefficient_stays_within_public_bounds_for_one_year() {
        let data = data();
        let mut at = must(UtcDateTime::parse_rfc3339("2026-01-01T00:00:00Z"));
        for _ in 0..365 {
            let coefficient = coefficient_after(&data, at)
                .unwrap_or_else(|| panic!("missing coefficient at {:?}", at.as_chrono()));
            assert!(coefficient.coefficient >= COEFFICIENT_MIN);
            assert!(coefficient.coefficient <= COEFFICIENT_MAX);
            at = at.add_seconds(24 * 60 * 60);
        }
    }

    #[test]
    fn higher_brest_high_water_gives_higher_coefficient() {
        let data = data();
        let brest = brest_station(&data).unwrap_or_else(|| panic!("missing Brest"));
        let from = must(UtcDateTime::parse_rfc3339("2026-08-01T00:00:00Z"));
        let to = must(UtcDateTime::parse_rfc3339("2026-09-15T00:00:00Z"));
        let mut highs = extrema_between(brest.model(), from, to)
            .into_iter()
            .filter(|extremum| extremum.kind() == TideExtremumKind::High)
            .collect::<Vec<_>>();
        highs.sort_by(|left, right| {
            left.height()
                .as_meters()
                .total_cmp(&right.height().as_meters())
        });

        for pair in highs.windows(2) {
            let left = from_brest_high(brest, pair[0]).coefficient;
            let right = from_brest_high(brest, pair[1]).coefficient;
            assert!(left <= right);
        }
    }

    #[test]
    fn spring_neap_cycle_has_fortnightly_peaks() {
        let data = data();
        let first_spring = coefficient_after(
            &data,
            must(UtcDateTime::parse_rfc3339("2026-08-01T00:00:00Z")),
        )
        .unwrap_or_else(|| panic!("missing first spring coefficient"))
        .coefficient;
        let first_neap = coefficient_after(
            &data,
            must(UtcDateTime::parse_rfc3339("2026-08-08T00:00:00Z")),
        )
        .unwrap_or_else(|| panic!("missing first neap coefficient"))
        .coefficient;
        let second_spring = coefficient_after(
            &data,
            must(UtcDateTime::parse_rfc3339("2026-08-15T00:00:00Z")),
        )
        .unwrap_or_else(|| panic!("missing second spring coefficient"))
        .coefficient;
        let second_neap = coefficient_after(
            &data,
            must(UtcDateTime::parse_rfc3339("2026-08-22T00:00:00Z")),
        )
        .unwrap_or_else(|| panic!("missing second neap coefficient"))
        .coefficient;

        assert!(first_spring > first_neap);
        assert!(second_spring > first_neap);
        assert!(second_spring > second_neap);
    }
}
