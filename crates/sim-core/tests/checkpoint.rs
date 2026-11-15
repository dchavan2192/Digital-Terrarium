//! Checkpoints are exact continuation artifacts for the same build and target.
use rand::RngCore;
use sim_core::checkpoint::{CHECKPOINT_VERSION, CheckpointError};
use sim_core::{Config, Simulation};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

fn config() -> Config {
    let mut c = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
    c.world.width = 200.0;
    c.world.height = 200.0;
    c.creature.initial_count = 16;
    c.creature.hard_cap = 50;
    c.creature.max_age = 5000;
    c.aging.initial_senescence = 0.0;
    c.aging.extrinsic_mortality = 0.0;
    c.food.regrowth_rate = 0.005;
    c.food.droughts.spawn_rate = 0.02;
    c.food.droughts.duration = 17;
    c.food.droughts.radius = 30.0;
    c.energy.base_metabolism = 0.01;
    c.energy.connection_cost = 0.00001;
    c.energy.activity_weight_cost = 0.00001;
    c.epigenetics.brain_update_interval = 7;
    c.activity_tracking.window_size = 13;
    c.activity_tracking.lineage_window = 17;
    c.validate().unwrap();
    c
}

fn step(s: &mut Simulation, count: usize) {
    for _ in 0..count {
        s.step();
    }
}

fn artifact(name: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    // Keep generated artifacts inside the repository's ignored output directory.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/checkpoint-tests")
        .join(format!(
            "{name}-{}-{}.json",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
}

#[test]
fn exact_continuation_matches_two_thousand_ticks_and_future_randomness() {
    let mut straight = Simulation::new(config());
    let mut split = Simulation::new(config());
    step(&mut straight, 2000);
    step(&mut split, 1000);
    let path = artifact("split");
    split.save_checkpoint(&path).unwrap();
    let mut resumed = Simulation::load_checkpoint(&path).unwrap();
    // Immediate state too: stats species and expressed weights must survive.
    assert_eq!(
        split.to_checkpoint_bytes().unwrap(),
        resumed.to_checkpoint_bytes().unwrap()
    );
    step(&mut resumed, 1000);
    assert_eq!(
        straight.to_checkpoint_bytes().unwrap(),
        resumed.to_checkpoint_bytes().unwrap()
    );
    for _ in 0..16 {
        assert_eq!(straight.rng.next_u64(), resumed.rng.next_u64());
    }
    step(&mut straight, 3);
    step(&mut resumed, 3);
    assert_eq!(
        straight.to_checkpoint_bytes().unwrap(),
        resumed.to_checkpoint_bytes().unwrap()
    );
}

#[test]
fn refresh_lag_preserves_live_brain_instead_of_rebuilding() {
    let mut c = config();
    c.creature.initial_count = 1;
    c.epigenetics.brain_update_interval = 100;
    c.food.energy_per_food = 0.0;
    c.food.fruit.enabled = false;
    c.niche_construction.enabled = false;
    let mut s = Simulation::new(c);
    s.organisms[0].energy = 10.0;
    let original = s.organisms[0].brain.connections.clone();
    s.step();
    assert!(
        s.organisms[0]
            .genome
            .connections
            .iter()
            .any(|c| c.methylation > 0.0)
    );
    assert_eq!(s.organisms[0].brain.connections, original);
    assert_ne!(
        s.organisms[0].brain.connections,
        s.organisms[0].genome.build_brain().connections
    );
    let restored = Simulation::from_checkpoint_bytes(&s.to_checkpoint_bytes().unwrap()).unwrap();
    assert_eq!(restored.organisms[0].brain.connections, original);
    assert_eq!(
        s.to_checkpoint_bytes().unwrap(),
        restored.to_checkpoint_bytes().unwrap()
    );
}

#[test]
fn initial_and_extinct_snapshots_are_valid() {
    let mut s = Simulation::new(config());
    let restored = Simulation::from_checkpoint_bytes(&s.to_checkpoint_bytes().unwrap()).unwrap();
    assert_eq!(restored.tick, 0);
    s.config.creature.max_age = 1;
    s.config.creature.hard_cap = s.organisms.len();
    s.step();
    assert!(s.is_extinct());
    let mut restored =
        Simulation::from_checkpoint_bytes(&s.to_checkpoint_bytes().unwrap()).unwrap();
    s.step();
    restored.step();
    assert_eq!(
        s.to_checkpoint_bytes().unwrap(),
        restored.to_checkpoint_bytes().unwrap()
    );
}

#[test]
fn version_build_truncation_and_checksum_errors_are_reported() {
    let s = Simulation::new(config());
    let bytes = s.to_checkpoint_bytes().unwrap();
    assert!(Simulation::from_checkpoint_bytes(&bytes[..bytes.len() / 2]).is_err());
    assert!(Simulation::from_checkpoint_bytes(b"not json").is_err());
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["version"] = serde_json::json!(CHECKPOINT_VERSION + 1);
    assert!(matches!(
        Simulation::from_checkpoint_bytes(&serde_json::to_vec(&value).unwrap()),
        Err(CheckpointError::UnsupportedVersion(_))
    ));
    value["version"] = serde_json::json!(CHECKPOINT_VERSION);
    value["core_build"] = serde_json::json!("different-build");
    assert!(matches!(
        Simulation::from_checkpoint_bytes(&serde_json::to_vec(&value).unwrap()),
        Err(CheckpointError::IncompatibleBuild { .. })
    ));
    value = serde_json::from_slice(&bytes).unwrap();
    value["state"]["organisms"][0]["energy"] = serde_json::json!(12.0);
    assert!(matches!(
        Simulation::from_checkpoint_bytes(&serde_json::to_vec(&value).unwrap()),
        Err(CheckpointError::InvalidState(_))
    ));
}

#[test]
fn malformed_live_brain_and_grid_are_rejected_without_panicking() {
    let mut s = Simulation::new(config());
    s.organisms[0].brain.connections[0].0 = u16::MAX;
    assert!(matches!(
        s.to_checkpoint_bytes(),
        Err(CheckpointError::InvalidState(_))
    ));
    let mut s = Simulation::new(config());
    s.organisms[0].genome.num_inputs = 19;
    s.organisms[0].brain.inputs = 19;
    assert!(matches!(
        s.to_checkpoint_bytes(),
        Err(CheckpointError::InvalidState(_))
    ));
    let mut s = Simulation::new(config());
    s.food_grid.cells.pop();
    assert!(matches!(
        s.to_checkpoint_bytes(),
        Err(CheckpointError::InvalidState(_))
    ));
}

#[test]
fn atomic_overwrite_and_genome_export_work() {
    let mut s = Simulation::new(config());
    let path = artifact("atomic");
    s.save_checkpoint(&path).unwrap();
    step(&mut s, 10);
    s.save_checkpoint(&path).unwrap();
    let restored = Simulation::load_checkpoint(&path).unwrap();
    assert_eq!(restored.tick, 10);
    let export = artifact("genomes");
    s.export_genomes(&export).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(export).unwrap()).unwrap();
    assert_eq!(value["format"], "terrarium-genomes");
    assert_eq!(
        value["genomes"].as_array().unwrap().len(),
        s.organisms.len()
    );
    assert!(value["genomes"][0]["genome"]["connections"].is_array());
    assert!(value["genomes"][0]["expressed_brain"]["connections"].is_array());
}
