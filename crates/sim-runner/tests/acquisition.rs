use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/acquisition-cli-tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn run(dir: &Path, config: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_sim-runner"))
        .args(["acquire-visible", "--protocol"])
        .arg(config)
        .arg("--arena")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/delayed_choice.toml"))
        .arg("--output")
        .arg(dir)
        .output()
        .unwrap()
}
fn json(path: PathBuf) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn small_protocol() -> String {
    include_str!("../../../configs/visible_acquisition.toml")
        .replace("population = 64", "population = 4")
        .replace("generations = 25", "generations = 2")
        .replace("elites = 8", "elites = 1")
        .replace("seeds = [2000, 2001, 2002]", "seeds = [700, 701]")
}

fn sessions(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    paths.sort();
    paths
}

fn successful_run(dir: &Path, config: &Path) {
    let output = run(dir, config);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_ranked(population: &[Value], progress_mode: bool) {
    assert_eq!(population.len(), 4);
    for candidate in population {
        assert!(candidate["correct"].as_u64().unwrap() <= 2);
        if progress_mode {
            let progress = candidate["terminal_progress"].as_f64().unwrap();
            assert!(progress.is_finite() && (0.0..=1.0).contains(&progress));
        } else {
            assert!(candidate.get("terminal_progress").is_none());
        }
    }
    for pair in population.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        let correct_left = left["correct"].as_u64().unwrap();
        let correct_right = right["correct"].as_u64().unwrap();
        assert!(
            correct_left >= correct_right,
            "correctness must rank first: {pair:?}"
        );
        if correct_left == correct_right {
            let tied = if progress_mode {
                let a = left["terminal_progress"].as_f64().unwrap();
                let b = right["terminal_progress"].as_f64().unwrap();
                assert!(
                    a >= b,
                    "progress must rank descending within a correctness tier: {pair:?}"
                );
                a == b
            } else {
                true
            };
            if tied {
                assert!(
                    left["id"].as_u64().unwrap() < right["id"].as_u64().unwrap(),
                    "persistent ID must break ties ascending: {pair:?}"
                );
            }
        }
    }
}
#[test]
fn replay_retains_all_runs_populations_and_fixed_budget() {
    let dir = fixture();
    let cfg = dir.join("protocol.toml");
    let source = small_protocol();
    std::fs::write(&cfg, source).unwrap();
    for _ in 0..2 {
        let out = run(&dir, &cfg);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let sessions: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|p| {
            let p = p.unwrap().path();
            p.is_dir().then_some(p)
        })
        .collect();
    assert_eq!(sessions.len(), 2);
    for seed in [700, 701] {
        let name = format!("seed_{seed}");
        for file in [
            "founders.json",
            "final_population.json",
            "generations.jsonl",
        ] {
            assert_eq!(
                std::fs::read(sessions[0].join(&name).join(file)).unwrap(),
                std::fs::read(sessions[1].join(&name).join(file)).unwrap()
            );
        }
        let pop = json(sessions[0].join(&name).join("final_population.json"));
        assert_eq!(pop.as_array().unwrap().len(), 4);
        for c in pop.as_array().unwrap() {
            assert!(
                c["genome"]["connections"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|e| e["recurrent"] == false && e["methylation"] == 0.0)
            );
        }
        let summary = json(sessions[0].join(name).join("summary.json"));
        assert_eq!(summary["evaluations"], 12);
        assert_eq!(summary["champion_training_trials"]["cue_visible"], true);
    }
    assert_eq!(
        json(sessions[0].join("summary.json"))["runs"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn omitted_and_explicit_terminal_only_preserve_identical_search_artifacts() {
    let dir = fixture();
    let omitted = dir.join("omitted.toml");
    let explicit = dir.join("explicit.toml");
    std::fs::write(&omitted, small_protocol()).unwrap();
    std::fs::write(
        &explicit,
        format!("fitness = \"terminal_only\"\n{}", small_protocol()),
    )
    .unwrap();
    let omitted_output = dir.join("omitted");
    let explicit_output = dir.join("explicit");
    successful_run(&omitted_output, &omitted);
    successful_run(&explicit_output, &explicit);
    let omitted_sessions = sessions(&omitted_output);
    let explicit_sessions = sessions(&explicit_output);
    assert_eq!(omitted_sessions.len(), 1);
    assert_eq!(explicit_sessions.len(), 1);
    let omitted_session = &omitted_sessions[0];
    let explicit_session = &explicit_sessions[0];
    for session in [omitted_session, explicit_session] {
        assert_eq!(
            json(session.join("manifest.json"))["protocol"]["fitness"],
            "terminal_only"
        );
    }
    for seed in [700, 701] {
        let seed = format!("seed_{seed}");
        for filename in [
            "founders.json",
            "final_population.json",
            "generations.jsonl",
        ] {
            assert_eq!(
                std::fs::read(omitted_session.join(&seed).join(filename)).unwrap(),
                std::fs::read(explicit_session.join(&seed).join(filename)).unwrap(),
                "explicit default must leave the seeded trajectory unchanged: {seed}/{filename}"
            );
        }
        for filename in ["founders.json", "final_population.json"] {
            let population = json(omitted_session.join(&seed).join(filename));
            assert_ranked(population.as_array().unwrap(), false);
        }
        let lines =
            std::fs::read_to_string(omitted_session.join(&seed).join("generations.jsonl")).unwrap();
        for line in lines.lines() {
            let generation: Value = serde_json::from_str(line).unwrap();
            assert!(generation.get("champion_terminal_progress").is_none());
            assert_ranked(generation["scores"].as_array().unwrap(), false);
        }
        for session in [omitted_session, explicit_session] {
            let summary = json(session.join(&seed).join("summary.json"));
            assert!(summary.get("founder_best_terminal_progress").is_none());
            assert!(summary.get("final_best_terminal_progress").is_none());
            for condition in ["intact", "same_network_state_cleared"] {
                let trials = &summary["champion_training_trials"][condition];
                assert!(trials.get("mean_terminal_progress").is_none());
                assert!(
                    trials["trials"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|trial| trial.get("terminal_progress").is_none())
                );
            }
        }
    }
}

#[test]
fn terminal_progress_replays_rank_correctness_then_progress_then_id() {
    let dir = fixture();
    let config = dir.join("progress.toml");
    std::fs::write(
        &config,
        format!("fitness = \"terminal_progress\"\n{}", small_protocol()),
    )
    .unwrap();
    successful_run(&dir, &config);
    successful_run(&dir, &config);
    let sessions = sessions(&dir);
    assert_eq!(sessions.len(), 2);
    let mut observed_progress_discrimination = false;
    for session in &sessions {
        let manifest = json(session.join("manifest.json"));
        assert_eq!(manifest["protocol"]["fitness"], "terminal_progress");
        assert_eq!(manifest["total_evaluations"], 24);
        let root = json(session.join("summary.json"));
        assert_eq!(root["status"], "completed");
        assert_eq!(root["technical_failures"], 0);
        assert_eq!(root["runs"].as_array().unwrap().len(), 2);
    }
    for seed in [700, 701] {
        let seed_dir = format!("seed_{seed}");
        for filename in [
            "founders.json",
            "final_population.json",
            "generations.jsonl",
        ] {
            assert_eq!(
                std::fs::read(sessions[0].join(&seed_dir).join(filename)).unwrap(),
                std::fs::read(sessions[1].join(&seed_dir).join(filename)).unwrap(),
                "progress mode must replay deterministically: {seed_dir}/{filename}"
            );
        }
        for session in &sessions {
            let seed_path = session.join(&seed_dir);
            let founders = json(seed_path.join("founders.json"));
            let final_population = json(seed_path.join("final_population.json"));
            for population in [&founders, &final_population] {
                assert_ranked(population.as_array().unwrap(), true);
            }
            let curve = std::fs::read_to_string(seed_path.join("generations.jsonl")).unwrap();
            let generations: Vec<Value> = curve
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(generations.len(), 3);
            for (index, generation) in generations.iter().enumerate() {
                assert_eq!(generation["generation"], index);
                let scores = generation["scores"].as_array().unwrap();
                assert_ranked(scores, true);
                assert_eq!(generation["champion_id"], scores[0]["id"]);
                assert_eq!(
                    generation["champion_terminal_progress"],
                    scores[0]["terminal_progress"]
                );
                observed_progress_discrimination |= scores.windows(2).any(|pair| {
                    pair[0]["correct"] == pair[1]["correct"]
                        && pair[0]["terminal_progress"] != pair[1]["terminal_progress"]
                });
            }
            let final_scores = generations.last().unwrap()["scores"].as_array().unwrap();
            let population = final_population.as_array().unwrap();
            for (score, candidate) in final_scores.iter().zip(population) {
                assert_eq!(score["id"], candidate["id"]);
                assert_eq!(score["correct"], candidate["correct"]);
                assert_eq!(score["terminal_progress"], candidate["terminal_progress"]);
            }
            let summary = json(seed_path.join("summary.json"));
            assert_eq!(summary["evaluations"], 12);
            assert_eq!(summary["champion_id"], population[0]["id"]);
            assert_eq!(
                summary["founder_best_terminal_progress"],
                founders[0]["terminal_progress"]
            );
            assert_eq!(
                summary["final_best_terminal_progress"],
                population[0]["terminal_progress"]
            );
            assert_eq!(
                summary["final_best_mean_reward"].as_f64().unwrap(),
                population[0]["correct"].as_u64().unwrap() as f64 / 2.0
            );
            assert_eq!(summary["champion_training_trials"]["cue_visible"], true);
            assert_eq!(
                summary["champion_training_trials"]["intact"]["mean_reward"],
                summary["final_best_mean_reward"]
            );
            for name in ["intact", "same_network_state_cleared"] {
                let condition = &summary["champion_training_trials"][name];
                let trials = condition["trials"].as_array().unwrap();
                assert_eq!(trials.len(), 2);
                let mean = trials
                    .iter()
                    .map(|trial| {
                        let progress = trial["terminal_progress"].as_f64().unwrap();
                        assert!(progress.is_finite() && (0.0..=1.0).contains(&progress));
                        progress
                    })
                    .sum::<f64>()
                    / 2.0;
                assert_eq!(condition["mean_terminal_progress"].as_f64().unwrap(), mean);
                assert_eq!(
                    condition["mean_terminal_progress"],
                    summary["final_best_terminal_progress"]
                );
            }
            assert_eq!(
                summary["champion_training_trials"]["intact"]["trials"],
                summary["champion_training_trials"]["same_network_state_cleared"]["trials"]
            );
        }
    }
    assert!(
        observed_progress_discrimination,
        "the declared fixtures must exercise progress ranking within a correctness tier"
    );
}
#[test]
fn invalid_protocol_creates_no_output() {
    let dir = fixture();
    let cfg = dir.join("bad.toml");
    let output = dir.join("outputs");
    let source = include_str!("../../../configs/visible_acquisition.toml");
    for bad in [
        source.replace("generations = 25", "generations = 4294967295"),
        source.replace(
            "add_recurrent_probability = 0.0",
            "add_recurrent_probability = 0.3",
        ),
        source.replace("trait_probability = 0.0", "trait_probability = 0.1"),
        source.replace("seeds = [2000, 2001, 2002]", "seeds = [2000, 2000]"),
        source.replace("weight_step = 0.2", "weight_step = 0.2\nunknown = 1"),
        format!("fitness = \"unknown_fitness\"\n{source}"),
    ] {
        std::fs::write(&cfg, bad).unwrap();
        assert!(!run(&output, &cfg).status.success());
        assert!(!output.exists());
    }
}
