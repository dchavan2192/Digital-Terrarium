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
#[test]
fn replay_retains_all_runs_populations_and_fixed_budget() {
    let dir = fixture();
    let cfg = dir.join("protocol.toml");
    let source = include_str!("../../../configs/visible_acquisition.toml")
        .replace("population = 64", "population = 4")
        .replace("generations = 25", "generations = 2")
        .replace("elites = 8", "elites = 1")
        .replace("seeds = [2000, 2001, 2002]", "seeds = [700, 701]");
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
    ] {
        std::fs::write(&cfg, bad).unwrap();
        assert!(!run(&output, &cfg).status.success());
        assert!(!output.exists());
    }
}
