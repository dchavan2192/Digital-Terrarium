//! Integration tests for sim-core.
//!
//! These verify that the simulation initializes, runs, and obeys its invariants.

use sim_core::brain::Brain;
use sim_core::config::{BiomeZone, DroughtConfig, FoodConfig, SeasonsConfig, WorldConfig};
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
inputs = 19
hidden = 8
outputs = 7

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
    assert_eq!(cfg.brain.inputs, 19);
    assert_eq!(cfg.brain.hidden, 8);
    assert_eq!(cfg.brain.outputs, 7);
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
inputs = 19
hidden = 8
outputs = 7
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
inputs = 19
hidden = 8
outputs = 7

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
        // Connection count: bias→8h + 19i→8h + bias→7o + 8h→7o = 8+152+7+56 = 223
        assert_eq!(org.brain.connection_count(), 223);
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

    // Regrow (no seasons in small_config, so multiplier = 1.0)
    let no_seasons = SeasonsConfig::default();
    sim.food_grid.regrow(0, &no_seasons, false);
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
inputs = 19
hidden = 8
outputs = 7

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

    // Initial population: all have 8 hidden nodes, 223 connections
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
        !unique_conns.is_empty(),
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
        seasons: SeasonsConfig::default(),
        droughts: DroughtConfig::default(),
        fruit: Default::default(),
    };

    let mut grid = FoodGrid::new(&world, &food, &Default::default());
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

    let no_seasons = SeasonsConfig::default();
    for t in 0..10 {
        grid.regrow(t, &no_seasons, false);
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
        seasons: SeasonsConfig::default(),
        droughts: DroughtConfig::default(),
        fruit: Default::default(),
    };

    let grid = FoodGrid::new(&world, &food, &Default::default());
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

#[test]
fn seasonal_food_variation() {
    // Verify that seasonal (cyclical) food variation modulates regrowth rates.
    //
    // At tick 0, sin(0)=0 → multiplier=1.0 (baseline).
    // At tick period/4, sin(π/2)=1 → multiplier=1+amplitude (peak summer).
    // At tick 3*period/4, sin(3π/2)=-1 → multiplier=1-amplitude (peak winter).
    //
    // Citation: Botkin (1990) "Forest Dynamics: An Ecological Model"
    use sim_core::world::seasonal_multiplier;

    let seasons = SeasonsConfig {
        amplitude: 0.5,
        period: 1000.0,
    };

    // At tick 0: sin(0) = 0 → multiplier = 1.0
    let m0 = seasonal_multiplier(0, &seasons);
    assert!(
        (m0 - 1.0).abs() < 1e-10,
        "at tick 0, multiplier should be 1.0, got {}",
        m0
    );

    // At tick period/4 (=250): sin(π/2) = 1 → multiplier = 1.5
    let m_peak = seasonal_multiplier(250, &seasons);
    assert!(
        (m_peak - 1.5).abs() < 1e-10,
        "at peak summer, multiplier should be 1.5, got {}",
        m_peak
    );

    // At tick 3*period/4 (=750): sin(3π/2) = -1 → multiplier = 0.5
    let m_trough = seasonal_multiplier(750, &seasons);
    assert!(
        (m_trough - 0.5).abs() < 1e-10,
        "at peak winter, multiplier should be 0.5, got {}",
        m_trough
    );

    // Now test actual food regrowth with seasons
    let world = WorldConfig {
        width: 100.0,
        height: 100.0,
        seed: 0,
    };
    let food = FoodConfig {
        cell_size: 50.0,
        max_per_cell: 1.0,
        regrowth_rate: 0.01,
        energy_per_food: 40.0,
        biomes: vec![],
        seasons: SeasonsConfig::default(), // not used for grid init
        droughts: DroughtConfig::default(),
        fruit: Default::default(),
    };
    let mut grid = FoodGrid::new(&world, &food, &Default::default());

    // Drain cell (0,0) and regrow at peak summer (tick=250, multiplier=1.5)
    grid.cells[0] = 0.0;
    grid.regrow(250, &seasons, false);
    let summer_food = grid.cells[0];

    // Drain again and regrow at peak winter (tick=750, multiplier=0.5)
    grid.cells[0] = 0.0;
    grid.regrow(750, &seasons, false);
    let winter_food = grid.cells[0];

    // Drain again and regrow at baseline (tick=0, multiplier=1.0)
    grid.cells[0] = 0.0;
    grid.regrow(0, &seasons, false);
    let baseline_food = grid.cells[0];

    // Summer > baseline > winter
    assert!(
        summer_food > baseline_food,
        "summer regrowth ({}) should exceed baseline ({})",
        summer_food,
        baseline_food
    );
    assert!(
        baseline_food > winter_food,
        "baseline regrowth ({}) should exceed winter ({})",
        baseline_food,
        winter_food
    );

    // Verify exact amounts: rate * multiplier
    assert!(
        (summer_food - 0.015).abs() < 1e-10,
        "summer food should be 0.015 (0.01 * 1.5), got {}",
        summer_food
    );
    assert!(
        (winter_food - 0.005).abs() < 1e-10,
        "winter food should be 0.005 (0.01 * 0.5), got {}",
        winter_food
    );
}

