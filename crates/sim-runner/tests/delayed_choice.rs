//! CLI checks for delayed choice, especially physical-state leakage and controls.
use serde_json::{Value, json};
use sim_core::{Config, Simulation};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn arena() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/delayed_choice.toml")
}

fn artifacts(label: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/delayed-choice-tests")
        .join(format!(
            "{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn command(name: &str, arena: &Path, input: Option<&Path>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sim-runner"));
    cmd.arg(name).arg("--arena").arg(arena);
    if let Some(input) = input {
        cmd.arg("--input").arg(input);
    }
    cmd.output().unwrap()
}

fn successful_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 1);
    serde_json::from_slice(&output.stdout).unwrap()
}

fn export_fixture(dir: &Path) -> PathBuf {
    let mut config = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
    config.creature.initial_count = 2;
    let simulation = Simulation::new(config);
    let path = dir.join("donor.json");
    simulation.export_genomes(&path).unwrap();
    let mut export: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for (index, record) in export["genomes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        let genome = &mut record["genome"];
        let turn = genome["num_inputs"].as_u64().unwrap() + 1;
        genome["hidden_nodes"] = json!([]);
        genome["body_size"] = json!(1.0);
        let mut edges = vec![
            json!({"from":0,"to":turn,"weight":2.0,"enabled":true,"methylation":0.0,"recurrent":false}),
            json!({"from":0,"to":turn+1,"weight":2.0,"enabled":true,"methylation":0.0,"recurrent":false}),
            json!({"from":13,"to":turn,"weight":5.0,"enabled":true,"methylation":0.0,"recurrent":false}),
        ];
        if index == 0 {
            edges.push(json!({"from":turn,"to":turn,"weight":5.0,"enabled":true,"methylation":0.0,"recurrent":true}));
        }
        genome["connections"] = json!(edges);
        // Expressed living brains are intentionally left untouched: the evaluator
        // must rebuild the edited exported genes, not use this stale live brain.
    }
    std::fs::write(&path, serde_json::to_vec(&export).unwrap()).unwrap();
    path
}

fn trials(condition: &Value) -> &[Value] {
    condition["trials"].as_array().unwrap()
}

fn no_response_cue_leakage(condition: &Value) {
    let pair = trials(condition);
    assert_eq!(pair.len(), 2);
    assert_eq!(pair[0]["cue"], -1);
    assert_eq!(pair[1]["cue"], 1);
    assert_eq!(pair[0]["initial_pose"], pair[1]["initial_pose"]);
    assert_eq!(
        pair[0]["trajectory"][0]["inputs"],
        pair[1]["trajectory"][0]["inputs"]
    );
    for trial in pair {
        for held in trial["hold_trace"].as_array().unwrap() {
            assert_eq!(held["pose"], trial["initial_pose"]);
        }
        for step in trial["trajectory"].as_array().unwrap() {
            let inputs = step["inputs"].as_array().unwrap();
            assert_eq!(inputs.len(), 20);
            assert_eq!(inputs[12].as_f64(), Some(0.0));
            assert!(step.get("target").is_none());
            assert!(step.get("reward").is_none());
            assert!(step.get("cue").is_none());
        }
    }
}

#[test]
fn synthetic_cli_scores_controls_and_prevents_physical_state_cue_leakage() {
    let output = command("delayed-choice", &arena(), None);
    let result = successful_json(&output);
    assert_eq!(result["instrument_validated"], true);
    let controls = result["controls"].as_array().unwrap();
    assert_eq!(controls.len(), 3);
    for (name, expected) in [
        ("recurrent_reference", 1.0),
        ("feed_forward_control", 0.5),
        ("same_network_state_cleared", 0.5),
    ] {
        let control = controls.iter().find(|c| c["name"] == name).unwrap();
        assert_eq!(control["mean_reward"].as_f64(), Some(expected));
        no_response_cue_leakage(control);
        if name != "recurrent_reference" {
            let pair = trials(control);
            assert_eq!(pair[0]["trajectory"], pair[1]["trajectory"]);
        }
    }
}

#[test]
fn full_export_rebuilds_genes_and_distinguishes_persistence_from_feed_forward() {
    let dir = artifacts("export-controls");
    let donor_path = export_fixture(&dir);
    let donor: Value = serde_json::from_slice(&std::fs::read(&donor_path).unwrap()).unwrap();
    let output = command("evaluate-delayed-choice", &arena(), Some(&donor_path));
    let result = successful_json(&output);
    let records = result["results"].as_array().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(result["source_export"]["config"], donor["config"]);
    for (index, record) in records.iter().enumerate() {
        assert_eq!(record["id"], donor["genomes"][index]["id"]);
        let evaluation = &record["evaluation"];
        assert_eq!(
            evaluation["intact"]["mean_reward"].as_f64(),
            Some(if index == 0 { 1.0 } else { 0.5 })
        );
        assert_eq!(
            evaluation["same_network_state_cleared"]["mean_reward"].as_f64(),
            Some(0.5)
        );
        for condition in [
            &evaluation["intact"],
            &evaluation["same_network_state_cleared"],
        ] {
            no_response_cue_leakage(condition);
            if index == 1 || condition["clear_state_each_tick"] == true {
                let pair = trials(condition);
                assert_eq!(pair[0]["trajectory"], pair[1]["trajectory"]);
            }
        }
    }
}

#[test]
fn malformed_arena_and_donor_layout_fail_without_partial_json() {
    let dir = artifacts("invalid-inputs");
    let original = std::fs::read_to_string(arena()).unwrap();
    let invalid = original
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("goal_radius =") {
                "goal_radius = -1.0"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(invalid, original.trim_end());
    let bad_arena = dir.join("bad_arena.toml");
    std::fs::write(&bad_arena, invalid).unwrap();
    let output = command("delayed-choice", &bad_arena, None);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let donor_path = export_fixture(&dir);
    let mut donor: Value = serde_json::from_slice(&std::fs::read(&donor_path).unwrap()).unwrap();
    donor["genomes"][0]["genome"]["num_inputs"] = json!(19);
    std::fs::write(&donor_path, serde_json::to_vec(&donor).unwrap()).unwrap();
    let output = command("evaluate-delayed-choice", &arena(), Some(&donor_path));
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("I/O"));
}
