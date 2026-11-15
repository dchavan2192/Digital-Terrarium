//! Forced causal fixtures for feature switches, rather than random-world divergence.
//! Each ON fixture activates the relevant pathway; OFF comparisons vary parameters
//! that should then be inert. These verify implementation, not evolutionary benefit.

use rand::Rng;
use sim_core::genome::ConnectionGene;
use sim_core::{Config, Simulation};

fn config(count: usize) -> Config {
    let mut c = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
    c.creature.initial_count = count;
    c.creature.hard_cap = count;
    c.creature.max_speed = 0.0;
    c.energy.starting = 50.0;
    c.energy.base_metabolism = 0.0;
    c.energy.movement_cost = 0.0;
    c.energy.brain_cost = 0.0;
    c.energy.connection_cost = 0.0;
    c.energy.activity_weight_cost = 0.0;
    c.density_pressure.cost_per_neighbor = 0.0;
    c.temperature.enabled = false;
    c.elevation.enabled = false;
    c.pheromones.enabled = false;
    c.daynight.enabled = false;
    c.epigenetics.enabled = false;
    c.niche_construction.enabled = false;
    c.activity_tracking.enabled = false;
    c.aging.extrinsic_mortality = 0.0;
    c.aging.initial_senescence = 0.0;
    c.food.droughts.spawn_rate = 0.0;
    c.food.seasons.amplitude = 0.0;
    c.food.regrowth_rate = 0.0;
    c.food.biomes.clear();
    c.food.fruit.enabled = false;
    c.reproduction.sexual = false;
    c
}

fn output(s: &mut Simulation, i: usize, slot: usize, bias: f64) {
    let g = &mut s.organisms[i].genome;
    let node = (g.num_inputs + 1 + slot) as u16;
    for edge in &mut g.connections {
        if edge.to == node {
            edge.weight = if edge.from == 0 { bias } else { 0.0 };
            edge.enabled = true;
            edge.methylation = 0.0;
        }
    }
    s.organisms[i].brain = g.build_brain();
}

fn sensor_to_output(s: &mut Simulation, sensor: usize, slot: usize, bias: f64, weight: f64) {
    output(s, 0, slot, bias);
    let g = &mut s.organisms[0].genome;
    g.connections.push(ConnectionGene {
        from: (sensor + 1) as u16,
        to: (g.num_inputs + 1 + slot) as u16,
        weight,
        enabled: true,
        methylation: 0.0,
    });
    s.organisms[0].brain = g.build_brain();
}

