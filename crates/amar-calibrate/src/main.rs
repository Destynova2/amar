use amar_core::{
    ConstituentId, CoreError, DatumId, Degrees, DegreesPerHour, HarmonicConstituent, Meters,
    PredictionMethod, TideModel, UtcDateTime, harmonic_basis, predict_height,
};
use amar_pack::{
    ConstituentPack, DegreesPerHourValue, DegreesValue, LatitudeDegValue, LongitudeDegValue,
    MetersValue, PeriodInfo, SCHEMA_VERSION, SourceInfo, StationPack, TidePack,
};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use clap::{Args, Parser, Subcommand};
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use thiserror::Error;

const REFMAR_BASE: &str = "https://services.data.shom.fr/maregraphie";
const DEFAULT_OBSERVATIONS: &str =
    "fixtures/refmar/brest_validated_hourly_2025-01-01_2026-07-01.csv";
const DEFAULT_TIDEGAUGE: &str = "fixtures/refmar/brest_tidegauge.json";
const DEFAULT_PACK: &str = "data/packs/amar-data-brest-experimental.json";
const DEFAULT_BENCHMARK: &str = "fixtures/refmar/benchmark_brest_v1.json";
const DEFAULT_GENERATED_AT: &str = "2026-07-06";
const DEFAULT_START: &str = "2025-01-01T00:00:00Z";
const DEFAULT_VALIDATION_START: &str = "2026-04-01T00:00:00Z";
const DEFAULT_END: &str = "2026-07-01T00:00:00Z";
const BREST_SHOM_ID: &str = "3";
const VALIDATED_HOURLY_SOURCE: u8 = 4;
const MIN_COVERAGE: f64 = 0.90;
const JUMP_THRESHOLD_M: f64 = 2.5;
const DRIFT_P95_LIMIT_CM: f64 = 100.0;
const DRIFT_BIAS_LIMIT_CM: f64 = 50.0;

const CONSTITUENTS: [ConstituentSpec; 16] = [
    ConstituentSpec::new("M2", 28.984_104),
    ConstituentSpec::new("S2", 30.0),
    ConstituentSpec::new("N2", 28.439_73),
    ConstituentSpec::new("K2", 30.082_138),
    ConstituentSpec::new("K1", 15.041_069),
    ConstituentSpec::new("O1", 13.943_035),
    ConstituentSpec::new("P1", 14.958_931),
    ConstituentSpec::new("Q1", 13.398_661),
    ConstituentSpec::new("M4", 57.968_21),
    ConstituentSpec::new("MS4", 58.984_104),
    ConstituentSpec::new("MN4", 57.423_832),
    ConstituentSpec::new("M6", 86.952_32),
    ConstituentSpec::new("MF", 1.098_033_1),
    ConstituentSpec::new("MM", 0.544_374_7),
    ConstituentSpec::new("SA", 0.041_068_6),
    ConstituentSpec::new("SSA", 0.082_137_3),
];

#[derive(Clone, Copy)]
struct ConstituentSpec {
    name: &'static str,
    speed_deg_per_hour: f64,
}

impl ConstituentSpec {
    const fn new(name: &'static str, speed_deg_per_hour: f64) -> Self {
        Self {
            name,
            speed_deg_per_hour,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "amar-calibrate")]
#[command(about = "Bounded Brest-only REFMAR calibration compiler")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    FetchRefmar(FetchRefmarArgs),
    BuildBrestPack(BuildBrestPackArgs),
}

#[derive(Debug, Args)]
struct FetchRefmarArgs {
    #[arg(long, default_value = DEFAULT_START)]
    start: String,
    #[arg(long, default_value = DEFAULT_END)]
    end: String,
    #[arg(long, default_value = BREST_SHOM_ID)]
    shom_id: String,
    #[arg(long, default_value_t = VALIDATED_HOURLY_SOURCE)]
    source: u8,
    #[arg(long, default_value = DEFAULT_OBSERVATIONS)]
    out: PathBuf,
    #[arg(long, default_value = DEFAULT_TIDEGAUGE)]
    tidegauge_out: PathBuf,
}

