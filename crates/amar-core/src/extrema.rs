use crate::{
    HeightEvaluator, Meters, TideExtremum, TideExtremumKind, TideModel, TideThresholdDirection,
    TideWindow, UtcDateTime,
};

const EXTREMUM_SAMPLE_STEP_SECONDS: i64 = 6 * 60;
const NEXT_EXTREMA_CHUNK_SECONDS: i64 = 18 * 60 * 60;
const NEXT_EXTREMA_OVERLAP_SECONDS: i64 = 2 * EXTREMUM_SAMPLE_STEP_SECONDS;
const WINDOW_SCAN_MARGIN_SECONDS: i64 = 48 * 60 * 60;
const THRESHOLD_ROOT_TOLERANCE_SECONDS: i64 = 1;
const THRESHOLD_ROOT_DEDUP_SECONDS: i64 = 1;
const THRESHOLD_VALUE_EPSILON: f64 = 1e-9;
const PARABOLIC_DENOMINATOR_EPSILON: f64 = 1e-12;

/// Detect high and low waters by sampling every six minutes, then refine each
/// local peak/trough with a quadratic fit through the bracketing triplet.
pub fn extrema_between(model: &TideModel, from: UtcDateTime, to: UtcDateTime) -> Vec<TideExtremum> {
    if to <= from {
        return Vec::new();
    }

    let mut evaluator = HeightEvaluator::new(model, from);
    let samples = sample_heights(&mut evaluator, from, to, EXTREMUM_SAMPLE_STEP_SECONDS);
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
            height: evaluator.predict_height(at).height,
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
    let mut next_high = None;
    let mut next_low = None;
    let mut chunk_from = search_from;
    while chunk_from < search_to {
        let chunk_to = chunk_from
            .add_seconds(NEXT_EXTREMA_CHUNK_SECONDS)
            .min(search_to);
        for extremum in extrema_between(model, chunk_from, chunk_to) {
            if extremum.at <= after {
                continue;
            }
            match extremum.kind {
                TideExtremumKind::High => {
                    if next_high.is_none_or(|current: TideExtremum| extremum.at < current.at) {
                        next_high = Some(extremum);
                    }
                }
                TideExtremumKind::Low => {
                    if next_low.is_none_or(|current: TideExtremum| extremum.at < current.at) {
                        next_low = Some(extremum);
                    }
                }
            }
        }
        if next_high.is_some() && next_low.is_some() {
            break;
        }
        if chunk_to >= search_to {
            break;
        }
        chunk_from = chunk_to.add_seconds(-NEXT_EXTREMA_OVERLAP_SECONDS);
    }
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
    let mut evaluator = HeightEvaluator::new(model, scan_from);
    let roots = threshold_crossings(&mut evaluator, scan_from, scan_to, threshold, direction);
    let mut boundaries = Vec::with_capacity(roots.len() + 2);
    boundaries.push(scan_from);
    boundaries.extend(roots);
    boundaries.push(scan_to);

    let mut windows = Vec::new();
    for pair in boundaries.windows(2) {
        let start = pair[0];
        let end = pair[1];
        if end <= start {
            continue;
        }
        let middle = start.add_seconds(end.seconds_since(start) / 2);
        if threshold_active(&mut evaluator, middle, threshold, direction) {
            push_clamped_window(&mut windows, start, end, from, to);
        }
    }
    windows
}

fn sample_heights(
    evaluator: &mut HeightEvaluator<'_>,
    from: UtcDateTime,
    to: UtcDateTime,
    step_seconds: i64,
) -> Vec<(UtcDateTime, f64)> {
    let mut samples = Vec::with_capacity(sample_capacity(from, to, step_seconds));
    let mut at = from;
    while at <= to {
        samples.push((at, evaluator.predict_height(at).height().as_meters()));
        at = at.add_seconds(step_seconds);
    }
    if samples.last().map(|(at, _)| *at) != Some(to) {
        samples.push((to, evaluator.predict_height(to).height().as_meters()));
    }
    samples
}

