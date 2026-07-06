use crate::FetchRefmarArgs;
use crate::common::{CalError, Observation, REFMAR_BASE, format_rfc3339, parse_rfc3339};
use crate::pack_out::{write_observations_csv, write_string};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct RefmarObservationResponse {
    data: Vec<RefmarObservation>,
}

#[derive(Debug, Deserialize)]
struct RefmarObservation {
    idsource: u8,
    value: f64,
    timestamp: String,
}

pub(crate) fn fetch_refmar(args: FetchRefmarArgs) -> Result<(), CalError> {
    let start = parse_rfc3339(&args.start)?;
    let end = parse_rfc3339(&args.end)?;
    if start >= end {
        return Err(CalError::InvalidTimestamp(format!(
            "{} must be before {}",
            args.start, args.end
        )));
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("amar-calibrate/0.1")
        .build()?;
    let tidegauge_url = format!("{REFMAR_BASE}/service/completetidegauge/{}", args.shom_id);
    let tidegauge_value = client
        .get(&tidegauge_url)
        .send()?
        .error_for_status()?
        .json::<serde_json::Value>()?;
    write_string(
        &args.tidegauge_out,
        &format!("{}\n", serde_json::to_string_pretty(&tidegauge_value)?),
    )?;

    let mut observations = BTreeMap::new();
    let mut cursor = start;
    while cursor < end {
        let window_end = (cursor + Duration::days(31)).min(end);
        let response = client
            .get(format!("{REFMAR_BASE}/observation/json/{}", args.shom_id))
            .query(&[
                ("sources", args.source.to_string()),
                ("dtStart", format_refmar_query_time(cursor)),
                ("dtEnd", format_refmar_query_time(window_end)),
            ])
            .send()?
            .error_for_status()?
            .json::<RefmarObservationResponse>()?;
        for raw in response.data {
            if raw.idsource != args.source {
                continue;
            }
            let at = parse_refmar_timestamp(&raw.timestamp)?;
            if at >= start && at < end {
                observations.insert(
                    at,
                    Observation {
                        at,
                        value_m: raw.value,
                        source: raw.idsource,
                    },
                );
            }
        }
        cursor = window_end;
    }

    if observations.is_empty() {
        return Err(CalError::EmptyObservations(args.shom_id));
    }
    write_observations_csv(&args.out, observations.values().copied())?;
    println!(
        "refmar shom_id={} source={} observations={} start={} end={} out={}",
        args.shom_id,
        args.source,
        observations.len(),
        format_rfc3339(start),
        format_rfc3339(end),
        args.out.display()
    );
    Ok(())
}

fn parse_refmar_timestamp(value: &str) -> Result<DateTime<Utc>, CalError> {
    NaiveDateTime::parse_from_str(value, "%Y/%m/%d %H:%M:%S")
        .map(|date| date.and_utc())
        .map_err(|_| CalError::InvalidTimestamp(value.to_string()))
}

fn format_refmar_query_time(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
