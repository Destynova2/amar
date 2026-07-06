use crate::constituents::constituent_definition;
use chrono::{DateTime, Datelike, TimeDelta, TimeZone, Timelike, Utc};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("{type_name} must be finite, got {value}")]
    NonFinite { type_name: &'static str, value: f64 },
    #[error("{type_name} must not be empty")]
    EmptyText { type_name: &'static str },
    #[error("duplicate constituent {0}")]
    DuplicateConstituent(String),
    #[error("unknown constituent {0}")]
    UnknownConstituent(String),
    #[error("model must contain at least one constituent")]
    EmptyConstituents,
    #[error("invalid UTC timestamp: {0}")]
    InvalidTimestamp(String),
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Degrees(pub(crate) f64);

impl Degrees {
    pub fn new(value: f64) -> Result<Self, CoreError> {
        ensure_finite("Degrees", value)?;
        Ok(Self(value))
    }

    pub fn as_degrees(self) -> f64 {
        self.0
    }

    pub fn to_radians(self) -> Radians {
        Radians(self.0.to_radians())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Radians(pub(crate) f64);

impl Radians {
    pub fn new(value: f64) -> Result<Self, CoreError> {
        ensure_finite("Radians", value)?;
        Ok(Self(value))
    }

    pub fn as_radians(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct DegreesPerHour(f64);

impl DegreesPerHour {
    pub fn new(value: f64) -> Result<Self, CoreError> {
        ensure_finite("DegreesPerHour", value)?;
        Ok(Self(value))
    }

    pub fn as_degrees_per_hour(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Meters(pub(crate) f64);

impl Meters {
    pub fn new(value: f64) -> Result<Self, CoreError> {
        ensure_finite("Meters", value)?;
        Ok(Self(value))
    }

    pub fn as_meters(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ConstituentId(Box<str>);

impl ConstituentId {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(CoreError::EmptyText {
                type_name: "ConstituentId",
            });
        }
        Ok(Self(trimmed.to_ascii_uppercase().into_boxed_str()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ConstituentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DatumId(Box<str>);

impl DatumId {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(CoreError::EmptyText {
                type_name: "DatumId",
            });
        }
        Ok(Self(trimmed.to_ascii_uppercase().into_boxed_str()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DatumId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UtcDateTime(DateTime<Utc>);

impl UtcDateTime {
    pub fn parse_rfc3339(value: &str) -> Result<Self, CoreError> {
        let parsed = DateTime::parse_from_rfc3339(value)
            .map_err(|error| CoreError::InvalidTimestamp(error.to_string()))?;
        Ok(Self(parsed.with_timezone(&Utc)))
    }

    pub fn from_utc(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn as_chrono(self) -> DateTime<Utc> {
        self.0
    }

    pub fn add_seconds(self, seconds: i64) -> Self {
        Self(self.0 + TimeDelta::seconds(seconds))
    }

    pub fn seconds_since(self, earlier: Self) -> i64 {
        (self.0 - earlier.0).num_seconds()
    }

    pub(crate) fn civil_year_start(self) -> Self {
        utc_datetime(self.0.year(), 1, 1, 0, 0, 0)
    }

    pub(crate) fn civil_year_midpoint(self) -> Self {
        let start = self.civil_year_start();
        let next_start = utc_datetime(self.0.year() + 1, 1, 1, 0, 0, 0);
        let year_seconds = (next_start.0 - start.0).num_seconds();
        start.add_seconds(year_seconds / 2)
    }

    pub(crate) fn hours_since(self, earlier: Self) -> f64 {
        (self.ordinal_days() - earlier.ordinal_days()) * 24.0
    }

    pub(crate) fn ordinal_days(self) -> f64 {
        let date = self.0.date_naive();
        let time = self.0.time();
        let year = i64::from(date.year());
        let days_before_year =
            365 * (year - 1) + (year - 1) / 4 - (year - 1) / 100 + (year - 1) / 400;
        let day_number = days_before_year + i64::from(date.ordinal());
        let seconds = f64::from(time.num_seconds_from_midnight());
        let nanos = f64::from(time.nanosecond());
        day_number as f64 + seconds / 86_400.0 + nanos / 86_400_000_000_000.0
    }
}

fn utc_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> UtcDateTime {
    match Utc
        .with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
    {
        Some(value) => UtcDateTime(value),
        None => unreachable!("valid UTC civil date"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredictionMethod {
    StationHarmonicsV0,
    HarmonicBasicNoNodal,
}

impl PredictionMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StationHarmonicsV0 => "station_harmonics_v0",
            Self::HarmonicBasicNoNodal => "harmonic_basic_no_nodal",
        }
    }
}

impl fmt::Display for PredictionMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct HarmonicConstituent {
    id: ConstituentId,
    amplitude: Meters,
    phase_gmt: Degrees,
    speed: DegreesPerHour,
}

impl HarmonicConstituent {
    pub fn new(
        id: ConstituentId,
        amplitude: Meters,
        phase_gmt: Degrees,
        speed: DegreesPerHour,
    ) -> Self {
        Self {
            id,
            amplitude,
            phase_gmt,
            speed,
        }
    }

    pub fn id(&self) -> &ConstituentId {
        &self.id
    }

    pub fn amplitude(&self) -> Meters {
        self.amplitude
    }

    pub fn phase_gmt(&self) -> Degrees {
        self.phase_gmt
    }

    pub fn speed(&self) -> DegreesPerHour {
        self.speed
    }
}

#[derive(Clone, Debug)]
pub struct TideModel {
    datum: DatumId,
    pub(crate) z0: Meters,
    constituents: Vec<HarmonicConstituent>,
    pub(crate) method: PredictionMethod,
}

impl TideModel {
    pub fn new(
        datum: DatumId,
        z0: Meters,
        mut constituents: Vec<HarmonicConstituent>,
        method: PredictionMethod,
    ) -> Result<Self, CoreError> {
        if constituents.is_empty() {
            return Err(CoreError::EmptyConstituents);
        }
        constituents.sort_by(|left, right| left.id.cmp(&right.id));
        for pair in constituents.windows(2) {
            if pair[0].id == pair[1].id {
                return Err(CoreError::DuplicateConstituent(pair[0].id.to_string()));
            }
        }
        for constituent in &constituents {
            if constituent_definition(constituent.id.as_str()).is_none() {
                return Err(CoreError::UnknownConstituent(constituent.id.to_string()));
            }
        }
        Ok(Self {
            datum,
            z0,
            constituents,
            method,
        })
    }

    pub fn datum(&self) -> &DatumId {
        &self.datum
    }

    pub fn method(&self) -> PredictionMethod {
        self.method
    }

    pub fn constituents(&self) -> &[HarmonicConstituent] {
        &self.constituents
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TidePrediction {
    pub(crate) height: Meters,
    pub(crate) method: PredictionMethod,
}

impl TidePrediction {
    pub fn height(&self) -> Meters {
        self.height
    }

    pub fn method(&self) -> PredictionMethod {
        self.method
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TideExtremumKind {
    High,
    Low,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TideExtremum {
    pub(crate) at: UtcDateTime,
    pub(crate) height: Meters,
    pub(crate) kind: TideExtremumKind,
}

impl TideExtremum {
    pub fn at(self) -> UtcDateTime {
        self.at
    }

    pub fn height(self) -> Meters {
        self.height
    }

    pub fn kind(self) -> TideExtremumKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TidePoint {
    pub(crate) at: UtcDateTime,
    pub(crate) height: Meters,
}

impl TidePoint {
    pub fn at(self) -> UtcDateTime {
        self.at
    }

    pub fn height(self) -> Meters {
        self.height
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TideThresholdDirection {
    Above,
    Below,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TideWindow {
    pub(crate) start: UtcDateTime,
    pub(crate) end: UtcDateTime,
}

impl TideWindow {
    pub fn start(self) -> UtcDateTime {
        self.start
    }

    pub fn end(self) -> UtcDateTime {
        self.end
    }
}

fn ensure_finite(type_name: &'static str, value: f64) -> Result<(), CoreError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(CoreError::NonFinite { type_name, value })
    }
}