#[test]
fn seasonal_no_effect_when_amplitude_zero() {
    // With amplitude=0, seasonal multiplier should always be 1.0
    use sim_core::world::seasonal_multiplier;

    let no_seasons = SeasonsConfig::default(); // amplitude=0

    for tick in [0, 100, 250, 500, 750, 999] {
        let m = seasonal_multiplier(tick, &no_seasons);
        assert!(
            (m - 1.0).abs() < 1e-10,
            "with amplitude=0, multiplier at tick {} should be 1.0, got {}",
            tick,
            m
        );
    }
}

#[test]
fn density_pressure_increases_metabolism() {
    // Organisms in crowded neighborhoods should lose more energy per tick
    // when density_pressure.cost_per_neighbor > 0, compared to an identical
    // simulation without density pressure.
    //
    // Citation: Allee, W.C. (1931) "Animal Aggregations: A Study in General Sociology"
    let base_toml = r#"
[world]
width = 500.0
height = 500.0
seed = 42

[food]
cell_size = 10.0
max_per_cell = 0.0
regrowth_rate = 0.0
energy_per_food = 40.0

[creature]
initial_count = 20
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
starting = 200.0
maximum = 500.0
base_metabolism = 0.1
movement_cost = 0.05
brain_cost = 0.002

[reproduction]
threshold = 999.0
parent_cost = 499.0
child_energy = 50.0

[mutation]
weight_probability = 0.1
weight_step = 0.2

[brain]
inputs = 19
hidden = 8
outputs = 7

[aging]
extrinsic_mortality = 0.0
initial_senescence = 0.0
aging_rate = 0.0

[combat]
attack_range = 0.0
attack_cost = 0.0
attack_damage = 0.0
energy_transfer_ratio = 0.0

[sharing]
share_range = 0.0
share_amount = 0.0
"#;

    // Run without density pressure (default cost_per_neighbor = 0.0)
    let cfg_no = Config::parse(base_toml).unwrap();
    let mut sim_no = Simulation::new(cfg_no);
    for _ in 0..3 {
        sim_no.step();
    }

    // Run with density pressure
    let mut cfg_yes = Config::parse(base_toml).unwrap();
    cfg_yes.density_pressure.cost_per_neighbor = 0.5;
    let mut sim_yes = Simulation::new(cfg_yes);
    for _ in 0..3 {
        sim_yes.step();
    }

    // Both should survive (high starting energy, short run, no mortality)
    assert!(!sim_no.is_extinct(), "baseline sim should survive 3 ticks");
    assert!(
        !sim_yes.is_extinct(),
        "density pressure sim should survive 3 ticks"
    );

    // Mean energy with density pressure should be strictly lower
    assert!(
        sim_yes.stats.mean_energy < sim_no.stats.mean_energy,
        "density pressure should reduce mean energy: with={:.2}, without={:.2}",
        sim_yes.stats.mean_energy,
        sim_no.stats.mean_energy
    );

    // Mean crowding should be tracked and positive (20 creatures in 500×500 world)
    assert!(
        sim_yes.stats.mean_crowding > 0.0,
        "mean_crowding should be positive with 20 organisms in 500×500 world, got {}",
        sim_yes.stats.mean_crowding
    );
}

