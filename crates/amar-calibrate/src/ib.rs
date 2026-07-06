use crate::{CalError, DiagnoseIbArgs, parse_rfc3339};
use amar_core::{UtcDateTime, predict_height};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const STANDARD_PRESSURE_HPA: f64 = 1013.25;
const IB_CM_PER_HPA: f64 = -0.9933;

#[derive(Debug, Deserialize)]
struct BrestBenchmark {
    samples: Vec<BrestBenchmarkSample>,
}

#[derive(Debug, Deserialize)]
struct BrestBenchmarkSample {
    timestamp: String,
    observed_m: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoPressure {
    hourly: OpenMeteoHourly,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoHourly {
    time: Vec<String>,
    surface_pressure: Vec<Option<f64>>,
}

#[derive(Clone, Copy)]
struct IbSample {
    residual_cm: f64,
    ib_cm: f64,
}

pub(crate) fn diagnose_ib(args: DiagnoseIbArgs) -> Result<(), CalError> {
    let data = amar_data::load_pack_from_path(&args.pack)?;
    let station = data
        .stations()
        .iter()
        .find(|station| station.pack().station_id == args.station_id)
        .ok_or_else(|| CalError::MissingStation(args.station_id.clone()))?;
    let benchmark = read_json::<BrestBenchmark>(&args.benchmark)?;
    let pressure = read_pressure(&args.pressure)?;
    let mut samples = Vec::new();
    let mut missing_pressure = 0_usize;
    for sample in &benchmark.samples {
        let Some(observed_m) = sample.observed_m else {
            continue;
        };
        let at = parse_rfc3339(&sample.timestamp)?;
        let Some(pressure_hpa) = pressure.get(&at).copied() else {
            missing_pressure += 1;
            continue;
        };
        let predicted = predict_height(station.model(), UtcDateTime::from_utc(at))
            .height()
            .as_meters();
        let residual_cm = (observed_m - predicted) * 100.0;
        let ib_cm = IB_CM_PER_HPA * (pressure_hpa - STANDARD_PRESSURE_HPA);
        samples.push(IbSample { residual_cm, ib_cm });
    }

    if samples.is_empty() {
        return Err(CalError::EmptyObservations("diagnose-ib".to_string()));
    }
    let residuals = samples
        .iter()
        .map(|sample| sample.residual_cm)
        .collect::<Vec<_>>();
    let ib = samples
        .iter()
        .map(|sample| sample.ib_cm)
        .collect::<Vec<_>>();
    let after = samples
        .iter()
        .map(|sample| sample.residual_cm - sample.ib_cm)
        .collect::<Vec<_>>();
    let corr = correlation(&residuals, &ib);
    let variance_before = variance(&residuals);
    let variance_after = variance(&after);
    let fixed_variance_delta_percent = if variance_before > 0.0 {
        (1.0 - variance_after / variance_before) * 100.0
    } else {
        0.0
    };
    println!(
        "samples,missing_pressure,corr,residual_r2_percent,fixed_ib_variance_delta_percent,rms_before_cm,rms_after_ib_cm,bias_before_cm,bias_after_ib_cm"
    );
    println!(
        "{},{},{:.3},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}",
        samples.len(),
        missing_pressure,
        corr,
        corr * corr * 100.0,
        fixed_variance_delta_percent,
        rms(&residuals),
        rms(&after),
        mean(&residuals),
        mean(&after),
    );
    Ok(())
}

fn read_pressure(path: &Path) -> Result<BTreeMap<DateTime<Utc>, f64>, CalError> {
    let response = read_json::<OpenMeteoPressure>(path)?;
    let mut values = BTreeMap::new();
    for (time, pressure) in response
        .hourly
        .time
        .into_iter()
        .zip(response.hourly.surface_pressure)
    {
        let Some(pressure) = pressure else {
            continue;
        };
        let at = NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M")
            .map(|date| date.and_utc())
            .map_err(|_| CalError::InvalidTimestamp(time.clone()))?;
        values.insert(at, pressure);
    }
    Ok(values)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, CalError> {
    let data = fs::read_to_string(path).map_err(|source| CalError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&data).map_err(CalError::from)
}

fn correlation(left: &[f64], right: &[f64]) -> f64 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let left_mean = mean(left);
    let right_mean = mean(right);
    let mut covariance = 0.0;
    let mut left_variance = 0.0;
    let mut right_variance = 0.0;
    for (left_value, right_value) in left.iter().zip(right) {
        let left_centered = left_value - left_mean;
        let right_centered = right_value - right_mean;
        covariance += left_centered * right_centered;
        left_variance += left_centered * left_centered;
        right_variance += right_centered * right_centered;
    }
    let denominator = (left_variance * right_variance).sqrt();
    if denominator > 0.0 {
        covariance / denominator
    } else {
        0.0
    }
}

fn variance(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let values_mean = mean(values);
    values
        .iter()
        .map(|value| {
            let centered = value - values_mean;
            centered * centered
        })
        .sum::<f64>()
        / values.len() as f64
}

fn rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt()
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}
