use crate::common::{
    CalError, Observation, REFMAR_BASE, VALIDATED_HOURLY_SOURCE, format_rfc3339, parse_rfc3339,
};
use crate::pack_out::{self, BenchmarkBuildInput};
use crate::qc::{self, ResidualStats};
use amar_core::{UtcDateTime, predict_height};
use amar_data::{LoadedStation, load_packs_from_paths};
use amar_pack::TideBenchmark;
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, TimeZone, Utc};
use clap::Args;
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration as StdDuration;

const DEFAULT_BREST_PACK: &str = "data/packs/amar-data-brest-experimental.json";
const DEFAULT_FRANCE_PACK: &str = "data/packs/amar-data-france-experimental.json";
const DEFAULT_BREST_BENCHMARK: &str = "fixtures/refmar/benchmark_brest_v1.json";
const DEFAULT_BENCHMARKS_DIR: &str = "fixtures/refmar/benchmarks";
const DEFAULT_BREST_DECENNIAL_BENCHMARK: &str = "fixtures/refmar/benchmark_brest_decennial_v1.json";
const DEFAULT_CACHE_DIR: &str = "target/refmar-cache";
const DEFAULT_START: &str = "2016-01-01T00:00:00Z";
const DEFAULT_END: &str = "2026-07-01T00:00:00Z";
const DEFAULT_GENERATED_AT: &str = "2026-07-08-v0.8-decennial";
const MIN_THROTTLE_MS: u64 = 500;
const DEFAULT_THROTTLE_MS: u64 = 600;

#[derive(Debug, Args)]
pub(crate) struct ValidateHistoryArgs {
    #[arg(long = "pack")]
    pack: Vec<PathBuf>,
    #[arg(long, default_value = DEFAULT_START)]
    start: String,
    #[arg(long, default_value = DEFAULT_END)]
    end: String,
    #[arg(long = "station")]
    stations: Vec<String>,
    #[arg(long, default_value = DEFAULT_CACHE_DIR)]
    cache_dir: PathBuf,
    #[arg(long, default_value = DEFAULT_BREST_BENCHMARK)]
    brest_benchmark: PathBuf,
    #[arg(long = "benchmark-dir", default_value = DEFAULT_BENCHMARKS_DIR)]
    benchmark_dir: PathBuf,
    #[arg(
        long = "brest-decennial-out",
        default_value = DEFAULT_BREST_DECENNIAL_BENCHMARK
    )]
    brest_decennial_out: PathBuf,
    #[arg(long, default_value = DEFAULT_GENERATED_AT)]
    generated_at: String,
    #[arg(long, default_value_t = VALIDATED_HOURLY_SOURCE)]
    source: u8,
    #[arg(long, default_value_t = DEFAULT_THROTTLE_MS)]
    throttle_ms: u64,
    #[arg(long = "alert-rms-factor", default_value_t = 2.0)]
    alert_rms_factor: f64,
    #[arg(long = "fail-on-alert")]
    fail_on_alert: bool,
    #[arg(long = "report-out")]
    report_out: Option<PathBuf>,
}

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

struct PoliteClient {
    client: Client,
    throttle: StdDuration,
}

struct HistoryReport {
    station_id: String,
    name: String,
    provider_station_id: String,
    observations: Vec<Observation>,
    yearly: Vec<YearReport>,
}

struct YearReport {
    year: i32,
    samples: usize,
    coverage: f64,
    stats: Option<ResidualStats>,
    alert: bool,
}

impl PoliteClient {
    fn new(throttle_ms: u64) -> Result<Self, CalError> {
        if throttle_ms < MIN_THROTTLE_MS {
            return Err(CalError::QualityGate(format!(
                "REFMAR throttle must be at least {MIN_THROTTLE_MS} ms"
            )));
        }
        Ok(Self {
            client: Client::builder()
                .user_agent("amar-calibrate/0.1")
                .timeout(StdDuration::from_secs(20))
                .build()?,
            throttle: StdDuration::from_millis(throttle_ms),
        })
    }

    fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        query: &[(String, String)],
        cache_path: &Path,
    ) -> Result<T, CalError> {
        let data = if cache_path.exists() {
            fs::read_to_string(cache_path).map_err(|source| CalError::Io {
                path: cache_path.to_path_buf(),
                source,
            })?
        } else {
            std::thread::sleep(self.throttle);
            let mut request = self.client.get(url);
            if !query.is_empty() {
                request = request.query(query);
            }
            let value = request
                .send()?
                .error_for_status()?
                .json::<serde_json::Value>()?;
            let data = format!("{}\n", serde_json::to_string_pretty(&value)?);
            pack_out::write_string(cache_path, &data)?;
            data
        };
        serde_json::from_str(&data).map_err(CalError::from)
    }
}

