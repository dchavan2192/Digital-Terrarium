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
    #[arg(
        short,
        long,
        default_value = "configs/default.toml",
        conflicts_with = "resume"
    )]
    config: PathBuf,

    /// Resume an exact same-build simulation checkpoint (uses its embedded config).
    #[arg(long)]
    resume: Option<PathBuf>,

    /// Save a complete checkpoint every N ticks, and at the end (single run only).
    #[arg(long)]
    checkpoint_every: Option<u64>,

    /// Export living genomes every N ticks, and at the end (single run only).
    #[arg(long)]
    export_genomes_every: Option<u64>,

    /// Absolute target tick (a resumed run executes only the remaining ticks).
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

        /// Fixed override applied to every run, KEY=NUMBER (repeatable),
        /// e.g. `--set creature.hard_cap=2000`.
        #[arg(long = "set", value_name = "KEY=NUMBER")]
        set: Vec<String>,

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
    if cli.command.is_some()
        && (cli.resume.is_some()
            || cli.checkpoint_every.is_some()
            || cli.export_genomes_every.is_some())
    {
        eprintln!(
            "Error: resume/checkpoint/genome-export options currently apply to single runs only"
        );
        std::process::exit(1);
    }

    let result = match cli.command {
        Some(Commands::Sweep {
            config,
            set,
            param,
            values,
            seeds,
            ticks,
            log_interval,
            output,
        }) => sweep::run_sweep(
            &config,
            &set,
            &param,
            &values,
            seeds,
            ticks,
            log_interval,
            &output,
        ),
        None => run::run_single(
            &cli.config,
            cli.ticks,
            cli.log_interval,
            &cli.output,
            &run::RunOptions {
                resume: cli.resume,
                checkpoint_every: cli.checkpoint_every,
                export_genomes_every: cli.export_genomes_every,
            },
        )
        .map(|_| ()),
    };
    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
