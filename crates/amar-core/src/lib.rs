//! Pure harmonic tide engine for amar.
//!
//! This crate has no I/O, no system clock access, and no local timezone logic.

mod astro;
mod constituents;
mod nodal;
mod types;

use astro::{astronomical_argument_degrees, astronomical_terms};
use constituents::constituent_definition;
use nodal::{nodal_correction, nodal_terms};

pub use types::{
    ConstituentId, CoreError, DatumId, Degrees, DegreesPerHour, HarmonicConstituent, Meters,
    PredictionMethod, Radians, TideModel, TidePrediction, UtcDateTime,
};

pub fn predict_height(model: &TideModel, at: UtcDateTime) -> TidePrediction {
    let mut height = model.z0.as_meters();
    let astro = astronomical_terms(at);
    let nodal = nodal_terms(&astro);
    for constituent in model.constituents() {
        let Some(definition) = constituent_definition(constituent.id().as_str()) else {
            unreachable!("constituents are validated by TideModel::new");
        };
        let correction = nodal_correction(definition, &nodal, model.method);
        let argument = astronomical_argument_degrees(definition, &astro);
        let phase = argument + correction.phase_degrees - constituent.phase_gmt().as_degrees();
        let contribution = correction.factor
            * constituent.amplitude().as_meters()
            * Degrees(phase).to_radians().as_radians().cos();
        height += contribution;
    }

    TidePrediction {
        height: Meters(height),
        method: model.method,
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
