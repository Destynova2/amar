use crate::common::{
    BREST_SHOM_ID, CalError, Observation, REFMAR_BASE, VALIDATED_HOURLY_SOURCE, format_rfc3339,
    parse_rfc3339,
};
use crate::solve::CalibrationResult;
use amar_core::PredictionMethod;
use amar_pack::{
    DatumReference, LatitudeDegValue, LongitudeDegValue, MetersValue, PeriodInfo, SCHEMA_VERSION,
    SourceInfo, StationPack, TideBenchmark, TideBenchmarkSample, TidePack,
};
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub(crate) struct TideGauge {
    pub(crate) shom_id: String,
    pub(crate) name: String,
    pub(crate) longitude: f64,
    pub(crate) latitude: f64,
    pub(crate) id_ram: Option<String>,
    pub(crate) niveau_moyen: Option<String>,
    #[serde(rename = "verticalRef")]
    pub(crate) vertical_ref: Option<VerticalRef>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct VerticalRef {
    pub(crate) zero_hydro: String,
    pub(crate) zh_ref: String,
    pub(crate) nom_ref: String,
}

pub(crate) struct PackBuildInput<'a> {
    pub(crate) tidegauge: &'a TideGauge,
    pub(crate) calibration: CalibrationResult,
    pub(crate) residual_p95_cm: f64,
    pub(crate) generated_at: &'a str,
    pub(crate) observations_sha256: &'a str,
    pub(crate) tidegauge_sha256: &'a str,
    pub(crate) calibration_start: DateTime<Utc>,
    pub(crate) validation_start: DateTime<Utc>,
    pub(crate) validation_end: DateTime<Utc>,
}

pub(crate) struct StationPackBuildInput<'a> {
    pub(crate) tidegauge: &'a TideGauge,
    pub(crate) station_id: &'a str,
    pub(crate) datum: &'a str,
    pub(crate) calibration: CalibrationResult,
    pub(crate) residual_p95_cm: f64,
    pub(crate) generated_at: &'a str,
    pub(crate) observations_sha256: &'a str,
    pub(crate) tidegauge_sha256: &'a str,
    pub(crate) calibration_start: DateTime<Utc>,
    pub(crate) validation_start: DateTime<Utc>,
    pub(crate) validation_end: DateTime<Utc>,
}

pub(crate) struct BenchmarkBuildInput<'a> {
    pub(crate) validation_samples: &'a [Observation],
    pub(crate) validation_start: DateTime<Utc>,
    pub(crate) validation_end: DateTime<Utc>,
    pub(crate) observations_sha256: &'a str,
    pub(crate) generated_at: &'a str,
    pub(crate) benchmark_id: &'a str,
    pub(crate) station_id: &'a str,
    pub(crate) provider_station_id: &'a str,
    pub(crate) station_name: &'a str,
    pub(crate) datum: &'a str,
}

pub(crate) struct ObservationCsvMetadata<'a> {
    pub(crate) station_name: &'a str,
    pub(crate) shom_id: &'a str,
    pub(crate) source: u8,
    pub(crate) datum: &'a str,
}

