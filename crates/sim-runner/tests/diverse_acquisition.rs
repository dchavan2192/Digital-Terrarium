//! Tiny CLI fixtures only; no archived policies or scientific campaign outcomes.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = root()
        .join("output/diverse-acquisition-cli-tests")
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

fn protocol() -> toml::Value {
    let mut config: toml::Value = include_str!("../../../configs/visible_diversity.toml")
        .parse()
        .unwrap();
    for (key, value) in [
        ("population", 4),
        ("generations", 2),
        ("elites", 1),
        ("tournament", 2),
    ] {
        config[key] = toml::Value::Integer(value);
    }
    config["seeds"] =
        toml::Value::Array(vec![toml::Value::Integer(700), toml::Value::Integer(701)]);
    config
}

fn run(protocol: &Path, output: &Path) -> Output {
    run_with_arena(
        protocol,
        &root().join("configs/delayed_choice.toml"),
        output,
    )
}

fn run_with_arena(protocol: &Path, arena: &Path, output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sim-runner"))
        .arg("acquire-visible-diversity")
        .arg("--protocol")
        .arg(protocol)
        .arg("--arena")
        .arg(arena)
        .arg("--output")
        .arg(output)
        .output()
        .unwrap()
}

fn session(output: &Path) -> PathBuf {
    let found: Vec<_> = std::fs::read_dir(output)
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
    assert_eq!(found.len(), 1);
    found[0].clone()
}

fn json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn population(path: &Path) -> Vec<Value> {
    json(path).as_array().unwrap().clone()
}

fn assert_file_identity(session: &Path, identity: &Value) {
    let bytes = std::fs::read(session.join(identity["path"].as_str().unwrap())).unwrap();
    let fingerprint = bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
    assert_eq!(identity["bytes"], bytes.len());
    assert_eq!(
        identity["fnv1a64_noncryptographic"],
        format!("{fingerprint:016x}")
    );
}

fn ranked(scores: &[Value]) {
    assert_eq!(scores.len(), 4);
    for candidate in scores {
        let cases = candidate["case_scores"].as_array().unwrap();
        assert_eq!(cases.len(), 3);
        let correct = cases
            .iter()
            .map(|case| {
                let count = case["correct"].as_u64().unwrap();
                assert!(count <= 2);
                count
            })
            .sum::<u64>();
        assert_eq!(candidate["correct"], correct);
        let progress = cases
            .iter()
            .map(|case| {
                let value = case["terminal_progress"].as_f64().unwrap();
                assert!(value.is_finite() && (0.0..=1.0).contains(&value));
                value
            })
            .sum::<f64>()
            / 3.0;
        assert!((candidate["terminal_progress"].as_f64().unwrap() - progress).abs() <= 1e-12);
    }
    for pair in scores.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        let ac = a["correct"].as_u64().unwrap();
        let bc = b["correct"].as_u64().unwrap();
        let ap = a["terminal_progress"].as_f64().unwrap();
        let bp = b["terminal_progress"].as_f64().unwrap();
        assert!(
            ac > bc
                || (ac == bc
                    && (ap > bp
                        || (ap == bp && a["id"].as_u64().unwrap() < b["id"].as_u64().unwrap())))
        );
    }
}

fn assert_ff_replay(case: &Value) {
    assert_eq!(case["status"], "completed");
    let evaluation = &case["evaluation"];
    assert_eq!(evaluation["cue_visible"], true);
    let trials = evaluation["intact"]["trials"].as_array().unwrap();
    assert_eq!(trials.len(), 2);
    assert_eq!(trials[0]["cue"], -1);
    assert_eq!(trials[1]["cue"], 1);
    assert_eq!(
        evaluation["intact"]["trials"],
        evaluation["same_network_state_cleared"]["trials"]
    );
}

