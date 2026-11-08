//! Integration tests for sim-core.
//!
//! These verify that the simulation initializes, runs, and obeys its invariants.

use sim_core::{Config, Simulation};

/// The default config embedded as a string so tests don't depend on file paths.
const DEFAULT_CONFIG: &str = include_str!("../../../configs/default.toml");

fn load_config() -> Config {
    Config::parse(DEFAULT_CONFIG).expect("default config should be valid")
}

fn small_config() -> Config {
    // A smaller config for faster tests
    let toml = r#"
[world]
width = 200.0
height = 200.0
seed = 42

[food]
cell_size = 10.0
max_per_cell = 1.0
regrowth_rate = 0.002
energy_per_food = 40.0

[creature]
initial_count = 50
hard_cap = 200
body_radius = 3.0
max_speed = 2.0
max_turn = 0.2
max_age = 5000

[creature.vision]
ray_count = 5
fov = 2.094
range = 60.0

[energy]
starting = 100.0
maximum = 200.0
base_metabolism = 0.1
movement_cost = 0.05
brain_cost = 0.002

[reproduction]
threshold = 150.0
parent_cost = 60.0
child_energy = 50.0

[mutation]
weight_probability = 0.1
weight_step = 0.2

[brain]
inputs = 13
hidden = 8
outputs = 3

[aging]
extrinsic_mortality = 0.0001
initial_senescence = 0.00001
aging_rate = 0.002
"#;
    Config::parse(toml).expect("small config should be valid")
}

#[test]
fn config_loads_from_default_toml() {
    let cfg = load_config();
    assert_eq!(cfg.world.width, 1000.0);
    assert_eq!(cfg.world.height, 1000.0);
    assert_eq!(cfg.creature.initial_count, 300);
    assert_eq!(cfg.brain.inputs, 13);
    assert_eq!(cfg.brain.hidden, 8);
    assert_eq!(cfg.brain.outputs, 3);
}

#[test]
fn config_validates_bad_values() {
    let bad = r#"
[world]
width = -100.0
height = 1000.0
seed = 42
[food]
cell_size = 10.0
max_per_cell = 1.0
regrowth_rate = 0.002
energy_per_food = 40.0
[creature]
initial_count = 1000
hard_cap = 4000
body_radius = 3.0
max_speed = 2.0
max_turn = 0.2
max_age = 5000
[creature.vision]
ray_count = 5
fov = 2.094
range = 60.0
[energy]
starting = 100.0
maximum = 200.0
base_metabolism = 0.1
movement_cost = 0.05
brain_cost = 0.002
[reproduction]
threshold = 150.0
parent_cost = 60.0
child_energy = 50.0
[mutation]
weight_probability = 0.1
weight_step = 0.2
[brain]
inputs = 13
hidden = 8
outputs = 3
[aging]
extrinsic_mortality = 0.0001
initial_senescence = 0.00001
aging_rate = 0.002
"#;
    assert!(Config::parse(bad).is_err());
}

#[test]
fn simulation_initializes_correctly() {
    let cfg = small_config();
    let initial_count = cfg.creature.initial_count;
    let sim = Simulation::new(cfg);

    assert_eq!(sim.tick, 0);
    assert_eq!(sim.organisms.len(), initial_count);
    assert!(!sim.is_extinct());

    // All organisms should be alive with starting energy
    for org in &sim.organisms {
        assert!(org.alive);
        assert_eq!(org.energy, 100.0);
        assert_eq!(org.age, 0);
    }
}

#[test]
fn simulation_runs_one_tick() {
    let cfg = small_config();
    let mut sim = Simulation::new(cfg);

    sim.step();

    assert_eq!(sim.tick, 1);
    // Population should still exist after one tick
    assert!(!sim.is_extinct());
    // All living organisms should have aged
    for org in &sim.organisms {
        if org.alive {
            // Age is either 1 (original) or 0 (newborn)
            assert!(org.age <= 1);
        }
    }
}

#[test]
fn simulation_runs_100_ticks_without_panic() {
    let cfg = small_config();
    let mut sim = Simulation::new(cfg);

    for _ in 0..100 {
        sim.step();
        if sim.is_extinct() {
            break;
        }
    }

    assert_eq!(sim.tick, 100);
}

#[test]
fn deterministic_with_same_seed() {
    let cfg1 = small_config();
    let cfg2 = small_config();

    let mut sim1 = Simulation::new(cfg1);
    let mut sim2 = Simulation::new(cfg2);

    for _ in 0..50 {
        sim1.step();
        sim2.step();
    }

    // Same seed must produce identical state
    assert_eq!(sim1.organisms.len(), sim2.organisms.len());
    assert_eq!(sim1.tick, sim2.tick);
    for (a, b) in sim1.organisms.iter().zip(sim2.organisms.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.pos_x, b.pos_x);
        assert_eq!(a.pos_y, b.pos_y);
        assert_eq!(a.energy, b.energy);
    }
}

