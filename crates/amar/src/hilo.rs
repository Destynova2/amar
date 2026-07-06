use crate::{
    CliError, HILO_P95_HEIGHT_LIMIT_M, HILO_P95_TIME_LIMIT_MIN, ValidateArgs, hilo_files,
    hilo_window_label,
};
use amar_core::{UtcDateTime, extrema_between};
use amar_data::{LoadedStation, OfficialExtremum, load_official_hilo_predictions, percentile};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn validate(args: ValidateArgs) -> Result<(), CliError> {
    let data = amar_data::load_pack_from_path(&args.pack)?;
    let mut failures = Vec::new();
    let mut sample_failures = Vec::new();

    for station in data.stations() {
        let report = validate_station(station, &args.fixtures)?;
        print_station_report(&report);
        failures.extend(report.failures);
        sample_failures.extend(report.sample_failures);
    }

    if data.stations().is_empty() {
        println!("no stations validated");
    }
    if !sample_failures.is_empty() {
        return Err(CliError::ValidationSamples {
            failures: sample_failures.join("\n"),
        });
    }
    if !failures.is_empty() {
        return Err(CliError::HiloThreshold {
            failures: failures.join("\n"),
        });
    }
    Ok(())
}

struct StationHiloReport {
    station_id: String,
    name: String,
    method: &'static str,
    stats: Option<HiloStats>,
    window_summaries: BTreeMap<String, HiloStats>,
    failures: Vec<String>,
    sample_failures: Vec<String>,
}

struct FixtureHiloReport {
    label: String,
    time_errors_min: Vec<f64>,
    height_errors_m: Vec<f64>,
    sample_failures: Vec<String>,
}

#[derive(Clone, Copy)]
struct HiloStats {
    samples: usize,
    p50_dt_min: f64,
    p95_dt_min: f64,
    max_dt_min: f64,
    p50_dh_m: f64,
    p95_dh_m: f64,
    max_dh_m: f64,
}

fn validate_station(
    station: &LoadedStation,
    fixtures: &Path,
) -> Result<StationHiloReport, CliError> {
    let station_dir = fixtures.join(&station.pack().provider_station_id);
    let mut time_errors_min = Vec::new();
    let mut height_errors_m = Vec::new();
    let mut sample_failures = Vec::new();
    let mut window_summaries = BTreeMap::new();

    for hilo_path in hilo_files(&station_dir)? {
        let fixture = validate_fixture(station, &hilo_path)?;
        time_errors_min.extend(fixture.time_errors_min.iter().copied());
        height_errors_m.extend(fixture.height_errors_m.iter().copied());
        if let Some(window_stats) = hilo_stats(&fixture.time_errors_min, &fixture.height_errors_m) {
            window_summaries.insert(fixture.label, window_stats);
        }
        sample_failures.extend(fixture.sample_failures);
    }

    let stats = hilo_stats(&time_errors_min, &height_errors_m);
    let mut failures = Vec::new();
    if let Some(stats) = stats {
        if stats.p95_dt_min > HILO_P95_TIME_LIMIT_MIN || stats.p95_dh_m > HILO_P95_HEIGHT_LIMIT_M {
            failures.push(format!(
                "{} all p95_dt_min={:.2} p95_dh_cm={:.1}",
                station.pack().station_id,
                stats.p95_dt_min,
                stats.p95_dh_m * 100.0
            ));
        }
    } else {
        sample_failures.push(format!("{} hilo_samples=0", station.pack().station_id));
    }

    for (window, window_stats) in &window_summaries {
        if window_stats.p95_dt_min > HILO_P95_TIME_LIMIT_MIN
            || window_stats.p95_dh_m > HILO_P95_HEIGHT_LIMIT_M
        {
            failures.push(format!(
                "{} {} p95_dt_min={:.2} p95_dh_cm={:.1}",
                station.pack().station_id,
                window,
                window_stats.p95_dt_min,
                window_stats.p95_dh_m * 100.0
            ));
        }
    }

    Ok(StationHiloReport {
        station_id: station.pack().station_id.clone(),
        name: station.pack().name.clone(),
        method: station.model().method().as_str(),
        stats,
        window_summaries,
        failures,
        sample_failures,
    })
}

