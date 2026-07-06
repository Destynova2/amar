//! JSON data-pack contract shared by readers and future pack writers.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const SCHEMA_VERSION: &str = "amar-pack-v0";

#[derive(Debug, Error)]
pub enum PackError {
    #[error("unsupported schema version {0}")]
    UnsupportedSchemaVersion(String),
    #[error("pack must contain at least one station")]
    EmptyStations,
    #[error("duplicate station id {0}")]
    DuplicateStation(String),
    #[error("station {station_id} has no constituents")]
    EmptyConstituents { station_id: String },
    #[error("duplicate constituent {constituent} in station {station_id}")]
    DuplicateConstituent {
        station_id: String,
        constituent: String,
    },
    #[error("experimental station {station_id} must define residual_benchmark_cm")]
    MissingExperimentalResidual { station_id: String },
    #[error("experimental station {station_id} must define validation_period")]
    MissingExperimentalValidationPeriod { station_id: String },
    #[error("{field} must be finite")]
    NonFinite { field: &'static str },
    #[error("{field} must be between {min} and {max}, got {value}")]
    OutOfRange {
        field: &'static str,
        value: f64,
        min: f64,
        max: f64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TidePack {
    pub schema_version: String,
    pub generated_at: String,
    pub stations: Vec<StationPack>,
}

impl TidePack {
    pub fn validate(&self) -> Result<(), PackError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(PackError::UnsupportedSchemaVersion(
                self.schema_version.clone(),
            ));
        }
        if self.stations.is_empty() {
            return Err(PackError::EmptyStations);
        }

        let mut station_ids = BTreeSet::new();
        for station in &self.stations {
            if !station_ids.insert(station.station_id.clone()) {
                return Err(PackError::DuplicateStation(station.station_id.clone()));
            }
            station.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationPack {
    pub station_id: String,
    pub provider_station_id: String,
    pub name: String,
    pub latitude_deg: LatitudeDegValue,
    pub longitude_deg: LongitudeDegValue,
    pub datum: String,
    pub z0_m: MetersValue,
    pub method: String,
    pub constituents: Vec<ConstituentPack>,
    pub source: SourceInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_official: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_shom: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_period: Option<PeriodInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_period: Option<PeriodInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disclaimer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datum_note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residual_benchmark_cm: Option<f64>,
}

impl StationPack {
    fn validate(&self) -> Result<(), PackError> {
        self.latitude_deg.ensure_finite("latitude_deg")?;
        self.longitude_deg.ensure_finite("longitude_deg")?;
        self.latitude_deg
            .ensure_range("latitude_deg", -90.0, 90.0)?;
        self.longitude_deg
            .ensure_range("longitude_deg", -180.0, 180.0)?;
        self.z0_m.ensure_finite("z0_m")?;
        if self.constituents.is_empty() {
            return Err(PackError::EmptyConstituents {
                station_id: self.station_id.clone(),
            });
        }
        let mut names = BTreeSet::new();
        for constituent in &self.constituents {
            constituent.amplitude_m.ensure_finite("amplitude_m")?;
            constituent.phase_gmt_deg.ensure_finite("phase_gmt_deg")?;
            constituent
                .speed_deg_per_hour
                .ensure_finite("speed_deg_per_hour")?;
            if !names.insert(constituent.name.clone()) {
                return Err(PackError::DuplicateConstituent {
                    station_id: self.station_id.clone(),
                    constituent: constituent.name.clone(),
                });
            }
        }
        if let Some(value) = self.residual_benchmark_cm {
            ensure_finite("residual_benchmark_cm", value)?;
        }
        if self.experimental == Some(true) {
            if self.residual_benchmark_cm.is_none() {
                return Err(PackError::MissingExperimentalResidual {
                    station_id: self.station_id.clone(),
                });
            }
            if self.validation_period.is_none() {
                return Err(PackError::MissingExperimentalValidationPeriod {
                    station_id: self.station_id.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeriodInfo {
    pub start: String,
    pub end: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrestBenchmark {
    pub schema_version: String,
    pub benchmark_id: String,
    pub generated_at: String,
    pub station_id: String,
    pub provider_station_id: String,
    pub station_name: String,
    pub datum: String,
    pub product: String,
    pub source: String,
    pub validation_period: PeriodInfo,
    pub observations_sha256: String,
    pub checksum_sha256: String,
    pub samples: Vec<BrestBenchmarkSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrestBenchmarkSample {
    pub timestamp: String,
    pub observed_m: Option<f64>,
    pub missing: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstituentPack {
    pub name: String,
    pub amplitude_m: MetersValue,
    pub phase_gmt_deg: DegreesValue,
    pub speed_deg_per_hour: DegreesPerHourValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceInfo {
    pub provider: String,
    pub license: String,
    pub extracted_at: String,
    pub station_url: String,
    pub datums_url: String,
    pub harcon_url: String,
    pub checksum_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observations_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observations_checksum_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tidegauge_checksum_sha256: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct MetersValue(f64);

impl MetersValue {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    fn ensure_finite(self, field: &'static str) -> Result<(), PackError> {
        ensure_finite(field, self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct DegreesValue(f64);

impl DegreesValue {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    fn ensure_finite(self, field: &'static str) -> Result<(), PackError> {
        ensure_finite(field, self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct DegreesPerHourValue(f64);

impl DegreesPerHourValue {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    fn ensure_finite(self, field: &'static str) -> Result<(), PackError> {
        ensure_finite(field, self.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct LatitudeDegValue(f64);

impl LatitudeDegValue {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    fn ensure_finite(self, field: &'static str) -> Result<(), PackError> {
        ensure_finite(field, self.0)
    }

    fn ensure_range(self, field: &'static str, min: f64, max: f64) -> Result<(), PackError> {
        if (min..=max).contains(&self.0) {
            Ok(())
        } else {
            Err(PackError::OutOfRange {
                field,
                value: self.0,
                min,
                max,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct LongitudeDegValue(f64);

impl LongitudeDegValue {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    fn ensure_finite(self, field: &'static str) -> Result<(), PackError> {
        ensure_finite(field, self.0)
    }

    fn ensure_range(self, field: &'static str, min: f64, max: f64) -> Result<(), PackError> {
        if (min..=max).contains(&self.0) {
            Ok(())
        } else {
            Err(PackError::OutOfRange {
                field,
                value: self.0,
                min,
                max,
            })
        }
    }
}

fn ensure_finite(field: &'static str, value: f64) -> Result<(), PackError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(PackError::NonFinite { field })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_pack() -> TidePack {
        TidePack {
            schema_version: SCHEMA_VERSION.to_string(),
            generated_at: "2026-07-06".to_string(),
            stations: vec![valid_station("noaa:8443970")],
        }
    }

    fn valid_station(station_id: &str) -> StationPack {
        StationPack {
            station_id: station_id.to_string(),
            provider_station_id: "8443970".to_string(),
            name: "Boston".to_string(),
            latitude_deg: LatitudeDegValue::new(42.3539),
            longitude_deg: LongitudeDegValue::new(-71.0503),
            datum: "MLLW".to_string(),
            z0_m: MetersValue::new(1.0),
            method: "station_harmonics_v0".to_string(),
            constituents: vec![ConstituentPack {
                name: "M2".to_string(),
                amplitude_m: MetersValue::new(0.5),
                phase_gmt_deg: DegreesValue::new(10.0),
                speed_deg_per_hour: DegreesPerHourValue::new(28.984_104),
            }],
            source: SourceInfo {
                provider: "NOAA CO-OPS".to_string(),
                license: "United States public domain".to_string(),
                extracted_at: "2026-07-06".to_string(),
                station_url: "https://example.test/station".to_string(),
                datums_url: "https://example.test/datums".to_string(),
                harcon_url: "https://example.test/harcon".to_string(),
                checksum_sha256: "abc".to_string(),
                attribution: None,
                product: None,
                observations_url: None,
                observations_checksum_sha256: None,
                tidegauge_checksum_sha256: None,
            },
            experimental: None,
            not_official: None,
            not_shom: None,
            calibration_period: None,
            validation_period: None,
            disclaimer: None,
            datum_note: None,
            residual_benchmark_cm: None,
        }
    }

    #[test]
    fn validate_rejects_unsupported_schema_version() {
        let mut pack = valid_pack();
        pack.schema_version = "amar-pack-v9".to_string();
        assert!(matches!(
            pack.validate(),
            Err(PackError::UnsupportedSchemaVersion(version)) if version == "amar-pack-v9"
        ));
    }

    #[test]
    fn validate_rejects_duplicate_station_ids() {
        let mut pack = valid_pack();
        pack.stations.push(valid_station("noaa:8443970"));
        assert!(matches!(
            pack.validate(),
            Err(PackError::DuplicateStation(station_id)) if station_id == "noaa:8443970"
        ));
    }

    #[test]
    fn validate_rejects_duplicate_constituents() {
        let mut pack = valid_pack();
        let duplicate = pack.stations[0].constituents[0].clone();
        pack.stations[0].constituents.push(duplicate);
        assert!(matches!(
            pack.validate(),
            Err(PackError::DuplicateConstituent { station_id, constituent })
                if station_id == "noaa:8443970" && constituent == "M2"
        ));
    }

    #[test]
    fn validate_rejects_non_finite_values() {
        let mut pack = valid_pack();
        pack.stations[0].constituents[0].amplitude_m = MetersValue::new(f64::NAN);
        assert!(matches!(
            pack.validate(),
            Err(PackError::NonFinite { field }) if field == "amplitude_m"
        ));
    }

    #[test]
    fn validate_rejects_coordinates_out_of_range() {
        let mut pack = valid_pack();
        pack.stations[0].latitude_deg = LatitudeDegValue::new(91.0);
        assert!(matches!(
            pack.validate(),
            Err(PackError::OutOfRange { field, value, min, max })
                if field == "latitude_deg" && value == 91.0 && min == -90.0 && max == 90.0
        ));

        let mut pack = valid_pack();
        pack.stations[0].longitude_deg = LongitudeDegValue::new(-181.0);
        assert!(matches!(
            pack.validate(),
            Err(PackError::OutOfRange { field, value, min, max })
                if field == "longitude_deg" && value == -181.0 && min == -180.0 && max == 180.0
        ));
    }

    #[test]
    fn validate_rejects_experimental_station_without_residual() {
        let mut pack = valid_pack();
        pack.stations[0].experimental = Some(true);
        pack.stations[0].validation_period = Some(PeriodInfo {
            start: "2026-04-01T00:00:00Z".to_string(),
            end: "2026-07-01T00:00:00Z".to_string(),
        });

        assert!(matches!(
            pack.validate(),
            Err(PackError::MissingExperimentalResidual { station_id })
                if station_id == "noaa:8443970"
        ));
    }

    #[test]
    fn validate_rejects_experimental_station_without_validation_period() {
        let mut pack = valid_pack();
        pack.stations[0].experimental = Some(true);
        pack.stations[0].residual_benchmark_cm = Some(26.6);

        assert!(matches!(
            pack.validate(),
            Err(PackError::MissingExperimentalValidationPeriod { station_id })
                if station_id == "noaa:8443970"
        ));
    }
}
