//! Parameter sweep: vary one parameter × multiple seeds, run in parallel.

use rayon::prelude::*;
use sim_core::Config;
use std::path::Path;

use crate::run;

/// Run a parameter sweep: for each value, run `seeds` simulations in parallel.
#[allow(clippy::too_many_arguments)]
pub fn run_sweep(
    config_path: &Path,
    overrides: &[String],
    param: &str,
    values: &[f64],
    seeds: u64,
    ticks: u64,
    log_interval: u64,
    output_dir: &Path,
) -> run::RunResult<()> {
    run::validate_request(ticks, log_interval)?;
    if seeds == 0 || values.is_empty() {
        return Err("--seeds must be positive and --values must not be empty".into());
    }
    let mut base_config = Config::from_file(config_path).unwrap_or_else(|e| {
        eprintln!("Failed to load config '{}': {}", config_path.display(), e);
        std::process::exit(1);
    });
    // Fixed overrides applied to every run (e.g. `--set creature.hard_cap=2000`), so
    // experiments don't need full copies of default.toml that drift out of date.
    for ov in overrides {
        let parsed = ov
            .split_once('=')
            .and_then(|(k, v)| v.trim().parse::<f64>().ok().map(|v| (k.trim(), v)));
        match parsed {
            Some((key, value)) if apply_param(&mut base_config, key, value) => {
                println!("override: {key} = {value}");
            }
            _ => {
                eprintln!("Bad --set '{ov}': expected KEY=NUMBER with a supported KEY:");
                print_supported_params();
                std::process::exit(1);
            }
        }
    }

    let total_runs = values.len() as u64 * seeds;
    println!(
        "=== SWEEP: {} = {:?}, {} seeds each, {} total runs ===",
        param, values, seeds, total_runs
    );

    // Build all (config, label) pairs
    let mut jobs: Vec<(Config, String)> = Vec::new();
    let mut labels = std::collections::HashSet::new();

    for &value in values {
        for seed in 0..seeds {
            let mut config = base_config.clone();
            config.world.seed = seed;

            if !apply_param(&mut config, param, value) {
                eprintln!("Unknown parameter: '{}'. Supported parameters:", param);
                print_supported_params();
                std::process::exit(1);
            }

            config
                .validate()
                .map_err(|e| format!("invalid generated job {param}={value}, seed={seed}: {e}"))?;
            let label = format!("{}={:.6}_seed{}", param, value, seed);
            if !labels.insert(label.clone()) {
                return Err(format!(
                    "duplicate output identity {label}: sweep values must be distinct at six decimal places"
                ));
            }
            jobs.push((config, label));
        }
    }

    let session = run::create_session(output_dir)?;

    // Run all jobs in parallel
    let results: Vec<run::RunResult<(String, u64, bool)>> = jobs
        .into_par_iter()
        .map(|(config, label)| {
            let sweep_dir = session.join("sweep");
            let (final_tick, survived) =
                run::run_with_config(config, ticks, log_interval, &sweep_dir, Some(&label))?;
            Ok((label, final_tick, survived))
        })
        .collect();

    let results: Vec<(String, u64, bool)> = results.into_iter().collect::<run::RunResult<_>>()?;

    // Write summary CSV
    let summary_path = session.join("sweep_summary.csv");
    let mut wtr = csv::Writer::from_path(&summary_path).expect("failed to create summary CSV");
    wtr.write_record(["label", "param", "value", "seed", "final_tick", "survived"])
        .expect("failed to write header");

    for (label, final_tick, survived) in &results {
        // Label format: "param.name=VALUE_seedN" — split on last "_seed"
        let (param_val, seed_str) = label.rsplit_once("_seed").unwrap_or((label.as_str(), "?"));
        let value_str = param_val.rsplit_once('=').map(|(_, v)| v).unwrap_or("?");

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
        let mean_tick: f64 =
            runs.iter().map(|(_, t, _)| *t as f64).sum::<f64>() / runs.len() as f64;
        println!(
            "  {} = {:.6}: {}/{} survived, mean final tick = {:.0}",
            param,
            value,
            survived,
            runs.len(),
            mean_tick
        );
    }
    Ok(())
}

