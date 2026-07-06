//! Pure harmonic tide engine for amar.
//!
//! This crate has no I/O, no system clock access, and no local timezone logic.

use chrono::{DateTime, Datelike, TimeDelta, Timelike, Utc};
use std::cmp::Ordering;
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
    #[error("model must contain at least one constituent")]
    EmptyConstituents,
    #[error("invalid UTC timestamp: {0}")]
    InvalidTimestamp(String),
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Degrees(f64);

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
pub struct Radians(f64);

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
pub struct Meters(f64);

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    fn ordinal_days(self) -> f64 {
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
    z0: Meters,
    constituents: Vec<HarmonicConstituent>,
    method: PredictionMethod,
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
    height: Meters,
    method: PredictionMethod,
}

impl TidePrediction {
    pub fn height(&self) -> Meters {
        self.height
    }

    pub fn method(&self) -> PredictionMethod {
        self.method
    }
}

pub fn predict_height(model: &TideModel, at: UtcDateTime) -> TidePrediction {
    let mut height = model.z0.as_meters();
    for constituent in &model.constituents {
        let correction = nodal_correction(constituent.id.as_str(), at, model.method);
        let argument = astronomical_argument_degrees(constituent, at);
        let phase = argument + correction.phase_degrees - constituent.phase_gmt.as_degrees();
        let contribution = correction.factor
            * constituent.amplitude.as_meters()
            * Degrees(phase).to_radians().as_radians().cos();
        height += contribution;
    }

    TidePrediction {
        height: Meters(height),
        method: model.method,
    }
}

#[derive(Clone, Copy, Debug)]
struct NodalCorrection {
    factor: f64,
    phase_degrees: f64,
}

fn nodal_correction(
    _constituent: &str,
    _at: UtcDateTime,
    method: PredictionMethod,
) -> NodalCorrection {
    match method {
        PredictionMethod::StationHarmonicsV0 => NodalCorrection {
            factor: 1.0,
            phase_degrees: 0.0,
        },
        PredictionMethod::HarmonicBasicNoNodal => NodalCorrection {
            factor: 1.0,
            phase_degrees: 0.0,
        },
    }
}

fn astronomical_argument_degrees(constituent: &HarmonicConstituent, at: UtcDateTime) -> f64 {
    if let Some(definition) = constituent_definition(constituent.id.as_str()) {
        let astro = astronomical_cycles(at);
        let mut cycles = definition.semi_cycles;
        for (coefficient, value) in definition.coefficients.iter().zip(astro) {
            cycles += f64::from(*coefficient) * value;
        }
        return cycles.rem_euclid(1.0) * 360.0;
    }

    let hours_since_j2000 = (at.ordinal_days() - 730_120.0) * 24.0;
    constituent.speed.as_degrees_per_hour() * hours_since_j2000
}

#[derive(Clone, Copy)]
struct ConstituentDefinition {
    coefficients: [i8; 6],
    semi_cycles: f64,
}

fn constituent_definition(name: &str) -> Option<ConstituentDefinition> {
    let coefficients = match name {
        "M2" => [2, 0, 0, 0, 0, 0],
        "S2" => [2, 2, -2, 0, 0, 0],
        "N2" => [2, -1, 0, 1, 0, 0],
        "K1" => [1, 1, 0, 0, 0, 0],
        "M4" => [4, 0, 0, 0, 0, 0],
        "O1" => [1, -1, 0, 0, 0, 0],
        "M6" => [6, 0, 0, 0, 0, 0],
        "MK3" => [3, 1, 0, 0, 0, 0],
        "S4" => [4, 4, -4, 0, 0, 0],
        "MN4" => [4, -1, 0, 1, 0, 0],
        "NU2" => [2, -1, 2, -1, 0, 0],
        "S6" => [6, 6, -6, 0, 0, 0],
        "MU2" => [2, -2, 2, 0, 0, 0],
        "2N2" => [2, -2, 0, 2, 0, 0],
        "OO1" => [1, 3, 0, 0, 0, 0],
        "LAM2" => [2, 1, -2, 1, 0, 0],
        "S1" => [1, 1, -1, 0, 0, 0],
        "M1" => [1, 0, 0, 1, 0, 0],
        "J1" => [1, 2, 0, -1, 0, 0],
        "MM" => [0, 1, 0, -1, 0, 0],
        "SSA" => [0, 0, 2, 0, 0, 0],
        "SA" => [0, 0, 1, 0, 0, 0],
        "MSF" => [0, 2, -2, 0, 0, 0],
        "MF" => [0, 2, 0, 0, 0, 0],
        "RHO" => [1, -2, 2, -1, 0, 0],
        "Q1" => [1, -2, 0, 1, 0, 0],
        "T2" => [2, 2, -3, 0, 0, 1],
        "R2" => [2, 2, -1, 0, 0, -1],
        "2Q1" => [1, -3, 0, 2, 0, 0],
        "P1" => [1, 1, -2, 0, 0, 0],
        "2SM2" => [2, 4, -4, 0, 0, 0],
        "M3" => [3, 0, 0, 0, 0, 0],
        "L2" => [2, 1, 0, -1, 0, 0],
        "2MK3" => [3, -1, 0, 0, 0, 0],
        "K2" => [2, 2, 0, 0, 0, 0],
        "M8" => [8, 0, 0, 0, 0, 0],
        "MS4" => [4, 2, -2, 0, 0, 0],
        _ => return None,
    };
    let semi_cycles = match name {
        "2Q1" | "J1" | "K1" | "M1" | "O1" | "OO1" | "P1" | "Q1" | "RHO" | "S1" => 0.25,
        _ => 0.0,
    };
    Some(ConstituentDefinition {
        coefficients,
        semi_cycles,
    })
}

