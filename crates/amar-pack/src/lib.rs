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
    #[error("{field} must be finite")]
    NonFinite { field: &'static str },
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
}

impl StationPack {
    fn validate(&self) -> Result<(), PackError> {
        self.latitude_deg.ensure_finite("latitude_deg")?;
        self.longitude_deg.ensure_finite("longitude_deg")?;
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
        Ok(())
    }
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
        if self.0.is_finite() {
            Ok(())
        } else {
            Err(PackError::NonFinite { field })
        }
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
        if self.0.is_finite() {
            Ok(())
        } else {
            Err(PackError::NonFinite { field })
        }
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
        if self.0.is_finite() {
            Ok(())
        } else {
            Err(PackError::NonFinite { field })
        }
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
        if self.0.is_finite() {
            Ok(())
        } else {
            Err(PackError::NonFinite { field })
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
        if self.0.is_finite() {
            Ok(())
        } else {
            Err(PackError::NonFinite { field })
        }
    }
}