fn sample_capacity(from: UtcDateTime, to: UtcDateTime, step_seconds: i64) -> usize {
    if to < from || step_seconds <= 0 {
        return 0;
    }
    (to.seconds_since(from) / step_seconds + 2) as usize
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
    if denominator.abs() < PARABOLIC_DENOMINATOR_EPSILON {
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
    evaluator: &mut HeightEvaluator<'_>,
    from: UtcDateTime,
    to: UtcDateTime,
    threshold: Meters,
    direction: TideThresholdDirection,
) -> Vec<UtcDateTime> {
    let mut roots = Vec::with_capacity(sample_capacity(from, to, EXTREMUM_SAMPLE_STEP_SECONDS));
    let mut left_at = from;
    let mut left_value = threshold_value(evaluator, left_at, threshold, direction);
    push_root_if_new(&mut roots, left_at, left_value);

    while left_at < to {
        let remaining_seconds = to.seconds_since(left_at);
        let step = remaining_seconds.min(EXTREMUM_SAMPLE_STEP_SECONDS);
        let right_at = left_at.add_seconds(step);
        let right_value = threshold_value(evaluator, right_at, threshold, direction);
        if left_value * right_value < 0.0 {
            roots.push(refine_threshold_crossing(
                evaluator,
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
    roots.dedup_by(|left, right| left.seconds_since(*right).abs() <= THRESHOLD_ROOT_DEDUP_SECONDS);
    roots
}

fn push_root_if_new(roots: &mut Vec<UtcDateTime>, at: UtcDateTime, value: f64) {
    if value.abs() > THRESHOLD_VALUE_EPSILON {
        return;
    }
    if roots
        .last()
        .is_some_and(|previous| at.seconds_since(*previous).abs() <= THRESHOLD_ROOT_DEDUP_SECONDS)
    {
        return;
    }
    roots.push(at);
}

fn refine_threshold_crossing(
    evaluator: &mut HeightEvaluator<'_>,
    mut left_at: UtcDateTime,
    mut left_value: f64,
    mut right_at: UtcDateTime,
    right_value: f64,
    threshold: Meters,
    direction: TideThresholdDirection,
) -> UtcDateTime {
    if left_value.abs() <= THRESHOLD_VALUE_EPSILON {
        return left_at;
    }
    if right_value.abs() <= THRESHOLD_VALUE_EPSILON {
        return right_at;
    }
    while right_at.seconds_since(left_at) > THRESHOLD_ROOT_TOLERANCE_SECONDS {
        let middle_at = left_at.add_seconds(right_at.seconds_since(left_at) / 2);
        let middle_value = threshold_value(evaluator, middle_at, threshold, direction);
        if left_value * middle_value <= 0.0 {
            right_at = middle_at;
        } else {
            left_at = middle_at;
            left_value = middle_value;
        }
    }
    left_at.add_seconds(right_at.seconds_since(left_at) / 2)
}

fn threshold_value(
    evaluator: &mut HeightEvaluator<'_>,
    at: UtcDateTime,
    threshold: Meters,
    direction: TideThresholdDirection,
) -> f64 {
    let delta = evaluator.predict_height(at).height().as_meters() - threshold.as_meters();
    match direction {
        TideThresholdDirection::Above => delta,
        TideThresholdDirection::Below => -delta,
    }
}

fn threshold_active(
    evaluator: &mut HeightEvaluator<'_>,
    at: UtcDateTime,
    threshold: Meters,
    direction: TideThresholdDirection,
) -> bool {
    threshold_value(evaluator, at, threshold, direction) >= 0.0
}

fn push_clamped_window(
    windows: &mut Vec<TideWindow>,
    start: UtcDateTime,
    end: UtcDateTime,
    from: UtcDateTime,
    to: UtcDateTime,
) {
    let start = start.max(from);
    let end = end.min(to);
    if end <= start {
        return;
    }
    if let Some(previous) = windows.last_mut()
        && start <= previous.end
    {
        previous.end = previous.end.max(end);
        return;
    }
    windows.push(TideWindow { start, end });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ConstituentId, DatumId, Degrees, DegreesPerHour, HarmonicConstituent, PredictionMethod,
        TideModel,
    };

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
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

    fn at(value: &str) -> UtcDateTime {
        must(UtcDateTime::parse_rfc3339(value))
    }

    #[test]
    fn threshold_crossings_cover_all_sampled_sign_changes_inside_range() {
        let model = single_m2_model();
        let from = at("2026-08-15T00:00:00Z");
        let to = at("2026-08-17T00:00:00Z");
        let threshold = must(Meters::new(1.2));
        let direction = TideThresholdDirection::Above;
        let mut evaluator = HeightEvaluator::new(&model, from);
        let roots = threshold_crossings(&mut evaluator, from, to, threshold, direction);

        assert!(roots.iter().all(|root| *root >= from && *root <= to));
        for pair in roots.windows(2) {
            assert!(pair[0] < pair[1]);
        }

        let mut left_at = from;
        let mut left_value = threshold_value(&mut evaluator, left_at, threshold, direction);
        while left_at < to {
            let right_at =
                left_at.add_seconds(to.seconds_since(left_at).min(EXTREMUM_SAMPLE_STEP_SECONDS));
            let right_value = threshold_value(&mut evaluator, right_at, threshold, direction);
            let sampled_crossing = left_value.abs() <= THRESHOLD_VALUE_EPSILON
                || right_value.abs() <= THRESHOLD_VALUE_EPSILON
                || left_value * right_value < 0.0;
            if sampled_crossing {
                assert!(
                    roots
                        .iter()
                        .any(|root| *root >= left_at && *root <= right_at),
                    "missing crossing between {:?} and {:?}",
                    left_at.as_chrono(),
                    right_at.as_chrono()
                );
            }
            left_at = right_at;
            left_value = right_value;
        }
    }
}