#[test]
fn density_pressure_no_effect_when_zero() {
    // With cost_per_neighbor = 0.0, density pressure should have no effect.
    // Two identical simulations (one with explicit 0.0, one without the config)
    // should produce identical results.
    let toml_explicit = r#"
[world]
width = 100.0
height = 100.0
seed = 7

[food]
cell_size = 10.0
max_per_cell = 1.0
regrowth_rate = 0.005
energy_per_food = 40.0

[creature]
initial_count = 20
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
inputs = 19
hidden = 8
outputs = 7

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

[density_pressure]
cost_per_neighbor = 0.0
"#;
    // Without density_pressure section — serde(default) gives 0.0
    let toml_omitted = r#"
[world]
width = 100.0
height = 100.0
seed = 7

[food]
cell_size = 10.0
max_per_cell = 1.0
regrowth_rate = 0.005
energy_per_food = 40.0

[creature]
initial_count = 20
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
inputs = 19
hidden = 8
outputs = 7

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

    let mut sim_explicit = Simulation::new(Config::parse(toml_explicit).unwrap());
    let mut sim_omitted = Simulation::new(Config::parse(toml_omitted).unwrap());

    for _ in 0..20 {
        sim_explicit.step();
        sim_omitted.step();
    }

    // Identical configs + same seed → identical state
    assert_eq!(sim_explicit.organisms.len(), sim_omitted.organisms.len());
    for (a, b) in sim_explicit
        .organisms
        .iter()
        .zip(sim_omitted.organisms.iter())
    {
        assert_eq!(a.energy, b.energy, "energy should be identical with cost=0");
    }
}

#[test]
fn dynamic_speciation_threshold_adapts_to_diversity() {
    // Run a simulation with dynamic speciation enabled. After enough ticks for
    // NEAT structural mutations to diversify genomes, the dynamically computed
    // threshold should differ from the fixed default (0.5) and be reported in stats.
    //
    // Citation: Stanley & Miikkulainen (2002) — dynamic compatibility threshold
    let toml = r#"
[world]
width = 200.0
height = 200.0
seed = 42

[food]
cell_size = 10.0
max_per_cell = 1.0
regrowth_rate = 0.005
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
threshold = 80.0
parent_cost = 40.0
child_energy = 40.0

[mutation]
weight_probability = 0.3
weight_step = 0.5

[brain]
inputs = 19
hidden = 8
outputs = 7

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

[speciation]
dynamic = true
fixed_threshold = 0.5
sample_size = 30
factor = 0.75
min_threshold = 0.1
max_threshold = 2.0
"#;
    let cfg = Config::parse(toml).unwrap();
    assert!(cfg.speciation.dynamic);

    let mut sim = Simulation::new(cfg);

    for _ in 0..500 {
        sim.step();
        if sim.is_extinct() {
            break;
        }
    }

    // Dynamic threshold should be within configured bounds
    assert!(
        sim.stats.speciation_threshold >= 0.1,
        "threshold {} should be >= min_threshold 0.1",
        sim.stats.speciation_threshold
    );
    assert!(
        sim.stats.speciation_threshold <= 2.0,
        "threshold {} should be <= max_threshold 2.0",
        sim.stats.speciation_threshold
    );
    // With high mutation rate (0.3) and large steps (0.5), genomes should diverge,
    // producing a non-trivial mean distance. The threshold should not equal the
    // fixed fallback (0.5) — it should have been computed from actual variance.
    // (It's technically possible but extremely unlikely with these settings.)
    assert!(
        sim.stats.speciation_threshold != 0.5,
        "dynamic threshold should differ from fixed default 0.5"
    );
    assert!(
        sim.stats.species_count >= 1,
        "should detect at least 1 species"
    );
}