pub(crate) fn validate_history(args: ValidateHistoryArgs) -> Result<(), CalError> {
    let start = parse_rfc3339(&args.start)?;
    let end = parse_rfc3339(&args.end)?;
    if start >= end {
        return Err(CalError::InvalidTimestamp(format!(
            "{} must be before {}",
            args.start, args.end
        )));
    }
    if args.alert_rms_factor <= 0.0 || !args.alert_rms_factor.is_finite() {
        return Err(CalError::QualityGate(
            "alert-rms-factor must be finite and positive".to_string(),
        ));
    }

    let data = load_packs_from_paths(&effective_pack_paths(&args.pack))?;
    let benchmark_rms = benchmark_rms_by_station(&args, data.stations())?;
    let selected = args
        .stations
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let stations = data
        .stations()
        .iter()
        .filter(|station| station.pack().station_id.starts_with("refmar:"))
        .filter(|station| {
            selected.is_empty()
                || selected.contains(station.pack().station_id.as_str())
                || selected.contains(station.pack().provider_station_id.as_str())
        })
        .collect::<Vec<_>>();
    if stations.is_empty() {
        return Err(CalError::QualityGate(
            "no REFMAR station selected from loaded packs".to_string(),
        ));
    }

    let client = PoliteClient::new(args.throttle_ms)?;
    let mut reports = Vec::new();
    let mut alerts = Vec::new();
    for station in stations {
        eprintln!(
            "history refmar:{} {}",
            station.pack().provider_station_id,
            station.pack().name
        );
        let observations = fetch_observations(
            &client,
            &args.cache_dir,
            &station.pack().provider_station_id,
            args.source,
            start,
            end,
        )?;
        let report = history_report(
            station,
            observations,
            start,
            end,
            benchmark_rms.get(&station.pack().station_id).copied(),
            args.alert_rms_factor,
        )?;
        for yearly in &report.yearly {
            if yearly.alert {
                alerts.push(format!(
                    "{} {} rms_cm={:.1} benchmark_rms_cm={:.1}",
                    report.station_id,
                    yearly.year,
                    yearly.stats.map(|stats| stats.rms_cm).unwrap_or(f64::NAN),
                    benchmark_rms
                        .get(&report.station_id)
                        .copied()
                        .unwrap_or(f64::NAN)
                ));
            }
        }
        if report.station_id == "refmar:3" {
            write_brest_decennial_benchmark(&args, &report, start, end)?;
        }
        reports.push(report);
    }

    let report_csv = history_reports_csv(&reports);
    print!("{report_csv}");
    if let Some(report_out) = &args.report_out {
        pack_out::write_string(report_out, &report_csv)?;
    }
    if !alerts.is_empty() && args.fail_on_alert {
        return Err(CalError::QualityGate(format!(
            "historical RMS alert exceeded:\n{}",
            alerts.join("\n")
        )));
    }
    for alert in alerts {
        eprintln!("historical RMS alert: {alert}");
    }
    Ok(())
}

fn effective_pack_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    if !paths.is_empty() {
        return paths.to_vec();
    }
    vec![
        PathBuf::from(DEFAULT_BREST_PACK),
        PathBuf::from(DEFAULT_FRANCE_PACK),
    ]
}

