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
    let astro = astronomical_terms(at);
    let nodal = nodal_terms(&astro);
    for constituent in &model.constituents {
        let Some(definition) = constituent_definition(constituent.id.as_str()) else {
            continue;
        };
        let correction = nodal_correction(definition, &nodal, model.method);
        let argument = astronomical_argument_degrees(definition, &astro);
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
    definition: ConstituentDefinition,
    nodal: &NodalTerms,
    method: PredictionMethod,
) -> NodalCorrection {
    match method {
        PredictionMethod::StationHarmonicsV0 => NodalCorrection {
            factor: definition.nodal_factor(nodal),
            phase_degrees: definition.nodal_phase_degrees(nodal),
        },
        PredictionMethod::HarmonicBasicNoNodal => NodalCorrection {
            factor: 1.0,
            phase_degrees: 0.0,
        },
    }
}

fn astronomical_argument_degrees(
    definition: ConstituentDefinition,
    astro: &AstronomicalTerms,
) -> f64 {
    let values = [astro.tau, astro.s, astro.h, astro.p, astro.p1];
    let mut cycles = definition.semi_cycles;
    for (coefficient, value) in definition.coefficients.iter().zip(values) {
        cycles += f64::from(*coefficient) * value;
    }
    cycles.rem_euclid(1.0) * 360.0
}

#[derive(Clone, Copy)]
struct ConstituentDefinition {
    coefficients: [i8; 5],
    semi_cycles: f64,
    u_coefficients: [i8; 7],
    factor_terms: [FactorTerm; 2],
}

impl ConstituentDefinition {
    fn new(
        coefficients: [i8; 5],
        semi_cycles: f64,
        u_coefficients: [i8; 7],
        factor_terms: [FactorTerm; 2],
    ) -> Self {
        Self {
            coefficients,
            semi_cycles,
            u_coefficients,
            factor_terms,
        }
    }

    fn nodal_phase_degrees(self, nodal: &NodalTerms) -> f64 {
        let values = [
            nodal.xi,
            nodal.nu,
            nodal.nu_prime,
            nodal.two_nu_double_prime,
            nodal.q,
            nodal.r,
            nodal.q_u,
        ];
        self.u_coefficients
            .iter()
            .zip(values)
            .map(|(coefficient, value)| f64::from(*coefficient) * value)
            .sum()
    }

    fn nodal_factor(self, nodal: &NodalTerms) -> f64 {
        self.factor_terms
            .iter()
            .filter(|term| term.power != 0)
            .map(|term| term.formula.value(nodal).powi(i32::from(term.power)))
            .product()
    }
}

#[derive(Clone, Copy)]
struct FactorTerm {
    formula: NodalFactorFormula,
    power: u8,
}

impl FactorTerm {
    const fn none() -> Self {
        Self {
            formula: NodalFactorFormula::Unity,
            power: 0,
        }
    }

    const fn new(formula: NodalFactorFormula, power: u8) -> Self {
        Self { formula, power }
    }
}

#[derive(Clone, Copy)]
enum NodalFactorFormula {
    Unity,
    Mm,
    Mf,
    O1,
    J1,
    Oo1,
    M2,
    M3,
    M1,
    L2,
    K1,
    K2,
}

impl NodalFactorFormula {
    fn value(self, nodal: &NodalTerms) -> f64 {
        match self {
            Self::Unity => 1.0,
            Self::Mm => f_mm(nodal.inclination),
            Self::Mf => f_mf(nodal.inclination),
            Self::O1 => f_o1(nodal.inclination),
            Self::J1 => f_j1(nodal.inclination),
            Self::Oo1 => f_oo1(nodal.inclination),
            Self::M2 => f_m2(nodal.inclination),
            Self::M3 => f_m3(nodal.inclination),
            Self::M1 => f_m1(nodal.inclination, nodal.p),
            Self::L2 => f_l2(nodal.inclination, nodal.p),
            Self::K1 => f_k1(nodal.inclination, nodal.nu),
            Self::K2 => f_k2(nodal.inclination, nodal.nu),
        }
    }
}