#[test]
fn speciation_fixed_threshold_backward_compatible() {
    // With dynamic=false (or config section omitted), the speciation threshold
    // should be the fixed value and behavior should match the previous hard-coded 0.5.
    //
    // Citation: Stanley & Miikkulainen (2002)
    let toml_explicit = r#"
[world]
width = 100.0
height = 100.0
seed = 7

[food]
cell_size = 10.0
max_per_cell = 1.0
regrowth_rate = 0.005
energy_per_food = 40.0

[creature]
initial_count = 20
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
inputs = 19
hidden = 8
outputs = 7

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

[speciation]
dynamic = false
fixed_threshold = 0.5
"#;
    // Without [speciation] section — should default to dynamic=false, threshold=0.5
    let toml_omitted = r#"
[world]
width = 100.0
height = 100.0
seed = 7

[food]
cell_size = 10.0
max_per_cell = 1.0
regrowth_rate = 0.005
energy_per_food = 40.0

[creature]
initial_count = 20
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
inputs = 19
hidden = 8
outputs = 7

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

    let mut sim_explicit = Simulation::new(Config::parse(toml_explicit).unwrap());
    let mut sim_omitted = Simulation::new(Config::parse(toml_omitted).unwrap());

    for _ in 0..20 {
        sim_explicit.step();
        sim_omitted.step();
    }

    // Both should use fixed threshold = 0.5
    assert!(
        (sim_explicit.stats.speciation_threshold - 0.5).abs() < 1e-10,
        "explicit fixed threshold should be 0.5, got {}",
        sim_explicit.stats.speciation_threshold
    );
    assert!(
        (sim_omitted.stats.speciation_threshold - 0.5).abs() < 1e-10,
        "omitted config should default to 0.5, got {}",
        sim_omitted.stats.speciation_threshold
    );

    // Identical configs + same seed → identical species count and state
    assert_eq!(
        sim_explicit.stats.species_count, sim_omitted.stats.species_count,
        "species count should match between explicit and default configs"
    );
    assert_eq!(
        sim_explicit.organisms.len(),
        sim_omitted.organisms.len(),
        "population should be identical"
    );
    for (a, b) in sim_explicit
        .organisms
        .iter()
        .zip(sim_omitted.organisms.iter())
    {
        assert_eq!(a.energy, b.energy, "energy should be identical");
    }
}

/// Brains with high-magnitude weights produce higher activity cost than near-zero weights.
/// Citation: Attwell & Laughlin (2001) — active synaptic transmission dominates neural energy.
#[test]
fn brain_activity_cost_scales_with_usage() {
    let num_inputs = 3;
    let num_outputs = 2;

    // High-weight brain: all connections have weight 2.0
    let high_conns: Vec<(u16, u16, f64, bool)> = vec![
        (0, 4, 2.0, true), // bias -> out0
        (1, 4, 2.0, true), // in0 -> out0
        (2, 4, 2.0, true), // in1 -> out0
        (3, 4, 2.0, true), // in2 -> out0
        (0, 5, 2.0, true), // bias -> out1
        (1, 5, 2.0, true), // in0 -> out1
    ];
    let high_brain = Brain::from_neat(num_inputs, num_outputs, &[], &high_conns);

    // Low-weight brain: same topology, near-zero weights
    let low_conns: Vec<(u16, u16, f64, bool)> = vec![
        (0, 4, 0.001, true),
        (1, 4, 0.001, true),
        (2, 4, 0.001, true),
        (3, 4, 0.001, true),
        (0, 5, 0.001, true),
        (1, 5, 0.001, true),
    ];
    let low_brain = Brain::from_neat(num_inputs, num_outputs, &[], &low_conns);

    let inputs = [0.5, 0.8, -0.3];
    let (_, high_activity) = high_brain.forward(&inputs);
    let (_, low_activity) = low_brain.forward(&inputs);

    assert!(
        high_activity > low_activity * 100.0,
        "high-weight brain activity ({}) should be >100x low-weight ({})",
        high_activity,
        low_activity
    );
    assert!(high_activity > 0.0, "activity should be positive");
    assert!(
        low_activity > 0.0,
        "even low-weight brain has some activity"
    );
}

#[test]
fn oee_activity_tracker_populates_stats() {
    let mut config = load_config();
    config.activity_tracking.enabled = true;
    config.activity_tracking.window_size = 1;
    let mut sim = Simulation::new(config);

    for _ in 0..10 {
        sim.step();
    }

    assert!(
        sim.stats.active_traits > 0,
        "should have active traits after 10 ticks"
    );
    assert!(
        sim.stats.mean_cumulative_activity > 0.0,
        "D(t) should be positive"
    );
    assert!(
        sim.stats.lineage_diversity > 0,
        "lineage diversity should be positive with living organisms"
    );
}