#[test]
fn matched_founders_equal_training_counts_deterministic_replay_and_all_holdouts() {
    let dir = fixture();
    let config = protocol();
    let config_path = dir.join("protocol.toml");
    std::fs::write(&config_path, toml::to_string(&config).unwrap()).unwrap();
    let mut sessions = Vec::new();
    for name in ["first", "replay"] {
        let output = dir.join(name);
        let result = run(&config_path, &output);
        assert!(
            result.status.success(),
            "stdout={} stderr={}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        sessions.push(session(&output));
    }
    let holdouts: Vec<_> = config["holdouts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(holdouts.len(), 6);
    for session in &sessions {
        let manifest = json(session.join("manifest.json"));
        assert_eq!(manifest["isolated_diet_mutation_probability"], 0.0);
        assert_eq!(manifest["budget"]["training_candidate_evaluations"], 48);
        assert_eq!(manifest["budget"]["scheduled_training_trials"], 288);
        assert_eq!(manifest["budget"]["training_trials_per_candidate"], 6);
        assert!(manifest["budget"]["max_neural_steps"].as_u64().unwrap() <= 3_000_000);
        assert!(manifest["budget"]["max_edge_steps"].as_u64().unwrap() <= 1_000_000_000);
        let summary = json(session.join("summary.json"));
        assert_eq!(summary["status"], "completed");
        assert_eq!(summary["technical_failures"], 0);
        assert_eq!(summary["runs"].as_array().unwrap().len(), 4);
        assert_eq!(summary["selection_lock"]["path"], "selection_lock.json");
        assert_file_identity(session, &summary["selection_lock"]);
        let lock = json(session.join("selection_lock.json"));
        assert_eq!(lock["status"], "locked");
        assert_eq!(lock["expected_searches"], 4);
        assert_eq!(lock["selected_champions"], 4);
        assert_eq!(lock["failed_searches"], 0);
        let locked = lock["runs"].as_array().unwrap();
        assert_eq!(locked.len(), 4);
        for (record, (seed, arm)) in locked.iter().zip([
            (700, "fixed"),
            (700, "varied"),
            (701, "varied"),
            (701, "fixed"),
        ]) {
            assert_eq!(record["seed"], seed);
            assert_eq!(record["arm"], arm);
            assert_eq!(record["status"], "training_completed");
            let directory = session.join(arm).join(format!("seed_{seed}"));
            let selection = json(directory.join("selection.json"));
            assert_eq!(record["champion_id"], selection["champion_id"]);
            assert_eq!(record["correct"], selection["correct"]);
            assert_eq!(record["terminal_progress"], selection["terminal_progress"]);
            let artifacts = record["artifacts"].as_array().unwrap();
            let expected: BTreeSet<_> = [
                "founder_genomes.json",
                "founders.json",
                "generations.jsonl",
                "final_population.json",
                "innovations.json",
                "selection.json",
                "training_summary.json",
            ]
            .iter()
            .map(|name| format!("{arm}/seed_{seed}/{name}"))
            .collect();
            assert_eq!(
                artifacts
                    .iter()
                    .map(|identity| identity["path"].as_str().unwrap().to_owned())
                    .collect::<BTreeSet<_>>(),
                expected
            );
            assert_eq!(artifacts.len(), 7);
            for identity in artifacts {
                assert_file_identity(session, identity);
            }
            let training = json(directory.join("training_summary.json"));
            assert_eq!(training["status"], "training_completed");
            assert!(training.get("holdout_cases").is_none());
            assert_eq!(training["champion_id"], selection["champion_id"]);
            let final_summary = json(directory.join("summary.json"));
            assert_eq!(final_summary["selection_lock"], summary["selection_lock"]);
        }
    }
    assert_eq!(
        std::fs::read(sessions[0].join("selection_lock.json")).unwrap(),
        std::fs::read(sessions[1].join("selection_lock.json")).unwrap()
    );
    let mut saw_partial_champion = false;
    for seed in [700, 701] {
        let relative = format!("seed_{seed}");
        let fixed = sessions[0].join("fixed").join(&relative);
        let varied = sessions[0].join("varied").join(&relative);
        assert_eq!(
            json(fixed.join("founder_genomes.json")),
            json(varied.join("founder_genomes.json"))
        );
        let by_id = |path: &Path| -> BTreeMap<u64, Value> {
            population(&path.join("founders.json"))
                .iter()
                .map(|candidate| {
                    (
                        candidate["id"].as_u64().unwrap(),
                        candidate["genome"].clone(),
                    )
                })
                .collect()
        };
        assert_eq!(by_id(&fixed), by_id(&varied));
        for arm in ["fixed", "varied"] {
            let seed_path = sessions[0].join(arm).join(&relative);
            for filename in [
                "founder_genomes.json",
                "founders.json",
                "final_population.json",
                "generations.jsonl",
                "innovations.json",
                "selection.json",
                "training_summary.json",
            ] {
                assert_eq!(
                    std::fs::read(seed_path.join(filename)).unwrap(),
                    std::fs::read(sessions[1].join(arm).join(&relative).join(filename)).unwrap(),
                    "{arm}/{relative}/{filename}"
                );
            }
            let founders = population(&seed_path.join("founders.json"));
            let final_population = population(&seed_path.join("final_population.json"));
            ranked(&founders);
            ranked(&final_population);
            let names: Vec<_> = if arm == "fixed" {
                vec!["reference_copy_1", "reference_copy_2", "reference_copy_3"]
            } else {
                vec!["reference", "near", "narrow"]
            };
            for candidate in founders.iter().chain(&final_population) {
                let scores = candidate["case_scores"].as_array().unwrap();
                assert_eq!(
                    scores
                        .iter()
                        .map(|score| score["case"].as_str().unwrap())
                        .collect::<Vec<_>>(),
                    names
                );
                if arm == "fixed" {
                    assert_eq!(candidate["correct"].as_u64().unwrap() % 3, 0);
                    for score in &scores[1..] {
                        assert_eq!(score["correct"], scores[0]["correct"]);
                        assert_eq!(score["terminal_progress"], scores[0]["terminal_progress"]);
                    }
                }
            }
            for candidate in &final_population {
                assert!(
                    candidate["genome"]["connections"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|edge| edge["recurrent"] == false && edge["methylation"] == 0.0)
                );
            }
            let generations: Vec<Value> =
                std::fs::read_to_string(seed_path.join("generations.jsonl"))
                    .unwrap()
                    .lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();
            assert_eq!(generations.len(), 3);
            for (index, generation) in generations.iter().enumerate() {
                assert_eq!(generation["generation"], index);
                let scores = generation["scores"].as_array().unwrap();
                ranked(scores);
                assert_eq!(generation["champion_id"], scores[0]["id"]);
                if index > 0 {
                    let previous = generations[index - 1]["scores"].as_array().unwrap();
                    let previous_ids: BTreeSet<_> =
                        previous.iter().map(|c| c["id"].as_u64().unwrap()).collect();
                    let elite = scores
                        .iter()
                        .find(|c| c["id"] == previous[0]["id"])
                        .unwrap();
                    assert_eq!(
                        elite, &previous[0],
                        "the single retained elite must be reevaluated unchanged"
                    );
                    let children: Vec<_> = scores
                        .iter()
                        .filter(|c| !previous_ids.contains(&c["id"].as_u64().unwrap()))
                        .collect();
                    assert_eq!(children.len(), 3);
                    for child in children {
                        assert_eq!(child["born_generation"], index);
                        assert!(previous_ids.contains(&child["parent_id"].as_u64().unwrap()));
                    }
                }
            }
            let founder_diets: BTreeMap<_, _> = founders
                .iter()
                .map(|candidate| {
                    (
                        candidate["id"].as_u64().unwrap(),
                        candidate["genome"]["diet_preference"].as_f64().unwrap(),
                    )
                })
                .collect();
            let mut parents = BTreeMap::new();
            for generation in &generations {
                for candidate in generation["scores"].as_array().unwrap() {
                    let id = candidate["id"].as_u64().unwrap();
                    let parent = candidate["parent_id"].as_u64();
                    if let Some(previous) = parents.insert(id, parent) {
                        assert_eq!(previous, parent, "persistent candidate lineage changed");
                    }
                }
            }
            for candidate in &final_population {
                let mut ancestor = candidate["id"].as_u64().unwrap();
                let mut visited = BTreeSet::new();
                while let Some(parent) = parents[&ancestor] {
                    assert!(visited.insert(ancestor), "lineage contains a cycle");
                    ancestor = parent;
                }
                let founder_diet = founder_diets[&ancestor];
                assert_eq!(
                    candidate["genome"]["diet_preference"].as_f64().unwrap(),
                    founder_diet,
                    "weight-only search must preserve diet from the traced founder"
                );
            }
            let result = json(seed_path.join("summary.json"));
            assert_eq!(result["arm"], arm);
            assert_eq!(result["seed"], seed);
            assert_eq!(result["champion_id"], final_population[0]["id"]);
            assert_eq!(result["training_correct"], final_population[0]["correct"]);
            assert_eq!(result["evaluations"], 12);
            assert_eq!(result["scheduled_training_trials"], 72);
            let selected = json(seed_path.join("selection.json"));
            assert_eq!(selected["champion_id"], final_population[0]["id"]);
            assert_eq!(selected["correct"], final_population[0]["correct"]);
            assert_eq!(
                selected["terminal_progress"],
                final_population[0]["terminal_progress"]
            );
            assert!(
                selected["basis"]
                    .as_str()
                    .unwrap()
                    .contains("training rank only")
            );
            assert!(selected.get("holdout_cases").is_none());
            saw_partial_champion |= result["training_correct"].as_u64().unwrap() < 6;
            let training = result["training_cases"].as_array().unwrap();
            assert_eq!(training.len(), 3);
            let validation = result["holdout_cases"].as_array().unwrap();
            assert_eq!(validation.len(), 6);
            assert_eq!(
                validation
                    .iter()
                    .map(|case| case["case"].as_str().unwrap().to_owned())
                    .collect::<Vec<_>>(),
                holdouts
            );
            for case in training.iter().chain(validation) {
                assert_ff_replay(case);
            }
        }
    }
    assert!(
        saw_partial_champion,
        "fixture must retain at least one partially successful training-selected champion"
    );
}

#[test]
fn invalid_or_overbudget_protocol_rejected_before_any_output_directory() {
    let dir = fixture();
    let source = protocol();
    let mut faults = Vec::new();
    for (key, value) in [("population", 65), ("generations", i64::MAX), ("hidden", 9)] {
        let mut bad = source.clone();
        bad[key] = toml::Value::Integer(value);
        faults.push(bad);
    }
    let mut duplicate_seed = source.clone();
    duplicate_seed["seeds"] = toml::Value::Array(vec![toml::Value::Integer(700); 2]);
    faults.push(duplicate_seed);
    let mut recurrence = source.clone();
    recurrence["mutation"]["add_recurrent_probability"] = toml::Value::Float(0.3);
    faults.push(recurrence);
    let mut unknown = source.clone();
    unknown
        .as_table_mut()
        .unwrap()
        .insert("unknown".into(), toml::Value::Integer(1));
    faults.push(unknown);
    let mut duplicate_holdout = source.clone();
    duplicate_holdout["holdouts"][1] = duplicate_holdout["holdouts"][0].clone();
    faults.push(duplicate_holdout);
    for (index, bad) in faults.iter().enumerate() {
        let path = dir.join(format!("bad_{index}.toml"));
        let output = dir.join(format!("bad_output_{index}"));
        std::fs::write(&path, toml::to_string(bad).unwrap()).unwrap();
        assert!(!run(&path, &output).status.success());
        assert!(!output.exists());
    }
}

#[test]
fn valid_scalar_limits_still_reject_excessive_work_before_outputs() {
    let dir = fixture();
    let protocol = dir.join("full_protocol.toml");
    std::fs::write(
        &protocol,
        include_str!("../../../configs/visible_diversity.toml"),
    )
    .unwrap();
    let mut arena: toml::Value = include_str!("../../../configs/delayed_choice.toml")
        .parse()
        .unwrap();
    arena["horizon"] = toml::Value::Integer(10_000);
    let arena_path = dir.join("long_arena.toml");
    std::fs::write(&arena_path, toml::to_string(&arena).unwrap()).unwrap();
    let output = dir.join("too_much_work");
    let result = run_with_arena(&protocol, &arena_path, &output);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("budget"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!output.exists());
}