fn constituent_definition(name: &str) -> Option<ConstituentDefinition> {
    let n = FactorTerm::none();
    let f = FactorTerm::new;
    let definition = match name {
        "2MK3" => ConstituentDefinition::new(
            [3, -1, 0, 0, 0],
            0.25,
            [4, -4, 1, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 2), f(NodalFactorFormula::K1, 1)],
        ),
        "2N2" => ConstituentDefinition::new(
            [2, -2, 0, 2, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "2Q1" => ConstituentDefinition::new(
            [1, -3, 0, 2, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "2SM2" => ConstituentDefinition::new(
            [2, 4, -4, 0, 0],
            0.0,
            [-2, 2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "J1" => ConstituentDefinition::new(
            [1, 2, 0, -1, 0],
            -0.25,
            [0, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::J1, 1), n],
        ),
        "K1" => ConstituentDefinition::new(
            [1, 1, 0, 0, 0],
            -0.25,
            [0, 0, -1, 0, 0, 0, 0],
            [f(NodalFactorFormula::K1, 1), n],
        ),
        "K2" => ConstituentDefinition::new(
            [2, 2, 0, 0, 0],
            0.0,
            [0, 0, 0, -1, 0, 0, 0],
            [f(NodalFactorFormula::K2, 1), n],
        ),
        "L2" => ConstituentDefinition::new(
            [2, 1, 0, -1, 0],
            0.5,
            [2, -2, 0, 0, 0, -1, 0],
            [f(NodalFactorFormula::L2, 1), n],
        ),
        "LAM2" | "LDA2" => ConstituentDefinition::new(
            [2, 1, -2, 1, 0],
            0.5,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "M1" => ConstituentDefinition::new(
            [1, 0, 0, 0, 0],
            -0.25,
            [1, -1, 0, 0, 1, 0, 0],
            [f(NodalFactorFormula::M1, 1), n],
        ),
        "M2" => ConstituentDefinition::new(
            [2, 0, 0, 0, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "M3" => ConstituentDefinition::new(
            [3, 0, 0, 0, 0],
            0.0,
            [3, -3, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M3, 1), n],
        ),
        "M4" => ConstituentDefinition::new(
            [4, 0, 0, 0, 0],
            0.0,
            [4, -4, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 2), n],
        ),
        "M6" => ConstituentDefinition::new(
            [6, 0, 0, 0, 0],
            0.0,
            [6, -6, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 3), n],
        ),
        "M8" => ConstituentDefinition::new(
            [8, 0, 0, 0, 0],
            0.0,
            [8, -8, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 4), n],
        ),
        "MF" => ConstituentDefinition::new(
            [0, 2, 0, 0, 0],
            0.0,
            [-2, 0, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::Mf, 1), n],
        ),
        "MK3" => ConstituentDefinition::new(
            [3, 1, 0, 0, 0],
            -0.25,
            [2, -2, -1, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), f(NodalFactorFormula::K1, 1)],
        ),
        "MM" => ConstituentDefinition::new(
            [0, 1, 0, -1, 0],
            0.0,
            [0, 0, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::Mm, 1), n],
        ),
        "MN4" => ConstituentDefinition::new(
            [4, -1, 0, 1, 0],
            0.0,
            [4, -4, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 2), n],
        ),
        "MS4" => ConstituentDefinition::new(
            [4, 2, -2, 0, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "MSF" => ConstituentDefinition::new(
            [0, 2, -2, 0, 0],
            0.0,
            [-2, 2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "MU2" => ConstituentDefinition::new(
            [2, -2, 2, 0, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "N2" => ConstituentDefinition::new(
            [2, -1, 0, 1, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "NU2" => ConstituentDefinition::new(
            [2, -1, 2, -1, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "O1" => ConstituentDefinition::new(
            [1, -1, 0, 0, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "OO1" => ConstituentDefinition::new(
            [1, 3, 0, 0, 0],
            -0.25,
            [-2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::Oo1, 1), n],
        ),
        "P1" => ConstituentDefinition::new([1, 1, -2, 0, 0], 0.25, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "Q1" => ConstituentDefinition::new(
            [1, -2, 0, 1, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "R2" => ConstituentDefinition::new([2, 2, -1, 0, -1], 0.5, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "RHO" | "RHO1" => ConstituentDefinition::new(
            [1, -2, 2, -1, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "S1" => ConstituentDefinition::new([1, 1, -1, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "S2" => ConstituentDefinition::new([2, 2, -2, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "S4" => ConstituentDefinition::new([4, 4, -4, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "S6" => ConstituentDefinition::new([6, 6, -6, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "SA" => ConstituentDefinition::new([0, 0, 1, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "SSA" => ConstituentDefinition::new([0, 0, 2, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "T2" => ConstituentDefinition::new([2, 2, -3, 0, 1], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        _ => return None,
    };
    Some(definition)
}

#[derive(Clone, Copy)]
struct AstronomicalTerms {
    tau: f64,
    s: f64,
    h: f64,
    p: f64,
    p1: f64,
    node_degrees: f64,
}

fn astronomical_terms(at: UtcDateTime) -> AstronomicalTerms {
    let d = at.ordinal_days() - 693_595.5;
    let julian_centuries = d / 36_525.0;
    let powers = [
        1.0,
        julian_centuries,
        julian_centuries * julian_centuries,
        julian_centuries * julian_centuries * julian_centuries,
    ];
    let s = polynomial_cycles(
        [
            270.0 + 26.0 / 60.0 + 14.72 / 3600.0,
            1336.0 * 360.0 + 1_108_411.2 / 3600.0,
            9.09 / 3600.0,
            0.0068 / 3600.0,
        ],
        powers,
    );
    let h = polynomial_cycles(
        [
            279.0 + 41.0 / 60.0 + 48.04 / 3600.0,
            129_602_768.13 / 3600.0,
            1.089 / 3600.0,
            0.0,
        ],
        powers,
    );
    let p = polynomial_cycles(
        [
            334.0 + 19.0 / 60.0 + 40.87 / 3600.0,
            11.0 * 360.0 + 392_515.94 / 3600.0,
            -37.24 / 3600.0,
            -0.045 / 3600.0,
        ],
        powers,
    );
    let p1 = polynomial_cycles(
        [
            281.0 + 13.0 / 60.0 + 15.0 / 3600.0,
            6189.03 / 3600.0,
            1.63 / 3600.0,
            0.012 / 3600.0,
        ],
        powers,
    );
    let node_degrees = polynomial_degrees(
        [
            259.0 + 10.0 / 60.0 + 57.12 / 3600.0,
            -(5.0 * 360.0 + 482_912.63 / 3600.0),
            7.58 / 3600.0,
            0.008 / 3600.0,
        ],
        powers,
    );
    let mean_sun_hour_angle = d.rem_euclid(1.0);
    let tau = (mean_sun_hour_angle + h - s).rem_euclid(1.0);

    AstronomicalTerms {
        tau,
        s,
        h,
        p,
        p1,
        node_degrees,
    }
}

fn polynomial_cycles(coefficients: [f64; 4], powers: [f64; 4]) -> f64 {
    (polynomial_degrees(coefficients, powers) / 360.0).rem_euclid(1.0)
}

fn polynomial_degrees(coefficients: [f64; 4], powers: [f64; 4]) -> f64 {
    let degrees = coefficients
        .iter()
        .zip(powers)
        .map(|(coefficient, power)| coefficient * power)
        .sum::<f64>();
    degrees.rem_euclid(360.0)
}

#[derive(Clone, Copy)]
struct NodalTerms {
    inclination: f64,
    xi: f64,
    nu: f64,
    nu_prime: f64,
    two_nu_double_prime: f64,
    q: f64,
    r: f64,
    q_u: f64,
    p: f64,
}

fn nodal_terms(astro: &AstronomicalTerms) -> NodalTerms {
    let node = astro.node_degrees;
    let inclination = inclination(node);
    let xi = xi(node);
    let nu = nu(node);
    let nu_prime = nu_prime(node);
    let two_nu_double_prime = two_nu_double_prime(node);
    let p = astro.p * 360.0 - xi;
    let q = q(p);
    let r = r(p, inclination);
    let q_u = p - q;

    NodalTerms {
        inclination,
        xi,
        nu,
        nu_prime,
        two_nu_double_prime,
        q,
        r,
        q_u,
        p,
    }
}

fn sind(degrees: f64) -> f64 {
    degrees.to_radians().sin()
}

fn cosd(degrees: f64) -> f64 {
    degrees.to_radians().cos()
}

fn tand(degrees: f64) -> f64 {
    degrees.to_radians().tan()
}

fn atan2d(y: f64, x: f64) -> f64 {
    y.atan2(x).to_degrees()
}

fn asind(value: f64) -> f64 {
    value.clamp(-1.0, 1.0).asin().to_degrees()
}

fn acosd(value: f64) -> f64 {
    value.clamp(-1.0, 1.0).acos().to_degrees()
}

fn cos_inclination(node: f64) -> f64 {
    const OBLIQUITY: f64 = 23.0 + 27.0 / 60.0 + 8.26 / 3600.0;
    const LUNAR_INCLINATION: f64 = 5.0 + 8.0 / 60.0 + 43.3546 / 3600.0;
    cosd(OBLIQUITY) * cosd(LUNAR_INCLINATION)
        - sind(OBLIQUITY) * sind(LUNAR_INCLINATION) * cosd(node)
}

fn sin_inclination(node: f64) -> f64 {
    let value = cos_inclination(node);
    (1.0 - value * value).sqrt()
}

fn inclination(node: f64) -> f64 {
    acosd(cos_inclination(node))
}

fn sin_nu(node: f64) -> f64 {
    const LUNAR_INCLINATION: f64 = 5.0 + 8.0 / 60.0 + 43.3546 / 3600.0;
    sind(LUNAR_INCLINATION) * sind(node) / sin_inclination(node)
}

fn cos_nu(node: f64) -> f64 {
    let value = sin_nu(node);
    (1.0 - value * value).sqrt()
}

fn sin_omega_arc(node: f64) -> f64 {
    const OBLIQUITY: f64 = 23.0 + 27.0 / 60.0 + 8.26 / 3600.0;
    sind(OBLIQUITY) * sind(node) / sin_inclination(node)
}

fn cos_omega_arc(node: f64) -> f64 {
    const OBLIQUITY: f64 = 23.0 + 27.0 / 60.0 + 8.26 / 3600.0;
    cosd(node) * cos_nu(node) + sind(node) * sin_nu(node) * cosd(OBLIQUITY)
}

fn xi(node: f64) -> f64 {
    node - atan2d(sin_omega_arc(node), cos_omega_arc(node))
}

fn nu(node: f64) -> f64 {
    asind(sin_nu(node))
}

fn nu_prime(node: f64) -> f64 {
    let inclination = inclination(node);
    let multiplier = sind(2.0 * inclination);
    atan2d(
        multiplier * sin_nu(node),
        multiplier * cos_nu(node) + 0.3347,
    )
}

fn two_nu_double_prime(node: f64) -> f64 {
    let sin_i = sin_inclination(node);
    let term = sin_i * sin_i;
    let double_nu = 2.0 * nu(node);
    atan2d(term * sind(double_nu), term * cosd(double_nu) + 0.0727)
}

fn q(p: f64) -> f64 {
    atan2d(0.483 * sind(p), cosd(p))
}

fn q_amplitude(p: f64) -> f64 {
    1.0 / (2.31 + 1.435 * cosd(2.0 * p)).sqrt()
}

fn r(p: f64, inclination: f64) -> f64 {
    let cot_i2 = 1.0 / tand(inclination / 2.0);
    atan2d(sind(2.0 * p), cot_i2 * cot_i2 / 6.0 - cosd(2.0 * p))
}

fn r_amplitude(p: f64, inclination: f64) -> f64 {
    let term = tand(inclination / 2.0).powi(2);
    1.0 / (1.0 - 12.0 * term * cosd(2.0 * p) + 36.0 * term * term).sqrt()
}

fn f_mm(inclination: f64) -> f64 {
    let sin_i = sind(inclination);
    (2.0 / 3.0 - sin_i * sin_i) / 0.5021
}

fn f_mf(inclination: f64) -> f64 {
    sind(inclination).powi(2) / 0.1578
}

fn f_o1(inclination: f64) -> f64 {
    sind(inclination) * cosd(inclination / 2.0).powi(2) / 0.38
}

fn f_j1(inclination: f64) -> f64 {
    sind(2.0 * inclination) / 0.7214
}

fn f_oo1(inclination: f64) -> f64 {
    sind(inclination) * sind(inclination / 2.0).powi(2) / 0.0164
}

fn f_m2(inclination: f64) -> f64 {
    cosd(inclination / 2.0).powi(4) / 0.9154
}

fn f_m3(inclination: f64) -> f64 {
    cosd(inclination / 2.0).powi(6) / 0.8758
}

fn f_m1(inclination: f64, p: f64) -> f64 {
    f_o1(inclination) / q_amplitude(p)
}

fn f_l2(inclination: f64, p: f64) -> f64 {
    f_m2(inclination) / r_amplitude(p, inclination)
}

fn f_k1(inclination: f64, nu: f64) -> f64 {
    let term = sind(2.0 * inclination);
    (0.8965 * term * term + 0.6001 * term * cosd(nu) + 0.1006).sqrt()
}

fn f_k2(inclination: f64, nu: f64) -> f64 {
    let term = sind(inclination).powi(2);
    (19.0444 * term * term + 2.7702 * term * cosd(2.0 * nu) + 0.0981).sqrt()
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

    #[test]
    fn congen_2026_start_arguments_match_nos_table() {
        let start = must(UtcDateTime::parse_rfc3339("2026-01-01T00:00:00Z"));
        let mid_year = must(UtcDateTime::parse_rfc3339("2026-07-02T12:00:00Z"));
        let start_astro = astronomical_terms(start);
        let mid_year_nodal = nodal_terms(&astronomical_terms(mid_year));
        let cases = [
            ("M2", 66.36, 0.9674),
            ("K1", 14.26, 1.1031),
            ("O1", 50.65, 1.1668),
            ("L2", 250.96, 1.3288),
            ("M1", 25.28, 1.0994),
            ("K2", 208.93, 1.2815),
            ("MF", 145.06, 1.4069),
            ("MM", 6.69, 0.8856),
        ];

        for (name, expected_argument, expected_factor) in cases {
            let definition = must_some(constituent_definition(name));
            let argument = (astronomical_argument_degrees(definition, &start_astro)
                + definition.nodal_phase_degrees(&mid_year_nodal))
            .rem_euclid(360.0);
            let factor = definition.nodal_factor(&mid_year_nodal);
            assert!(
                (argument - expected_argument).abs() < 0.02,
                "{name} argument={argument}"
            );
            assert!(
                (factor - expected_factor).abs() < 0.0002,
                "{name} factor={factor}"
            );
        }
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
