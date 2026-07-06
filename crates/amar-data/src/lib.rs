//! Data-pack loading, validation, NOAA fixture parsing, and station lookup.

use amar_core::{
    ConstituentId, DatumId, Degrees, DegreesPerHour, HarmonicConstituent, Meters, PredictionMethod,
    TideModel, UtcDateTime, predict_height,
};
use amar_pack::{
    ConstituentPack, DegreesPerHourValue, DegreesValue, LatitudeDegValue, LongitudeDegValue,
    MetersValue, SCHEMA_VERSION, SourceInfo, StationPack, TidePack,
};
use chrono::{NaiveDateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("pack error: {0}")]
    Pack(#[from] amar_pack::PackError),
    #[error("core error: {0}")]
    Core(#[from] amar_core::CoreError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("station {0} is missing datum {1}")]
    MissingDatum(String, &'static str),
    #[error("station fixture {0} is empty")]
    EmptyStationFixture(String),
    #[error("unsupported method {0}")]
    UnsupportedMethod(String),
    #[error("no supported source within {max_distance_km:.1} km")]
    NoSupportedSource { max_distance_km: f64 },
    #[error("invalid NOAA prediction timestamp {0}")]
    InvalidPredictionTime(String),
    #[error("invalid NOAA prediction value {0}")]
    InvalidPredictionValue(String),
}

#[derive(Clone, Debug)]
pub struct DataSet {
    pack: TidePack,
    stations: Vec<LoadedStation>,
}

impl DataSet {
    pub fn from_pack(pack: TidePack) -> Result<Self, DataError> {
        pack.validate()?;
        let mut stations = Vec::with_capacity(pack.stations.len());
        for station in &pack.stations {
            stations.push(LoadedStation {
                model: station_to_model(station)?,
                pack: station.clone(),
            });
        }
        Ok(Self { pack, stations })
    }

    pub fn pack(&self) -> &TidePack {
        &self.pack
    }

    pub fn stations(&self) -> &[LoadedStation] {
        &self.stations
    }

    pub fn nearest_station(
        &self,
        latitude_deg: f64,
        longitude_deg: f64,
        max_distance_km: f64,
    ) -> Result<StationMatch<'_>, DataError> {
        let best = self.closest_station(latitude_deg, longitude_deg);

        match best {
            Some(station_match) if station_match.distance_km <= max_distance_km => {
                Ok(station_match)
            }
            _ => Err(DataError::NoSupportedSource { max_distance_km }),
        }
    }

    pub fn closest_station(
        &self,
        latitude_deg: f64,
        longitude_deg: f64,
    ) -> Option<StationMatch<'_>> {
        self.stations
            .iter()
            .map(|station| {
                let distance_km = haversine_km(
                    latitude_deg,
                    longitude_deg,
                    station.pack.latitude_deg.get(),
                    station.pack.longitude_deg.get(),
                );
                StationMatch {
                    station,
                    distance_km,
                }
            })
            .min_by(|left, right| left.distance_km.total_cmp(&right.distance_km))
    }
}

#[derive(Clone, Debug)]
pub struct LoadedStation {
    pack: StationPack,
    model: TideModel,
}

impl LoadedStation {
    pub fn pack(&self) -> &StationPack {
        &self.pack
    }

