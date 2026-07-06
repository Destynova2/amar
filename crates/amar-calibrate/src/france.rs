use crate::common::{
    CalError, Observation, REFMAR_BASE, VALIDATED_HOURLY_SOURCE, format_rfc3339, parse_rfc3339,
};
use crate::pack_out::{
    self, BenchmarkBuildInput, ObservationCsvMetadata, StationPackBuildInput, TideGauge,
    title_case_station,
};
use crate::qc::{self, QcReport, ResidualStats};
use crate::solve::{self, ConstituentSet};
use amar_core::{UtcDateTime, predict_height};
use amar_pack::{PeriodInfo, SCHEMA_VERSION, StationPack, TideBenchmark, TidePack};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use clap::Args;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration as StdDuration;

const DEFAULT_FRANCE_PACK: &str = "data/packs/amar-data-france-experimental.json";
const DEFAULT_BENCHMARKS_DIR: &str = "fixtures/refmar/benchmarks";
const DEFAULT_MANIFESTS_DIR: &str = "fixtures/refmar/manifests";
const DEFAULT_CACHE_DIR: &str = "target/refmar-cache";
const DEFAULT_GENERATED_AT: &str = "2026-07-06-v0.4-france";
const DEFAULT_START: &str = "2021-01-01T00:00:00Z";
const DEFAULT_VALIDATION_START: &str = "2026-04-01T00:00:00Z";
const DEFAULT_END: &str = "2026-07-01T00:00:00Z";
const MIN_THROTTLE_MS: u64 = 500;
const DEFAULT_THROTTLE_MS: u64 = 600;
const DEFAULT_P95_LIMIT_CM: f64 = 40.0;
const DEFAULT_MIN_RMS_FACTOR: f64 = 2.0;
const FRANCE_MIN_LAT: f64 = 41.0;
const FRANCE_MAX_LAT: f64 = 52.0;
const FRANCE_MIN_LON: f64 = -6.0;
const FRANCE_MAX_LON: f64 = 10.0;

const PRIORITY_STATIONS: [&str; 10] =
    ["410", "4", "13", "37", "34", "160", "152", "54", "2", "111"];