#[derive(Debug, Args)]
struct BuildBrestPackArgs {
    #[arg(long, default_value = DEFAULT_OBSERVATIONS)]
    observations: PathBuf,
    #[arg(long, default_value = DEFAULT_TIDEGAUGE)]
    tidegauge: PathBuf,
    #[arg(long, default_value = DEFAULT_START)]
    calibration_start: String,
    #[arg(long, default_value = DEFAULT_VALIDATION_START)]
    validation_start: String,
    #[arg(long, default_value = DEFAULT_END)]
    validation_end: String,
    #[arg(long, default_value = DEFAULT_PACK)]
    out: PathBuf,
    #[arg(long, default_value = DEFAULT_BENCHMARK)]
    benchmark_out: PathBuf,
    #[arg(long, default_value = DEFAULT_GENERATED_AT)]
    generated_at: String,
}

#[derive(Debug, Error)]
enum CalError {
    #[error("{0}")]
    Core(#[from] CoreError),
    #[error("{0}")]
    Pack(#[from] amar_pack::PackError),
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid timestamp {0}")]
    InvalidTimestamp(String),
    #[error("invalid observation CSV line {line}: {reason}")]
    InvalidCsvLine { line: String, reason: String },
    #[error("no observations available for {0}")]
    EmptyObservations(String),
    #[error("least-squares solve failed")]
    SolveFailed,
    #[error("quality gate failed: {0}")]
    QualityGate(String),
}

#[derive(Clone, Copy, Debug)]
struct Observation {
    at: DateTime<Utc>,
    value_m: f64,
    source: u8,
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

#[derive(Debug, Deserialize)]
struct TideGauge {
    shom_id: String,
    name: String,
    longitude: f64,
    latitude: f64,
    id_ram: Option<String>,
    #[serde(rename = "verticalRef")]
    vertical_ref: Option<VerticalRef>,
}

#[derive(Debug, Deserialize)]
struct VerticalRef {
    zero_hydro: String,
    zh_ref: String,
    nom_ref: String,
}

#[derive(Debug)]
struct QcReport {
    expected: usize,
    observed: usize,
    coverage: f64,
    gaps: Vec<Gap>,
    jumps: Vec<Jump>,
}

#[derive(Debug)]
struct Gap {
    after: DateTime<Utc>,
    before: DateTime<Utc>,
    hours: i64,
}

#[derive(Debug)]
struct Jump {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    delta_m: f64,
}

#[derive(Debug)]
struct CalibrationResult {
    z0_m: f64,
    constituents: Vec<ConstituentPack>,
    model: TideModel,
}

struct PackBuildInput<'a> {
    tidegauge: &'a TideGauge,
    calibration: CalibrationResult,
    residual_p95_cm: f64,
    generated_at: &'a str,
    observations_sha256: &'a str,
    tidegauge_sha256: &'a str,
    calibration_start: DateTime<Utc>,
    validation_start: DateTime<Utc>,
    validation_end: DateTime<Utc>,
}

#[derive(Clone, Copy)]
struct ResidualStats {
    samples: usize,
    bias_cm: f64,
    p95_cm: f64,
}

#[derive(Debug, Serialize)]
struct Benchmark {
    schema_version: String,
    benchmark_id: String,
    generated_at: String,
    station_id: String,
    provider_station_id: String,
    station_name: String,
    datum: String,
    product: String,
    source: String,
    validation_period: BenchmarkPeriod,
    observations_sha256: String,
    checksum_sha256: String,
    samples: Vec<BenchmarkSample>,
}

#[derive(Debug, Serialize)]
struct BenchmarkPeriod {
    start: String,
    end: String,
}

#[derive(Debug, Serialize)]
struct BenchmarkSample {
    timestamp: String,
    observed_m: Option<f64>,
    missing: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), CalError> {
    let cli = Cli::parse();
    match cli.command {
        Command::FetchRefmar(args) => fetch_refmar(args),
        Command::BuildBrestPack(args) => build_brest_pack(args),
    }
}

fn fetch_refmar(args: FetchRefmarArgs) -> Result<(), CalError> {
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

fn build_brest_pack(args: BuildBrestPackArgs) -> Result<(), CalError> {
    let calibration_start = parse_rfc3339(&args.calibration_start)?;
    let validation_start = parse_rfc3339(&args.validation_start)?;
    let validation_end = parse_rfc3339(&args.validation_end)?;
    if !(calibration_start < validation_start && validation_start < validation_end) {
        return Err(CalError::InvalidTimestamp(
            "calibration_start < validation_start < validation_end is required".to_string(),
        ));
    }

    let observations = read_observations_csv(&args.observations)?;
    let tidegauge = read_json::<TideGauge>(&args.tidegauge)?;
    let all_qc = qc_report(&observations, calibration_start, validation_end);
    let calibration_samples = observations
        .iter()
        .copied()
        .filter(|obs| obs.at >= calibration_start && obs.at < validation_start)
        .collect::<Vec<_>>();
    let validation_samples = observations
        .iter()
        .copied()
        .filter(|obs| obs.at >= validation_start && obs.at < validation_end)
        .collect::<Vec<_>>();
    let calibration_qc = qc_report(&calibration_samples, calibration_start, validation_start);
    let validation_qc = qc_report(&validation_samples, validation_start, validation_end);
    enforce_qc("calibration", &calibration_qc)?;
    enforce_qc("validation", &validation_qc)?;

    let calibration = calibrate(&calibration_samples)?;
    let residuals = validation_samples
        .iter()
        .map(|observation| {
            let at = UtcDateTime::from_utc(observation.at);
            let predicted = predict_height(&calibration.model, at).height().as_meters();
            observation.value_m - predicted
        })
        .collect::<Vec<_>>();
    let residual_stats = residual_stats(&residuals)
        .ok_or_else(|| CalError::EmptyObservations("validation".to_string()))?;
    if residual_stats.p95_cm > DRIFT_P95_LIMIT_CM
        || residual_stats.bias_cm.abs() > DRIFT_BIAS_LIMIT_CM
    {
        return Err(CalError::QualityGate(format!(
            "Brest résidu aberrant p95={:.1} cm biais={:.1} cm",
            residual_stats.p95_cm, residual_stats.bias_cm
        )));
    }

    let observations_sha256 = sha256_file(&args.observations)?;
    let tidegauge_sha256 = sha256_file(&args.tidegauge)?;
    let benchmark = build_benchmark(
        &validation_samples,
        validation_start,
        validation_end,
        &observations_sha256,
        &args.generated_at,
    );
    write_json(&args.benchmark_out, &benchmark)?;

    let pack = build_pack(PackBuildInput {
        tidegauge: &tidegauge,
        calibration,
        residual_p95_cm: residual_stats.p95_cm,
        generated_at: &args.generated_at,
        observations_sha256: &observations_sha256,
        tidegauge_sha256: &tidegauge_sha256,
        calibration_start,
        validation_start,
        validation_end,
    })?;
    write_json(&args.out, &pack)?;

    println!(
        "brest calibration observations={} coverage={:.3} gaps={} jumps={}",
        all_qc.observed,
        all_qc.coverage,
        all_qc.gaps.len(),
        all_qc.jumps.len()
    );
    print_qc("calibration", &calibration_qc);
    print_qc("validation", &validation_qc);
    println!(
        "z0_m={:.3} validation_samples={} résidu_biais_cm={:.1} résidu_p95_cm={:.1}",
        pack.stations[0].z0_m.get(),
        residual_stats.samples,
        residual_stats.bias_cm,
        residual_stats.p95_cm,
    );
    for constituent in &pack.stations[0].constituents {
        println!(
            "constituent={} amplitude_m={:.4} phase_gmt_deg={:.2}",
            constituent.name,
            constituent.amplitude_m.get(),
            constituent.phase_gmt_deg.get()
        );
    }
    Ok(())
}

fn calibrate(samples: &[Observation]) -> Result<CalibrationResult, CalError> {
    if samples.is_empty() {
        return Err(CalError::EmptyObservations("calibration".to_string()));
    }
    let columns = 1 + CONSTITUENTS.len() * 2;
    let mut matrix = Vec::with_capacity(samples.len() * columns);
    let mut values = Vec::with_capacity(samples.len());
    let ids = CONSTITUENTS
        .iter()
        .map(|spec| {
            Ok((
                spec,
                ConstituentId::new(spec.name)?,
                DegreesPerHour::new(spec.speed_deg_per_hour)?,
            ))
        })
        .collect::<Result<Vec<_>, CalError>>()?;

    for sample in samples {
        matrix.push(1.0);
        let at = UtcDateTime::from_utc(sample.at);
        for (_, id, speed) in &ids {
            let basis = harmonic_basis(id, *speed, PredictionMethod::StationHarmonicsV0, at)?;
            let radians = basis.argument_degrees.to_radians();
            matrix.push(basis.nodal_factor * radians.cos());
            matrix.push(basis.nodal_factor * radians.sin());
        }
        values.push(sample.value_m);
    }

    let a = DMatrix::from_row_slice(samples.len(), columns, &matrix);
    let y = DVector::from_row_slice(&values);
    let solution = a
        .svd(true, true)
        .solve(&y, 1.0e-10)
        .map_err(|_| CalError::SolveFailed)?;

    let z0_m = solution[0];
    let mut constituents = Vec::with_capacity(CONSTITUENTS.len());
    let mut model_constituents = Vec::with_capacity(CONSTITUENTS.len());
    for (index, (spec, id, speed)) in ids.iter().enumerate() {
        let cos_coefficient = solution[1 + index * 2];
        let sin_coefficient = solution[1 + index * 2 + 1];
        let amplitude_m = cos_coefficient.hypot(sin_coefficient);
        let phase_gmt_deg = sin_coefficient
            .atan2(cos_coefficient)
            .to_degrees()
            .rem_euclid(360.0);
        constituents.push(ConstituentPack {
            name: spec.name.to_string(),
            amplitude_m: MetersValue::new(amplitude_m),
            phase_gmt_deg: DegreesValue::new(phase_gmt_deg),
            speed_deg_per_hour: DegreesPerHourValue::new(spec.speed_deg_per_hour),
        });
        model_constituents.push(HarmonicConstituent::new(
            id.clone(),
            Meters::new(amplitude_m)?,
            Degrees::new(phase_gmt_deg)?,
            *speed,
        ));
    }
    let model = TideModel::new(
        DatumId::new("zero_hydrographique_brest")?,
        Meters::new(z0_m)?,
        model_constituents,
        PredictionMethod::StationHarmonicsV0,
    )?;

    Ok(CalibrationResult {
        z0_m,
        constituents,
        model,
    })
}

fn build_pack(input: PackBuildInput<'_>) -> Result<TidePack, CalError> {
    let tidegauge = input.tidegauge;
    let vertical_ref = tidegauge.vertical_ref.as_ref();
    let datum_note = vertical_ref
        .map(|reference| {
            format!(
                "{}; ZH = {} m relative to {}; RAM id {} in REFMAR tide-gauge metadata",
                reference.zero_hydro,
                reference.zh_ref,
                reference.nom_ref,
                tidegauge.id_ram.as_deref().unwrap_or("NEEDS-REVIEW")
            )
        })
        .unwrap_or_else(|| "zero_hydrographique; REFMAR vertical reference missing".to_string());
    let station_url = format!(
        "{REFMAR_BASE}/service/completetidegauge/{}",
        tidegauge.shom_id
    );
    let observations_url = format!(
        "{REFMAR_BASE}/observation/json/{}?sources={VALIDATED_HOURLY_SOURCE}",
        tidegauge.shom_id
    );
    let station = StationPack {
        station_id: "refmar:3".to_string(),
        provider_station_id: tidegauge.shom_id.clone(),
        name: title_case_station(&tidegauge.name),
        latitude_deg: LatitudeDegValue::new(tidegauge.latitude),
        longitude_deg: LongitudeDegValue::new(tidegauge.longitude),
        datum: "zero_hydrographique_brest".to_string(),
        z0_m: MetersValue::new(input.calibration.z0_m),
        method: PredictionMethod::StationHarmonicsV0.as_str().to_string(),
        constituents: input.calibration.constituents,
        source: SourceInfo {
            provider: "Shom / REFMAR".to_string(),
            license: "Licence Ouverte 2.0 Etalab".to_string(),
            extracted_at: input.generated_at.to_string(),
            station_url: station_url.clone(),
            datums_url: station_url,
            harcon_url: "not applicable: constants calibrated from REFMAR observations".to_string(),
            checksum_sha256: input.observations_sha256.to_string(),
            attribution: Some("Shom / REFMAR".to_string()),
            product: Some("Données horaires validées REFMAR, source 4".to_string()),
            observations_url: Some(observations_url),
            observations_checksum_sha256: Some(input.observations_sha256.to_string()),
            tidegauge_checksum_sha256: Some(input.tidegauge_sha256.to_string()),
        },
        experimental: Some(true),
        not_official: Some(true),
        not_shom: Some(true),
        calibration_period: Some(PeriodInfo {
            start: format_rfc3339(input.calibration_start),
            end: format_rfc3339(input.validation_start),
        }),
        validation_period: Some(PeriodInfo {
            start: format_rfc3339(input.validation_start),
            end: format_rfc3339(input.validation_end),
        }),
        disclaimer: Some(
            "constantes dérivées des observations REFMAR, non équivalentes aux constantes SHOM"
                .to_string(),
        ),
        datum_note: Some(datum_note),
        residual_benchmark_cm: Some(input.residual_p95_cm),
    };
    let pack = TidePack {
        schema_version: SCHEMA_VERSION.to_string(),
        generated_at: input.generated_at.to_string(),
        stations: vec![station],
    };
    pack.validate()?;
    Ok(pack)
}

fn build_benchmark(
    validation_samples: &[Observation],
    validation_start: DateTime<Utc>,
    validation_end: DateTime<Utc>,
    observations_sha256: &str,
    generated_at: &str,
) -> Benchmark {
    let by_time = validation_samples
        .iter()
        .map(|observation| (observation.at, observation.value_m))
        .collect::<BTreeMap<_, _>>();
    let mut samples = Vec::new();
    let mut checksum_input = String::new();
    let mut cursor = validation_start;
    while cursor < validation_end {
        let observed_m = by_time.get(&cursor).copied();
        samples.push(BenchmarkSample {
            timestamp: format_rfc3339(cursor),
            observed_m,
            missing: observed_m.is_none(),
        });
        checksum_input.push_str(&format!(
            "{},{}\n",
            format_rfc3339(cursor),
            observed_m
                .map(|value| format!("{value:.3}"))
                .unwrap_or_else(|| "NA".to_string())
        ));
        cursor += Duration::hours(1);
    }
    Benchmark {
        schema_version: "benchmark_brest_v1".to_string(),
        benchmark_id: "benchmark_brest_v1".to_string(),
        generated_at: generated_at.to_string(),
        station_id: "refmar:3".to_string(),
        provider_station_id: BREST_SHOM_ID.to_string(),
        station_name: "Brest".to_string(),
        datum: "zero_hydrographique_brest".to_string(),
        product: "Données horaires validées REFMAR, source 4".to_string(),
        source: "Shom / REFMAR".to_string(),
        validation_period: BenchmarkPeriod {
            start: format_rfc3339(validation_start),
            end: format_rfc3339(validation_end),
        },
        observations_sha256: observations_sha256.to_string(),
        checksum_sha256: sha256_hex(checksum_input.as_bytes()),
        samples,
    }
}

fn qc_report(observations: &[Observation], start: DateTime<Utc>, end: DateTime<Utc>) -> QcReport {
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
        let delta_hours = (next.at - previous.at).num_minutes();
        if delta_hours > 90 {
            gaps.push(Gap {
                after: previous.at,
                before: next.at,
                hours: delta_hours / 60,
            });
        }
        let delta_m = next.value_m - previous.value_m;
        if delta_m.abs() > JUMP_THRESHOLD_M {
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

fn enforce_qc(label: &str, report: &QcReport) -> Result<(), CalError> {
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

fn print_qc(label: &str, report: &QcReport) {
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
            "{label} gap after={} before={} hours={}",
            format_rfc3339(gap.after),
            format_rfc3339(gap.before),
            gap.hours
        );
    }
}

fn residual_stats(residuals_m: &[f64]) -> Option<ResidualStats> {
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
    let index = ((absolute.len() - 1) as f64 * 0.95).ceil() as usize;
    Some(ResidualStats {
        samples,
        bias_cm: bias_m * 100.0,
        p95_cm: absolute[index] * 100.0,
    })
}

fn read_observations_csv(path: &Path) -> Result<Vec<Observation>, CalError> {
    let data = fs::read_to_string(path).map_err(|source| CalError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut observations = BTreeMap::new();
    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("timestamp,") {
            continue;
        }
        let fields = line.split(',').collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err(CalError::InvalidCsvLine {
                line: line.to_string(),
                reason: "expected timestamp,value_m,source".to_string(),
            });
        }
        let at = parse_rfc3339(fields[0])?;
        let value_m = fields[1]
            .parse::<f64>()
            .map_err(|error| CalError::InvalidCsvLine {
                line: line.to_string(),
                reason: error.to_string(),
            })?;
        let source = fields[2]
            .parse::<u8>()
            .map_err(|error| CalError::InvalidCsvLine {
                line: line.to_string(),
                reason: error.to_string(),
            })?;
        observations.insert(
            at,
            Observation {
                at,
                value_m,
                source,
            },
        );
    }
    if observations.is_empty() {
        return Err(CalError::EmptyObservations(path.display().to_string()));
    }
    Ok(observations.values().copied().collect())
}

fn write_observations_csv(
    path: &Path,
    observations: impl IntoIterator<Item = Observation>,
) -> Result<(), CalError> {
    let mut output = String::new();
    output.push_str("# station=BREST\n");
    output.push_str("# shom_id=3\n");
    output.push_str("# provider=Shom / REFMAR\n");
    output.push_str("# license=Licence Ouverte 2.0 Etalab\n");
    output.push_str("# product=Donnees horaires validees REFMAR, source 4\n");
    output.push_str("# datum=zero_hydrographique\n");
    output.push_str("# unit=m\n");
    output.push_str("timestamp,value_m,source\n");
    for observation in observations {
        output.push_str(&format!(
            "{},{:.3},{}\n",
            format_rfc3339(observation.at),
            observation.value_m,
            observation.source
        ));
    }
    write_string(path, &output)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, CalError> {
    let data = fs::read_to_string(path).map_err(|source| CalError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&data).map_err(CalError::from)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), CalError> {
    write_string(path, &format!("{}\n", serde_json::to_string_pretty(value)?))
}

fn write_string(path: &Path, data: &str) -> Result<(), CalError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CalError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, data).map_err(|source| CalError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn sha256_file(path: &Path) -> Result<String, CalError> {
    let data = fs::read(path).map_err(|source| CalError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(sha256_hex(&data))
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

fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>, CalError> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(|_| CalError::InvalidTimestamp(value.to_string()))
}

fn parse_refmar_timestamp(value: &str) -> Result<DateTime<Utc>, CalError> {
    NaiveDateTime::parse_from_str(value, "%Y/%m/%d %H:%M:%S")
        .map(|date| date.and_utc())
        .map_err(|_| CalError::InvalidTimestamp(value.to_string()))
}

fn format_refmar_query_time(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn format_rfc3339(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn title_case_station(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_uppercase(),
                    chars.as_str().to_ascii_lowercase()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
