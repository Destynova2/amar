use crate::DiagnoseArgs;
use crate::common::{CalError, Observation, format_rfc3339, parse_rfc3339};
use crate::pack_out::read_observations_csv;
use crate::solve::M22_RAYLEIGH37;
use amar_core::{UtcDateTime, predict_height};
use amar_pack::{BrestBenchmark, BrestBenchmarkSample};
use chrono::{DateTime, Utc};
use nalgebra::{DMatrix, DVector};
use std::fs;

const LOW_PASS_RADIUS_HOURS: usize = 24;
const TIDAL_DECISION_FLOOR_CM: f64 = 3.0;

#[derive(Clone, Copy)]
struct ResidualSample {
    at: DateTime<Utc>,
    residual_m: f64,
}

#[derive(Clone, Copy)]
struct SpectralSummary {
    samples: usize,
    rms_cm: f64,
    bias_cm: f64,
    low_pass_cm: f64,
    diurnal_cm: f64,
    semidiurnal_cm: f64,
    compound_cm: f64,
    coherent_tidal_cm: f64,
}

#[derive(Clone, Copy)]
enum HarmonicBand {
    Low,
    Diurnal,
    Semidiurnal,
    Compound,
    Other,
}

pub(crate) fn diagnose(args: DiagnoseArgs) -> Result<(), CalError> {
    let data = amar_data::load_pack_from_path(&args.pack)?;
    let station = data
        .stations()
        .iter()
        .find(|station| station.pack().station_id == args.station_id)
        .ok_or_else(|| CalError::MissingStation(args.station_id.clone()))?;
    let calibration_period = station.pack().calibration_period.as_ref().ok_or_else(|| {
        CalError::MissingStationPeriod {
            station_id: args.station_id.clone(),
            field: "calibration_period",
        }
    })?;
    let calibration_start = parse_rfc3339(&calibration_period.start)?;
    let calibration_end = parse_rfc3339(&calibration_period.end)?;
    let observations = read_observations_csv(&args.observations)?;
    let calibration_residuals = residuals_from_observations(
        station.model(),
        observations
            .iter()
            .copied()
            .filter(|sample| sample.at >= calibration_start && sample.at < calibration_end),
    );
    let benchmark = read_benchmark(&args.benchmark)?;
    let benchmark_start = parse_rfc3339(&benchmark.validation_period.start)?;
    let benchmark_end = parse_rfc3339(&benchmark.validation_period.end)?;
    let benchmark_residuals = residuals_from_benchmark(station.model(), &benchmark.samples)?;

    let calibration = spectral_summary(&calibration_residuals)?;
    let validation = spectral_summary(&benchmark_residuals)?;
    print_summary(
        "calibration",
        calibration_start,
        calibration_end,
        calibration,
    );
    print_summary("benchmark", benchmark_start, benchmark_end, validation);
    println!(
        "tidal_decision_floor_cm={:.1} calibration_tidal_cm={:.2} benchmark_tidal_cm={:.2}",
        TIDAL_DECISION_FLOOR_CM, calibration.coherent_tidal_cm, validation.coherent_tidal_cm,
    );
    if calibration.coherent_tidal_cm < TIDAL_DECISION_FLOOR_CM
        && validation.coherent_tidal_cm < TIDAL_DECISION_FLOOR_CM
    {
        println!("decision=stop: coherent tidal residual is below the diagnostic floor");
    } else {
        println!("decision=continue: coherent tidal residual is above the diagnostic floor");
    }
    Ok(())
}

fn residuals_from_observations(
    model: &amar_core::TideModel,
    observations: impl Iterator<Item = Observation>,
) -> Vec<ResidualSample> {
    observations
        .map(|observation| {
            let at = UtcDateTime::from_utc(observation.at);
            let predicted = predict_height(model, at).height().as_meters();
            ResidualSample {
                at: observation.at,
                residual_m: observation.value_m - predicted,
            }
        })
        .collect()
}

fn residuals_from_benchmark(
    model: &amar_core::TideModel,
    samples: &[BrestBenchmarkSample],
) -> Result<Vec<ResidualSample>, CalError> {
    let mut residuals = Vec::new();
    for sample in samples {
        let Some(observed_m) = sample.observed_m else {
            continue;
        };
        let at = parse_rfc3339(&sample.timestamp)?;
        let predicted = predict_height(model, UtcDateTime::from_utc(at))
            .height()
            .as_meters();
        residuals.push(ResidualSample {
            at,
            residual_m: observed_m - predicted,
        });
    }
    Ok(residuals)
}

fn spectral_summary(samples: &[ResidualSample]) -> Result<SpectralSummary, CalError> {
    if samples.is_empty() {
        return Err(CalError::EmptyObservations("diagnose".to_string()));
    }
    let values = samples
        .iter()
        .map(|sample| sample.residual_m)
        .collect::<Vec<_>>();
    let samples_len = values.len();
    let bias_m = values.iter().sum::<f64>() / samples_len as f64;
    let rms_m = rms(&values);
    let low_pass_cm = moving_low_pass_rms(&values, bias_m) * 100.0;
    let harmonic = harmonic_summary(samples)?;
    Ok(SpectralSummary {
        samples: samples_len,
        rms_cm: rms_m * 100.0,
        bias_cm: bias_m * 100.0,
        low_pass_cm,
        diurnal_cm: harmonic.diurnal_cm,
        semidiurnal_cm: harmonic.semidiurnal_cm,
        compound_cm: harmonic.compound_cm,
        coherent_tidal_cm: harmonic.coherent_tidal_cm,
    })
}

