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
    PredictionMethod, Radians, TideExtremum, TideExtremumKind, TideModel, TidePoint,
    TidePrediction, TideThresholdDirection, TideWindow, UtcDateTime,
};

const EXTREMUM_SAMPLE_STEP_SECONDS: i64 = 6 * 60;
const WINDOW_SCAN_MARGIN_SECONDS: i64 = 48 * 60 * 60;
const THRESHOLD_ROOT_TOLERANCE_SECONDS: i64 = 1;

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

    let mut offset_seconds = 0_i64;
    while offset_seconds <= duration_seconds {
        let at = from.add_seconds(offset_seconds);
        points.push(TidePoint {
            at,
            height: predict_height(model, at).height,
        });
        offset_seconds += step_seconds;
    }
    points
}

/// Detect high and low waters by sampling every six minutes, then refine each
/// local peak/trough with a quadratic fit through the bracketing triplet.
pub fn extrema_between(model: &TideModel, from: UtcDateTime, to: UtcDateTime) -> Vec<TideExtremum> {
    if to <= from {
        return Vec::new();
    }

    let samples = sample_heights(model, from, to, EXTREMUM_SAMPLE_STEP_SECONDS);
    let mut extrema = Vec::new();
    for triplet in samples.windows(3) {
        let (left_at, left_height) = triplet[0];
        let (middle_at, middle_height) = triplet[1];
        let (right_at, right_height) = triplet[2];
        let kind = if middle_height > left_height && middle_height >= right_height {
            TideExtremumKind::High
        } else if middle_height < left_height && middle_height <= right_height {
            TideExtremumKind::Low
        } else {
            continue;
        };
        let at = parabolic_vertex_time(
            left_at,
            left_height,
            middle_at,
            middle_height,
            right_at,
            right_height,
        );
        extrema.push(TideExtremum {
            at,
            height: predict_height(model, at).height,
            kind,
        });
    }
    dedup_extrema(extrema)
}

pub fn next_extrema_after(
    model: &TideModel,
    after: UtcDateTime,
    horizon_h: u32,
) -> (Option<TideExtremum>, Option<TideExtremum>) {
    let search_from = after.add_seconds(-EXTREMUM_SAMPLE_STEP_SECONDS);
    let search_to = after.add_seconds(i64::from(horizon_h) * 60 * 60);
    let extrema = extrema_between(model, search_from, search_to);
    let next_high = extrema
        .iter()
        .copied()
        .filter(|extremum| extremum.kind == TideExtremumKind::High && extremum.at > after)
        .min_by_key(|extremum| extremum.at);
    let next_low = extrema
        .iter()
        .copied()
        .filter(|extremum| extremum.kind == TideExtremumKind::Low && extremum.at > after)
        .min_by_key(|extremum| extremum.at);
    (next_high, next_low)
}

pub fn tide_windows(
    model: &TideModel,
    from: UtcDateTime,
    to: UtcDateTime,
    threshold: Meters,
    direction: TideThresholdDirection,
) -> Vec<TideWindow> {
    if to <= from {
        return Vec::new();
    }
    let scan_from = from.add_seconds(-WINDOW_SCAN_MARGIN_SECONDS);
    let scan_to = to.add_seconds(WINDOW_SCAN_MARGIN_SECONDS);
    let roots = threshold_crossings(model, scan_from, scan_to, threshold, direction);
    let mut windows = Vec::new();
    for pair in roots.windows(2) {
        let start = pair[0];
        let end = pair[1];
        if end <= start {
            continue;
        }
        let middle = start.add_seconds(end.seconds_since(start) / 2);
        if threshold_active(model, middle, threshold, direction) && end > from && start < to {
            windows.push(TideWindow { start, end });
        }
    }
    windows
}

fn sample_heights(
    model: &TideModel,
    from: UtcDateTime,
    to: UtcDateTime,
    step_seconds: i64,
) -> Vec<(UtcDateTime, f64)> {
    let mut samples = Vec::new();
    let mut at = from;
    while at <= to {
        samples.push((at, predict_height(model, at).height().as_meters()));
        at = at.add_seconds(step_seconds);
    }
    if samples.last().map(|(at, _)| *at) != Some(to) {
        samples.push((to, predict_height(model, to).height().as_meters()));
    }
    samples
}

