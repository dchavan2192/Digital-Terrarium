use serde_json::Value;
use sim_core::{Config, Simulation};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn evaluates_a_real_export_in_full_without_treating_feed_forward_as_memory() {
    let mut config = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
    config.creature.initial_count = 2;
    let sim = Simulation::new(config);
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/exported-assay-tests")
        .join(format!(
            "terrarium_export_assay_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("donor.json");
    sim.export_genomes(&path).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_sim-runner"))
        .args(["evaluate-delayed-cue", "--input"])
        .arg(&path)
        .args(["--cue-channel", "12", "--blank-delay", "5"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stderr.is_empty());
    let output: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(output["sampling"]["evaluated"], 2);
    assert_eq!(output["source_export"]["tick"], 0);
    assert_eq!(output["protocol"]["cue_channel_zero_based"], 12);
    assert!(
        output["protocol"]["probe_inputs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v.as_f64() == Some(0.0))
    );
    for (record, organism) in output["results"]
        .as_array()
        .unwrap()
        .iter()
        .zip(sim.organisms.iter())
    {
        assert_eq!(record["id"], organism.id);
        assert_eq!(record["enabled_recurrent_connections"], 0);
        assert_eq!(record["intact"]["accuracy"], 0.5);
        assert_eq!(record["same_network_state_cleared"]["accuracy"], 0.5);
    }
    // Finite exported weights can overflow summed activity. Reject them without
    // emitting a partial JSON result (rather than silently serializing null).
    let mut extreme: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let turn = sim.config.brain.inputs + 1;
    extreme["genomes"][0]["genome"]["hidden_nodes"] = serde_json::json!([]);
    extreme["genomes"][0]["genome"]["connections"] = serde_json::json!([
        {"from":0,"to":turn,"weight":f64::MAX,"enabled":true,"methylation":0.0,"recurrent":false},
        {"from":13,"to":turn,"weight":f64::MAX,"enabled":true,"methylation":0.0,"recurrent":false}
    ]);
    let extreme_path = dir.join("extreme.json");
    std::fs::write(&extreme_path, serde_json::to_vec(&extreme).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_sim-runner"))
        .args(["evaluate-delayed-cue", "--input"])
        .arg(&extreme_path)
        .args(["--cue-channel", "12"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("non-finite neural response"));
    // Assay inputs/outputs are deliberately left as reviewable test artifacts.
}

#[test]
fn cli_requires_explicit_channel_and_reports_bad_input_without_json() {
    for args in [
        vec!["evaluate-delayed-cue", "--input", "missing.json"],
        vec![
            "evaluate-delayed-cue",
            "--input",
            "missing.json",
            "--cue-channel",
            "0",
        ],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_sim-runner"))
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
}