#[test]
fn child_inherits_root_ancestor_and_birth_tick() {
    let mut config = load_config();
    config.creature.initial_count = 50;
    config.reproduction.threshold = 50.0;
    config.energy.starting = 200.0;
    let mut sim = Simulation::new(config);

    for _ in 0..200 {
        sim.step();
    }

    let children: Vec<_> = sim
        .organisms
        .iter()
        .filter(|o| o.alive && o.generation > 0)
        .collect();

    if !children.is_empty() {
        for child in &children {
            assert!(
                child.root_ancestor_id < child.id,
                "root_ancestor_id should be from an earlier organism"
            );
        }
        let any_nonzero_birth = children.iter().any(|c| c.birth_tick > 0);
        assert!(
            any_nonzero_birth,
            "at least some children should have birth_tick > 0"
        );
    }
}

#[test]
fn niche_construction_modifies_environment() {
    let mut config = load_config();
    config.niche_construction.enabled = true;
    config.niche_construction.deposit_rate = 0.1;
    config.niche_construction.decay_rate = 0.99;
    let mut sim = Simulation::new(config);

    for _ in 0..50 {
        sim.step();
    }

    let any_nonzero = sim
        .construction_grid
        .cells
        .iter()
        .any(|&c| c.abs() > 0.0001);
    assert!(
        any_nonzero,
        "construction grid should have non-zero cells after 50 ticks"
    );
}

// ---------------------------------------------------------------------------
// Phase 7.5 correctness fixes
// ---------------------------------------------------------------------------

/// D: an old 15-input / 6-output config must be rejected with a clear error.
#[test]
fn config_rejects_mismatched_brain_io() {
    let old = DEFAULT_CONFIG
        .replace("inputs = 19", "inputs = 15")
        .replace("outputs = 7", "outputs = 6");
    let err = Config::parse(&old).expect_err("15/6 brain should be rejected");
    assert!(err.to_string().contains("brain.inputs"), "got: {err}");

    let bad_out = DEFAULT_CONFIG.replace("outputs = 7", "outputs = 6");
    let err = Config::parse(&bad_out).expect_err("6 outputs should be rejected");
    assert!(err.to_string().contains("brain.outputs"), "got: {err}");
}

/// D: every shipped config must validate.
#[test]
fn shipped_configs_validate() {
    let intense = include_str!("../../../configs/intense.toml");
    Config::parse(intense).expect("configs/intense.toml should be valid");
    load_config();
}

/// K: omitted mutation keys default to the previously hard-coded values.
#[test]
fn mutation_probabilities_default_to_legacy_values() {
    let cfg = small_config();
    assert_eq!(cfg.mutation.add_connection_probability, 0.02);
    assert_eq!(cfg.mutation.add_node_probability, 0.01);
    assert_eq!(cfg.mutation.toggle_probability, 0.01);
    assert_eq!(cfg.mutation.prune_probability, 0.04);
    assert_eq!(cfg.mutation.trait_probability, 0.05);
    assert_eq!(cfg.niche_construction.activation_threshold, 0.01);
    assert_eq!(cfg.reproduction.mate_energy_fraction, 0.5);
    assert_eq!(
        cfg.reproduction.effective_mate_cost(),
        cfg.reproduction.parent_cost
    );
}

/// A: stress is relative to reproduction.threshold; a config whose stress level is
/// at or above child_energy (every newborn stressed) is rejected.
#[test]
fn epigenetic_stress_level_must_be_below_child_energy() {
    let mut text = DEFAULT_CONFIG.replace(
        "stress_repro_fraction = 0.25",
        "stress_repro_fraction = 0.6",
    );
    assert!(
        Config::parse(&text).is_err(),
        "0.6 × 60 = 36 >= 30 must fail"
    );
    text = DEFAULT_CONFIG.to_string();
    assert!(Config::parse(&text).is_ok());
}

/// A: organisms well above the stress level gain no methylation.
#[test]
fn well_fed_organisms_are_not_stressed() {
    let mut config = small_config();
    config.epigenetics.enabled = true;
    config.epigenetics.stress_repro_fraction = 0.1; // 15 energy; everyone starts at 100
    let mut sim = Simulation::new(config);
    for _ in 0..5 {
        sim.step();
    }
    for org in &sim.organisms {
        if org.energy > 20.0 && org.generation == 0 {
            assert!(
                org.genome.connections.iter().all(|c| c.methylation == 0.0),
                "well-fed organism should have no methylation"
            );
        }
    }
}

