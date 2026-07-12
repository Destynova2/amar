use amar_pack::TideBenchmark;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const FROZEN_SHA256: &str = r#"
d36f445c320c17ba323fbe572e0cb93d45eba846aeff0260ee9d1b3631a6bf6f  fixtures/refmar/benchmark_brest_v1.json
d371be7da00d4324ce92fd016d3601c5669c3d452dd4373a5b3347f5ed80b5e5  fixtures/refmar/benchmark_brest_decennial_v1.json
5c2434a5537b6e2616de681ed9591aa3cdd4c21c8276d071ba2e905bae5e677b  fixtures/refmar/benchmarks/benchmark_arcachon_eyrac_v1.json
6432dcab336f9de3c5f5b56f65830d0189845f1479c4841959b96e9fef33b572  fixtures/refmar/benchmarks/benchmark_boucau_bayonne_v1.json
930c78a79cb7cd556ac277008875de03842fe03f004b36998918de31907cddb5  fixtures/refmar/benchmarks/benchmark_boulogne_sur_mer_v1.json
2cd4d23435704780710cf6d709023cbd8e8df2654e2dac6af73822b7ccc6732a  fixtures/refmar/benchmarks/benchmark_calais_v1.json
aa3f5ba43339252e4344c824829f3794944ced962076b99cf63fd069965853e3  fixtures/refmar/benchmarks/benchmark_cherbourg_v1.json
7d8ac4f72f7ae0bcf1bc1139f5147cffbfefab8771855aacb9dc759dd18f82f4  fixtures/refmar/benchmarks/benchmark_concarneau_v1.json
b2ca6f965b26834d631bdbf9d274d41dfc476ff28078415c8e80c255000032c0  fixtures/refmar/benchmarks/benchmark_dielette_v1.json
a72d5492f2301519f57c914788922e58f2e0eaf68430269d26c865960d669d32  fixtures/refmar/benchmarks/benchmark_dieppe_v1.json
25e3dbb227caf9114f5c95dab486d80e46dcdde4eb5e49de39be44285af47c3d  fixtures/refmar/benchmarks/benchmark_dunkerque_v1.json
622f1be4191b07101aec02e773582f310f11dee5146077a221a97c3f66f83696  fixtures/refmar/benchmarks/benchmark_herbaudiere_v1.json
41601ae44993acfca695a460041f0de537523fb513a4ab0df9f33bea15d856a3  fixtures/refmar/benchmarks/benchmark_la_rochelle_pallice_v1.json
13582265924ed514b9e4bb7e70b8f1b45ddf62d24b2a75322cabe8516d914136  fixtures/refmar/benchmarks/benchmark_le_conquet_v1.json
30fbce3857f0dba249b748301ba25ef59fbb8cf31920513eaa36526713688134  fixtures/refmar/benchmarks/benchmark_le_crouesty_v1.json
3a5711dff057d62ad7dc5e467866cf89bc92956e3957aec13bcafe5a1952b876  fixtures/refmar/benchmarks/benchmark_le_havre_v1.json
42864645e6dca81549b583dfb0304ff3859dd43687ec5538d4be0ef365d943fe  fixtures/refmar/benchmarks/benchmark_le_havre_v2.json
cf51e82704a307f300b263697515e9f554b84990509c8950018f09736e1cbadf  fixtures/refmar/benchmarks/benchmark_les_sables_d_olonne_v1.json
0a9811879063f797ab4af41df5a577e5f0d6a8b49307037e0b6ada6268bcef3e  fixtures/refmar/benchmarks/benchmark_mimizan_v1.json
e716e4a64fc66ac4f4270cc0bae04866694a67353b5b5b49c7ea264365ceb229  fixtures/refmar/benchmarks/benchmark_noumea_numbo_v1.json
509f2aab46c35f5d1bde54535e0ad440179b04c499a4f21dc9583d7ca79b14a2  fixtures/refmar/benchmarks/benchmark_ouistreham_v1.json
b6d33b73916bc5fd2ab5d55b3bede4ef562a442a7942a74664819fef49422696  fixtures/refmar/benchmarks/benchmark_pointe_des_galets_v1.json
35dab7c47758c4af3af084e0a893af5bdd5a04923e15acf9c6ddfff837f674e5  fixtures/refmar/benchmarks/benchmark_port_tudy_v1.json
5a8eeb2b6023557812edfe69172f2558da2b75225d0368619713309f918ee64e  fixtures/refmar/benchmarks/benchmark_roscoff_v1.json
34276362f570fee0b2e6d2b7a6e67d155907636162dd9c0b05cfb07eb23b747c  fixtures/refmar/benchmarks/benchmark_saint_malo_v1.json
fb4e14d78b4b0947b5a864c7c276fa9d8187a110cad2f93bf5f5f3b55ad7f321  fixtures/refmar/benchmarks/benchmark_saint_nazaire_v1.json
4ceb3f4a0b7a343ca46abf007f8ef69521be4d5ec517967e5b3b853bba99a2a8  data/packs/noaa_m0.json
a53b833324780937e57cf43a8d51cef7c28175c280ef239e9b0257663b125435  data/packs/amar-data-brest-experimental.json
b357bb33d67999e1a21aaf96d5168f77c26b58231c05e182897f052cf279971d  data/packs/amar-data-france-experimental.json
"#;