#[derive(Debug, Args)]
pub(crate) struct CalibrateFranceArgs {
    #[arg(long, default_value = DEFAULT_START)]
    start: String,
    #[arg(long, default_value = DEFAULT_VALIDATION_START)]
    validation_start: String,
    #[arg(long, default_value = DEFAULT_END)]
    end: String,
    #[arg(long = "station")]
    stations: Vec<String>,
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long, default_value_t = VALIDATED_HOURLY_SOURCE)]
    source: u8,
    #[arg(long, default_value = DEFAULT_CACHE_DIR)]
    cache_dir: PathBuf,
    #[arg(long, default_value = DEFAULT_FRANCE_PACK)]
    pack_out: PathBuf,
    #[arg(long, default_value = DEFAULT_BENCHMARKS_DIR)]
    benchmarks_dir: PathBuf,
    #[arg(long, default_value = DEFAULT_MANIFESTS_DIR)]
    manifests_dir: PathBuf,
    #[arg(long, default_value = DEFAULT_GENERATED_AT)]
    generated_at: String,
    #[arg(long, default_value_t = DEFAULT_THROTTLE_MS)]
    throttle_ms: u64,
    #[arg(long, default_value_t = DEFAULT_P95_LIMIT_CM)]
    p95_limit_cm: f64,
    #[arg(long, default_value_t = DEFAULT_MIN_RMS_FACTOR)]
    min_rms_factor: f64,
    #[arg(long)]
    include_brest: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CatalogStation {
    shom_id: String,
    name: String,
    longitude: f64,
    latitude: f64,
    state: String,
    reseau: Option<String>,
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

#[derive(Debug)]
struct StationArtifacts {
    station: StationPack,
    benchmark: TideBenchmark,
    manifest: CalibrationManifest,
    decision: DecisionRow,
}

#[derive(Debug)]
struct StationExclusion {
    shom_id: String,
    name: String,
    reason: String,
    metrics: Option<DecisionMetrics>,
    manifest: Option<CalibrationManifest>,
}

enum StationRunError {
    Error(CalError),
    Excluded(Box<StationExclusion>),
}

impl From<CalError> for StationRunError {
    fn from(error: CalError) -> Self {
        Self::Error(error)
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
struct DecisionMetrics {
    coverage: f64,
    rms_cm: f64,
    p95_cm: f64,
    baseline_rms_cm: f64,
    rms_factor: f64,
}

#[derive(Debug, Serialize)]
struct DecisionRow {
    shom_id: String,
    station_id: String,
    name: String,
    calibration_period: String,
    validation_period: String,
    coverage: f64,
    rms_cm: f64,
    p95_cm: f64,
    baseline_rms_cm: f64,
    rms_factor: f64,
    included: bool,
    reason: String,
}

#[derive(Debug, Serialize)]
struct CalibrationManifest {
    schema_version: &'static str,
    generated_at: String,
    station_id: String,
    provider_station_id: String,
    station_name: String,
    source: u8,
    datum: String,
    datum_note: String,
    station_url: String,
    observations_url: String,
    input_period: PeriodInfo,
    calibration_period: PeriodInfo,
    validation_period: PeriodInfo,
    observations_sha256: String,
    tidegauge_sha256: String,
    qc: ManifestQc,
    benchmark: Option<ManifestBenchmark>,
    decision: ManifestDecision,
}

#[derive(Debug, Serialize)]
struct ManifestQc {
    expected: usize,
    observed: usize,
    coverage: f64,
    gaps: usize,
    jumps: usize,
}

#[derive(Debug, Serialize)]
struct ManifestBenchmark {
    benchmark_id: String,
    checksum_sha256: String,
    calibrated_rms_cm: f64,
    calibrated_mae_cm: f64,
    calibrated_p95_cm: f64,
    calibrated_max_cm: f64,
    z0_rms_cm: f64,
    z0_mae_cm: f64,
    z0_p95_cm: f64,
    z0_max_cm: f64,
    rms_factor_vs_z0: f64,
}

#[derive(Debug, Serialize)]
struct ManifestDecision {
    included: bool,
    reason: String,
}

struct PoliteClient {
    client: Client,
    throttle: StdDuration,
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

pub(crate) fn calibrate_france(args: CalibrateFranceArgs) -> Result<(), CalError> {
    let start = parse_rfc3339(&args.start)?;
    let validation_start = parse_rfc3339(&args.validation_start)?;
    let end = parse_rfc3339(&args.end)?;
    if !(start < validation_start && validation_start < end) {
        return Err(CalError::InvalidTimestamp(
            "start < validation_start < end is required".to_string(),
        ));
    }

    let client = PoliteClient::new(args.throttle_ms)?;
    let catalog = fetch_catalog(&client, &args.cache_dir)?;
    let selected = select_stations(&catalog, &args);
    let selected_ids = selected
        .iter()
        .map(|station| station.shom_id.as_str())
        .collect::<BTreeSet<_>>();
    let missing_requested = args
        .stations
        .iter()
        .filter(|station| !selected_ids.contains(station.as_str()))
        .map(|station| StationExclusion {
            shom_id: station.clone(),
            name: station.clone(),
            reason: "station not found in eligible REFMAR mainland catalog".to_string(),
            metrics: None,
            manifest: None,
        })
        .collect::<Vec<_>>();

    let mut artifacts = Vec::new();
    let mut exclusions = missing_requested;
    for station in selected {
        eprintln!(
            "calibrate refmar:{} {}",
            station.shom_id,
            title_case_station(&station.name)
        );
        match calibrate_station(&client, &args, &station, start, validation_start, end) {
            Ok(artifact) => {
                write_station_artifacts(&args, &artifact)?;
                eprintln!(
                    "include {} {} p95={:.1}cm factor={:.2}",
                    artifact.decision.station_id,
                    artifact.decision.name,
                    artifact.decision.p95_cm,
                    artifact.decision.rms_factor
                );
                artifacts.push(artifact);
            }
            Err(exclusion) => {
                let exclusion = *exclusion;
                if let Some(manifest) = &exclusion.manifest {
                    let path = args.manifests_dir.join(format!(
                        "{}_observations.json",
                        station_slug(&exclusion.name)
                    ));
                    pack_out::write_json(&path, manifest)?;
                }
                eprintln!(
                    "skip refmar:{} {}: {}",
                    exclusion.shom_id, exclusion.name, exclusion.reason
                );
                exclusions.push(exclusion);
            }
        }
    }

    if artifacts.is_empty() {
        return Err(CalError::QualityGate(
            "no REFMAR France station passed calibration gates".to_string(),
        ));
    }

    let mut stations = artifacts
        .iter()
        .map(|artifact| artifact.station.clone())
        .collect::<Vec<_>>();
    stations.sort_by(|left, right| left.station_id.cmp(&right.station_id));
    let pack = TidePack {
        schema_version: SCHEMA_VERSION.to_string(),
        generated_at: args.generated_at.clone(),
        stations,
    };
    pack.validate()?;
    pack_out::write_json(&args.pack_out, &pack)?;

    print_decisions(&artifacts, &exclusions);
    Ok(())
}

fn fetch_catalog(client: &PoliteClient, cache_dir: &Path) -> Result<Vec<CatalogStation>, CalError> {
    let cache_path = cache_dir.join("catalog").join("tidegauges.json");
    client.get_json(
        &format!("{REFMAR_BASE}/service/tidegauges"),
        &[],
        &cache_path,
    )
}

fn select_stations(catalog: &[CatalogStation], args: &CalibrateFranceArgs) -> Vec<CatalogStation> {
    let requested = args
        .stations
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut stations = catalog
        .iter()
        .filter(|station| {
            if !requested.is_empty() {
                return requested.contains(station.shom_id.as_str());
            }
            station.state == "OK" && is_mainland_france(station)
        })
        .filter(|station| args.include_brest || station.shom_id != crate::common::BREST_SHOM_ID)
        .cloned()
        .collect::<Vec<_>>();

    stations.sort_by(|left, right| {
        priority_rank(&left.shom_id)
            .cmp(&priority_rank(&right.shom_id))
            .then_with(|| left.name.cmp(&right.name))
    });
    if let Some(limit) = args.limit {
        stations.truncate(limit);
    }
    stations
}

fn priority_rank(shom_id: &str) -> usize {
    PRIORITY_STATIONS
        .iter()
        .position(|priority| *priority == shom_id)
        .unwrap_or(usize::MAX)
}

fn is_mainland_france(station: &CatalogStation) -> bool {
    (FRANCE_MIN_LAT..=FRANCE_MAX_LAT).contains(&station.latitude)
        && (FRANCE_MIN_LON..=FRANCE_MAX_LON).contains(&station.longitude)
}

fn calibrate_station(
    client: &PoliteClient,
    args: &CalibrateFranceArgs,
    catalog_station: &CatalogStation,
    start: DateTime<Utc>,
    validation_start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<StationArtifacts, Box<StationExclusion>> {
    match calibrate_station_inner(client, args, catalog_station, start, validation_start, end) {
        Ok(artifacts) => Ok(artifacts),
        Err(StationRunError::Excluded(exclusion)) => Err(exclusion),
        Err(StationRunError::Error(error)) => Err(Box::new(StationExclusion {
            shom_id: catalog_station.shom_id.clone(),
            name: title_case_station(&catalog_station.name),
            reason: error.to_string(),
            metrics: None,
            manifest: None,
        })),
    }
}

fn calibrate_station_inner(
    client: &PoliteClient,
    args: &CalibrateFranceArgs,
    catalog_station: &CatalogStation,
    start: DateTime<Utc>,
    validation_start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<StationArtifacts, StationRunError> {
    let tidegauge = fetch_tidegauge(client, &args.cache_dir, &catalog_station.shom_id)?;
    let tidegauge_path = tidegauge_cache_path(&args.cache_dir, &catalog_station.shom_id);
    let tidegauge_sha256 = pack_out::sha256_file(&tidegauge_path)?;
    let observations = fetch_observations(
        client,
        &args.cache_dir,
        &catalog_station.shom_id,
        args.source,
        start,
        end,
    )?;
    let display_name = title_case_station(&tidegauge.name);
    let slug = station_slug(&display_name);
    let station_id = format!("refmar:{}", tidegauge.shom_id);
    let datum = format!("zero_hydrographique_{slug}");
    let observations_csv = args
        .cache_dir
        .join("stations")
        .join(&tidegauge.shom_id)
        .join(format!(
            "{}_validated_hourly_{}_{}.csv",
            slug,
            date_label(start),
            date_label(end)
        ));
    pack_out::write_station_observations_csv(
        &observations_csv,
        ObservationCsvMetadata {
            station_name: &tidegauge.name,
            shom_id: &tidegauge.shom_id,
            source: args.source,
            datum: "zero_hydrographique",
        },
        observations.iter().copied(),
    )?;
    let observations_sha256 = pack_out::sha256_file(&observations_csv)?;

    let all_qc = qc::qc_report(&observations, start, end);
    let calibration_samples = observations
        .iter()
        .copied()
        .filter(|obs| obs.at >= start && obs.at < validation_start)
        .collect::<Vec<_>>();
    let validation_samples = observations
        .iter()
        .copied()
        .filter(|obs| obs.at >= validation_start && obs.at < end)
        .collect::<Vec<_>>();
    let calibration_qc = qc::qc_report(&calibration_samples, start, validation_start);
    let validation_qc = qc::qc_report(&validation_samples, validation_start, end);
    qc::enforce_qc("calibration", &calibration_qc)?;
    qc::enforce_qc("validation", &validation_qc)?;

    let calibration = solve::calibrate(
        &calibration_samples,
        start,
        validation_start,
        ConstituentSet::M22Rayleigh37,
    )?;
    let residuals = validation_samples
        .iter()
        .map(|observation| {
            let predicted =
                predict_height(&calibration.model, UtcDateTime::from_utc(observation.at))
                    .height()
                    .as_meters();
            observation.value_m - predicted
        })
        .collect::<Vec<_>>();
    let z0_residuals = validation_samples
        .iter()
        .map(|observation| observation.value_m - calibration.z0_m)
        .collect::<Vec<_>>();
    let calibrated_stats = qc::residual_stats(&residuals)
        .ok_or_else(|| CalError::EmptyObservations("validation".to_string()))?;
    let baseline_stats = qc::residual_stats(&z0_residuals)
        .ok_or_else(|| CalError::EmptyObservations("validation baseline".to_string()))?;
    let rms_factor = if calibrated_stats.rms_cm > 0.0 {
        baseline_stats.rms_cm / calibrated_stats.rms_cm
    } else {
        f64::INFINITY
    };
    let metrics = DecisionMetrics {
        coverage: validation_qc.coverage,
        rms_cm: calibrated_stats.rms_cm,
        p95_cm: calibrated_stats.p95_cm,
        baseline_rms_cm: baseline_stats.rms_cm,
        rms_factor,
    };

    let benchmark_id = format!("benchmark_{slug}_v1");
    let benchmark = pack_out::build_station_benchmark(BenchmarkBuildInput {
        validation_samples: &validation_samples,
        validation_start,
        validation_end: end,
        observations_sha256: &observations_sha256,
        generated_at: &args.generated_at,
        benchmark_id: &benchmark_id,
        station_id: &station_id,
        provider_station_id: &tidegauge.shom_id,
        station_name: &display_name,
        datum: &datum,
    });
    let included =
        calibrated_stats.p95_cm <= args.p95_limit_cm && rms_factor >= args.min_rms_factor;
    let reason = if included {
        "included".to_string()
    } else if calibrated_stats.p95_cm > args.p95_limit_cm {
        format!(
            "p95 {:.1} cm exceeds {:.1} cm",
            calibrated_stats.p95_cm, args.p95_limit_cm
        )
    } else {
        format!(
            "RMS factor {:.2} below {:.2}",
            rms_factor, args.min_rms_factor
        )
    };
    let datum_note = datum_note(&tidegauge);
    let manifest = manifest_for_station(ManifestForStation {
        args,
        tidegauge: &tidegauge,
        station_id: &station_id,
        display_name: &display_name,
        datum: &datum,
        datum_note: &datum_note,
        start,
        validation_start,
        end,
        observations_sha256: &observations_sha256,
        tidegauge_sha256: &tidegauge_sha256,
        all_qc: &all_qc,
        benchmark: Some(&benchmark),
        calibrated_stats,
        baseline_stats,
        rms_factor,
        included,
        reason: &reason,
    });

    if !included {
        return Err(StationRunError::Excluded(Box::new(StationExclusion {
            shom_id: tidegauge.shom_id.clone(),
            name: display_name,
            reason,
            metrics: Some(metrics),
            manifest: Some(manifest),
        })));
    }

    let station = pack_out::build_station_pack(StationPackBuildInput {
        tidegauge: &tidegauge,
        station_id: &station_id,
        datum: &datum,
        calibration,
        residual_p95_cm: calibrated_stats.p95_cm,
        generated_at: &args.generated_at,
        observations_sha256: &observations_sha256,
        tidegauge_sha256: &tidegauge_sha256,
        calibration_start: start,
        validation_start,
        validation_end: end,
    })?;
    let decision = DecisionRow {
        shom_id: tidegauge.shom_id.clone(),
        station_id,
        name: display_name,
        calibration_period: format!(
            "{}/{}",
            format_rfc3339(start),
            format_rfc3339(validation_start)
        ),
        validation_period: format!(
            "{}/{}",
            format_rfc3339(validation_start),
            format_rfc3339(end)
        ),
        coverage: metrics.coverage,
        rms_cm: metrics.rms_cm,
        p95_cm: metrics.p95_cm,
        baseline_rms_cm: metrics.baseline_rms_cm,
        rms_factor: metrics.rms_factor,
        included,
        reason,
    };
    Ok(StationArtifacts {
        station,
        benchmark,
        manifest,
        decision,
    })
}

struct ManifestForStation<'a> {
    args: &'a CalibrateFranceArgs,
    tidegauge: &'a TideGauge,
    station_id: &'a str,
    display_name: &'a str,
    datum: &'a str,
    datum_note: &'a str,
    start: DateTime<Utc>,
    validation_start: DateTime<Utc>,
    end: DateTime<Utc>,
    observations_sha256: &'a str,
    tidegauge_sha256: &'a str,
    all_qc: &'a QcReport,
    benchmark: Option<&'a TideBenchmark>,
    calibrated_stats: ResidualStats,
    baseline_stats: ResidualStats,
    rms_factor: f64,
    included: bool,
    reason: &'a str,
}

fn manifest_for_station(input: ManifestForStation<'_>) -> CalibrationManifest {
    CalibrationManifest {
        schema_version: "refmar-observation-manifest-v1",
        generated_at: input.args.generated_at.clone(),
        station_id: input.station_id.to_string(),
        provider_station_id: input.tidegauge.shom_id.clone(),
        station_name: input.display_name.to_string(),
        source: input.args.source,
        datum: input.datum.to_string(),
        datum_note: input.datum_note.to_string(),
        station_url: format!(
            "{REFMAR_BASE}/service/completetidegauge/{}",
            input.tidegauge.shom_id
        ),
        observations_url: format!(
            "{REFMAR_BASE}/observation/json/{}?sources={}",
            input.tidegauge.shom_id, input.args.source
        ),
        input_period: PeriodInfo {
            start: format_rfc3339(input.start),
            end: format_rfc3339(input.end),
        },
        calibration_period: PeriodInfo {
            start: format_rfc3339(input.start),
            end: format_rfc3339(input.validation_start),
        },
        validation_period: PeriodInfo {
            start: format_rfc3339(input.validation_start),
            end: format_rfc3339(input.end),
        },
        observations_sha256: input.observations_sha256.to_string(),
        tidegauge_sha256: input.tidegauge_sha256.to_string(),
        qc: ManifestQc {
            expected: input.all_qc.expected,
            observed: input.all_qc.observed,
            coverage: input.all_qc.coverage,
            gaps: input.all_qc.gaps.len(),
            jumps: input.all_qc.jumps.len(),
        },
        benchmark: input.benchmark.map(|benchmark| ManifestBenchmark {
            benchmark_id: benchmark.benchmark_id.clone(),
            checksum_sha256: benchmark.checksum_sha256.clone(),
            calibrated_rms_cm: input.calibrated_stats.rms_cm,
            calibrated_mae_cm: input.calibrated_stats.mae_cm,
            calibrated_p95_cm: input.calibrated_stats.p95_cm,
            calibrated_max_cm: input.calibrated_stats.max_cm,
            z0_rms_cm: input.baseline_stats.rms_cm,
            z0_mae_cm: input.baseline_stats.mae_cm,
            z0_p95_cm: input.baseline_stats.p95_cm,
            z0_max_cm: input.baseline_stats.max_cm,
            rms_factor_vs_z0: input.rms_factor,
        }),
        decision: ManifestDecision {
            included: input.included,
            reason: input.reason.to_string(),
        },
    }
}

fn fetch_tidegauge(
    client: &PoliteClient,
    cache_dir: &Path,
    shom_id: &str,
) -> Result<TideGauge, CalError> {
    client.get_json(
        &format!("{REFMAR_BASE}/service/completetidegauge/{shom_id}"),
        &[],
        &tidegauge_cache_path(cache_dir, shom_id),
    )
}

fn tidegauge_cache_path(cache_dir: &Path, shom_id: &str) -> PathBuf {
    cache_dir
        .join("stations")
        .join(shom_id)
        .join("tidegauge.json")
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

fn write_station_artifacts(
    args: &CalibrateFranceArgs,
    artifact: &StationArtifacts,
) -> Result<(), CalError> {
    let slug = station_slug(&artifact.decision.name);
    pack_out::write_json(
        &args
            .benchmarks_dir
            .join(format!("benchmark_{slug}_v1.json")),
        &artifact.benchmark,
    )?;
    pack_out::write_json(
        &args.manifests_dir.join(format!("{slug}_observations.json")),
        &artifact.manifest,
    )?;
    Ok(())
}

fn datum_note(tidegauge: &TideGauge) -> String {
    tidegauge
        .vertical_ref
        .as_ref()
        .map(|reference| {
            format!(
                "{}; ZH = {} m relative to {}; RAM id {} in REFMAR tide-gauge metadata",
                reference.zero_hydro,
                reference.zh_ref,
                reference.nom_ref,
                tidegauge.id_ram.as_deref().unwrap_or("NEEDS-REVIEW")
            )
        })
        .unwrap_or_else(|| "zero_hydrographique; REFMAR vertical reference missing".to_string())
}

fn print_decisions(artifacts: &[StationArtifacts], exclusions: &[StationExclusion]) {
    println!("station_id,name,coverage,rms_cm,p95_cm,z0_rms_cm,rms_factor,included,reason");
    for artifact in artifacts {
        let decision = &artifact.decision;
        println!(
            "{},{},{:.3},{:.1},{:.1},{:.1},{:.2},{},{}",
            decision.station_id,
            decision.name,
            decision.coverage,
            decision.rms_cm,
            decision.p95_cm,
            decision.baseline_rms_cm,
            decision.rms_factor,
            decision.included,
            decision.reason
        );
    }
    for exclusion in exclusions {
        if let Some(metrics) = exclusion.metrics {
            println!(
                "refmar:{},{},{:.3},{:.1},{:.1},{:.1},{:.2},false,{}",
                exclusion.shom_id,
                exclusion.name,
                metrics.coverage,
                metrics.rms_cm,
                metrics.p95_cm,
                metrics.baseline_rms_cm,
                metrics.rms_factor,
                exclusion.reason
            );
        } else {
            println!(
                "refmar:{},{},NA,NA,NA,NA,NA,false,{}",
                exclusion.shom_id, exclusion.name, exclusion.reason
            );
        }
    }
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

fn date_label(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d").to_string()
}

fn station_slug(name: &str) -> String {
    let mut output = String::new();
    let mut previous_separator = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            previous_separator = false;
        } else if !previous_separator {
            output.push('_');
            previous_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}