fn parabolic_vertex_time(
    left_at: UtcDateTime,
    left_height: f64,
    middle_at: UtcDateTime,
    middle_height: f64,
    right_at: UtcDateTime,
    right_height: f64,
) -> UtcDateTime {
    let left_seconds = middle_at.seconds_since(left_at);
    let right_seconds = right_at.seconds_since(middle_at);
    if left_seconds != right_seconds || left_seconds <= 0 {
        return middle_at;
    }
    let denominator = left_height - 2.0 * middle_height + right_height;
    if denominator.abs() < 1e-12 {
        return middle_at;
    }
    let step_seconds = left_seconds as f64;
    let offset = 0.5 * (left_height - right_height) / denominator * step_seconds;
    let offset = offset.clamp(-step_seconds, step_seconds).round() as i64;
    middle_at.add_seconds(offset)
}

fn dedup_extrema(mut extrema: Vec<TideExtremum>) -> Vec<TideExtremum> {
    extrema.sort_by_key(|extremum| extremum.at);
    let mut deduped: Vec<TideExtremum> = Vec::new();
    for extremum in extrema {
        if let Some(previous) = deduped.last()
            && previous.kind == extremum.kind
            && extremum.at.seconds_since(previous.at).abs() <= EXTREMUM_SAMPLE_STEP_SECONDS
        {
            continue;
        }
        deduped.push(extremum);
    }
    deduped
}

fn threshold_crossings(
    model: &TideModel,
    from: UtcDateTime,
    to: UtcDateTime,
    threshold: Meters,
    direction: TideThresholdDirection,
) -> Vec<UtcDateTime> {
    let mut roots = Vec::new();
    let mut left_at = from;
    let mut left_value = threshold_value(model, left_at, threshold, direction);
    push_root_if_new(&mut roots, left_at, left_value);

    while left_at < to {
        let remaining_seconds = to.seconds_since(left_at);
        let step = remaining_seconds.min(EXTREMUM_SAMPLE_STEP_SECONDS);
        let right_at = left_at.add_seconds(step);
        let right_value = threshold_value(model, right_at, threshold, direction);
        if left_value * right_value < 0.0 {
            roots.push(refine_threshold_crossing(
                model,
                left_at,
                left_value,
                right_at,
                right_value,
                threshold,
                direction,
            ));
        }
        push_root_if_new(&mut roots, right_at, right_value);
        left_at = right_at;
        left_value = right_value;
    }

    roots.sort();
    roots.dedup_by(|left, right| left.seconds_since(*right).abs() <= 1);
    roots
}

fn push_root_if_new(roots: &mut Vec<UtcDateTime>, at: UtcDateTime, value: f64) {
    if value.abs() > 1e-9 {
        return;
    }
    if roots
        .last()
        .is_some_and(|previous| at.seconds_since(*previous).abs() <= 1)
    {
        return;
    }
    roots.push(at);
}

fn refine_threshold_crossing(
    model: &TideModel,
    mut left_at: UtcDateTime,
    mut left_value: f64,
    mut right_at: UtcDateTime,
    mut right_value: f64,
    threshold: Meters,
    direction: TideThresholdDirection,
) -> UtcDateTime {
    if left_value.abs() <= 1e-9 {
        return left_at;
    }
    if right_value.abs() <= 1e-9 {
        return right_at;
    }
    while right_at.seconds_since(left_at) > THRESHOLD_ROOT_TOLERANCE_SECONDS {
        let middle_at = left_at.add_seconds(right_at.seconds_since(left_at) / 2);
        let middle_value = threshold_value(model, middle_at, threshold, direction);
        if left_value * middle_value <= 0.0 {
            right_at = middle_at;
            right_value = middle_value;
        } else {
            left_at = middle_at;
            left_value = middle_value;
        }
    }
    let _ = right_value;
    left_at.add_seconds(right_at.seconds_since(left_at) / 2)
}

fn threshold_value(
    model: &TideModel,
    at: UtcDateTime,
    threshold: Meters,
    direction: TideThresholdDirection,
) -> f64 {
    let delta = predict_height(model, at).height().as_meters() - threshold.as_meters();
    match direction {
        TideThresholdDirection::Above => delta,
        TideThresholdDirection::Below => -delta,
    }
}

fn threshold_active(
    model: &TideModel,
    at: UtcDateTime,
    threshold: Meters,
    direction: TideThresholdDirection,
) -> bool {
    threshold_value(model, at, threshold, direction) >= 0.0
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
            let start_height = predict_height(&model, window.start()).height().as_meters();
            let end_height = predict_height(&model, window.end()).height().as_meters();
            assert!((start_height - threshold.as_meters()).abs() < 0.001);
            assert!((end_height - threshold.as_meters()).abs() < 0.001);
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