const BREST_OBSERVATIONS_CSV: &str =
    "fixtures/refmar/brest_validated_hourly_2025-01-01_2026-07-01.csv";
const BREST_DECENNIAL_OBSERVATIONS_SHA256: &str =
    "5a086d64fdc612e7e0a8955bc0c2a11c233d5b074467c03fe63a9919d352e2dd";

#[derive(Debug)]
struct FrozenArtifact {
    path: &'static str,
    sha256: &'static str,
}

type StationShaSet = BTreeSet<(String, String)>;

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn expected_artifacts() -> Vec<FrozenArtifact> {
    FROZEN_SHA256
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let Some((sha256, path)) = line.split_once("  ") else {
                panic!("invalid frozen SHA line: {line}");
            };
            Some(FrozenArtifact { path, sha256 })
        })
        .collect()
}

fn expected_artifact_paths() -> BTreeSet<&'static str> {
    expected_artifacts()
        .into_iter()
        .map(|artifact| artifact.path)
        .collect()
}

fn actual_artifact_paths(root: &Path) -> BTreeSet<String> {
    let mut paths = BTreeSet::from([
        "fixtures/refmar/benchmark_brest_v1.json".to_string(),
        "fixtures/refmar/benchmark_brest_decennial_v1.json".to_string(),
        "data/packs/noaa_m0.json".to_string(),
        "data/packs/amar-data-brest-experimental.json".to_string(),
        "data/packs/amar-data-france-experimental.json".to_string(),
    ]);
    for entry in must(fs::read_dir(root.join("fixtures/refmar/benchmarks"))) {
        let entry = must(entry);
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        paths.insert(relative_path(root, &path));
    }
    paths
}

fn relative_path(root: &Path, path: &Path) -> String {
    let relative = match path.strip_prefix(root) {
        Ok(value) => value,
        Err(error) => panic!("{error:?}"),
    };
    relative.to_string_lossy().replace('\\', "/")
}