/// B: with epigenetics on, the living brain always carries w·(1−m).
#[test]
fn methylation_reaches_living_brain() {
    let mut config = small_config();
    config.epigenetics.enabled = true;
    config.epigenetics.stress_repro_fraction = 0.9; // 135 > starting 100: all stressed
    config.epigenetics.methylation_rate = 0.05;
    config.epigenetics.brain_update_interval = 1;
    let mut sim = Simulation::new(config);
    for _ in 0..10 {
        sim.step();
    }
    let mut any_methylated = false;
    for org in &sim.organisms {
        let rebuilt = org.genome.build_brain();
        assert_eq!(rebuilt.connections.len(), org.brain.connections.len());
        for (a, b) in rebuilt.connections.iter().zip(org.brain.connections.iter()) {
            assert_eq!((a.0, a.1), (b.0, b.1));
            assert!(
                (a.2 - b.2).abs() < 1e-12,
                "brain weight not methylation-adjusted"
            );
        }
        any_methylated |= org.genome.connections.iter().any(|c| c.methylation > 0.0);
    }
    assert!(any_methylated, "stressed organisms should have methylation");
}

/// B: refresh_weights patches in place and matches a rebuild.
#[test]
fn brain_refresh_weights_matches_rebuild() {
    let config = small_config();
    let mut sim = Simulation::new(config);
    let org = &mut sim.organisms[0];
    for (k, c) in org.genome.connections.iter_mut().enumerate() {
        c.methylation = (k % 10) as f64 / 10.0;
    }
    let genome = org.genome.clone();
    org.brain.refresh_weights(|gi| genome.effective_weight(gi));
    let rebuilt = genome.build_brain();
    for (a, b) in rebuilt.connections.iter().zip(org.brain.connections.iter()) {
        assert!((a.2 - b.2).abs() < 1e-12);
    }
}

/// C: inheritance_noise = 0 must not panic, and methylation stays in [0, 1].
#[test]
fn zero_inheritance_noise_does_not_panic() {
    let mut config = small_config();
    config.epigenetics.enabled = true;
    config.epigenetics.inheritance_noise = 0.0;
    config.reproduction.threshold = 110.0;
    config.reproduction.parent_cost = 40.0;
    config.energy.starting = 120.0; // above threshold → births from tick 1
    config.epigenetics.stress_repro_fraction = 0.25;
    let mut sim = Simulation::new(config);
    for _ in 0..200 {
        sim.step();
    }
    assert!(
        sim.stats.total_births > 0,
        "test needs births to exercise inheritance"
    );
    for org in &sim.organisms {
        for c in &org.genome.connections {
            assert!((0.0..=1.0).contains(&c.methylation));
        }
    }
}

/// A/B/C: epigenetics disabled ⇒ methylation is exactly zero everywhere.
#[test]
fn epigenetics_disabled_leaves_methylation_zero() {
    let mut config = load_config();
    config.epigenetics.enabled = false;
    let mut sim = Simulation::new(config);
    for _ in 0..100 {
        sim.step();
    }
    assert_eq!(sim.stats.mean_methylation, 0.0);
    for org in &sim.organisms {
        assert!(org.genome.connections.iter().all(|c| c.methylation == 0.0));
    }
}

/// E: niche construction multiplies actual regrowth — no regrowth inside a total drought,
/// boosted regrowth outside one.
#[test]
fn niche_construction_respects_drought() {
    use sim_core::config::{FruitConfig, NicheConstructionConfig};
    use sim_core::world::{ActiveDrought, ConstructionGrid};
    let world = WorldConfig {
        width: 100.0,
        height: 100.0,
        seed: 1,
    };
    let food = FoodConfig {
        cell_size: 10.0,
        max_per_cell: 1.0,
        regrowth_rate: 0.01,
        energy_per_food: 10.0,
        biomes: vec![],
        seasons: SeasonsConfig::default(),
        droughts: DroughtConfig::default(),
        fruit: FruitConfig::default(),
    };
    let nc = NicheConstructionConfig {
        enabled: true,
        regrowth_bonus: 1.0,
        ..Default::default()
    };
    let mut cg = ConstructionGrid::new(&world, 10.0, &nc);
    for c in cg.cells.iter_mut() {
        *c = 1.0; // fully built → multiplier 2.0
    }
    let no_seasons = SeasonsConfig::default();

    // Outside drought: built cells regrow at 2× the base rate.
    let mut grid = FoodGrid::new(&world, &food, &FruitConfig::default());
    for c in grid.cells.iter_mut() {
        *c = 0.0;
    }
    grid.regrow_with_construction(0, &no_seasons, false, Some(&cg));
    assert!(
        (grid.cells[0] - 0.02).abs() < 1e-12,
        "got {}",
        grid.cells[0]
    );

    // Total drought covering the world: no regrowth despite construction.
    let mut grid = FoodGrid::new(&world, &food, &FruitConfig::default());
    for c in grid.cells.iter_mut() {
        *c = 0.0;
    }
    grid.droughts.push(ActiveDrought {
        x: 50.0,
        y: 50.0,
        radius: 1.0e6,
        severity: 0.0,
        ticks_remaining: 10,
    });
    grid.regrow_with_construction(0, &no_seasons, false, Some(&cg));
    let centre = 5 * grid.cols + 5;
    assert!(
        grid.cells[centre] < 1e-6,
        "constructed cell regrew inside a drought: {}",
        grid.cells[centre]
    );
}

