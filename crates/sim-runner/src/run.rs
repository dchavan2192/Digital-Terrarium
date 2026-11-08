//! Single-run logic: load config, simulate, write stats CSV.

use sim_core::{Config, Simulation};
use std::path::Path;
use std::time::Instant;

/// Run a single simulation and write stats to CSV.
/// Returns (final_tick, survived: bool).
pub fn run_single(
    config_path: &Path,
    ticks: u64,
    log_interval: u64,
    output_dir: &Path,
) -> (u64, bool) {
    let config = Config::from_file(config_path).unwrap_or_else(|e| {
        eprintln!("Failed to load config '{}': {}", config_path.display(), e);
        std::process::exit(1);
    });

    run_with_config(config, ticks, log_interval, output_dir, None)
}

/// Run a simulation from an in-memory config.
/// `label` is an optional suffix for the output filename (used by sweeps).
pub fn run_with_config(
    config: Config,
    ticks: u64,
    log_interval: u64,
    output_dir: &Path,
    label: Option<&str>,
) -> (u64, bool) {
    let seed = config.world.seed;

    // Save the config alongside results for reproducibility
    std::fs::create_dir_all(output_dir).expect("failed to create output directory");

    let csv_name = match label {
        Some(l) => format!("stats_seed{}_{}.csv", seed, l),
        None => format!("stats_seed{}.csv", seed),
    };
    let csv_path = output_dir.join(&csv_name);

    // Save config
    let config_name = match label {
        Some(l) => format!("config_seed{}_{}.toml", seed, l),
        None => format!("config_seed{}.toml", seed),
    };
    let config_toml = toml::to_string_pretty(&config).expect("failed to serialize config");
    std::fs::write(output_dir.join(&config_name), config_toml).expect("failed to write config");

    // Set up CSV writer
    let mut wtr = csv::Writer::from_path(&csv_path).expect("failed to create CSV file");

    let mut sim = Simulation::new(config);
    let start = Instant::now();

    println!(
        "[seed={}{}] Starting {} ticks...",
        seed,
        label.map(|l| format!(" {}", l)).unwrap_or_default(),
        ticks
    );

    // Log initial state
    wtr.serialize(&sim.stats).expect("failed to write CSV row");

    for t in 1..=ticks {
        sim.step();

        if t % log_interval == 0 {
            wtr.serialize(&sim.stats).expect("failed to write CSV row");
        }

        if sim.is_extinct() {
            // Log the extinction tick
            wtr.serialize(&sim.stats).expect("failed to write CSV row");
            break;
        }

        // Progress report every 10% of the run
        if t % (ticks / 10).max(1) == 0 {
            let pct = (t as f64 / ticks as f64 * 100.0) as u32;
            let elapsed = start.elapsed().as_secs_f64();
            let tps = t as f64 / elapsed;
            eprint!(
                "\r[seed={}{}] {}% — tick {}/{}, pop={}, {:.0} ticks/s",
                seed,
                label.map(|l| format!(" {}", l)).unwrap_or_default(),
                pct,
                t,
                ticks,
                sim.stats.population,
                tps
            );
        }
    }

    wtr.flush().expect("failed to flush CSV");
    let elapsed = start.elapsed();
    let survived = !sim.is_extinct();

    eprintln!();
    println!(
        "[seed={}{}] Done in {:.1}s — {} at tick {}, pop={}. CSV: {}",
        seed,
        label.map(|l| format!(" {}", l)).unwrap_or_default(),
        elapsed.as_secs_f64(),
        if survived { "SURVIVED" } else { "EXTINCT" },
        sim.tick,
        sim.stats.population,
        csv_path.display(),
    );

    (sim.tick, survived)
}