fn sha256_file(path: &Path) -> String {
    let bytes = must(fs::read(path));
    sha256_bytes(&bytes)
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

fn benchmark_checksum(benchmark: &TideBenchmark) -> String {
    let mut checksum_input = String::new();
    for sample in &benchmark.samples {
        let observed = match (sample.missing, sample.observed_m) {
            (true, None) => "NA".to_string(),
            (false, Some(value)) => format!("{value:.3}"),
            _ => panic!(
                "{} has inconsistent missing/observed_m at {}",
                benchmark.benchmark_id, sample.timestamp
            ),
        };
        checksum_input.push_str(&format!("{},{}\n", sample.timestamp, observed));
    }
    sha256_bytes(checksum_input.as_bytes())
}

fn manifest_indexes(root: &Path) -> (StationShaSet, StationShaSet) {
    let mut observation_shas = BTreeSet::new();
    let mut benchmark_checksums = BTreeSet::new();
    for entry in must(fs::read_dir(root.join("fixtures/refmar/manifests"))) {
        let entry = must(entry);
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let value: Value = must(serde_json::from_slice(&must(fs::read(&path))));
        let station_id = json_string(&value, "station_id", &path);
        let observations_sha256 = json_string(&value, "observations_sha256", &path);
        observation_shas.insert((station_id.clone(), observations_sha256));
        if let Some(benchmark) = value.get("benchmark") {
            let checksum_sha256 = json_string(benchmark, "checksum_sha256", &path);
            benchmark_checksums.insert((station_id, checksum_sha256));
        }
    }
    (observation_shas, benchmark_checksums)
}

fn json_string(value: &Value, field: &str, path: &Path) -> String {
    match value.get(field).and_then(Value::as_str) {
        Some(value) => value.to_string(),
        None => panic!("missing {field} in {}", path.display()),
    }
}

fn frozen_benchmarks(root: &Path) -> BTreeMap<&'static str, TideBenchmark> {
    expected_artifacts()
        .into_iter()
        .filter(|artifact| artifact.path.contains("benchmark"))
        .map(|artifact| {
            let path = root.join(artifact.path);
            let benchmark = must(serde_json::from_slice(&must(fs::read(&path))));
            (artifact.path, benchmark)
        })
        .collect()
}

#[test]
fn frozen_artifact_paths_are_fully_pinned() {
    let root = workspace_root();
    let expected = expected_artifact_paths()
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let actual = actual_artifact_paths(&root);

    assert_eq!(actual, expected);
}

#[test]
fn frozen_artifact_shas_match_pinned_values() {
    let root = workspace_root();
    for artifact in expected_artifacts() {
        let actual = sha256_file(&root.join(artifact.path));
        assert_eq!(actual, artifact.sha256, "{}", artifact.path);
    }
}

#[test]
fn frozen_benchmark_checksums_match_samples() {
    let root = workspace_root();
    for (path, benchmark) in frozen_benchmarks(&root) {
        assert_eq!(benchmark.schema_version, benchmark.benchmark_id, "{path}");
        assert_eq!(
            benchmark_checksum(&benchmark),
            benchmark.checksum_sha256,
            "{path}"
        );
        assert_eq!(benchmark.observations_sha256.len(), 64, "{path}");
    }
}

#[test]
fn frozen_benchmark_observation_shas_match_sources() {
    let root = workspace_root();
    let (manifest_observations, manifest_benchmarks) = manifest_indexes(&root);

    for (path, benchmark) in frozen_benchmarks(&root) {
        match path {
            "fixtures/refmar/benchmark_brest_v1.json" => {
                assert_eq!(
                    sha256_file(&root.join(BREST_OBSERVATIONS_CSV)),
                    benchmark.observations_sha256,
                    "{path}"
                );
            }
            "fixtures/refmar/benchmark_brest_decennial_v1.json" => {
                assert_eq!(
                    benchmark.observations_sha256, BREST_DECENNIAL_OBSERVATIONS_SHA256,
                    "{path}"
                );
            }
            _ => {
                let observation_key = (
                    benchmark.station_id.clone(),
                    benchmark.observations_sha256.clone(),
                );
                assert!(
                    manifest_observations.contains(&observation_key),
                    "{path} observations_sha256 missing from manifests"
                );
                let benchmark_key = (benchmark.station_id.clone(), benchmark.checksum_sha256);
                assert!(
                    manifest_benchmarks.contains(&benchmark_key),
                    "{path} checksum_sha256 missing from manifests"
                );
            }
        }
    }
}
