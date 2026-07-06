use crate::common::{
    BREST_SHOM_ID, CalError, Observation, REFMAR_BASE, VALIDATED_HOURLY_SOURCE, format_rfc3339,
    parse_rfc3339,
};
use crate::solve::CalibrationResult;
use amar_core::PredictionMethod;
use amar_pack::{
    BrestBenchmark, BrestBenchmarkSample, LatitudeDegValue, LongitudeDegValue, MetersValue,
    PeriodInfo, SCHEMA_VERSION, SourceInfo, StationPack, TidePack,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub(crate) struct TideGauge {
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

pub(crate) fn read_tidegauge(path: &Path) -> Result<TideGauge, CalError> {
    read_json(path)
}

pub(crate) fn build_pack(input: PackBuildInput<'_>) -> Result<TidePack, CalError> {
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

pub(crate) fn build_benchmark(
    validation_samples: &[Observation],
    validation_start: DateTime<Utc>,
    validation_end: DateTime<Utc>,
    observations_sha256: &str,
    generated_at: &str,
) -> BrestBenchmark {
    let by_time = validation_samples
        .iter()
        .map(|observation| (observation.at, observation.value_m))
        .collect::<BTreeMap<_, _>>();
    let mut samples = Vec::new();
    let mut checksum_input = String::new();
    let mut cursor = validation_start;
    while cursor < validation_end {
        let observed_m = by_time.get(&cursor).copied();
        samples.push(BrestBenchmarkSample {
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
    BrestBenchmark {
        schema_version: "benchmark_brest_v1".to_string(),
        benchmark_id: "benchmark_brest_v1".to_string(),
        generated_at: generated_at.to_string(),
        station_id: "refmar:3".to_string(),
        provider_station_id: BREST_SHOM_ID.to_string(),
        station_name: "Brest".to_string(),
        datum: "zero_hydrographique_brest".to_string(),
        product: "Données horaires validées REFMAR, source 4".to_string(),
        source: "Shom / REFMAR".to_string(),
        validation_period: PeriodInfo {
            start: format_rfc3339(validation_start),
            end: format_rfc3339(validation_end),
        },
        observations_sha256: observations_sha256.to_string(),
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
}