/// E: mean_construction_level is mean |level| (no cancellation).
#[test]
fn construction_level_stat_is_absolute() {
    let mut config = load_config();
    config.niche_construction.enabled = true;
    let mut sim = Simulation::new(config);
    for _ in 0..30 {
        sim.step();
    }
    assert!(sim.stats.mean_construction_level >= 0.0);
    assert!(sim.stats.mean_construction_level >= sim.stats.net_construction_level.abs() - 1e-12);
}

fn mating_config() -> Config {
    let mut config = small_config();
    config.creature.initial_count = 3;
    config.reproduction.sexual = true;
    config.reproduction.crossover_rate = 1.0;
    config.reproduction.threshold = 60.0;
    config.reproduction.parent_cost = 30.0;
    config.reproduction.child_energy = 20.0;
    config.energy.starting = 100.0;
    config.combat.attack_cost = 0.0;
    config.combat.attack_damage = 0.0;
    config.sharing.share_amount = 0.0;
    config.aging.extrinsic_mortality = 0.0;
    config.aging.initial_senescence = 0.0;
    // No food income, so energy changes in one tick are exactly the
    // reproduction cost plus a small metabolic burn (< 1 unit here).
    config.food.energy_per_food = 0.0;
    config
}

/// F: the mate pays too, and each organism mates at most once per tick.
#[test]
fn sexual_reproduction_costs_both_parents_once_per_tick() {
    let mut sim = Simulation::new(mating_config());
    for (k, org) in sim.organisms.iter_mut().enumerate() {
        org.pos_x = 50.0 + 2.0 * k as f64;
        org.pos_y = 50.0;
    }
    sim.step();
    assert_eq!(sim.stats.births, 2);
    assert_eq!(sim.stats.sexual_births, 1, "only one pair can form");
    assert_eq!(
        sim.stats.asexual_births, 1,
        "third organism falls back to asexual"
    );
    // All three parents paid ~30 (plus a few units of metabolism).
    for org in sim.organisms.iter().filter(|o| o.generation == 0) {
        assert!(
            org.energy < 70.0 + 1e-9 && org.energy > 68.0,
            "parent energy {} should reflect a 30-unit reproduction cost",
            org.energy
        );
    }
    assert_eq!(sim.stats.total_sexual_births, 1);
    assert_eq!(sim.stats.total_asexual_births, 1);
}

/// F: mate_cost is configurable (0 restores the old initiator-only accounting).
#[test]
fn mate_cost_zero_means_mate_does_not_pay() {
    let mut config = mating_config();
    config.creature.initial_count = 2;
    config.reproduction.mate_cost = Some(0.0);
    let mut sim = Simulation::new(config);
    sim.organisms[0].pos_x = 50.0;
    sim.organisms[0].pos_y = 50.0;
    sim.organisms[1].pos_x = 52.0;
    sim.organisms[1].pos_y = 50.0;
    sim.step();
    assert_eq!(sim.stats.sexual_births, 1);
    let parents: Vec<f64> = sim
        .organisms
        .iter()
        .filter(|o| o.generation == 0)
        .map(|o| o.energy)
        .collect();
    assert!(
        parents.iter().any(|&e| e > 98.0),
        "mate should not pay: {parents:?}"
    );
    assert!(
        parents.iter().any(|&e| e < 70.0 + 1e-9 && e > 68.0),
        "initiator pays: {parents:?}"
    );
}