    pub fn model(&self) -> &TideModel {
        &self.model
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StationMatch<'a> {
    pub station: &'a LoadedStation,
    pub distance_km: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct OfficialPrediction {
    pub at: UtcDateTime,
    pub height: Meters,
}

pub fn load_pack_from_path(path: impl AsRef<Path>) -> Result<DataSet, DataError> {
    let path = path.as_ref();
    let data = fs::read_to_string(path).map_err(|source| DataError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    load_pack_from_str(&data)
}

pub fn load_pack_from_str(data: &str) -> Result<DataSet, DataError> {
    let pack = serde_json::from_str::<TidePack>(data)?;
    DataSet::from_pack(pack)
}

pub fn build_noaa_pack(
    fixtures_dir: impl AsRef<Path>,
    extracted_at: &str,
    station_ids: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<TidePack, DataError> {
    let fixtures_dir = fixtures_dir.as_ref();
    let mut stations = Vec::new();
    for station_id in station_ids {
        let station_id = station_id.as_ref();
        stations.push(build_noaa_station(fixtures_dir, station_id, extracted_at)?);
    }
    stations.sort_by(|left, right| left.station_id.cmp(&right.station_id));
    let pack = TidePack {
        schema_version: SCHEMA_VERSION.to_string(),
        generated_at: extracted_at.to_string(),
        stations,
    };
    pack.validate()?;
    Ok(pack)
}

pub fn load_official_predictions(
    path: impl AsRef<Path>,
) -> Result<Vec<OfficialPrediction>, DataError> {
    let path = path.as_ref();
    let data = fs::read_to_string(path).map_err(|source| DataError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let raw = serde_json::from_str::<NoaaPredictions>(&data)?;
    raw.predictions
        .into_iter()
        .map(|prediction| {
            let naive = NaiveDateTime::parse_from_str(&prediction.t, "%Y-%m-%d %H:%M")
                .map_err(|_| DataError::InvalidPredictionTime(prediction.t.clone()))?;
            let value = prediction
                .v
                .parse::<f64>()
                .map_err(|_| DataError::InvalidPredictionValue(prediction.v.clone()))?;
            Ok(OfficialPrediction {
                at: UtcDateTime::from_utc(naive.and_utc().with_timezone(&Utc)),
                height: Meters::new(value)?,
            })
        })
        .collect()
}

pub fn prediction_error_meters(model: &TideModel, official: OfficialPrediction) -> f64 {
    let predicted = predict_height(model, official.at).height().as_meters();
    (predicted - official.height.as_meters()).abs()
}

pub fn percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() || !(0.0..=1.0).contains(&percentile) {
        return None;
    }
    let index = ((sorted_values.len() - 1) as f64 * percentile).ceil() as usize;
    sorted_values.get(index).copied()
}

fn build_noaa_station(
    fixtures_dir: &Path,
    station_id: &str,
    extracted_at: &str,
) -> Result<StationPack, DataError> {
    let station_dir = fixtures_dir.join(station_id);
    let station = read_json::<NoaaStationResponse>(&station_dir.join("station.json"))?;
    let station = station
        .stations
        .into_iter()
        .next()
        .ok_or_else(|| DataError::EmptyStationFixture(station_id.to_string()))?;
    let datums = read_json::<NoaaDatums>(&station_dir.join("datums.json"))?;
    let harcon_path = station_dir.join("harcon.json");
    let harcon_bytes = fs::read(&harcon_path).map_err(|source| DataError::Io {
        path: harcon_path.clone(),
        source,
    })?;
    let harcon = serde_json::from_slice::<NoaaHarcon>(&harcon_bytes)?;
    let datum_values = datums
        .datums
        .into_iter()
        .map(|datum| (datum.name, datum.value))
        .collect::<BTreeMap<_, _>>();
    let mtl = datum_values
        .get("MTL")
        .copied()
        .ok_or_else(|| DataError::MissingDatum(station_id.to_string(), "MTL"))?;
    let mllw = datum_values
        .get("MLLW")
        .copied()
        .ok_or_else(|| DataError::MissingDatum(station_id.to_string(), "MLLW"))?;

    let mut constituents = harcon
        .harmonic_constituents
        .into_iter()
        .map(|constituent| ConstituentPack {
            name: constituent.name,
            amplitude_m: MetersValue::new(constituent.amplitude),
            phase_gmt_deg: DegreesValue::new(constituent.phase_gmt),
            speed_deg_per_hour: DegreesPerHourValue::new(constituent.speed),
        })
        .collect::<Vec<_>>();
    constituents.sort_by(|left, right| left.name.cmp(&right.name));

    let station_url = format!(
        "https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/{station_id}.json?expand=details"
    );
    let datums_url = format!(
        "https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/{station_id}/datums.json?units=metric"
    );
    let harcon_url = format!(
        "https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/{station_id}/harcon.json?units=metric"
    );

    Ok(StationPack {
        station_id: format!("noaa:{station_id}"),
        provider_station_id: station_id.to_string(),
        name: station.name,
        latitude_deg: LatitudeDegValue::new(station.lat),
        longitude_deg: LongitudeDegValue::new(station.lng),
        datum: "MLLW".to_string(),
        z0_m: MetersValue::new(mtl - mllw),
        method: PredictionMethod::StationHarmonicsV0.as_str().to_string(),
        constituents,
        source: SourceInfo {
            provider: "NOAA CO-OPS".to_string(),
            license: "United States public domain".to_string(),
            extracted_at: extracted_at.to_string(),
            station_url,
            datums_url,
            harcon_url,
            checksum_sha256: sha256_hex(&harcon_bytes),
        },
    })
}

fn station_to_model(station: &StationPack) -> Result<TideModel, DataError> {
    let method = match station.method.as_str() {
        "station_harmonics_v0" => PredictionMethod::StationHarmonicsV0,
        "harmonic_basic_no_nodal" => PredictionMethod::HarmonicBasicNoNodal,
        other => return Err(DataError::UnsupportedMethod(other.to_string())),
    };
    let constituents = station
        .constituents
        .iter()
        .map(|constituent| {
            Ok(HarmonicConstituent::new(
                ConstituentId::new(&constituent.name)?,
                Meters::new(constituent.amplitude_m.get())?,
                Degrees::new(constituent.phase_gmt_deg.get())?,
                DegreesPerHour::new(constituent.speed_deg_per_hour.get())?,
            ))
        })
        .collect::<Result<Vec<_>, DataError>>()?;
    TideModel::new(
        DatumId::new(&station.datum)?,
        Meters::new(station.z0_m.get())?,
        constituents,
        method,
    )
    .map_err(DataError::from)
}

fn haversine_km(left_lat: f64, left_lon: f64, right_lat: f64, right_lon: f64) -> f64 {
    let earth_radius_km = 6_371.0;
    let d_lat = (right_lat - left_lat).to_radians();
    let d_lon = (right_lon - left_lon).to_radians();
    let left_lat = left_lat.to_radians();
    let right_lat = right_lat.to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + left_lat.cos() * right_lat.cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * earth_radius_km * a.sqrt().asin()
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, DataError> {
    let data = fs::read_to_string(path).map_err(|source| DataError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&data).map_err(DataError::from)
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[derive(Debug, Deserialize)]
struct NoaaStationResponse {
    stations: Vec<NoaaStation>,
}

#[derive(Debug, Deserialize)]
struct NoaaStation {
    name: String,
    lat: f64,
    lng: f64,
}

#[derive(Debug, Deserialize)]
struct NoaaDatums {
    datums: Vec<NoaaDatum>,
}

#[derive(Debug, Deserialize)]
struct NoaaDatum {
    name: String,
    value: f64,
}

#[derive(Debug, Deserialize)]
struct NoaaHarcon {
    #[serde(rename = "HarmonicConstituents")]
    harmonic_constituents: Vec<NoaaConstituent>,
}

#[derive(Debug, Deserialize)]
struct NoaaConstituent {
    name: String,
    amplitude: f64,
    #[serde(rename = "phase_GMT")]
    phase_gmt: f64,
    speed: f64,
}

#[derive(Debug, Deserialize)]
struct NoaaPredictions {
    predictions: Vec<NoaaPrediction>,
}

#[derive(Debug, Deserialize)]
struct NoaaPrediction {
    t: String,
    v: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_keeps_identical_points_at_zero() {
        assert!(haversine_km(37.806, -122.465, 37.806, -122.465) < 0.001);
    }
}
