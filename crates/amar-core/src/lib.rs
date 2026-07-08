//! Pure harmonic tide engine for amar.
//!
//! This crate has no I/O, no system clock access, and no local timezone logic.

mod astro;
mod constituents;
mod extrema;
mod nodal;
mod types;

use astro::{astronomical_argument_degrees, astronomical_terms};
use chrono::{Datelike, TimeZone, Utc};
use constituents::constituent_definition;
use nodal::{NodalTerms, nodal_terms};

pub use constituents::{
    constituent_speed_degrees_per_hour, port_selection_constituent_names,
    supported_constituent_names,
};
pub use extrema::{extrema_between, next_extrema_after, tide_windows};
pub use types::{
    ConstituentId, CoreError, DatumId, Degrees, DegreesPerHour, HarmonicConstituent, Meters,
    PredictionMethod, Radians, TideExtremum, TideExtremumKind, TideModel, TidePoint,
    TidePrediction, TideThresholdDirection, TideWindow, UtcDateTime,
};

pub fn predict_height(model: &TideModel, at: UtcDateTime) -> TidePrediction {
    predict_height_direct(model, at)
}

fn predict_height_direct(model: &TideModel, at: UtcDateTime) -> TidePrediction {
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
    }
}

#[derive(Clone, Debug)]
pub struct CompiledTideModel {
    z0: Meters,
    year_start: UtcDateTime,
    constituents: Vec<CompiledConstituent>,
}

#[derive(Clone, Copy, Debug)]
struct CompiledConstituent {
    amp_eff_m: f64,
    phase0_degrees: f64,
    speed_degrees_per_hour: f64,
}

impl CompiledTideModel {
    pub fn for_year(model: &TideModel, year: i32) -> Result<Self, CoreError> {
        match model.method {
            PredictionMethod::StationHarmonicsV0 => {}
            PredictionMethod::HarmonicBasicNoNodal => {
                return Err(CoreError::InvalidTimestamp(
                    "compiled tide model requires station_harmonics_v0".to_string(),
                ));
            }
        }
        let year_start = utc_year_start(year)?;
        let start_astro = astronomical_terms(year_start);
        let mid_year_astro = astronomical_terms(year_start.civil_year_midpoint());
        let mid_year_nodal = nodal_terms(&mid_year_astro);
        let mut constituents = Vec::with_capacity(model.constituents().len());

        for constituent in model.constituents() {
            let Some(definition) = constituent_definition(constituent.id().as_str()) else {
                unreachable!("constituents are validated by TideModel::new");
            };
            let argument0 = astronomical_argument_degrees(definition, &start_astro);
            let nodal_phase = definition.nodal_phase_degrees(&mid_year_nodal);
            let nodal_factor = definition.nodal_factor(&mid_year_nodal);
            constituents.push(CompiledConstituent {
                amp_eff_m: nodal_factor * constituent.amplitude().as_meters(),
                phase0_degrees: argument0 + nodal_phase - constituent.phase_gmt().as_degrees(),
                speed_degrees_per_hour: constituent.speed().as_degrees_per_hour(),
            });
        }

        Ok(Self {
            z0: model.z0,
            year_start,
            constituents,
        })
    }

    pub fn contains(&self, at: UtcDateTime) -> bool {
        at.civil_year_start() == self.year_start
    }

    pub fn predict_height(&self, at: UtcDateTime) -> Result<TidePrediction, CoreError> {
        if !self.contains(at) {
            return Err(CoreError::InvalidTimestamp(
                "compiled tide model does not match timestamp year".to_string(),
            ));
        }
        Ok(self.predict_height_unchecked(at))
    }

    fn predict_height_unchecked(&self, at: UtcDateTime) -> TidePrediction {
        let hours = at.hours_since(self.year_start);
        let mut height = self.z0.as_meters();
        for constituent in &self.constituents {
            let phase = constituent.phase0_degrees + constituent.speed_degrees_per_hour * hours;
            height += constituent.amp_eff_m * Degrees(phase).to_radians().as_radians().cos();
        }
        TidePrediction {
            height: Meters(height),
        }
    }
}

fn utc_year_start(year: i32) -> Result<UtcDateTime, CoreError> {
    Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0)
        .single()
        .map(UtcDateTime::from_utc)
        .ok_or_else(|| CoreError::InvalidTimestamp(format!("invalid UTC year {year}")))
}

pub(crate) struct HeightEvaluator<'a> {
    model: &'a TideModel,
    mode: HeightEvaluatorMode,
}

enum HeightEvaluatorMode {
    Direct,
    Compiled(CompiledTideModel),
}

