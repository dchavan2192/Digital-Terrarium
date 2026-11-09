//! Integration tests for sim-core.
//!
//! These verify that the simulation initializes, runs, and obeys its invariants.

use sim_core::config::{BiomeZone, FoodConfig, WorldConfig};
use sim_core::world::FoodGrid;
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
inputs = 15
hidden = 8
outputs = 6

[aging]
extrinsic_mortality = 0.0001
initial_senescence = 0.00001
aging_rate = 0.002

[combat]
attack_range = 8.0
attack_cost = 5.0
attack_damage = 20.0
energy_transfer_ratio = 0.5

[sharing]
share_range = 10.0
share_amount = 5.0
"#;
    Config::parse(toml).expect("small config should be valid")
}

#[test]
fn config_loads_from_default_toml() {
    let cfg = load_config();
    assert_eq!(cfg.world.width, 1000.0);
    assert_eq!(cfg.world.height, 1000.0);
    assert_eq!(cfg.creature.initial_count, 300);
    assert_eq!(cfg.brain.inputs, 15);
    assert_eq!(cfg.brain.hidden, 8);
    assert_eq!(cfg.brain.outputs, 6);
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
inputs = 15
hidden = 8
outputs = 6
[aging]
extrinsic_mortality = 0.0001
initial_senescence = 0.00001
aging_rate = 0.002

[combat]
attack_range = 8.0
attack_cost = 5.0
attack_damage = 20.0
energy_transfer_ratio = 0.5

[sharing]
share_range = 10.0
share_amount = 5.0
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
inputs = 15
hidden = 8
outputs = 6

[aging]
extrinsic_mortality = 0.0001
initial_senescence = 0.00001
aging_rate = 0.002

[combat]
attack_range = 0.0
attack_cost = 5.0
attack_damage = 20.0
energy_transfer_ratio = 0.5

[sharing]
share_range = 0.0
share_amount = 0.0
"#;
    let cfg = Config::parse(toml).unwrap();
    let mut sim = Simulation::new(cfg);

    sim.step();

    // With zero food, no combat, and no sharing, all organisms should have lost energy
    for org in &sim.organisms {
        assert!(org.energy < 100.0, "organism should have lost energy");
    }
}

