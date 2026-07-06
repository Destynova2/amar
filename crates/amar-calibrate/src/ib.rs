use crate::DiagnoseIbArgs;
use crate::common::{CalError, parse_rfc3339};
use amar_core::{UtcDateTime, predict_height};
use amar_pack::BrestBenchmark;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const STANDARD_PRESSURE_HPA: f64 = 1013.25;
const IB_CM_PER_HPA: f64 = -0.9933;

#[derive(Debug, Deserialize)]
struct OpenMeteoPressure {
    timezone: String,
    utc_offset_seconds: i32,
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
    if response.timezone != "GMT" || response.utc_offset_seconds != 0 {
        return Err(CalError::OpenMeteoTimezone {
            timezone: response.timezone,
            utc_offset_seconds: response.utc_offset_seconds,
        });
    }
    let time_len = response.hourly.time.len();
    let pressure_len = response.hourly.surface_pressure.len();
    if time_len != pressure_len {
        return Err(CalError::OpenMeteoHourlyLength {
            time_len,
            pressure_len,
        });
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        }
    }

    fn pressure_file(name: &str, body: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "amar-open-meteo-{name}-{}-{nanos}.json",
            std::process::id()
        ));
        must(fs::write(&path, body));
        path
    }

    fn read_pressure_body(body: &str) -> Result<BTreeMap<DateTime<Utc>, f64>, CalError> {
        let path = pressure_file("pressure", body);
        let result = read_pressure(&path);
        let _ = fs::remove_file(path);
        result
    }

    #[test]
    fn read_pressure_rejects_desynchronized_hourly_arrays() {
        let result = read_pressure_body(
            r#"{"timezone":"GMT","utc_offset_seconds":0,"hourly":{"time":["2026-04-01T00:00","2026-04-01T01:00"],"surface_pressure":[1013.2]}}"#,
        );

        match result {
            Err(CalError::OpenMeteoHourlyLength {
                time_len,
                pressure_len,
            }) => {
                assert_eq!(time_len, 2);
                assert_eq!(pressure_len, 1);
            }
            other => panic!("expected hourly length error, got {other:?}"),
        }
    }

    #[test]
    fn read_pressure_requires_gmt_timezone() {
        let result = read_pressure_body(
            r#"{"timezone":"Europe/Paris","utc_offset_seconds":3600,"hourly":{"time":["2026-04-01T00:00"],"surface_pressure":[1013.2]}}"#,
        );

        match result {
            Err(CalError::OpenMeteoTimezone {
                timezone,
                utc_offset_seconds,
            }) => {
                assert_eq!(timezone, "Europe/Paris");
                assert_eq!(utc_offset_seconds, 3600);
            }
            other => panic!("expected timezone error, got {other:?}"),
        }
    }

    #[test]
    fn read_pressure_parses_gmt_hourly_samples() {
        let values = must(read_pressure_body(
            r#"{"timezone":"GMT","utc_offset_seconds":0,"hourly":{"time":["2026-04-01T00:00","2026-04-01T01:00"],"surface_pressure":[1013.2,null]}}"#,
        ));

        let at = must(DateTime::parse_from_rfc3339("2026-04-01T00:00:00Z")).with_timezone(&Utc);
        assert_eq!(values.len(), 1);
        assert_eq!(values.get(&at).copied(), Some(1013.2));
    }
}