impl<'a> HeightEvaluator<'a> {
    pub(crate) fn new(model: &'a TideModel, at: UtcDateTime) -> Self {
        let mode = match model.method {
            PredictionMethod::StationHarmonicsV0 => {
                let compiled = CompiledTideModel::for_year(model, at.as_chrono().year())
                    .expect("valid UTC year from timestamp");
                HeightEvaluatorMode::Compiled(compiled)
            }
            PredictionMethod::HarmonicBasicNoNodal => HeightEvaluatorMode::Direct,
        };
        Self { model, mode }
    }

    pub(crate) fn predict_height(&mut self, at: UtcDateTime) -> TidePrediction {
        match &mut self.mode {
            HeightEvaluatorMode::Direct => predict_height_direct(self.model, at),
            HeightEvaluatorMode::Compiled(compiled) => {
                if !compiled.contains(at) {
                    *compiled = CompiledTideModel::for_year(self.model, at.as_chrono().year())
                        .expect("valid UTC year from timestamp");
                }
                compiled.predict_height_unchecked(at)
            }
        }
    }
}

pub fn predict_series(
    model: &TideModel,
    from: UtcDateTime,
    duration_h: u32,
    step_min: u32,
) -> Vec<TidePoint> {
    let duration_seconds = i64::from(duration_h) * 60 * 60;
    let step_seconds = i64::from(step_min) * 60;
    let mut points = Vec::new();
    if step_seconds <= 0 {
        return points;
    }

    let mut evaluator = HeightEvaluator::new(model, from);
    let mut offset_seconds = 0_i64;
    while offset_seconds <= duration_seconds {
        let at = from.add_seconds(offset_seconds);
        points.push(TidePoint {
            at,
            height: evaluator.predict_height(at).height,
        });
        offset_seconds += step_seconds;
    }
    points
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HarmonicBasis {
    pub argument_degrees: f64,
    pub nodal_factor: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HarmonicYearBasis {
    pub argument0_degrees: f64,
    pub nodal_phase_degrees: f64,
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

    pub fn year_start(self) -> UtcDateTime {
        self.year_start
    }

    pub fn hours_since_year_start(self, at: UtcDateTime) -> Result<f64, CoreError> {
        if !self.contains(at) {
            return Err(CoreError::InvalidTimestamp(
                "harmonic year context does not match timestamp".to_string(),
            ));
        }
        Ok(at.hours_since(self.year_start))
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
        let annual = self.annual_basis(constituent_id)?;
        Ok(HarmonicBasis {
            argument_degrees: annual.argument0_degrees
                + speed.as_degrees_per_hour() * at.hours_since(self.year_start)
                + annual.nodal_phase_degrees,
            nodal_factor: annual.nodal_factor,
        })
    }

    pub fn annual_basis(
        self,
        constituent_id: &ConstituentId,
    ) -> Result<HarmonicYearBasis, CoreError> {
        let definition = constituent_definition(constituent_id.as_str())
            .ok_or_else(|| CoreError::UnknownConstituent(constituent_id.to_string()))?;
        Ok(HarmonicYearBasis {
            argument0_degrees: astronomical_argument_degrees(definition, &self.start_astro),
            nodal_phase_degrees: definition.nodal_phase_degrees(&self.mid_year_nodal),
            nodal_factor: definition.nodal_factor(&self.mid_year_nodal),
        })
    }
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
    use chrono::{Datelike, TimeZone, Utc};
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

    fn extended_catalog_model() -> TideModel {
        let constituents = supported_constituent_names()
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let speed = must_some(constituent_speed_degrees_per_hour(name));
                HarmonicConstituent::new(
                    must(ConstituentId::new(*name)),
                    must(Meters::new(0.01 + index as f64 * 0.002)),
                    must(Degrees::new((index as f64 * 29.0).rem_euclid(360.0))),
                    must(DegreesPerHour::new(speed)),
                )
            })
            .collect::<Vec<_>>();
        must(TideModel::new(
            must(DatumId::new("zero_hydrographique_synthetic")),
            must(Meters::new(2.0)),
            constituents,
            PredictionMethod::StationHarmonicsV0,
        ))
    }

    fn assert_compiled_matches_direct(model: &TideModel, at: UtcDateTime) {
        let compiled = must(CompiledTideModel::for_year(model, at.as_chrono().year()));
        let direct = predict_height_direct(model, at).height().as_meters();
        let compiled = must(compiled.predict_height(at)).height().as_meters();
        let delta = (direct - compiled).abs();
        assert!(delta <= 1e-9, "at={:?} delta={delta}", at.as_chrono());
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

    #[test]
    fn supported_catalog_has_definitions_and_speeds() {
        assert!(supported_constituent_names().len() >= 68);
        assert_eq!(port_selection_constituent_names().len(), 68);

        for name in supported_constituent_names() {
            assert!(constituent_definition(name).is_some(), "{name}");
            assert!(
                must_some(constituent_speed_degrees_per_hour(name)).is_finite(),
                "{name}"
            );
        }
        for name in port_selection_constituent_names() {
            assert!(supported_constituent_names().contains(name), "{name}");
        }
    }

    #[test]
    fn shallow_constituents_follow_documented_combinations() {
        let cases = [
            ("M4", &[(2, "M2")][..]),
            ("M6", &[(3, "M2")][..]),
            ("M8", &[(4, "M2")][..]),
            ("MS4", &[(1, "M2"), (1, "S2")][..]),
            ("MN4", &[(1, "M2"), (1, "N2")][..]),
            ("2MS6", &[(2, "M2"), (1, "S2")][..]),
            ("2MN6", &[(2, "M2"), (1, "N2")][..]),
            ("MSN2", &[(1, "M2"), (1, "S2"), (-1, "N2")][..]),
            ("MK3", &[(1, "M2"), (1, "K1")][..]),
            ("2MK3", &[(2, "M2"), (-1, "K1")][..]),
        ];

        for (name, terms) in cases {
            let terms = terms
                .iter()
                .map(|(coefficient, term_name)| {
                    crate::constituents::CompoundTerm::new(*coefficient, term_name)
                })
                .collect::<Vec<_>>();
            let actual = must_some(constituent_definition(name));
            let expected = must_some(crate::constituents::compound_definition(&terms));
            assert_eq!(actual.coefficients, expected.coefficients, "{name}");
            assert_eq!(actual.u_coefficients, expected.u_coefficients, "{name}");
            assert_eq!(actual.factor_terms, expected.factor_terms, "{name}");
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

        #[test]
        fn compiled_model_matches_direct_model_random_2020_2030(seconds in 1_577_836_800_i64..1_924_992_000_i64) {
            let at = UtcDateTime::from_utc(must_some(Utc.timestamp_opt(seconds, 0).single()));
            let model = extended_catalog_model();
            let compiled = must(CompiledTideModel::for_year(&model, at.as_chrono().year()));
            let direct = predict_height_direct(&model, at).height().as_meters();
            let compiled = must(compiled.predict_height(at)).height().as_meters();
            prop_assert!((direct - compiled).abs() <= 1e-9);
        }
    }

    #[test]
    fn compiled_model_matches_direct_model_around_year_boundaries() {
        let model = extended_catalog_model();
        for year in 2020..=2031 {
            let boundary = must_some(Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).single());
            for offset_seconds in -60..=60 {
                let at =
                    UtcDateTime::from_utc(boundary + chrono::TimeDelta::seconds(offset_seconds));
                assert_compiled_matches_direct(&model, at);
            }
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
    fn series_is_bounded_by_duration() {
        let model = single_m2_model();
        let at = must(UtcDateTime::parse_rfc3339("2026-08-15T00:00:00Z"));
        let series = predict_series(&model, at, 2, 30);

        assert_eq!(series.len(), 5);
        assert_eq!(series[0].at(), at);
        assert_eq!(series[4].at(), at.add_seconds(2 * 60 * 60));
    }

    #[test]
    fn extrema_alternate_on_single_constituent_model() {
        let model = single_m2_model();
        let from = must(UtcDateTime::parse_rfc3339("2026-08-15T00:00:00Z"));
        let to = must(UtcDateTime::parse_rfc3339("2026-08-16T00:00:00Z"));
        let extrema = extrema_between(&model, from, to);

        assert!(extrema.len() >= 3);
        for pair in extrema.windows(2) {
            assert_ne!(pair[0].kind(), pair[1].kind());
        }
    }

    #[test]
    fn next_extrema_finds_high_and_low_after_timestamp() {
        let model = single_m2_model();
        let at = must(UtcDateTime::parse_rfc3339("2026-08-15T00:00:00Z"));
        let (next_high, next_low) = next_extrema_after(&model, at, 36);

        assert!(next_high.is_some());
        assert!(next_low.is_some());
        assert!(next_high.unwrap_or_else(|| unreachable!()).at() > at);
        assert!(next_low.unwrap_or_else(|| unreachable!()).at() > at);
    }

    #[test]
    fn window_boundaries_are_threshold_crossings() {
        let model = single_m2_model();
        let from = must(UtcDateTime::parse_rfc3339("2026-08-15T00:00:00Z"));
        let to = must(UtcDateTime::parse_rfc3339("2026-08-16T00:00:00Z"));
        let threshold = must(Meters::new(1.2));
        let windows = tide_windows(&model, from, to, threshold, TideThresholdDirection::Above);

        assert!(!windows.is_empty());
        for window in windows {
            assert!(window.start() >= from);
            assert!(window.end() <= to);
            if window.start() != from {
                let start_height = predict_height(&model, window.start()).height().as_meters();
                assert!((start_height - threshold.as_meters()).abs() < 0.001);
            }
            if window.end() != to {
                let end_height = predict_height(&model, window.end()).height().as_meters();
                assert!((end_height - threshold.as_meters()).abs() < 0.001);
            }
        }
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
