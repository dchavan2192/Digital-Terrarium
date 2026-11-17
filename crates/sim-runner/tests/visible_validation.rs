//! Bounded CLI fixtures, not evaluation of any archived scientific cohort.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const SUITE: &str = include_str!("../../../configs/visible_validation.toml");
const CASES: [&str; 7] = [
    "training_reference",
    "rotation_minus_45",
    "rotation_plus_45",
    "near",
    "far_forward",
    "narrow",
    "wide",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = root()
        .join("output/visible-validation-cli-tests")
        .join(format!(
            "{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn expect_success(output: Output) {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn one_session(parent: &Path) -> PathBuf {
    let sessions: Vec<_> = std::fs::read_dir(parent)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("session_")
        })
        .collect();
    assert_eq!(sessions.len(), 1);
    sessions[0].clone()
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn source_session(dir: &Path, progress: bool) -> PathBuf {
    let protocol = include_str!("../../../configs/visible_acquisition.toml")
        .replace("population = 64", "population = 4")
        .replace("generations = 25", "generations = 2")
        .replace("elites = 8", "elites = 1")
        .replace("seeds = [2000, 2001, 2002]", "seeds = [700, 701]");
    let protocol = if progress {
        format!("fitness = \"terminal_progress\"\n{protocol}")
    } else {
        protocol
    };
    let path = dir.join("source_protocol.toml");
    std::fs::write(&path, protocol).unwrap();
    let output = dir.join("source_runs");
    expect_success(
        Command::new(env!("CARGO_BIN_EXE_sim-runner"))
            .arg("acquire-visible")
            .arg("--protocol")
            .arg(path)
            .arg("--arena")
            .arg(root().join("configs/delayed_choice.toml"))
            .arg("--output")
            .arg(&output)
            .output()
            .unwrap(),
    );
    one_session(&output)
}

fn validate(source: &Path, suite: &Path, output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sim-runner"))
        .arg("validate-visible")
        .arg("--session")
        .arg(source)
        .arg("--suite")
        .arg(suite)
        .arg("--output")
        .arg(output)
        .output()
        .unwrap()
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(base: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(base, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(base).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

fn without_progress(mut value: Value) -> Value {
    for name in ["intact", "same_network_state_cleared"] {
        value[name]
            .as_object_mut()
            .unwrap()
            .remove("mean_terminal_progress");
        for trial in value[name]["trials"].as_array_mut().unwrap() {
            trial.as_object_mut().unwrap().remove("terminal_progress");
        }
    }
    value
}

fn check_source_metadata(result: &Value, source: &Path, failed_seed: bool) {
    let metadata = &result["source"];
    let canonical = source.canonicalize().unwrap();
    assert_eq!(
        metadata["source_kind"],
        "saved_acquisition_training_selection"
    );
    assert_eq!(
        metadata["session_path"].as_str().unwrap(),
        canonical.to_str().unwrap()
    );
    let manifest = read_json(&source.join("manifest.json"));
    assert_eq!(metadata["training_manifest"], manifest);
    for key in [
        "source_revision",
        "source_dirty",
        "core_build",
        "target",
        "compiler",
        "build_profile",
    ] {
        assert_eq!(metadata["source_provenance"][key], manifest[key]);
    }
    let files = metadata["input_files"].as_array().unwrap();
    assert_eq!(files.len(), if failed_seed { 4 } else { 6 });
    let mut total = 0;
    let mut unique = std::collections::BTreeSet::new();
    for record in files {
        let path = PathBuf::from(record["path"].as_str().unwrap());
        assert!(path.starts_with(&canonical));
        assert!(unique.insert(path.clone()));
        let bytes = std::fs::read(&path).unwrap();
        total += bytes.len() as u64;
        assert_eq!(record["bytes"].as_u64(), Some(bytes.len() as u64));
        let hash = bytes.iter().fold(0xcbf29ce484222325_u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
        });
        assert_eq!(record["fnv1a64_noncryptographic"], format!("{hash:016x}"));
        if failed_seed {
            assert!(!path.starts_with(canonical.join("seed_701")));
        }
    }
    assert_eq!(metadata["total_input_bytes"].as_u64(), Some(total));
}

#[test]
fn validates_all_cases_for_default_and_progress_sources_without_mutating_training() {
    for progress in [false, true] {
        let dir = fixture();
        let source = source_session(&dir, progress);
        let before = snapshot(&source);
        let output = dir.join("validation");
        expect_success(validate(
            &source,
            &root().join("configs/visible_validation.toml"),
            &output,
        ));
        let result = read_json(&one_session(&output).join("result.json"));
        assert_eq!(snapshot(&source), before);
        assert_eq!(result["schema_version"], 1);
        assert_eq!(result["task"], "saved_policy_visible_validation");
        assert_eq!(result["status"], "completed");
        check_source_metadata(&result, &source, false);
        assert_eq!(
            result["source"]["resolved_protocol"]["fitness"],
            if progress {
                "terminal_progress"
            } else {
                "terminal_only"
            }
        );
        assert_eq!(
            result["baseline_instrument_controls"]["instrument_validated"],
            true
        );
        assert_eq!(result["resolved_cases"].as_array().unwrap().len(), 7);
        assert_eq!(result["case_controls"].as_array().unwrap().len(), 7);
        let policies = result["policies"].as_array().unwrap();
        assert_eq!(policies.len(), 2);
        for (policy, seed) in policies.iter().zip([700, 701]) {
            assert_eq!(policy["seed"], seed);
            assert_eq!(policy["status"], "evaluated");
            let training = read_json(&source.join(format!("seed_{seed}/summary.json")));
            assert_eq!(policy["training"], training);
            let cases = policy["cases"].as_array().unwrap();
            assert_eq!(cases.len(), 7);
            assert_eq!(cases[0]["matches_training_replay"], true);
            assert_eq!(
                cases[0]["evaluation"],
                without_progress(training["champion_training_trials"].clone())
            );
            for (case, expected) in cases.iter().zip(CASES) {
                assert_eq!(case["case"], expected);
                let evaluation = &case["evaluation"];
                assert_eq!(evaluation["cue_visible"], true);
                assert_eq!(
                    evaluation["intact"]["trials"],
                    evaluation["same_network_state_cleared"]["trials"]
                );
                assert_eq!(
                    evaluation["intact"]["mean_reward"],
                    evaluation["same_network_state_cleared"]["mean_reward"]
                );
                assert_eq!(evaluation["intact"]["clear_state_each_tick"], false);
                assert_eq!(
                    evaluation["same_network_state_cleared"]["clear_state_each_tick"],
                    true
                );
            }
        }
    }
}

#[test]
fn invalid_suites_and_missing_or_partial_sources_fail_before_output_creation() {
    let dir = fixture();
    let source = source_session(&dir, false);
    let before = snapshot(&source);
    for (index, bad) in [
        format!("unknown_suite_field = 1\n{SUITE}"),
        SUITE.replace(
            "name = \"rotation_minus_45\"",
            "name = \"training_reference\"",
        ),
        SUITE.replace("forward = 3.0", "forward = nan"),
    ]
    .iter()
    .enumerate()
    {
        let suite = dir.join(format!("bad_{index}.toml"));
        std::fs::write(&suite, bad).unwrap();
        let output = dir.join(format!("bad_output_{index}"));
        assert!(!validate(&source, &suite, &output).status.success());
        assert!(!output.exists());
    }
    assert_eq!(snapshot(&source), before);
    let output = dir.join("missing_source_output");
    assert!(
        !validate(
            &dir.join("no_source"),
            &root().join("configs/visible_validation.toml"),
            &output
        )
        .status
        .success()
    );
    assert!(!output.exists());
    let summary_path = source.join("summary.json");
    let mut summary = read_json(&summary_path);
    summary["status"] = json!("running");
    std::fs::write(summary_path, serde_json::to_vec(&summary).unwrap()).unwrap();
    let output = dir.join("partial_source_output");
    assert!(
        !validate(
            &source,
            &root().join("configs/visible_validation.toml"),
            &output
        )
        .status
        .success()
    );
    assert!(!output.exists());
}

#[test]
fn source_failure_is_retained_without_loading_its_corrupt_partial_population() {
    let dir = fixture();
    let source = source_session(&dir, true);
    let summary_path = source.join("summary.json");
    let mut summary = read_json(&summary_path);
    summary["runs"][1] =
        json!({"seed":701,"status":"failed","error":"declared fixture technical failure"});
    summary["technical_failures"] = json!(1);
    summary["status"] = json!("completed_with_failures");
    std::fs::write(summary_path, serde_json::to_vec(&summary).unwrap()).unwrap();
    std::fs::write(
        source.join("seed_701/final_population.json"),
        b"corrupt partial population; must not be read",
    )
    .unwrap();
    let before = snapshot(&source);
    let output = dir.join("validation");
    expect_success(validate(
        &source,
        &root().join("configs/visible_validation.toml"),
        &output,
    ));
    let result = read_json(&one_session(&output).join("result.json"));
    assert_eq!(result["status"], "completed_with_source_failures");
    assert_eq!(snapshot(&source), before);
    let policies = result["policies"].as_array().unwrap();
    assert_eq!(policies.len(), 2);
    assert_eq!(policies[0]["seed"], 700);
    assert_eq!(policies[0]["status"], "evaluated");
    assert_eq!(policies[0]["cases"].as_array().unwrap().len(), 7);
    assert_eq!(policies[1]["seed"], 701);
    assert_eq!(policies[1]["status"], "source_training_failed");
    assert!(policies[1]["cases"].is_null());
    assert_eq!(policies[1]["training"], summary["runs"][1]);
    check_source_metadata(&result, &source, true);
}