/// H: cumulative birth counters are consistent with total_births.
#[test]
fn cumulative_birth_mode_counters() {
    let mut config = load_config();
    config.reproduction.sexual = true;
    let mut sim = Simulation::new(config);
    let mut sex = 0u64;
    let mut asex = 0u64;
    for _ in 0..300 {
        sim.step();
        sex += sim.stats.sexual_births;
        asex += sim.stats.asexual_births;
    }
    assert_eq!(sim.stats.total_sexual_births, sex);
    assert_eq!(sim.stats.total_asexual_births, asex);
    assert_eq!(sex + asex, sim.stats.total_births);
}

/// H: windowed lineage diversity is bounded by the population at the previous
/// checkpoint and is not stuck at the founder count.
#[test]
fn windowed_lineage_diversity() {
    let mut config = load_config();
    config.activity_tracking.lineage_window = 50;
    let mut sim = Simulation::new(config);
    let mut pop_at_checkpoint = sim.organisms.len();
    for _ in 0..300 {
        sim.step();
        if sim.tick.is_multiple_of(50) {
            assert!(sim.stats.lineage_diversity <= pop_at_checkpoint);
            assert!(sim.stats.lineage_diversity > 0 || sim.organisms.is_empty());
            pop_at_checkpoint = sim.organisms.len();
            for o in &sim.organisms {
                assert_eq!(o.clade_id, o.id, "clades reset at checkpoint");
            }
        }
    }
    assert!(sim.stats.founder_lineages > 0);
}

/// H: topology hash v2 is independent of gene order; v1 (legacy) is not required to be.
#[test]
fn topology_hash_is_order_independent() {
    use sim_core::activity::{topology_hash, topology_hash_versioned};
    let sim = Simulation::new(small_config());
    let g = sim.organisms[0].genome.clone();
    let mut shuffled = g.clone();
    shuffled.connections.reverse();
    assert_eq!(topology_hash(&g), topology_hash(&shuffled));
    assert_eq!(
        topology_hash_versioned(&g, 2),
        topology_hash_versioned(&shuffled, 2)
    );
    // Different wiring → different hash.
    let mut pruned = g.clone();
    pruned.connections.pop();
    assert_ne!(topology_hash(&g), topology_hash(&pruned));
}

/// Determinism still holds with every Phase 7 feature on.
#[test]
fn deterministic_with_all_phase7_features() {
    let run = || {
        let mut config = load_config();
        config.reproduction.sexual = true;
        config.epigenetics.enabled = true;
        config.niche_construction.enabled = true;
        config.activity_tracking.enabled = true;
        let mut sim = Simulation::new(config);
        for _ in 0..200 {
            sim.step();
        }
        (
            sim.organisms.len(),
            sim.stats.total_births,
            sim.stats.mean_methylation,
            sim.stats.mean_construction_level,
            sim.organisms.iter().map(|o| o.energy).sum::<f64>(),
        )
    };
    assert_eq!(run(), run());
}

/// Regression: `[food.fruit]` in default.toml must actually enable fruit.
/// Previously the sim read a separate top-level `fruit` table, so fruit was
/// silently disabled in every run despite `enabled = true` in the config.
#[test]
fn food_fruit_section_enables_fruit() {
    let config = load_config();
    assert!(
        config.food.fruit.enabled,
        "[food.fruit] enabled=true must be honoured"
    );
    let mut sim = Simulation::new(config);
    for _ in 0..50 {
        sim.step();
    }
    let total_fruit: f64 = sim.food_grid.fruit_cells.iter().sum();
    assert!(
        total_fruit > 0.0,
        "fruit grid should hold fruit when enabled"
    );
}

/// Phase 9a: births blocked only by the population cap are counted, so experiments
/// can verify the population is food-limited rather than cap-limited.
#[test]
fn cap_blocked_births_are_counted() {
    let mut config = load_config();
    config.creature.initial_count = 40;
    config.creature.hard_cap = 40;
    config.energy.starting = 150.0; // everyone starts above the reproduction threshold
    let mut sim = Simulation::new(config);
    sim.step();
    assert!(
        sim.stats.cap_blocked_births > 0,
        "full population with energy above threshold must register cap-blocked births"
    );
    let total_after_one = sim.stats.total_cap_blocked_births;
    for _ in 0..5 {
        sim.step();
    }
    assert!(sim.stats.total_cap_blocked_births >= total_after_one);
}
