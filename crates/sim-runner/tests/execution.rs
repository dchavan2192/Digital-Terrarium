//! CLI regressions: invalid designs cannot start jobs; results preserve provenance.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
static ID: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn output(name: &str) -> PathBuf {
    root().join("output/runner-tests").join(format!(
        "{name}_{}_{}",
        std::process::id(),
        ID.fetch_add(1, Ordering::Relaxed)
    ))
}
fn run(args: &[&str], output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sim-runner"))
        .current_dir(root())
        .args(args)
        .arg("--output")
        .arg(output)
        .output()
        .unwrap()
}
#[test]
fn invalid_generated_job_is_rejected_before_artifacts() {
    let out = output("invalid-config");
    let result = run(
        &[
            "sweep",
            "--param",
            "energy.base_metabolism",
            "--values=-1",
            "--seeds",
            "1",
            "--ticks",
            "1",
        ],
        &out,
    );
    assert!(!result.status.success());
    assert!(!out.exists());
    // Fixed overrides are also validated after mutation.
    let result = run(
        &[
            "sweep",
            "--param",
            "food.regrowth_rate",
            "--values",
            "0.002",
            "--set",
            "creature.hard_cap=1",
            "--seeds",
            "1",
            "--ticks",
            "1",
        ],
        &out,
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("invalid generated job"));
    assert!(!out.exists());
}
#[test]
fn zero_intervals_and_seeds_are_rejected() {
    for args in [
        vec!["--log-interval", "0", "--ticks", "1"],
        vec![
            "sweep",
            "--param",
            "food.regrowth_rate",
            "--values",
            "0.002",
            "--seeds",
            "0",
        ],
    ] {
        let out = output("zero");
        assert!(!run(&args, &out).status.success());
        assert!(!out.exists());
    }
}
#[test]
fn repeated_invocations_preserve_both_completed_runs() {
    let out = output("repeat");
    for _ in 0..2 {
        let result = run(&["--ticks", "3", "--log-interval", "2"], &out);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let sessions: Vec<_> = std::fs::read_dir(&out).unwrap().collect();
    assert_eq!(sessions.len(), 2);
    for session in sessions {
        let path = session.unwrap().path();
        let manifest: toml::Value =
            toml::from_str(&std::fs::read_to_string(path.join("manifest_seed42.toml")).unwrap())
                .unwrap();
        assert_eq!(manifest["status"].as_str(), Some("completed"));
        assert_eq!(manifest["completed_ticks"].as_integer(), Some(3));
        assert_eq!(manifest["config"]["world"]["seed"].as_integer(), Some(42));
        assert!(manifest["source_revision"].as_str().unwrap().len() >= 7);
        let mut csv = csv::Reader::from_path(path.join("stats_seed42.csv")).unwrap();
        let rows: Vec<_> = csv.records().map(Result::unwrap).collect();
        let tick = csv
            .headers()
            .unwrap()
            .iter()
            .position(|x| x == "tick")
            .unwrap();
        assert_eq!(&rows.last().unwrap()[tick], "3");
        assert_eq!(
            rows.len(),
            2,
            "log tick 2 plus final tick 3, no bogus zero row"
        );
    }
}

fn only_session(output: &Path) -> PathBuf {
    let mut sessions = std::fs::read_dir(output)
        .unwrap()
        .map(|e| e.unwrap().path());
    let path = sessions.next().unwrap();
    assert!(sessions.next().is_none());
    path
}

#[test]
fn resume_matches_uninterrupted_checkpoint_and_preserves_provenance() {
    let straight = output("straight");
    let first = output("split-first");
    let resumed = output("split-resumed");
    let args = [
        "--ticks",
        "12",
        "--log-interval",
        "5",
        "--checkpoint-every",
        "5",
        "--export-genomes-every",
        "5",
    ];
    let result = run(&args, &straight);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result = run(&["--ticks", "5", "--checkpoint-every", "5"], &first);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let first_session = only_session(&first);
    let checkpoint = first_session.join("checkpoint_seed42_tick5.json");
    let result = run(
        &[
            "--resume",
            checkpoint.to_str().unwrap(),
            "--ticks",
            "12",
            "--log-interval",
            "5",
            "--checkpoint-every",
            "5",
            "--export-genomes-every",
            "5",
        ],
        &resumed,
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let straight_session = only_session(&straight);
    let resumed_session = only_session(&resumed);
    assert_eq!(
        std::fs::read(straight_session.join("checkpoint_seed42_tick12.json")).unwrap(),
        std::fs::read(resumed_session.join("checkpoint_seed42_tick12.json")).unwrap(),
        "resume must preserve complete state and RNG"
    );
    for tick in [5, 10, 12] {
        assert!(
            straight_session
                .join(format!("genomes_seed42_tick{tick}.json"))
                .exists()
        );
    }
    assert!(resumed_session.join("genomes_seed42_tick12.json").exists());
    let manifest: toml::Value = toml::from_str(
        &std::fs::read_to_string(resumed_session.join("manifest_seed42.toml")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["starting_tick"].as_integer(), Some(5));
    assert_eq!(manifest["completed_ticks"].as_integer(), Some(12));
    assert_eq!(manifest["simulated_ticks"].as_integer(), Some(7));
    assert_eq!(manifest["resumed_from"].as_str(), checkpoint.to_str());
    assert_eq!(
        manifest["core_build_id"].as_str(),
        Some(sim_core::checkpoint::CORE_BUILD_ID)
    );
    let invalid = output("resume-past-target");
    let result = run(
        &["--resume", checkpoint.to_str().unwrap(), "--ticks", "5"],
        &invalid,
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("absolute target"));
    assert!(!invalid.exists());
}

#[test]
fn invalid_resume_or_snapshot_interval_cannot_start_an_experiment() {
    let out = output("invalid-checkpoint");
    let result = run(&["--resume", "does-not-exist.json", "--ticks", "12"], &out);
    assert!(!result.status.success());
    assert!(!out.exists());
    let result = run(&["--checkpoint-every", "0", "--ticks", "1"], &out);
    assert!(!result.status.success());
    assert!(!out.exists());
    let result = run(&["--export-genomes-every", "0", "--ticks", "1"], &out);
    assert!(!result.status.success());
    assert!(!out.exists());
}
