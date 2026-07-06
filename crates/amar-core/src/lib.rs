//! Pure harmonic tide engine for amar.
//!
//! This crate has no I/O, no system clock access, and no local timezone logic.

mod astro;
mod constituents;
mod nodal;
mod types;

use astro::{astronomical_argument_degrees, astronomical_terms};
use constituents::constituent_definition;
use nodal::{NodalTerms, nodal_terms};

pub use types::{
    ConstituentId, CoreError, DatumId, Degrees, DegreesPerHour, HarmonicConstituent, Meters,
    PredictionMethod, Radians, TideModel, TidePrediction, UtcDateTime,
};

pub fn predict_height(model: &TideModel, at: UtcDateTime) -> TidePrediction {
    let mut height = model.z0.as_meters();
    let convention = PredictionConvention::new(model.method, at);
    for constituent in model.constituents() {
        let Some(definition) = constituent_definition(constituent.id().as_str()) else {
            unreachable!("constituents are validated by TideModel::new");
        };
        let basis = convention.basis(definition, constituent);
        let phase = basis.argument_degrees - constituent.phase_gmt().as_degrees();
        let contribution = basis.nodal_factor
            * constituent.amplitude().as_meters()
            * Degrees(phase).to_radians().as_radians().cos();
        height += contribution;
    }

    TidePrediction {
        height: Meters(height),
        method: model.method,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HarmonicBasis {
    pub argument_degrees: f64,
    pub nodal_factor: f64,
}

#[derive(Clone, Copy)]
pub struct HarmonicYearContext {
    year_start: UtcDateTime,
    start_astro: astro::AstronomicalTerms,
    mid_year_nodal: NodalTerms,
}

impl HarmonicYearContext {
    pub fn new(at: UtcDateTime) -> Self {
        let year_start = at.civil_year_start();
        let start_astro = astronomical_terms(year_start);
        let mid_year_astro = astronomical_terms(at.civil_year_midpoint());
        let mid_year_nodal = nodal_terms(&mid_year_astro);
        Self {
            year_start,
            start_astro,
            mid_year_nodal,
        }
    }

    pub fn contains(self, at: UtcDateTime) -> bool {
        at.civil_year_start() == self.year_start
    }

    pub fn basis(
        self,
        constituent_id: &ConstituentId,
        speed: DegreesPerHour,
        at: UtcDateTime,
    ) -> Result<HarmonicBasis, CoreError> {
        if !self.contains(at) {
            return Err(CoreError::InvalidTimestamp(
                "harmonic year context does not match timestamp".to_string(),
            ));
        }
        let definition = constituent_definition(constituent_id.as_str())
            .ok_or_else(|| CoreError::UnknownConstituent(constituent_id.to_string()))?;
        Ok(HarmonicBasis {
            argument_degrees: astronomical_argument_degrees(definition, &self.start_astro)
                + speed.as_degrees_per_hour() * at.hours_since(self.year_start)
                + definition.nodal_phase_degrees(&self.mid_year_nodal),
            nodal_factor: definition.nodal_factor(&self.mid_year_nodal),
        })
    }
}

pub fn harmonic_basis(
    constituent_id: &ConstituentId,
    speed: DegreesPerHour,
    method: PredictionMethod,
    at: UtcDateTime,
) -> Result<HarmonicBasis, CoreError> {
    if method == PredictionMethod::StationHarmonicsV0 {
        return HarmonicYearContext::new(at).basis(constituent_id, speed, at);
    }
    let definition = constituent_definition(constituent_id.as_str())
        .ok_or_else(|| CoreError::UnknownConstituent(constituent_id.to_string()))?;
    let constituent = HarmonicConstituent::new(
        constituent_id.clone(),
        Meters::new(1.0)?,
        Degrees::new(0.0)?,
        speed,
    );
    Ok(PredictionConvention::new(method, at).basis(definition, &constituent))
}

struct PredictionConvention {
    mode: PredictionConventionMode,
}

enum PredictionConventionMode {
    AnnualNoaa {
        start_astro: astro::AstronomicalTerms,
        mid_year_nodal: NodalTerms,
        hours_since_year_start: f64,
    },
    InstantNoNodal {
        astro: astro::AstronomicalTerms,
    },
}

impl PredictionConvention {
    fn new(method: PredictionMethod, at: UtcDateTime) -> Self {
        let mode = match method {
            PredictionMethod::StationHarmonicsV0 => {
                let year_start = at.civil_year_start();
                let start_astro = astronomical_terms(year_start);
                let mid_year_astro = astronomical_terms(at.civil_year_midpoint());
                let mid_year_nodal = nodal_terms(&mid_year_astro);
                PredictionConventionMode::AnnualNoaa {
                    start_astro,
                    mid_year_nodal,
                    hours_since_year_start: at.hours_since(year_start),
                }
            }
            PredictionMethod::HarmonicBasicNoNodal => PredictionConventionMode::InstantNoNodal {
                astro: astronomical_terms(at),
            },
        };
        Self { mode }
    }

    fn argument_degrees(
        &self,
        definition: constituents::ConstituentDefinition,
        constituent: &HarmonicConstituent,
    ) -> f64 {
        match &self.mode {
            PredictionConventionMode::AnnualNoaa {
                start_astro,
                hours_since_year_start,
                ..
            } => {
                astronomical_argument_degrees(definition, start_astro)
                    + constituent.speed().as_degrees_per_hour() * hours_since_year_start
            }
            PredictionConventionMode::InstantNoNodal { astro } => {
                astronomical_argument_degrees(definition, astro)
            }
        }
    }

    fn basis(
        &self,
        definition: constituents::ConstituentDefinition,
        constituent: &HarmonicConstituent,
    ) -> HarmonicBasis {
        HarmonicBasis {
            argument_degrees: self.argument_degrees(definition, constituent)
                + self.nodal_phase_degrees(definition),
            nodal_factor: self.nodal_factor(definition),
        }
    }

    fn nodal_phase_degrees(&self, definition: constituents::ConstituentDefinition) -> f64 {
        match &self.mode {
            PredictionConventionMode::AnnualNoaa { mid_year_nodal, .. } => {
                definition.nodal_phase_degrees(mid_year_nodal)
            }
            PredictionConventionMode::InstantNoNodal { .. } => 0.0,
        }
    }

    fn nodal_factor(&self, definition: constituents::ConstituentDefinition) -> f64 {
        match &self.mode {
            PredictionConventionMode::AnnualNoaa { mid_year_nodal, .. } => {
                definition.nodal_factor(mid_year_nodal)
            }
            PredictionConventionMode::InstantNoNodal { .. } => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astro::{astronomical_argument_degrees, astronomical_terms};
    use crate::constituents::constituent_definition;
    use crate::nodal::nodal_terms;
    use chrono::{TimeZone, Utc};
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

    #[test]
    fn unknown_constituents_are_rejected() {
        let result = TideModel::new(
            must(DatumId::new("MLLW")),
            must(Meters::new(0.0)),
            vec![HarmonicConstituent::new(
                must(ConstituentId::new("ZZ9")),
                must(Meters::new(1.0)),
                must(Degrees::new(0.0)),
                must(DegreesPerHour::new(28.984_104)),
            )],
            PredictionMethod::StationHarmonicsV0,
        );
        assert!(matches!(
            result,
            Err(CoreError::UnknownConstituent(name)) if name == "ZZ9"
        ));
    }
}