fn validate_fixture(
    station: &LoadedStation,
    hilo_path: &Path,
) -> Result<FixtureHiloReport, CliError> {
    let label = hilo_window_label(hilo_path);
    let official = load_official_hilo_predictions(hilo_path)?;
    let Some((from, to)) = official_time_bounds(&official) else {
        return Ok(FixtureHiloReport {
            label: label.clone(),
            time_errors_min: Vec::new(),
            height_errors_m: Vec::new(),
            sample_failures: vec![format!("{} {label} samples=0", station.pack().station_id)],
        });
    };
    let predicted = extrema_between(
        station.model(),
        from.add_seconds(-12 * 60 * 60),
        to.add_seconds(12 * 60 * 60),
    );
    let mut time_errors_min = Vec::new();
    let mut height_errors_m = Vec::new();
    let mut sample_failures = Vec::new();

    for official_extremum in official {
        let Some(predicted_extremum) = closest_extremum(&predicted, official_extremum) else {
            sample_failures.push(format!(
                "{} {} missing predicted {:?}",
                station.pack().station_id,
                label,
                official_extremum.kind
            ));
            continue;
        };
        let dt_min = predicted_extremum
            .at()
            .seconds_since(official_extremum.at)
            .abs() as f64
            / 60.0;
        let dh_m =
            (predicted_extremum.height().as_meters() - official_extremum.height.as_meters()).abs();
        time_errors_min.push(dt_min);
        height_errors_m.push(dh_m);
    }

    Ok(FixtureHiloReport {
        label,
        time_errors_min,
        height_errors_m,
        sample_failures,
    })
}

fn print_station_report(report: &StationHiloReport) {
    let Some(stats) = report.stats else {
        println!(
            "{} {} method={} hilo_samples=0 p50_dt_min=NA p95_dt_min=NA max_dt_min=NA p50_dh_cm=NA p95_dh_cm=NA max_dh_cm=NA",
            report.station_id, report.name, report.method,
        );
        return;
    };

    println!(
        "{} {} method={} hilo_samples={} p50_dt_min={:.2} p95_dt_min={:.2} max_dt_min={:.2} p50_dh_cm={:.1} p95_dh_cm={:.1} max_dh_cm={:.1}",
        report.station_id,
        report.name,
        report.method,
        stats.samples,
        stats.p50_dt_min,
        stats.p95_dt_min,
        stats.max_dt_min,
        stats.p50_dh_m * 100.0,
        stats.p95_dh_m * 100.0,
        stats.max_dh_m * 100.0
    );
    for (window, window_stats) in &report.window_summaries {
        println!(
            "{} {} window={} hilo_samples={} p50_dt_min={:.2} p95_dt_min={:.2} max_dt_min={:.2} p50_dh_cm={:.1} p95_dh_cm={:.1} max_dh_cm={:.1}",
            report.station_id,
            report.name,
            window,
            window_stats.samples,
            window_stats.p50_dt_min,
            window_stats.p95_dt_min,
            window_stats.max_dt_min,
            window_stats.p50_dh_m * 100.0,
            window_stats.p95_dh_m * 100.0,
            window_stats.max_dh_m * 100.0
        );
    }
}

fn official_time_bounds(official: &[OfficialExtremum]) -> Option<(UtcDateTime, UtcDateTime)> {
    let first = official.first()?.at;
    let mut from = first;
    let mut to = first;
    for extremum in official {
        from = from.min(extremum.at);
        to = to.max(extremum.at);
    }
    Some((from, to))
}

fn closest_extremum(
    predicted: &[amar_core::TideExtremum],
    official: OfficialExtremum,
) -> Option<amar_core::TideExtremum> {
    predicted
        .iter()
        .copied()
        .filter(|extremum| extremum.kind() == official.kind)
        .min_by_key(|extremum| extremum.at().seconds_since(official.at).abs())
}

fn hilo_stats(time_errors_min: &[f64], height_errors_m: &[f64]) -> Option<HiloStats> {
    if time_errors_min.is_empty() || time_errors_min.len() != height_errors_m.len() {
        return None;
    }
    let mut sorted_time = time_errors_min.to_vec();
    sorted_time.sort_by(|left, right| left.total_cmp(right));
    let mut sorted_height = height_errors_m.to_vec();
    sorted_height.sort_by(|left, right| left.total_cmp(right));
    Some(HiloStats {
        samples: sorted_time.len(),
        p50_dt_min: percentile(&sorted_time, 0.50)?,
        p95_dt_min: percentile(&sorted_time, 0.95)?,
        max_dt_min: sorted_time.last().copied().unwrap_or(0.0),
        p50_dh_m: percentile(&sorted_height, 0.50)?,
        p95_dh_m: percentile(&sorted_height, 0.95)?,
        max_dh_m: sorted_height.last().copied().unwrap_or(0.0),
    })
}
