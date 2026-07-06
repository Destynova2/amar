use crate::common::{CalError, Observation, format_rfc3339};
use chrono::{DateTime, Utc};

const MIN_COVERAGE: f64 = 0.90;
const JUMP_THRESHOLD_M: f64 = 2.5;
const MAX_JUMP_INTERVAL_MINUTES: i64 = 90;

#[derive(Debug)]
pub(crate) struct QcReport {
    pub(crate) expected: usize,
    pub(crate) observed: usize,
    pub(crate) coverage: f64,
    pub(crate) gaps: Vec<Gap>,
    pub(crate) jumps: Vec<Jump>,
}

#[derive(Debug)]
pub(crate) struct Gap {
    pub(crate) after: DateTime<Utc>,
    pub(crate) before: DateTime<Utc>,
    pub(crate) minutes: i64,
}

#[derive(Debug)]
pub(crate) struct Jump {
    pub(crate) from: DateTime<Utc>,
    pub(crate) to: DateTime<Utc>,
    pub(crate) delta_m: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct ResidualStats {
    pub(crate) samples: usize,
    pub(crate) bias_cm: f64,
    pub(crate) p95_cm: f64,
}

pub(crate) fn qc_report(
    observations: &[Observation],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> QcReport {
    let expected = (end - start).num_hours().max(0) as usize;
    let observed = observations.len();
    let coverage = if expected == 0 {
        0.0
    } else {
        observed as f64 / expected as f64
    };
    let mut gaps = Vec::new();
    let mut jumps = Vec::new();
    let mut sorted = observations.to_vec();
    sorted.sort_by_key(|observation| observation.at);
    for pair in sorted.windows(2) {
        let previous = pair[0];
        let next = pair[1];
        let delta_minutes = (next.at - previous.at).num_minutes();
        if delta_minutes > MAX_JUMP_INTERVAL_MINUTES {
            gaps.push(Gap {
                after: previous.at,
                before: next.at,
                minutes: delta_minutes,
            });
        }
        let delta_m = next.value_m - previous.value_m;
        if delta_minutes <= MAX_JUMP_INTERVAL_MINUTES && delta_m.abs() > JUMP_THRESHOLD_M {
            jumps.push(Jump {
                from: previous.at,
                to: next.at,
                delta_m,
            });
        }
    }
    QcReport {
        expected,
        observed,
        coverage,
        gaps,
        jumps,
    }
}

pub(crate) fn enforce_qc(label: &str, report: &QcReport) -> Result<(), CalError> {
    if report.coverage < MIN_COVERAGE {
        return Err(CalError::QualityGate(format!(
            "{label} coverage {:.3} below {:.3}",
            report.coverage, MIN_COVERAGE
        )));
    }
    if !report.jumps.is_empty() {
        let jump = &report.jumps[0];
        return Err(CalError::QualityGate(format!(
            "{label} aberrant jump {:.3} m between {} and {}",
            jump.delta_m,
            format_rfc3339(jump.from),
            format_rfc3339(jump.to)
        )));
    }
    Ok(())
}

pub(crate) fn print_qc(label: &str, report: &QcReport) {
    println!(
        "{label} expected={} observed={} coverage={:.3} gaps={} jumps={}",
        report.expected,
        report.observed,
        report.coverage,
        report.gaps.len(),
        report.jumps.len()
    );
    for gap in report.gaps.iter().take(5) {
        println!(
            "{label} gap after={} before={} minutes={}",
            format_rfc3339(gap.after),
            format_rfc3339(gap.before),
            gap.minutes
        );
    }
}

pub(crate) fn residual_stats(residuals_m: &[f64]) -> Option<ResidualStats> {
    if residuals_m.is_empty() {
        return None;
    }
    let samples = residuals_m.len();
    let bias_m = residuals_m.iter().sum::<f64>() / samples as f64;
    let mut absolute = residuals_m
        .iter()
        .map(|residual| residual.abs())
        .collect::<Vec<_>>();
    absolute.sort_by(|left, right| left.total_cmp(right));
    let p95_m = amar_data::percentile(&absolute, 0.95)?;
    Some(ResidualStats {
        samples,
        bias_cm: bias_m * 100.0,
        p95_cm: p95_m * 100.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn at(value: &str) -> DateTime<Utc> {
        match crate::common::parse_rfc3339(value) {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        }
    }

    fn observation(at: DateTime<Utc>, value_m: f64) -> Observation {
        Observation {
            at,
            value_m,
            source: 4,
        }
    }

    #[test]
    fn qc_report_counts_bounds_gaps_jumps_and_coverage() {
        let start = at("2026-01-01T00:00:00Z");
        let end = at("2026-01-01T05:00:00Z");
        let observations = [
            observation(start, 0.0),
            observation(start + Duration::hours(1), 0.1),
            observation(start + Duration::hours(3), 5.0),
            observation(start + Duration::hours(4), 8.0),
        ];

        let report = qc_report(&observations, start, end);

        assert_eq!(report.expected, 5);
        assert_eq!(report.observed, 4);
        assert!((report.coverage - 0.8).abs() < f64::EPSILON);
        assert_eq!(report.gaps.len(), 1);
        assert_eq!(report.gaps[0].minutes, 120);
        assert_eq!(report.jumps.len(), 1);
        assert_eq!(report.jumps[0].from, start + Duration::hours(3));
        assert_eq!(report.jumps[0].to, start + Duration::hours(4));
    }

    #[test]
    fn qc_report_ignores_large_level_change_across_gap() {
        let start = at("2026-01-01T00:00:00Z");
        let observations = [
            observation(start, 0.0),
            observation(start + Duration::hours(3), 5.0),
        ];

        let report = qc_report(&observations, start, start + Duration::hours(4));

        assert_eq!(report.gaps.len(), 1);
        assert!(report.jumps.is_empty());
    }

    #[test]
    fn qc_report_clamps_negative_expected_window() {
        let start = at("2026-01-02T00:00:00Z");
        let end = at("2026-01-01T00:00:00Z");

        let report = qc_report(&[], start, end);

        assert_eq!(report.expected, 0);
        assert_eq!(report.coverage, 0.0);
    }
}
