//! Forced-action accounting fixtures. These test specific energy pathways rather
//! than relying on an evolved/random world to happen to exercise the mechanism.
use sim_core::{Config, Simulation};

fn config(count: usize) -> Config {
    let mut c = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
    c.creature.initial_count = count;
    c.creature.hard_cap = 30;
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
    c.epigenetics.enabled = false;
    c.niche_construction.enabled = false;
    c.aging.extrinsic_mortality = 0.0;
    c.aging.initial_senescence = 0.0;
    c.food.droughts.spawn_rate = 0.0;
    c.food.regrowth_rate = 0.0;
    c.food.fruit.enabled = false;
    c.reproduction.sexual = false;
    c
}

fn output(s: &mut Simulation, i: usize, slot: usize, value: f64) {
    let g = &mut s.organisms[i].genome;
    let node = (g.num_inputs + 1 + slot) as u16;
    for c in &mut g.connections {
        if c.to == node {
            c.weight = if c.from == 0 { value } else { 0.0 };
            c.enabled = true;
            c.methylation = 0.0;
        }
    }
    s.organisms[i].brain = g.build_brain();
}

fn setup(c: Config) -> Simulation {
    let mut s = Simulation::new(c);
    for i in 0..s.organisms.len() {
        for slot in 0..7 {
            output(&mut s, i, slot, -50.0);
        }
        s.organisms[i].pos_x = 500.0 + i as f64 * 2.0;
        s.organisms[i].pos_y = 500.0;
        s.organisms[i].genome.body_size = 1.0;
    }
    s.food_grid.cells.fill(0.0);
    s.food_grid.fruit_cells.fill(0.0);
    s
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-8, "{actual} != {expected}");
}

fn balanced(s: &Simulation) {
    close(s.stats.energy_balance_residual, 0.0);
    close(
        s.stats.energy_after,
        s.organisms.iter().map(|o| o.energy).sum(),
    );
}

#[test]
fn reciprocal_sharing_without_food_dissipates_energy() {
    let mut c = config(2);
    c.sharing.share_amount = 5.0;
    c.sharing.transfer_efficiency = 0.9;
    let mut s = setup(c);
    for i in 0..2 {
        output(&mut s, i, 5, 50.0);
    }
    for _ in 0..10 {
        s.step();
        assert_eq!(s.stats.shares, 2);
        close(s.stats.energy_sharing_loss, 1.0);
        close(s.stats.energy_food_input + s.stats.energy_fruit_input, 0.0);
        close(s.stats.energy_after - s.stats.energy_before, -1.0);
        balanced(&s);
    }
}

#[test]
fn clipped_sharing_records_wasted_gain() {
    let mut c = config(2);
    c.sharing.transfer_efficiency = 1.0;
    c.sharing.share_amount = 5.0;
    // No offspring despite high reserves.
    c.creature.hard_cap = 2;
    let mut s = setup(c);
    s.organisms[1].energy = s.config.energy.maximum;
    output(&mut s, 0, 5, 50.0);
    s.step();
    close(s.stats.energy_sharing_loss, 0.0);
    close(s.stats.energy_cap_overflow, 5.0);
    close(s.stats.energy_after - s.stats.energy_before, -5.0);
    balanced(&s);
}

#[test]
fn feeding_records_harvest_and_overflow_separately() {
    let mut c = config(1);
    c.creature.hard_cap = 1;
    c.food.fruit.enabled = true;
    let mut s = setup(c);
    s.food_grid.cells.fill(1.0);
    s.food_grid.fruit_cells.fill(1.0);
    s.organisms[0].genome.diet_preference = 0.5;
    s.organisms[0].energy = s.config.energy.maximum - 1.0;
    output(&mut s, 0, 2, 50.0);
    s.step();
    let food = s.config.food.energy_per_food * 0.65;
    let fruit = s.config.food.fruit.energy_per_fruit * 0.65;
    close(s.stats.energy_food_input, food);
    close(s.stats.energy_fruit_input, fruit);
    close(s.stats.energy_cap_overflow, food + fruit - 1.0);
    close(s.stats.energy_after - s.stats.energy_before, 1.0);
    balanced(&s);
}

#[test]
fn births_record_both_parent_expenditure_and_newborn_reserves() {
    let mut c = config(1);
    c.reproduction.parent_cost = 35.0;
    c.reproduction.child_energy = 30.0;
    let mut s = setup(c);
    s.organisms[0].energy = s.config.reproduction.threshold;
    s.step();
    assert_eq!(s.stats.births, 1);
    close(s.stats.energy_reproduction_cost, 35.0);
    close(s.stats.energy_offspring_input, 30.0);
    close(s.stats.energy_after - s.stats.energy_before, -5.0);
    balanced(&s);
}

