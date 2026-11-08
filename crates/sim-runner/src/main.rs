//! sim-runner: Headless CLI for running Terrarium experiments.
//!
//! Modes:
//! - **Single run:** Load a config, run N ticks, log stats to CSV.
//! - **Sweep:** Vary one parameter across values × multiple seeds, run all in parallel.
//!
//! Usage:
//!   cargo run -p sim-runner --release -- --config configs/default.toml --ticks 100000
//!   cargo run -p sim-runner --release -- sweep --config configs/default.toml --param food.regrowth_rate --values 0.001,0.002,0.005,0.01 --seeds 30 --ticks 100000

mod run;
mod sweep;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "sim-runner",
    about = "Headless experiment runner for Terrarium"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the TOML config file.
    #[arg(short, long, default_value = "configs/default.toml")]
    config: PathBuf,

    /// Total ticks to simulate.
    #[arg(short, long, default_value_t = 100_000)]
    ticks: u64,

    /// Log stats every N ticks.
    #[arg(short, long, default_value_t = 100)]
    log_interval: u64,

    /// Output directory for CSV logs.
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Sweep one parameter across multiple values and seeds.
    Sweep {
        /// Path to the base TOML config file.
        #[arg(short, long, default_value = "configs/default.toml")]
        config: PathBuf,

        /// Parameter to sweep (dot-separated path, e.g. "food.regrowth_rate").
        #[arg(short, long)]
        param: String,

        /// Comma-separated list of values to try.
        #[arg(short, long, value_delimiter = ',')]
        values: Vec<f64>,

        /// Number of seeds per value.
        #[arg(short, long, default_value_t = 30)]
        seeds: u64,

        /// Total ticks per run.
        #[arg(short, long, default_value_t = 100_000)]
        ticks: u64,

        /// Log stats every N ticks.
        #[arg(short, long, default_value_t = 100)]
        log_interval: u64,

        /// Output directory.
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Sweep {
            config,
            param,
            values,
            seeds,
            ticks,
            log_interval,
            output,
        }) => {
            sweep::run_sweep(
                &config,
                &param,
                &values,
                seeds,
                ticks,
                log_interval,
                &output,
            );
        }
        None => {
            run::run_single(&cli.config, cli.ticks, cli.log_interval, &cli.output);
        }
    }
}
