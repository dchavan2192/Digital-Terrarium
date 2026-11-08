//! Parameter sweep: vary one parameter × multiple seeds, run in parallel.

use rayon::prelude::*;
use sim_core::Config;
use std::path::Path;

use crate::run;

/// Run a parameter sweep: for each value, run `seeds` simulations in parallel.
pub fn run_sweep(
    config_path: &Path,
    param: &str,
    values: &[f64],
    seeds: u64,
    ticks: u64,
    log_interval: u64,
    output_dir: &Path,
) {
    let base_config = Config::from_file(config_path).unwrap_or_else(|e| {
        eprintln!("Failed to load config '{}': {}", config_path.display(), e);
        std::process::exit(1);
    });

    let total_runs = values.len() as u64 * seeds;
    println!(
        "=== SWEEP: {} = {:?}, {} seeds each, {} total runs ===",
        param, values, seeds, total_runs
    );

    // Build all (config, label) pairs
    let mut jobs: Vec<(Config, String)> = Vec::new();

    for &value in values {
        for seed in 0..seeds {
            let mut config = base_config.clone();
            config.world.seed = seed;

            if !apply_param(&mut config, param, value) {
                eprintln!("Unknown parameter: '{}'. Supported parameters:", param);
                print_supported_params();
                std::process::exit(1);
            }

            let label = format!("{}={:.6}_seed{}", param, value, seed);
            jobs.push((config, label));
        }
    }

    // Run all jobs in parallel
    let results: Vec<(String, u64, bool)> = jobs
        .into_par_iter()
        .map(|(config, label)| {
            let sweep_dir = output_dir.join("sweep");
            let (final_tick, survived) =
                run::run_with_config(config, ticks, log_interval, &sweep_dir, Some(&label));
            (label, final_tick, survived)
        })
        .collect();

    // Write summary CSV
    let summary_path = output_dir.join("sweep_summary.csv");
    let mut wtr = csv::Writer::from_path(&summary_path).expect("failed to create summary CSV");
    wtr.write_record(["label", "param", "value", "seed", "final_tick", "survived"])
        .expect("failed to write header");

    for (label, final_tick, survived) in &results {
        // Label format: "param.name=VALUE_seedN" — split on last "_seed"
        let (param_val, seed_str) = label
            .rsplit_once("_seed")
            .map(|(pv, s)| (pv, s))
            .unwrap_or((label.as_str(), "?"));
        let value_str = param_val
            .rsplit_once('=')
            .map(|(_, v)| v)
            .unwrap_or("?");

        wtr.write_record([
            label.as_str(),
            param,
            value_str,
            seed_str,
            &final_tick.to_string(),
            &survived.to_string(),
        ])
        .expect("failed to write summary row");
    }
    wtr.flush().expect("failed to flush summary CSV");

    // Print summary
    println!("\n=== SWEEP COMPLETE ===");
    println!("Summary: {}", summary_path.display());

    for &value in values {
        let runs: Vec<&(String, u64, bool)> = results
            .iter()
            .filter(|(l, _, _)| l.starts_with(&format!("{}={:.6}", param, value)))
            .collect();
        let survived = runs.iter().filter(|(_, _, s)| *s).count();
        let mean_tick: f64 = runs.iter().map(|(_, t, _)| *t as f64).sum::<f64>() / runs.len() as f64;
        println!(
            "  {} = {:.6}: {}/{} survived, mean final tick = {:.0}",
            param,
            value,
            survived,
            runs.len(),
            mean_tick
        );
    }
}

/// Apply a parameter value to a config by dot-separated path.
/// Returns false if the parameter is unknown.
fn apply_param(config: &mut Config, param: &str, value: f64) -> bool {
    match param {
        // World
        "world.width" => config.world.width = value,
        "world.height" => config.world.height = value,

        // Food
        "food.cell_size" => config.food.cell_size = value,
        "food.max_per_cell" => config.food.max_per_cell = value,
        "food.regrowth_rate" => config.food.regrowth_rate = value,
        "food.energy_per_food" => config.food.energy_per_food = value,

        // Creature
        "creature.initial_count" => config.creature.initial_count = value as usize,
        "creature.hard_cap" => config.creature.hard_cap = value as usize,
        "creature.body_radius" => config.creature.body_radius = value,
        "creature.max_speed" => config.creature.max_speed = value,
        "creature.max_turn" => config.creature.max_turn = value,
        "creature.max_age" => config.creature.max_age = value as u64,
        "creature.vision.ray_count" => config.creature.vision.ray_count = value as usize,
        "creature.vision.fov" => config.creature.vision.fov = value,
        "creature.vision.range" => config.creature.vision.range = value,

        // Energy
        "energy.starting" => config.energy.starting = value,
        "energy.maximum" => config.energy.maximum = value,
        "energy.base_metabolism" => config.energy.base_metabolism = value,
        "energy.movement_cost" => config.energy.movement_cost = value,
        "energy.brain_cost" => config.energy.brain_cost = value,

        // Reproduction
        "reproduction.threshold" => config.reproduction.threshold = value,
        "reproduction.parent_cost" => config.reproduction.parent_cost = value,
        "reproduction.child_energy" => config.reproduction.child_energy = value,

        // Mutation
        "mutation.weight_probability" => config.mutation.weight_probability = value,
        "mutation.weight_step" => config.mutation.weight_step = value,

        _ => return false,
    }
    true
}

fn print_supported_params() {
    let params = [
        "world.width", "world.height",
        "food.cell_size", "food.max_per_cell", "food.regrowth_rate", "food.energy_per_food",
        "creature.initial_count", "creature.hard_cap", "creature.body_radius",
        "creature.max_speed", "creature.max_turn", "creature.max_age",
        "creature.vision.ray_count", "creature.vision.fov", "creature.vision.range",
        "energy.starting", "energy.maximum", "energy.base_metabolism",
        "energy.movement_cost", "energy.brain_cost",
        "reproduction.threshold", "reproduction.parent_cost", "reproduction.child_energy",
        "mutation.weight_probability", "mutation.weight_step",
    ];
    for p in params {
        eprintln!("  {}", p);
    }
}