#[test]
fn sexual_births_include_mate_expenditure() {
    let mut c = config(2);
    c.reproduction.sexual = true;
    c.reproduction.crossover_rate = 1.0;
    let mut s = setup(c);
    for o in &mut s.organisms {
        o.energy = 100.0;
    }
    s.step();
    assert_eq!(s.stats.sexual_births, 1);
    close(s.stats.energy_reproduction_cost, 60.0);
    close(s.stats.energy_offspring_input, 30.0);
    balanced(&s);
}

#[test]
fn deaths_remove_positive_reserves_or_signed_overdrafts() {
    let mut s = setup(config(1));
    s.organisms[0].age = s.config.creature.max_age;
    s.step();
    assert_eq!(s.stats.deaths, 1);
    close(s.stats.energy_death_removal, 50.0);
    close(s.stats.energy_after, 0.0);
    balanced(&s);

    let mut c = config(1);
    c.energy.base_metabolism = 2.0;
    let mut s = setup(c);
    s.organisms[0].energy = 0.5;
    s.step();
    assert_eq!(s.stats.deaths, 1);
    close(s.stats.energy_basal_cost, 2.0);
    close(s.stats.energy_death_removal, -1.5);
    close(s.stats.energy_after, 0.0);
    balanced(&s);
}

#[test]
fn repeated_attacks_on_depleted_prey_record_only_available_energy() {
    let mut c = config(3);
    c.combat.attack_cost = 2.0;
    c.combat.attack_damage = 20.0;
    c.combat.energy_transfer_ratio = 0.75;
    let mut s = setup(c);
    // Put victim between attackers; both select the victim as nearest.
    s.organisms[0].pos_x = 498.0;
    s.organisms[1].pos_x = 502.0;
    s.organisms[2].pos_x = 500.0;
    s.organisms[2].energy = 1.0;
    output(&mut s, 0, 3, 50.0);
    output(&mut s, 1, 3, 50.0);
    s.step();
    assert_eq!(s.stats.attacks, 2);
    close(s.stats.energy_attack_cost, 4.0);
    close(s.stats.energy_combat_loss, 0.25);
    close(s.stats.energy_after - s.stats.energy_before, -4.25);
    balanced(&s);
}

#[test]
fn combat_overflow_and_idle_expenditure_are_accounted() {
    let mut c = config(2);
    c.creature.hard_cap = 2;
    c.combat.attack_cost = 0.0;
    c.combat.energy_transfer_ratio = 0.75;
    let mut s = setup(c);
    s.organisms[0].energy = s.config.energy.maximum;
    output(&mut s, 0, 3, 50.0);
    s.step();
    close(s.stats.energy_combat_loss, 5.0);
    close(s.stats.energy_cap_overflow, 15.0);
    balanced(&s);

    let mut c = config(1);
    c.combat.idle_attack_cost = 3.0;
    let mut s = setup(c);
    output(&mut s, 0, 3, 50.0);
    s.step();
    close(s.stats.energy_attack_cost, 3.0);
    balanced(&s);
}

#[test]
fn mixed_world_ledger_balances_across_ecological_events() {
    let mut c = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
    c.creature.initial_count = 30;
    c.creature.hard_cap = 60;
    let mut s = Simulation::new(c);
    let mut max_residual = 0.0_f64;
    for _ in 0..150 {
        s.step();
        max_residual = max_residual.max(s.stats.energy_balance_residual.abs());
        balanced(&s);
    }
    assert!(max_residual < 1e-8);
    eprintln!("mixed-world maximum absolute ledger residual: {max_residual:e}");
}

#[test]
fn metabolic_and_construction_components_are_measured_independently() {
    let mut c = config(2);
    c.creature.max_speed = 1.0;
    c.energy.base_metabolism = 0.3;
    c.energy.movement_cost = 0.2;
    c.energy.brain_cost = 0.001;
    c.energy.connection_cost = 0.002;
    c.energy.activity_weight_cost = 0.0001;
    c.density_pressure.cost_per_neighbor = 0.01;
    c.temperature.enabled = true;
    c.temperature.cost_factor = 0.4;
    c.niche_construction.enabled = true;
    c.niche_construction.energy_cost = 0.5;
    let mut s = setup(c);
    for i in 0..2 {
        s.organisms[i].genome.preferred_temp = 0.0;
        // Pin turn to zero so y remains exactly 500.
        output(&mut s, i, 0, 0.0);
        s.organisms[i].heading = 0.0;
        output(&mut s, i, 1, 50.0);
        output(&mut s, i, 6, 50.0);
    }
    let structure: f64 = s
        .organisms
        .iter()
        .map(|o| o.brain.neuron_count() as f64 * 0.001 + o.brain.connection_count() as f64 * 0.002)
        .sum();
    s.step();
    close(s.stats.energy_basal_cost, 0.6);
    close(s.stats.energy_movement_cost, 0.4);
    close(s.stats.energy_brain_structure_cost, structure);
    assert!(s.stats.energy_brain_activity_cost > 0.0);
    close(s.stats.energy_crowding_cost, 0.02);
    close(s.stats.energy_temperature_cost, 0.12);
    close(s.stats.energy_construction_cost, 1.0);
    balanced(&s);
}