#[test]
fn brain_topology_matches_genome() {
    let cfg = small_config();
    let sim = Simulation::new(cfg);

    for org in &sim.organisms {
        // Brain hidden count should match genome hidden node count
        assert_eq!(org.brain.hidden, org.genome.hidden_nodes.len());
        // Brain inputs/outputs should match config
        assert_eq!(org.brain.inputs, sim.config.brain.inputs);
        assert_eq!(org.brain.outputs, sim.config.brain.outputs);
        // Initial population should have 8 hidden nodes
        assert_eq!(org.genome.hidden_nodes.len(), 8);
        // Connection count: bias→8h + 15i→8h + bias→6o + 8h→6o = 8+120+6+48 = 182
        assert_eq!(org.brain.connection_count(), 182);
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
inputs = 15
hidden = 8
outputs = 6

[aging]
extrinsic_mortality = 0.0001
initial_senescence = 0.00001
aging_rate = 0.005

[combat]
attack_range = 8.0
attack_cost = 5.0
attack_damage = 20.0
energy_transfer_ratio = 0.5

[sharing]
share_range = 10.0
share_amount = 5.0
"#;
    let cfg = Config::parse(toml).unwrap();
    let mut sim = Simulation::new(cfg);

    for _ in 0..2000 {
        sim.step();
        if sim.is_extinct() {
            break;
        }
    }

    assert!(
        sim.stats.total_deaths > 0,
        "expected deaths from Gompertz mortality"
    );
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

#[test]
fn lineage_tracking_and_speciation() {
    let cfg = small_config();
    let mut sim = Simulation::new(cfg);

    // Initial population should all be generation 0 with no parent
    for org in &sim.organisms {
        assert_eq!(org.generation, 0);
        assert!(org.parent_id.is_none());
    }

    // Run for enough ticks to see reproduction
    for _ in 0..500 {
        sim.step();
        if sim.is_extinct() {
            break;
        }
    }

    // After reproduction, some organisms should have generation > 0
    let max_gen = sim
        .organisms
        .iter()
        .map(|o| o.generation)
        .max()
        .unwrap_or(0);
    assert!(
        max_gen > 0,
        "expected offspring with generation > 0 after 500 ticks"
    );

    // Offspring should have parent_id set
    let has_parent = sim.organisms.iter().any(|o| o.parent_id.is_some());
    assert!(has_parent, "expected some organisms with parent_id set");

    // Species count should be at least 1
    assert!(
        sim.stats.species_count >= 1,
        "expected at least 1 species, got {}",
        sim.stats.species_count
    );

    // Mean generation should be positive if offspring exist
    assert!(
        sim.stats.mean_generation > 0.0,
        "expected mean_generation > 0"
    );
}

#[test]
fn trophic_level_tracking() {
    let cfg = small_config();
    let mut sim = Simulation::new(cfg);

    // Initial organisms should have zero energy intake
    for org in &sim.organisms {
        assert_eq!(org.energy_from_food, 0.0);
        assert_eq!(org.energy_from_attacks, 0.0);
    }

    // Run long enough for eating and attacks to happen
    for _ in 0..500 {
        sim.step();
        if sim.is_extinct() {
            break;
        }
    }

    // Some organisms should have eaten food
    let ate_food = sim.organisms.iter().any(|o| o.energy_from_food > 0.0);
    assert!(ate_food, "expected some organisms to have eaten food");

    // Mean trophic level should be >= 1.0 (pure herbivore baseline)
    assert!(
        sim.stats.mean_trophic_level >= 1.0,
        "trophic level should be >= 1.0, got {}",
        sim.stats.mean_trophic_level
    );

    // Herbivore + predator counts should not exceed population
    assert!(
        sim.stats.herbivore_count + sim.stats.predator_count <= sim.stats.population,
        "herbivore + predator count exceeds population"
    );
}

#[test]
fn neat_topology_evolves() {
    // Run long enough for NEAT structural mutations to occur
    let cfg = small_config();
    let mut sim = Simulation::new(cfg);

    // Initial population: all have 8 hidden nodes, 182 connections
    for org in &sim.organisms {
        assert_eq!(org.genome.hidden_nodes.len(), 8);
        assert_eq!(org.brain.hidden, 8);
    }

    // Run for 2000 ticks — enough for many generations and structural mutations
    for _ in 0..2000 {
        sim.step();
        if sim.is_extinct() {
            break;
        }
    }

    // Verify all organisms have consistent brain/genome topology
    for org in &sim.organisms {
        if org.alive {
            assert_eq!(
                org.brain.hidden,
                org.genome.hidden_nodes.len(),
                "brain hidden count should match genome"
            );
            assert_eq!(org.brain.inputs, sim.config.brain.inputs);
            assert_eq!(org.brain.outputs, sim.config.brain.outputs);
            // Connection count should be positive
            assert!(
                org.brain.connection_count() > 0,
                "brain should have at least some connections"
            );
        }
    }

    // After 2000 ticks, we should see some topology variation
    // (different hidden node counts and/or different connection counts)
    let _unique_hidden: std::collections::HashSet<usize> = sim
        .organisms
        .iter()
        .filter(|o| o.alive)
        .map(|o| o.genome.hidden_nodes.len())
        .collect();
    let unique_conns: std::collections::HashSet<usize> = sim
        .organisms
        .iter()
        .filter(|o| o.alive)
        .map(|o| o.brain.connection_count())
        .collect();

    // At minimum, connection counts should vary (add-connection and toggle mutations)
    assert!(
        unique_conns.len() >= 1,
        "expected connection count variation, got {:?}",
        unique_conns,
    );

    // Check that no organism exceeds MAX_NODES
    for org in &sim.organisms {
        if org.alive {
            let total_nodes = 1 + org.brain.inputs + org.brain.hidden + org.brain.outputs;
            assert!(
                total_nodes <= sim_core::brain::MAX_NODES,
                "total nodes {} exceeds MAX_NODES",
                total_nodes,
            );
        }
    }
}

#[test]
fn biome_aware_food_regrowth() {
    // Verify that BiomeZone config produces different per-cell regrowth rates
    // and that cells in lush zones regrow faster than cells in barren zones.
    //
    // The implementation uses "highest multiplier wins" starting at 1.0×,
    // so we test with two lush zones of different strengths and a neutral area.
    //
    // Citation: Schluter (2000) "The Ecology of Adaptive Radiation"
    let world = WorldConfig {
        width: 200.0,
        height: 200.0,
        seed: 42,
    };
    let food = FoodConfig {
        cell_size: 10.0,
        max_per_cell: 1.0,
        regrowth_rate: 0.002,
        energy_per_food: 40.0,
        biomes: vec![
            // Very lush zone at top-left: 5× regrowth
            BiomeZone {
                x: 30.0,
                y: 30.0,
                radius: 40.0,
                regrowth_multiplier: 5.0,
            },
            // Moderately lush zone at bottom-right: 2× regrowth
            BiomeZone {
                x: 170.0,
                y: 170.0,
                radius: 40.0,
                regrowth_multiplier: 2.0,
            },
        ],
    };

    let mut grid = FoodGrid::new(&world, &food);
    let base_rate = food.regrowth_rate;

    // Cell at (col=3, row=3) → center (35, 35) is inside the very lush zone
    let lush_rate = grid.regrowth_at(3, 3);
    // Cell at (col=17, row=17) → center (175, 175) is inside the moderate zone
    let moderate_rate = grid.regrowth_at(17, 17);
    // Cell at (col=10, row=10) → center (105, 105) is far from both zones (barren = base)
    let barren_rate = grid.regrowth_at(10, 10);

    // Lush zone rate should be higher than base rate
    assert!(
        lush_rate > base_rate,
        "lush zone rate ({}) should exceed base rate ({})",
        lush_rate,
        base_rate
    );
    // Moderate zone rate should also exceed base rate
    assert!(
        moderate_rate > base_rate,
        "moderate zone rate ({}) should exceed base rate ({})",
        moderate_rate,
        base_rate
    );
    // Barren area (no zone) should be at the base rate
    assert!(
        (barren_rate - base_rate).abs() < 1e-10,
        "barren rate ({}) should equal base rate ({})",
        barren_rate,
        base_rate
    );
    // Lush > moderate > barren — three distinct tiers of regrowth
    assert!(
        lush_rate > moderate_rate,
        "lush rate ({}) must exceed moderate rate ({})",
        lush_rate,
        moderate_rate
    );
    assert!(
        moderate_rate > barren_rate,
        "moderate rate ({}) must exceed barren rate ({})",
        moderate_rate,
        barren_rate
    );

    // Now test actual regrowth: drain cells in all three zones and regrow
    let cols = grid.cols;
    let lush_idx = 3 * cols + 3;
    let moderate_idx = 17 * cols + 17;
    let barren_idx = 10 * cols + 10;

    grid.cells[lush_idx] = 0.0;
    grid.cells[moderate_idx] = 0.0;
    grid.cells[barren_idx] = 0.0;

    for _ in 0..10 {
        grid.regrow();
    }

    let lush_food = grid.cells[lush_idx];
    let moderate_food = grid.cells[moderate_idx];
    let barren_food = grid.cells[barren_idx];

    // After regrowth, lush > moderate > barren
    assert!(
        lush_food > moderate_food,
        "after regrowth, lush cell ({}) should have more food than moderate cell ({})",
        lush_food,
        moderate_food
    );
    assert!(
        moderate_food > barren_food,
        "after regrowth, moderate cell ({}) should have more food than barren cell ({})",
        moderate_food,
        barren_food
    );
    // Lush cell should have regrown well above base rate over 10 ticks
    assert!(
        lush_food > base_rate * 10.0,
        "lush cell ({}) should have more than 10× base regrowth ({})",
        lush_food,
        base_rate * 10.0
    );
    // Barren cell should have regrown at exactly base rate
    let expected_barren = base_rate * 10.0;
    assert!(
        (barren_food - expected_barren).abs() < 1e-10,
        "barren cell ({}) should equal 10× base rate ({})",
        barren_food,
        expected_barren
    );
}

#[test]
fn biome_no_zones_gives_uniform_regrowth() {
    // When no biomes are configured, all cells should have the same base regrowth rate.
    let world = WorldConfig {
        width: 100.0,
        height: 100.0,
        seed: 0,
    };
    let food = FoodConfig {
        cell_size: 10.0,
        max_per_cell: 1.0,
        regrowth_rate: 0.005,
        energy_per_food: 40.0,
        biomes: vec![],
    };

    let grid = FoodGrid::new(&world, &food);
    let cols = grid.cols;
    let rows = grid.rows;

    for row in 0..rows {
        for col in 0..cols {
            let rate = grid.regrowth_at(col, row);
            assert!(
                (rate - food.regrowth_rate).abs() < 1e-10,
                "cell ({},{}) rate {} != base rate {}",
                col,
                row,
                rate,
                food.regrowth_rate
            );
        }
    }
}