#[derive(Clone, Copy)]
struct HarmonicSummary {
    diurnal_cm: f64,
    semidiurnal_cm: f64,
    compound_cm: f64,
    coherent_tidal_cm: f64,
}

fn harmonic_summary(samples: &[ResidualSample]) -> Result<HarmonicSummary, CalError> {
    let columns = 1 + M22_RAYLEIGH37.len() * 2;
    let mut matrix = Vec::with_capacity(samples.len() * columns);
    let mut values = Vec::with_capacity(samples.len());
    let first = samples[0].at;
    for sample in samples {
        let days = (sample.at - first).num_seconds() as f64 / 86_400.0;
        matrix.push(1.0);
        for spec in M22_RAYLEIGH37 {
            let cycles_per_day = spec.speed_deg_per_hour / 15.0;
            let radians = std::f64::consts::TAU * cycles_per_day * days;
            matrix.push(radians.cos());
            matrix.push(radians.sin());
        }
        values.push(sample.residual_m);
    }

    let a = DMatrix::from_row_slice(samples.len(), columns, &matrix);
    let y = DVector::from_row_slice(&values);
    let solution = a
        .svd(true, true)
        .solve(&y, 1.0e-10)
        .map_err(|_| CalError::SolveFailed)?;

    let mut diurnal = Vec::with_capacity(samples.len());
    let mut semidiurnal = Vec::with_capacity(samples.len());
    let mut compound = Vec::with_capacity(samples.len());
    let mut coherent_tidal = Vec::with_capacity(samples.len());
    for sample in samples {
        let days = (sample.at - first).num_seconds() as f64 / 86_400.0;
        let mut sample_diurnal = 0.0;
        let mut sample_semidiurnal = 0.0;
        let mut sample_compound = 0.0;
        for (index, spec) in M22_RAYLEIGH37.iter().enumerate() {
            let cycles_per_day = spec.speed_deg_per_hour / 15.0;
            let radians = std::f64::consts::TAU * cycles_per_day * days;
            let component = solution[1 + index * 2] * radians.cos()
                + solution[1 + index * 2 + 1] * radians.sin();
            match harmonic_band(cycles_per_day) {
                HarmonicBand::Low | HarmonicBand::Other => {}
                HarmonicBand::Diurnal => sample_diurnal += component,
                HarmonicBand::Semidiurnal => sample_semidiurnal += component,
                HarmonicBand::Compound => sample_compound += component,
            }
        }
        let tidal = sample_diurnal + sample_semidiurnal + sample_compound;
        diurnal.push(sample_diurnal);
        semidiurnal.push(sample_semidiurnal);
        compound.push(sample_compound);
        coherent_tidal.push(tidal);
    }

    Ok(HarmonicSummary {
        diurnal_cm: rms(&diurnal) * 100.0,
        semidiurnal_cm: rms(&semidiurnal) * 100.0,
        compound_cm: rms(&compound) * 100.0,
        coherent_tidal_cm: rms(&coherent_tidal) * 100.0,
    })
}

fn harmonic_band(cycles_per_day: f64) -> HarmonicBand {
    if cycles_per_day < 0.5 {
        HarmonicBand::Low
    } else if (0.75..1.25).contains(&cycles_per_day) {
        HarmonicBand::Diurnal
    } else if (1.75..2.25).contains(&cycles_per_day) {
        HarmonicBand::Semidiurnal
    } else if cycles_per_day >= 2.5 {
        HarmonicBand::Compound
    } else {
        HarmonicBand::Other
    }
}

fn moving_low_pass_rms(values: &[f64], bias_m: f64) -> f64 {
    let centered = values
        .iter()
        .map(|value| value - bias_m)
        .collect::<Vec<_>>();
    let mut prefix = Vec::with_capacity(centered.len() + 1);
    prefix.push(0.0);
    for value in &centered {
        let previous = prefix.last().copied().unwrap_or(0.0);
        prefix.push(previous + value);
    }
    let mut smoothed = Vec::with_capacity(centered.len());
    for index in 0..centered.len() {
        let left = index.saturating_sub(LOW_PASS_RADIUS_HOURS);
        let right = (index + LOW_PASS_RADIUS_HOURS + 1).min(centered.len());
        let count = (right - left) as f64;
        smoothed.push((prefix[right] - prefix[left]) / count);
    }
    rms(&smoothed)
}

fn rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt()
}

fn print_summary(label: &str, start: DateTime<Utc>, end: DateTime<Utc>, summary: SpectralSummary) {
    println!(
        "| {label} | {} | {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |",
        format_rfc3339(start),
        format_rfc3339(end),
        summary.samples,
        summary.rms_cm,
        summary.bias_cm,
        summary.low_pass_cm,
        summary.diurnal_cm,
        summary.semidiurnal_cm,
        summary.compound_cm,
        summary.coherent_tidal_cm,
    );
}

fn read_benchmark(path: &std::path::Path) -> Result<BrestBenchmark, CalError> {
    let data = fs::read_to_string(path).map_err(|source| CalError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&data).map_err(CalError::from)
}