fn setup(c: Config) -> Simulation {
    c.validate().unwrap();
    let mut s = Simulation::new(c);
    for i in 0..s.organisms.len() {
        for slot in 0..7 {
            output(&mut s, i, slot, -50.0);
        }
        output(&mut s, i, 0, 0.0);
        s.organisms[i].pos_x = 500.0 + i as f64 * 2.0;
        s.organisms[i].pos_y = 500.0;
        s.organisms[i].heading = 0.0;
        s.organisms[i].genome.body_size = 1.0;
    }
    s.food_grid.cells.fill(0.0);
    s.food_grid.fruit_cells.fill(0.0);
    s
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

fn same_physical_state(a: &Simulation, b: &Simulation) {
    assert_eq!(a.tick, b.tick);
    assert_eq!(a.organisms.len(), b.organisms.len());
    for (x, y) in a.organisms.iter().zip(&b.organisms) {
        assert_eq!(x.id, y.id);
        assert_eq!(x.age, y.age);
        close(x.pos_x, y.pos_x);
        close(x.pos_y, y.pos_y);
        close(x.heading, y.heading);
        close(x.energy, y.energy);
    }
    assert_eq!(a.food_grid.cells, b.food_grid.cells);
    assert_eq!(a.food_grid.fruit_cells, b.food_grid.fruit_cells);
    assert_eq!(a.pheromone_grid.cells, b.pheromone_grid.cells);
    assert_eq!(a.construction_grid.cells, b.construction_grid.cells);
    // Observation-only flags and ignored parameters must not consume extra RNG.
    assert_eq!(a.rng.clone().r#gen::<u64>(), b.rng.clone().r#gen::<u64>());
}

fn food_cell(s: &Simulation, x: f64, y: f64) -> usize {
    (y / s.food_grid.cell_size) as usize * s.food_grid.cols + (x / s.food_grid.cell_size) as usize
}

#[test]
fn fruit_toggle_controls_actual_consumption_and_disabled_yield_is_inert() {
    let run = |enabled, yield_energy| {
        let mut c = config(1);
        c.food.fruit.enabled = enabled;
        c.food.fruit.energy_per_fruit = yield_energy;
        let mut s = setup(c);
        s.organisms[0].genome.diet_preference = 1.0;
        output(&mut s, 0, 2, 50.0);
        let cell = food_cell(&s, 500.0, 500.0);
        s.food_grid.fruit_cells[cell] = 1.0;
        s.step();
        s
    };
    let on = run(true, 20.0);
    let off = run(false, 20.0);
    let off_changed = run(false, 100.0);
    close(on.stats.energy_fruit_input, 20.0);
    close(on.stats.energy_after - on.stats.energy_before, 20.0);
    close(off.stats.energy_fruit_input, 0.0);
    close(off.food_grid.fruit_at(500.0, 500.0), 1.0);
    close(on.food_grid.fruit_at(500.0, 500.0), 0.0);
    same_physical_state(&off, &off_changed);
}

#[test]
fn temperature_toggle_controls_mismatch_expenditure() {
    let run = |enabled, factor| {
        let mut c = config(1);
        c.energy.base_metabolism = 0.4;
        c.temperature.enabled = enabled;
        c.temperature.cost_factor = factor;
        let mut s = setup(c);
        s.organisms[0].pos_y = 250.0;
        s.organisms[0].genome.preferred_temp = 1.0;
        s.step();
        s
    };
    let on = run(true, 0.5);
    let off = run(false, 0.5);
    let off_changed = run(false, 10.0);
    close(on.stats.energy_temperature_cost, 0.15);
    close(off.stats.energy_temperature_cost, 0.0);
    close(off.organisms[0].energy - on.organisms[0].energy, 0.15);
    same_physical_state(&off, &off_changed);
}

#[test]
fn pheromone_toggle_controls_deposition_decay_and_sensed_action() {
    let run = |enabled, deposit| {
        let mut c = config(1);
        c.pheromones.enabled = enabled;
        c.pheromones.deposit_rate = deposit;
        c.pheromones.decay_rate = 0.5;
        c.pheromones.max_level = 2.0;
        let mut s = setup(c);
        // Input 17 (zero-index 16): pheromone level. Observe through turn.
        sensor_to_output(&mut s, 16, 0, 0.0, 2.0);
        s.step();
        close(s.organisms[0].heading, 0.0); // senses before first deposit
        let expected = if enabled { deposit * 0.5 } else { 0.0 };
        close(s.pheromone_grid.cells.iter().sum(), expected);
        s.step();
        s
    };
    let on = run(true, 0.4);
    let off = run(false, 0.4);
    let off_changed = run(false, 1.0);
    close(on.pheromone_grid.cells.iter().sum(), 0.3);
    assert!(on.organisms[0].heading > 0.0);
    close(off.organisms[0].heading, 0.0);
    same_physical_state(&off, &off_changed);
}

#[test]
fn daynight_toggle_changes_light_sensor_and_disabled_period_is_inert() {
    let run = |enabled, period| {
        let mut c = config(1);
        c.daynight.enabled = enabled;
        c.daynight.period = period;
        let mut s = setup(c);
        s.tick = 30; // darkest point of a 40-tick cycle
        sensor_to_output(&mut s, 17, 0, 0.0, 1.0);
        close(s.light_level(), if enabled { 0.0 } else { 1.0 });
        s.step();
        s
    };
    let night = run(true, 40.0);
    let off = run(false, 40.0);
    let off_changed = run(false, 80.0);
    close(night.organisms[0].heading, 0.0);
    assert!(off.organisms[0].heading > 0.1);
    same_physical_state(&off, &off_changed);
}

#[test]
fn night_vision_excludes_a_food_cue_beyond_half_range() {
    let run = |enabled| {
        let mut c = config(1);
        c.daynight.enabled = enabled;
        c.daynight.period = 40.0;
        let mut s = setup(c);
        s.tick = 30;
        // Centre-ray type: algae=0; nothing=-1. Cue gates eating locally.
        sensor_to_output(&mut s, 5, 2, 0.5, 1.0);
        let here = food_cell(&s, 500.0, 500.0);
        let cue = food_cell(&s, 540.0, 500.0);
        s.food_grid.cells[here] = 1.0;
        s.food_grid.cells[cue] = 1.0;
        s.step();
        s
    };
    let night = run(true);
    let day = run(false);
    close(night.stats.energy_food_input, 0.0);
    close(day.stats.energy_food_input, day.config.food.energy_per_food);
    close(night.food_grid.food_at(500.0, 500.0), 1.0);
    close(day.food_grid.food_at(500.0, 500.0), 0.0);
}

#[test]
fn elevation_toggle_controls_slope_motion_and_disabled_slope_is_inert() {
    let run = |enabled, slope| {
        let mut c = config(1);
        c.elevation.enabled = enabled;
        c.elevation.slope_factor = slope;
        c.creature.max_speed = 1.0;
        let mut s = setup(c);
        output(&mut s, 0, 1, 50.0);
        let expected = if enabled {
            s.elevation_map.speed_multiplier(500.0, 500.0, 501.0, 500.0)
        } else {
            1.0
        };
        assert!(expected > 0.0);
        s.step();
        close(s.organisms[0].pos_x - 500.0, expected);
        s
    };
    let on = run(true, 20.0);
    let off = run(false, 20.0);
    let off_changed = run(false, 200.0);
    assert!(on.organisms[0].pos_x < off.organisms[0].pos_x - 1e-5);
    same_physical_state(&off, &off_changed);
}

#[test]
fn construction_toggle_controls_forced_cost_and_local_regrowth() {
    let run = |enabled, bonus| {
        let mut c = config(1);
        c.niche_construction.enabled = enabled;
        c.niche_construction.deposit_rate = 0.4;
        c.niche_construction.decay_rate = 1.0;
        c.niche_construction.regrowth_bonus = bonus;
        c.niche_construction.energy_cost = 0.5;
        c.food.regrowth_rate = 0.01;
        let mut s = setup(c);
        output(&mut s, 0, 6, 50.0);
        s.step();
        s
    };
    let on = run(true, 2.0);
    let off = run(false, 2.0);
    let off_changed = run(false, 20.0);
    close(on.stats.energy_construction_cost, 0.5);
    close(off.stats.energy_construction_cost, 0.0);
    close(on.food_grid.food_at(500.0, 500.0), 0.018);
    close(off.food_grid.food_at(500.0, 500.0), 0.01);
    close(on.construction_grid.cells.iter().sum(), 0.4);
    same_physical_state(&off, &off_changed);
}

#[test]
fn epigenetic_toggle_controls_forced_stress_and_live_weight_suppression() {
    let run = |enabled, rate| {
        let mut c = config(1);
        c.epigenetics.enabled = enabled;
        c.epigenetics.methylation_rate = rate;
        c.epigenetics.brain_update_interval = 1;
        let mut s = setup(c);
        s.organisms[0].energy = 10.0; // below the 15-energy stress threshold
        output(&mut s, 0, 0, 0.5);
        s.step();
        s
    };
    let on = run(true, 0.1);
    let off = run(false, 0.1);
    let off_changed = run(false, 0.5);
    let turn_node = (on.config.brain.inputs + 1) as u16;
    let bias = |s: &Simulation| {
        s.organisms[0]
            .brain
            .connections
            .iter()
            .find(|&&(from, to, _)| from == 0 && to == turn_node)
            .unwrap()
            .2
    };
    close(bias(&on), 0.45);
    close(bias(&off), 0.5);
    assert!(
        on.organisms[0]
            .genome
            .connections
            .iter()
            .any(|c| c.methylation > 0.0)
    );
    assert!(
        off.organisms[0]
            .genome
            .connections
            .iter()
            .all(|c| c.methylation == 0.0)
    );
    same_physical_state(&off, &off_changed);
}

#[test]
fn sexual_toggle_changes_birth_mode_and_disabled_crossover_rate_is_inert() {
    let run = |sexual, crossover_rate| {
        let mut c = config(2);
        c.creature.hard_cap = 6;
        c.reproduction.sexual = sexual;
        c.reproduction.crossover_rate = crossover_rate;
        let mut s = setup(c);
        for o in &mut s.organisms {
            o.energy = 100.0;
        }
        s.step();
        s
    };
    let on = run(true, 1.0);
    let off = run(false, 1.0);
    let off_changed = run(false, 0.0);
    assert_eq!((on.stats.sexual_births, on.stats.asexual_births), (1, 0));
    assert_eq!((off.stats.sexual_births, off.stats.asexual_births), (0, 2));
    close(on.stats.energy_reproduction_cost, 60.0);
    close(on.stats.energy_offspring_input, 30.0);
    close(off.stats.energy_offspring_input, 60.0);
    same_physical_state(&off, &off_changed);
}

#[test]
fn activity_tracking_observes_forced_topology_without_perturbing_state_or_rng() {
    let run = |enabled, window| {
        let mut c = config(1);
        c.activity_tracking.enabled = enabled;
        c.activity_tracking.window_size = window;
        let mut s = setup(c);
        for _ in 0..3 {
            s.step();
        }
        s
    };
    let on = run(true, 1);
    let off = run(false, 1);
    let off_changed = run(false, 10);
    assert_eq!(on.stats.active_traits, 1);
    assert!(on.stats.mean_cumulative_activity > 0.0);
    assert_eq!(off.stats.active_traits, 0);
    close(off.stats.mean_cumulative_activity, 0.0);
    same_physical_state(&on, &off);
    same_physical_state(&off, &off_changed);
}