fn benchmark_rms_by_station(
    args: &ValidateHistoryArgs,
    stations: &[LoadedStation],
) -> Result<BTreeMap<String, f64>, CalError> {
    let mut paths = Vec::new();
    if args.brest_benchmark.exists() {
        paths.push(args.brest_benchmark.clone());
    }
    if args.benchmark_dir.exists() {
        for entry in fs::read_dir(&args.benchmark_dir).map_err(|source| CalError::Io {
            path: args.benchmark_dir.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| CalError::Io {
                path: args.benchmark_dir.clone(),
                source,
            })?;
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    paths.dedup();

    let mut output = BTreeMap::new();
    for path in paths {
        let data = fs::read_to_string(&path).map_err(|source| CalError::Io {
            path: path.clone(),
            source,
        })?;
        let benchmark = serde_json::from_str::<TideBenchmark>(&data)?;
        let Some(station) = stations
            .iter()
            .find(|station| station.pack().station_id == benchmark.station_id)
        else {
            continue;
        };
        let residuals = benchmark
            .samples
            .iter()
            .filter_map(|sample| {
                let observed_m = sample.observed_m?;
                let at = UtcDateTime::parse_rfc3339(&sample.timestamp).ok()?;
                let predicted = predict_height(station.model(), at).height().as_meters();
                Some(observed_m - predicted)
            })
            .collect::<Vec<_>>();
        if let Some(stats) = qc::residual_stats(&residuals) {
            output.insert(benchmark.station_id, stats.rms_cm);
        }
    }
    Ok(output)
}

fn history_report(
    station: &LoadedStation,
    observations: Vec<Observation>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    benchmark_rms_cm: Option<f64>,
    alert_rms_factor: f64,
) -> Result<HistoryReport, CalError> {
    let mut yearly = Vec::new();
    let calibration_start = station
        .pack()
        .valid_from
        .as_deref()
        .and_then(|value| parse_rfc3339(value).ok());
    for year in start.year()..=end.year() {
        let year_start = utc_year_start(year).max(start);
        let year_end = utc_year_start(year + 1).min(end);
        if year_start >= year_end {
            continue;
        }
        let samples = observations
            .iter()
            .copied()
            .filter(|observation| observation.at >= year_start && observation.at < year_end)
            .collect::<Vec<_>>();
        let qc_report = qc::qc_report(&samples, year_start, year_end);
        let residuals = samples
            .iter()
            .map(|observation| {
                let predicted =
                    predict_height(station.model(), UtcDateTime::from_utc(observation.at))
                        .height()
                        .as_meters();
                observation.value_m - predicted
            })
            .collect::<Vec<_>>();
        let stats = qc::residual_stats(&residuals);
        let before_calibration = calibration_start.is_none_or(|boundary| year_start < boundary);
        let alert = before_calibration
            && stats.is_some_and(|stats| {
                benchmark_rms_cm
                    .is_some_and(|benchmark| stats.rms_cm > benchmark * alert_rms_factor)
            });
        yearly.push(YearReport {
            year,
            samples: samples.len(),
            coverage: qc_report.coverage,
            stats,
            alert,
        });
    }
    Ok(HistoryReport {
        station_id: station.pack().station_id.clone(),
        name: station.pack().name.clone(),
        provider_station_id: station.pack().provider_station_id.clone(),
        observations,
        yearly,
    })
}

fn write_brest_decennial_benchmark(
    args: &ValidateHistoryArgs,
    report: &HistoryReport,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(), CalError> {
    let observations_sha256 = observations_sha256(&report.observations);
    let benchmark = pack_out::build_station_benchmark(BenchmarkBuildInput {
        validation_samples: &report.observations,
        validation_start: start,
        validation_end: end,
        observations_sha256: &observations_sha256,
        generated_at: &args.generated_at,
        benchmark_id: "benchmark_brest_decennial_v1",
        station_id: &report.station_id,
        provider_station_id: &report.provider_station_id,
        station_name: &report.name,
        datum: "zero_hydrographique_brest",
    });
    pack_out::write_json(&args.brest_decennial_out, &benchmark)
}

fn observations_sha256(observations: &[Observation]) -> String {
    let mut input = String::new();
    for observation in observations {
        input.push_str(&format!(
            "{},{:.3},{}\n",
            format_rfc3339(observation.at),
            observation.value_m,
            observation.source
        ));
    }
    sha256_hex(input.as_bytes())
}

fn history_reports_csv(reports: &[HistoryReport]) -> String {
    let mut output = String::from("station_id,name,year,N,coverage,rms_cm,bias_cm,p95_cm,alert\n");
    for report in reports {
        for yearly in &report.yearly {
            if let Some(stats) = yearly.stats {
                output.push_str(&format!(
                    "{},{},{},{},{:.3},{:.1},{:.1},{:.1},{}",
                    report.station_id,
                    report.name,
                    yearly.year,
                    yearly.samples,
                    yearly.coverage,
                    stats.rms_cm,
                    stats.bias_cm,
                    stats.p95_cm,
                    yearly.alert
                ));
            } else {
                output.push_str(&format!(
                    "{},{},{},{},{:.3},NA,NA,NA,{}",
                    report.station_id,
                    report.name,
                    yearly.year,
                    yearly.samples,
                    yearly.coverage,
                    yearly.alert
                ));
            }
            output.push('\n');
        }
    }
    output
}

fn fetch_observations(
    client: &PoliteClient,
    cache_dir: &Path,
    shom_id: &str,
    source: u8,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<Observation>, CalError> {
    let mut observations = BTreeMap::new();
    let mut cursor = start;
    while cursor < end {
        let window_end = (cursor + Duration::days(31)).min(end);
        let response = client.get_json::<RefmarObservationResponse>(
            &format!("{REFMAR_BASE}/observation/json/{shom_id}"),
            &[
                ("sources".to_string(), source.to_string()),
                ("dtStart".to_string(), format_refmar_query_time(cursor)),
                ("dtEnd".to_string(), format_refmar_query_time(window_end)),
            ],
            &observation_window_cache_path(cache_dir, shom_id, cursor, window_end),
        )?;
        for raw in response.data {
            if raw.idsource != source {
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
        return Err(CalError::EmptyObservations(shom_id.to_string()));
    }
    Ok(observations.values().copied().collect())
}

fn observation_window_cache_path(
    cache_dir: &Path,
    shom_id: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> PathBuf {
    cache_dir
        .join("stations")
        .join(shom_id)
        .join("observations")
        .join(format!(
            "{}_{}.json",
            cache_time_label(start),
            cache_time_label(end)
        ))
}

fn parse_refmar_timestamp(value: &str) -> Result<DateTime<Utc>, CalError> {
    NaiveDateTime::parse_from_str(value, "%Y/%m/%d %H:%M:%S")
        .map(|date| date.and_utc())
        .map_err(|_| CalError::InvalidTimestamp(value.to_string()))
}

fn format_refmar_query_time(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn cache_time_label(value: DateTime<Utc>) -> String {
    format_rfc3339(value).replace(':', "")
}

fn utc_year_start(year: i32) -> DateTime<Utc> {
    match Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).single() {
        Some(value) => value,
        None => unreachable!("valid UTC year boundary"),
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