fn astronomical_cycles(at: UtcDateTime) -> [f64; 6] {
    let jd = at.ordinal_days();
    let d = jd - 693_595.5;
    let big_d = d / 10_000.0;
    let powers = [1.0, d, big_d * big_d, big_d * big_d * big_d];
    let s = polynomial_cycles(
        [270.434_164, 13.176_396_526_8, -0.000_085_0, 0.000_000_039],
        powers,
    );
    let h = polynomial_cycles([279.696_678, 0.985_647_335_4, 0.000_022_67, 0.0], powers);
    let p = polynomial_cycles(
        [334.329_556, 0.111_404_080_3, -0.000_773_9, -0.000_000_26],
        powers,
    );
    let negative_node = polynomial_cycles(
        [-259.183_275, 0.052_953_922_2, -0.000_155_7, -0.000_000_050],
        powers,
    );
    let pp = polynomial_cycles(
        [281.220_844, 0.000_047_068_4, 0.000_033_9, 0.000_000_070],
        powers,
    );
    let tau = (jd.rem_euclid(1.0) + h - s).rem_euclid(1.0);
    [tau, s, h, p, negative_node, pp]
}

fn polynomial_cycles(coefficients: [f64; 4], powers: [f64; 4]) -> f64 {
    let degrees = coefficients
        .iter()
        .zip(powers)
        .map(|(coefficient, power)| coefficient * power)
        .sum::<f64>();
    (degrees / 360.0).rem_euclid(1.0)
}

fn ensure_finite(type_name: &'static str, value: f64) -> Result<(), CoreError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(CoreError::NonFinite { type_name, value })
    }
}

impl PartialEq for TideModel {
    fn eq(&self, other: &Self) -> bool {
        self.datum == other.datum
            && self.z0 == other.z0
            && self.constituents.len() == other.constituents.len()
            && self.method == other.method
    }
}

impl Eq for TideModel {}

impl PartialOrd for TideModel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.datum.cmp(&other.datum))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use proptest::prelude::*;

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

    fn single_m2_model() -> TideModel {
        must(TideModel::new(
            must(DatumId::new("MLLW")),
            must(Meters::new(1.0)),
            vec![HarmonicConstituent::new(
                must(ConstituentId::new("M2")),
                must(Meters::new(0.5)),
                must(Degrees::new(0.0)),
                must(DegreesPerHour::new(28.984_104)),
            )],
            PredictionMethod::HarmonicBasicNoNodal,
        ))
    }

    proptest! {
        #[test]
        fn harmonic_height_is_continuous(seconds in 1_609_459_200_i64..1_893_456_000_i64) {
            let at = UtcDateTime::from_utc(must_some(Utc.timestamp_opt(seconds, 0).single()));
            let model = single_m2_model();
            let first = predict_height(&model, at).height().as_meters();
            let second = predict_height(&model, at.add_seconds(60)).height().as_meters();
            prop_assert!((first - second).abs() < 0.04);
        }
    }

    #[test]
    fn m2_is_approximately_periodic() {
        let model = single_m2_model();
        let at = must(UtcDateTime::parse_rfc3339("2026-08-15T12:00:00Z"));
        let period_seconds = (360.0_f64 / 28.984_104_f64 * 3600.0_f64).round() as i64;
        let first = predict_height(&model, at).height().as_meters();
        let second = predict_height(&model, at.add_seconds(period_seconds))
            .height()
            .as_meters();
        assert!((first - second).abs() < 0.01);
    }

    #[test]
    fn duplicate_constituents_are_rejected() {
        let constituent = HarmonicConstituent::new(
            must(ConstituentId::new("M2")),
            must(Meters::new(1.0)),
            must(Degrees::new(0.0)),
            must(DegreesPerHour::new(28.984_104)),
        );
        let result = TideModel::new(
            must(DatumId::new("MLLW")),
            must(Meters::new(0.0)),
            vec![constituent.clone(), constituent],
            PredictionMethod::HarmonicBasicNoNodal,
        );
        assert!(matches!(result, Err(CoreError::DuplicateConstituent(_))));
    }
}