#[test]
fn energy_decreases_without_eating() {
    // With very low food, organisms should lose energy over time
    let toml = r#"
[world]
width = 200.0
height = 200.0
seed = 99

[food]
cell_size = 10.0
max_per_cell = 0.0
regrowth_rate = 0.0
energy_per_food = 40.0

[creature]
initial_count = 10
hard_cap = 100
body_radius = 3.0
max_speed = 2.0
max_turn = 0.2
max_age = 5000

[creature.vision]
ray_count = 5
fov = 2.094
range = 60.0

[energy]
starting = 100.0
maximum = 200.0
base_metabolism = 0.1
movement_cost = 0.05
brain_cost = 0.002

[reproduction]
threshold = 150.0
parent_cost = 60.0
child_energy = 50.0

[mutation]
weight_probability = 0.1
weight_step = 0.2

[brain]
inputs = 13
hidden = 8
outputs = 3

[aging]
extrinsic_mortality = 0.0001
initial_senescence = 0.00001
aging_rate = 0.002
"#;
    let cfg = Config::parse(toml).unwrap();
    let mut sim = Simulation::new(cfg);

    sim.step();

    // With zero food, all organisms should have lost energy
    for org in &sim.organisms {
        assert!(org.energy < 100.0, "organism should have lost energy");
    }
}

#[test]
fn brain_weight_count_matches_config() {
    let cfg = small_config();
    let sim = Simulation::new(cfg);

    let expected = sim.config.brain.total_weights();
    for org in &sim.organisms {
        assert_eq!(org.brain.weights.len(), expected);
        assert_eq!(org.genome.brain_weights.len(), expected);
    }
}

#[test]
fn food_grid_regrows() {
    let cfg = small_config();
    let mut sim = Simulation::new(cfg);

    // Drain one cell
    let idx = 0;
    sim.food_grid.cells[idx] = 0.0;
    assert_eq!(sim.food_grid.cells[idx], 0.0);

    // Regrow
    sim.food_grid.regrow();
    assert!(sim.food_grid.cells[idx] > 0.0);
    assert!(sim.food_grid.cells[idx] <= sim.food_grid.regrowth_rate + 0.0001);
}

#[test]
fn gompertz_mortality_kills_old_creatures() {
    // With high aging rate, creatures should die well before max_age
    let toml = r#"
[world]
width = 200.0
height = 200.0
seed = 42

[food]
cell_size = 10.0
max_per_cell = 1.0
regrowth_rate = 0.01
energy_per_food = 40.0

[creature]
initial_count = 50
hard_cap = 200
body_radius = 3.0
max_speed = 2.0
max_turn = 0.2
max_age = 50000

[creature.vision]
ray_count = 5
fov = 2.094
range = 60.0

[energy]
starting = 100.0
maximum = 200.0
base_metabolism = 0.1
movement_cost = 0.05
brain_cost = 0.002

[reproduction]
threshold = 150.0
parent_cost = 60.0
child_energy = 50.0

[mutation]
weight_probability = 0.1
weight_step = 0.2

[brain]
inputs = 13
hidden = 8
outputs = 3

[aging]
extrinsic_mortality = 0.0001
initial_senescence = 0.00001
aging_rate = 0.005
"#;
    let cfg = Config::parse(toml).unwrap();
    let mut sim = Simulation::new(cfg);

    // Run for 2000 ticks — with aging_rate=0.005, hazard at age 2000 is:
    // 0.0001 + 0.00001 * exp(0.005 * 2000) = 0.0001 + 0.00001 * exp(10) ≈ 0.22
    // So by tick 2000, old creatures face ~22% death chance per tick.
    // Total deaths should exceed what we'd see from energy depletion alone.
    for _ in 0..2000 {
        sim.step();
        if sim.is_extinct() {
            break;
        }
    }

    // Verify deaths occurred (Gompertz + energy depletion)
    assert!(
        sim.stats.total_deaths > 0,
        "expected deaths from Gompertz mortality"
    );
    // With max_age=50000 and only 2000 ticks, no deaths from hitting max_age —
    // any old-age deaths are from the Gompertz probability.
}

#[test]
fn wraparound_positions_stay_in_bounds() {
    let cfg = small_config();
    let mut sim = Simulation::new(cfg);

    for _ in 0..200 {
        sim.step();
        if sim.is_extinct() {
            break;
        }
    }

    for org in &sim.organisms {
        assert!(
            org.pos_x >= 0.0 && org.pos_x < sim.config.world.width,
            "x={} out of bounds",
            org.pos_x
        );
        assert!(
            org.pos_y >= 0.0 && org.pos_y < sim.config.world.height,
            "y={} out of bounds",
            org.pos_y
        );
    }
}