pub(crate) fn read_observations_csv(path: &Path) -> Result<Vec<Observation>, CalError> {
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

pub(crate) fn write_observations_csv(
    path: &Path,
    observations: impl IntoIterator<Item = Observation>,
) -> Result<(), CalError> {
    write_station_observations_csv(
        path,
        ObservationCsvMetadata {
            station_name: "BREST",
            shom_id: BREST_SHOM_ID,
            source: VALIDATED_HOURLY_SOURCE,
            datum: "zero_hydrographique",
        },
        observations,
    )
}

pub(crate) fn write_station_observations_csv(
    path: &Path,
    metadata: ObservationCsvMetadata<'_>,
    observations: impl IntoIterator<Item = Observation>,
) -> Result<(), CalError> {
    let mut output = String::new();
    output.push_str(&format!("# station={}\n", metadata.station_name));
    output.push_str(&format!("# shom_id={}\n", metadata.shom_id));
    output.push_str("# provider=Shom / REFMAR\n");
    output.push_str("# license=Licence Ouverte 2.0 Etalab\n");
    output.push_str(&format!(
        "# product=Donnees horaires validees REFMAR, source {}\n",
        metadata.source
    ));
    output.push_str(&format!("# datum={}\n", metadata.datum));
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

pub(crate) fn read_tidegauge(path: &Path) -> Result<TideGauge, CalError> {
    read_json(path)
}

pub(crate) fn build_pack(input: PackBuildInput<'_>) -> Result<TidePack, CalError> {
    let station = build_station_pack(StationPackBuildInput {
        tidegauge: input.tidegauge,
        station_id: "refmar:3",
        datum: "zero_hydrographique_brest",
        calibration: input.calibration,
        residual_p95_cm: input.residual_p95_cm,
        generated_at: input.generated_at,
        observations_sha256: input.observations_sha256,
        tidegauge_sha256: input.tidegauge_sha256,
        calibration_start: input.calibration_start,
        validation_start: input.validation_start,
        validation_end: input.validation_end,
    })?;
    let pack = TidePack {
        schema_version: SCHEMA_VERSION.to_string(),
        generated_at: input.generated_at.to_string(),
        stations: vec![station],
    };
    pack.validate()?;
    Ok(pack)
}

pub(crate) fn build_station_pack(
    input: StationPackBuildInput<'_>,
) -> Result<StationPack, CalError> {
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
        station_id: input.station_id.to_string(),
        provider_station_id: tidegauge.shom_id.clone(),
        name: title_case_station(&tidegauge.name),
        latitude_deg: LatitudeDegValue::new(tidegauge.latitude),
        longitude_deg: LongitudeDegValue::new(tidegauge.longitude),
        datum: input.datum.to_string(),
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
        data_version: Some(input.generated_at.to_string()),
        valid_from: Some(format_rfc3339(input.calibration_start)),
        valid_until: Some(format_rfc3339(valid_until(input.validation_start)?)),
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
        datum_reference: Some(datum_reference(
            tidegauge,
            input.calibration.z0_m,
            input.calibration_start,
            input.validation_start,
        )),
        residual_benchmark_cm: Some(input.residual_p95_cm),
    };
    TidePack {
        schema_version: SCHEMA_VERSION.to_string(),
        generated_at: input.generated_at.to_string(),
        stations: vec![station.clone()],
    }
    .validate()?;
    Ok(station)
}

fn valid_until(calibration_end: DateTime<Utc>) -> Result<DateTime<Utc>, CalError> {
    let target_year = calibration_end.year() + 5;
    let mut day = calibration_end.day();
    while day >= 1 {
        if let Some(value) = Utc
            .with_ymd_and_hms(
                target_year,
                calibration_end.month(),
                day,
                calibration_end.hour(),
                calibration_end.minute(),
                calibration_end.second(),
            )
            .single()
        {
            return Ok(value);
        }
        day -= 1;
    }
    Err(CalError::InvalidTimestamp(format!(
        "invalid validity boundary for {}",
        format_rfc3339(calibration_end)
    )))
}

fn datum_reference(
    tidegauge: &TideGauge,
    internal_mean_level_m: f64,
    calibration_start: DateTime<Utc>,
    calibration_end: DateTime<Utc>,
) -> DatumReference {
    let vertical_ref = tidegauge.vertical_ref.as_ref();
    let vertical_ref_name = vertical_ref.map(|reference| reference.nom_ref.clone());
    let offset_vertical_ref_m =
        vertical_ref.and_then(|reference| parse_optional_f64(&reference.zh_ref));
    let offset_ign69_m = match (vertical_ref_name.as_deref(), offset_vertical_ref_m) {
        (Some("IGN69"), Some(offset)) => Some(MetersValue::new(offset)),
        _ => None,
    };
    let mean_level_official_m = tidegauge
        .niveau_moyen
        .as_deref()
        .and_then(parse_optional_f64);
    let offset_zh_officiel_m =
        mean_level_official_m.map(|official| MetersValue::new(official - internal_mean_level_m));
    let recent_minus_official_mean_m =
        mean_level_official_m.map(|official| MetersValue::new(internal_mean_level_m - official));
    let status = if offset_vertical_ref_m.is_some() && mean_level_official_m.is_some() {
        "complete"
    } else if offset_vertical_ref_m.is_some() {
        "ram_only"
    } else {
        "incomplete"
    };

    DatumReference {
        source: "Shom / REFMAR completetidegauge RAM".to_string(),
        status: status.to_string(),
        vertical_ref: vertical_ref_name,
        offset_vertical_ref_m: offset_vertical_ref_m.map(MetersValue::new),
        offset_ign69_m,
        offset_zh_officiel_m,
        mean_level_official_m: mean_level_official_m.map(MetersValue::new),
        mean_level_official_epoch: mean_level_official_m.map(|_| {
            "RAM public Shom / REFMAR; epoch not exposed by completetidegauge API".to_string()
        }),
        mean_level_recent_m: Some(MetersValue::new(internal_mean_level_m)),
        mean_level_recent_period: Some(PeriodInfo {
            start: format_rfc3339(calibration_start),
            end: format_rfc3339(calibration_end),
        }),
        recent_minus_official_mean_m,
        note: Some(
            "datum offsets are output transforms only; harmonic constants and z0_m stay internal"
                .to_string(),
        ),
    }
}

fn parse_optional_f64(value: &str) -> Option<f64> {
    value.trim().parse::<f64>().ok()
}

pub(crate) fn build_benchmark(
    validation_samples: &[Observation],
    validation_start: DateTime<Utc>,
    validation_end: DateTime<Utc>,
    observations_sha256: &str,
    generated_at: &str,
) -> TideBenchmark {
    build_station_benchmark(BenchmarkBuildInput {
        validation_samples,
        validation_start,
        validation_end,
        observations_sha256,
        generated_at,
        benchmark_id: "benchmark_brest_v1",
        station_id: "refmar:3",
        provider_station_id: BREST_SHOM_ID,
        station_name: "Brest",
        datum: "zero_hydrographique_brest",
    })
}

pub(crate) fn build_station_benchmark(input: BenchmarkBuildInput<'_>) -> TideBenchmark {
    let by_time = input
        .validation_samples
        .iter()
        .map(|observation| (observation.at, observation.value_m))
        .collect::<BTreeMap<_, _>>();
    let mut samples = Vec::new();
    let mut checksum_input = String::new();
    let mut cursor = input.validation_start;
    while cursor < input.validation_end {
        let observed_m = by_time.get(&cursor).copied();
        samples.push(TideBenchmarkSample {
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
    TideBenchmark {
        schema_version: input.benchmark_id.to_string(),
        benchmark_id: input.benchmark_id.to_string(),
        generated_at: input.generated_at.to_string(),
        station_id: input.station_id.to_string(),
        provider_station_id: input.provider_station_id.to_string(),
        station_name: input.station_name.to_string(),
        datum: input.datum.to_string(),
        product: "Données horaires validées REFMAR, source 4".to_string(),
        source: "Shom / REFMAR".to_string(),
        validation_period: PeriodInfo {
            start: format_rfc3339(input.validation_start),
            end: format_rfc3339(input.validation_end),
        },
        observations_sha256: input.observations_sha256.to_string(),
        checksum_sha256: sha256_hex(checksum_input.as_bytes()),
        samples,
    }
}

pub(crate) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), CalError> {
    write_string(path, &format!("{}\n", serde_json::to_string_pretty(value)?))
}

pub(crate) fn write_string(path: &Path, data: &str) -> Result<(), CalError> {
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

pub(crate) fn sha256_file(path: &Path) -> Result<String, CalError> {
    let data = fs::read(path).map_err(|source| CalError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(sha256_hex(&data))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, CalError> {
    let data = fs::read_to_string(path).map_err(|source| CalError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&data).map_err(CalError::from)
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

pub(crate) fn title_case_station(value: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "amar-calibrate-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn at(value: &str) -> DateTime<Utc> {
        match parse_rfc3339(value) {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        }
    }

    #[test]
    fn observations_csv_round_trips() {
        let path = temp_path("observations.csv");
        let observations = [
            Observation {
                at: at("2026-04-01T00:00:00Z"),
                value_m: 4.1234,
                source: 4,
            },
            Observation {
                at: at("2026-04-01T01:00:00Z"),
                value_m: 4.5678,
                source: 4,
            },
        ];

        match write_observations_csv(&path, observations) {
            Ok(()) => {}
            Err(error) => panic!("{error:?}"),
        }
        let actual = match read_observations_csv(&path) {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        };
        let _ = fs::remove_file(&path);

        assert_eq!(actual.len(), 2);
        assert_eq!(actual[0].at, observations[0].at);
        assert_eq!(actual[0].value_m, 4.123);
        assert_eq!(actual[1].at, observations[1].at);
        assert_eq!(actual[1].value_m, 4.568);
    }

    #[test]
    fn valid_until_clamps_leap_day_to_february_28() {
        let actual = match valid_until(at("2024-02-29T00:00:00Z")) {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        };

        assert_eq!(format_rfc3339(actual), "2029-02-28T00:00:00Z");
    }
}