/// Apply a parameter value to a config by dot-separated path.
/// Returns false if the parameter is unknown.
fn apply_param(config: &mut Config, param: &str, value: f64) -> bool {
    if !value.is_finite() {
        return false;
    }
    let integer = matches!(
        param,
        "creature.initial_count"
            | "creature.hard_cap"
            | "creature.max_age"
            | "creature.vision.ray_count"
    );
    if integer && (value < 0.0 || value.fract() != 0.0 || value >= usize::MAX as f64) {
        return false;
    }
    if param == "creature.vision.ray_count" && value > sim_core::config::MAX_RAYS as f64 {
        return false;
    }
    let toggle = matches!(
        param,
        "reproduction.sexual" | "niche_construction.enabled" | "epigenetics.enabled"
    );
    if toggle && value != 0.0 && value != 1.0 {
        return false;
    }
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
        "creature.vision.ray_count" => {
            // Brain inputs are 2 per ray + fixed scalar senses; keep them in sync.
            config.creature.vision.ray_count = value as usize;
            config.brain.inputs =
                2 * config.creature.vision.ray_count + sim_core::config::NUM_SCALAR_INPUTS;
        }
        "creature.vision.fov" => config.creature.vision.fov = value,
        "creature.vision.range" => config.creature.vision.range = value,

        // Energy
        "energy.starting" => config.energy.starting = value,
        "energy.maximum" => config.energy.maximum = value,
        "energy.base_metabolism" => config.energy.base_metabolism = value,
        "energy.movement_cost" => config.energy.movement_cost = value,
        "energy.brain_cost" => config.energy.brain_cost = value,
        "energy.connection_cost" => config.energy.connection_cost = value,
        "energy.activity_weight_cost" => config.energy.activity_weight_cost = value,
        "sharing.transfer_efficiency" => config.sharing.transfer_efficiency = value,
        "reproduction.mate_cost" => config.reproduction.mate_cost = Some(value),

        // Reproduction
        "reproduction.threshold" => config.reproduction.threshold = value,
        "reproduction.parent_cost" => config.reproduction.parent_cost = value,
        "reproduction.child_energy" => config.reproduction.child_energy = value,

        // Mutation
        "mutation.weight_probability" => config.mutation.weight_probability = value,
        "mutation.weight_step" => config.mutation.weight_step = value,

        // Aging (Gompertz-Makeham)
        "aging.extrinsic_mortality" => config.aging.extrinsic_mortality = value,
        "aging.initial_senescence" => config.aging.initial_senescence = value,
        "aging.aging_rate" => config.aging.aging_rate = value,

        // Combat
        "combat.attack_range" => config.combat.attack_range = value,
        "combat.attack_cost" => config.combat.attack_cost = value,
        "combat.attack_damage" => config.combat.attack_damage = value,
        "combat.energy_transfer_ratio" => config.combat.energy_transfer_ratio = value,

        // Sharing
        "sharing.share_range" => config.sharing.share_range = value,
        "sharing.share_amount" => config.sharing.share_amount = value,

        // Rule parameters changed on 2026-11-15 (old values reproduce the old rules)
        "combat.idle_attack_cost" => config.combat.idle_attack_cost = value,
        "niche_construction.activation_threshold" => {
            config.niche_construction.activation_threshold = value
        }
        "mutation.prune_probability" => config.mutation.prune_probability = value,
        "creature.feeding_speed_factor" => config.creature.feeding_speed_factor = value,

        // Feature toggles (0 = off, nonzero = on) for ON/OFF ablation experiments
        "reproduction.sexual" => config.reproduction.sexual = value != 0.0,
        "niche_construction.enabled" => config.niche_construction.enabled = value != 0.0,
        "epigenetics.enabled" => config.epigenetics.enabled = value != 0.0,

        _ => return false,
    }
    true
}

fn print_supported_params() {
    let params = [
        "world.width",
        "world.height",
        "food.cell_size",
        "food.max_per_cell",
        "food.regrowth_rate",
        "food.energy_per_food",
        "creature.initial_count",
        "creature.hard_cap",
        "creature.body_radius",
        "creature.max_speed",
        "creature.max_turn",
        "creature.max_age",
        "creature.vision.ray_count",
        "creature.vision.fov",
        "creature.vision.range",
        "energy.starting",
        "energy.maximum",
        "energy.base_metabolism",
        "energy.movement_cost",
        "energy.brain_cost",
        "energy.connection_cost",
        "energy.activity_weight_cost",
        "sharing.transfer_efficiency",
        "reproduction.mate_cost",
        "reproduction.threshold",
        "reproduction.parent_cost",
        "reproduction.child_energy",
        "mutation.weight_probability",
        "mutation.weight_step",
        "aging.extrinsic_mortality",
        "aging.initial_senescence",
        "aging.aging_rate",
        "combat.attack_range",
        "combat.attack_cost",
        "combat.attack_damage",
        "combat.energy_transfer_ratio",
        "combat.idle_attack_cost",
        "niche_construction.activation_threshold",
        "mutation.prune_probability",
        "creature.feeding_speed_factor",
        "reproduction.sexual",
        "niche_construction.enabled",
        "epigenetics.enabled",
        "sharing.share_range",
        "sharing.share_amount",
    ];
    for p in params {
        eprintln!("  {}", p);
    }
}
