mod common;
mod diagnose;
mod fetch;
mod ib;
mod pack_out;
mod qc;
mod solve;

use amar_core::{UtcDateTime, predict_height};
use chrono::{DateTime, Datelike, TimeZone, Utc};
use clap::{Args, Parser, Subcommand};
use common::{BREST_SHOM_ID, CalError, Observation, VALIDATED_HOURLY_SOURCE, parse_rfc3339};
use std::path::PathBuf;
use std::process::ExitCode;

const DEFAULT_OBSERVATIONS: &str =
    "fixtures/refmar/brest_validated_hourly_2021-01-01_2026-07-01.csv";
const DEFAULT_TIDEGAUGE: &str = "fixtures/refmar/brest_tidegauge.json";
const DEFAULT_PACK: &str = "data/packs/amar-data-brest-experimental.json";
const DEFAULT_BENCHMARK: &str = "fixtures/refmar/benchmark_brest_v1.json";
const DEFAULT_GENERATED_AT: &str = "2026-07-06-m2.2";
const DEFAULT_START: &str = "2021-01-01T00:00:00Z";
const DEFAULT_VALIDATION_START: &str = "2026-04-01T00:00:00Z";
const DEFAULT_END: &str = "2026-07-01T00:00:00Z";
const DRIFT_P95_LIMIT_CM: f64 = 100.0;
const DRIFT_BIAS_LIMIT_CM: f64 = 50.0;

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
    Diagnose(DiagnoseArgs),
    DiagnoseIb(DiagnoseIbArgs),
}

#[derive(Debug, Args)]
pub(crate) struct FetchRefmarArgs {
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
    #[arg(long)]
    benchmark_out: Option<PathBuf>,
    #[arg(long, default_value = DEFAULT_GENERATED_AT)]
    generated_at: String,
    #[arg(long, value_enum, default_value_t = solve::ConstituentSet::M22Rayleigh37)]
    constituent_set: solve::ConstituentSet,
}

#[derive(Debug, Args)]
struct DiagnoseArgs {
    #[arg(long, default_value = DEFAULT_OBSERVATIONS)]
    observations: PathBuf,
    #[arg(long, default_value = DEFAULT_PACK)]
    pack: PathBuf,
    #[arg(long, default_value = DEFAULT_BENCHMARK)]
    benchmark: PathBuf,
    #[arg(long, default_value = "refmar:3")]
    station_id: String,
}

#[derive(Debug, Args)]
struct DiagnoseIbArgs {
    #[arg(long, default_value = DEFAULT_PACK)]
    pack: PathBuf,
    #[arg(long, default_value = DEFAULT_BENCHMARK)]
    benchmark: PathBuf,
    #[arg(
        long,
        default_value = "fixtures/open_meteo/brest_surface_pressure_2026-04-01_2026-06-30.json"
    )]
    pressure: PathBuf,
    #[arg(long, default_value = "refmar:3")]
    station_id: String,
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
        Command::FetchRefmar(args) => fetch::fetch_refmar(args),
        Command::BuildBrestPack(args) => build_brest_pack(args),
        Command::Diagnose(args) => diagnose::diagnose(args),
        Command::DiagnoseIb(args) => ib::diagnose_ib(args),
    }
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

    let observations = pack_out::read_observations_csv(&args.observations)?;
    let tidegauge = pack_out::read_tidegauge(&args.tidegauge)?;
    let all_qc = qc::qc_report(&observations, calibration_start, validation_end);
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
    let calibration_qc = qc::qc_report(&calibration_samples, calibration_start, validation_start);
    let validation_qc = qc::qc_report(&validation_samples, validation_start, validation_end);
    qc::enforce_qc("calibration", &calibration_qc)?;
    qc::enforce_qc("validation", &validation_qc)?;

    let calibration = solve::calibrate(
        &calibration_samples,
        calibration_start,
        validation_start,
        args.constituent_set,
    )?;
    let residuals = validation_samples
        .iter()
        .map(|observation| {
            let at = UtcDateTime::from_utc(observation.at);
            let predicted = predict_height(&calibration.model, at).height().as_meters();
            observation.value_m - predicted
        })
        .collect::<Vec<_>>();
    let residual_stats = qc::residual_stats(&residuals)
        .ok_or_else(|| CalError::EmptyObservations("validation".to_string()))?;
    if residual_stats.p95_cm > DRIFT_P95_LIMIT_CM
        || residual_stats.bias_cm.abs() > DRIFT_BIAS_LIMIT_CM
    {
        return Err(CalError::QualityGate(format!(
            "Brest résidu aberrant p95={:.1} cm biais={:.1} cm",
            residual_stats.p95_cm, residual_stats.bias_cm
        )));
    }

    let observations_sha256 = pack_out::sha256_file(&args.observations)?;
    let tidegauge_sha256 = pack_out::sha256_file(&args.tidegauge)?;
    if let Some(benchmark_out) = &args.benchmark_out {
        let benchmark = pack_out::build_benchmark(
            &validation_samples,
            validation_start,
            validation_end,
            &observations_sha256,
            &args.generated_at,
        );
        pack_out::write_json(benchmark_out, &benchmark)?;
    }

    let pack = pack_out::build_pack(pack_out::PackBuildInput {
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
    pack_out::write_json(&args.out, &pack)?;

    println!(
        "brest calibration observations={} coverage={:.3} gaps={} jumps={}",
        all_qc.observed,
        all_qc.coverage,
        all_qc.gaps.len(),
        all_qc.jumps.len()
    );
    print_yearly_qc(&observations, calibration_start, validation_end);
    qc::print_qc("calibration", &calibration_qc);
    qc::print_qc("validation", &validation_qc);
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

fn print_yearly_qc(observations: &[Observation], start: DateTime<Utc>, end: DateTime<Utc>) {
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
        let report = qc::qc_report(&samples, year_start, year_end);
        qc::print_qc(&format!("year {year}"), &report);
    }
}

fn utc_year_start(year: i32) -> DateTime<Utc> {
    match Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).single() {
        Some(value) => value,
        None => unreachable!("valid UTC year boundary"),
    }
}
