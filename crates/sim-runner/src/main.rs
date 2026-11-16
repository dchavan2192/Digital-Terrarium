//! sim-runner: Headless CLI for running Terrarium experiments.
//!
//! Modes:
//! - **Single run:** Load a config, run N ticks, log stats to CSV.
//! - **Sweep:** Vary one parameter across values × multiple seeds, run all in parallel.
//!
//! Usage:
//!   cargo run -p sim-runner --release -- --config configs/default.toml --ticks 100000
//!   cargo run -p sim-runner --release -- sweep --config configs/default.toml --param food.regrowth_rate --values 0.001,0.002,0.005,0.01 --seeds 30 --ticks 100000

mod assay;
mod exported_assay;
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

    /// Seed for a fresh single run, overriding `world.seed` in the config.
    #[arg(long, conflicts_with = "resume")]
    seed: Option<u64>,

    /// Override for a fresh single run, KEY=NUMBER (repeatable),
    /// e.g. `--set food.regrowth_rate=0.008`.
    #[arg(long = "set", value_name = "KEY=NUMBER", conflicts_with = "resume")]
    set: Vec<String>,

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
    /// Clamp synthetic delayed cues into every inherited genome of one export.
    /// Prints JSON; scores do not establish learning or ecological usefulness.
    EvaluateDelayedCue {
        /// A terrarium-genomes version 1 JSON export, evaluated in full.
        #[arg(long)]
        input: PathBuf,
        /// Zero-based input channel receiving the signed synthetic cue.
        #[arg(long)]
        cue_channel: usize,
        /// Number of all-zero blank ticks before one identical all-zero probe.
        #[arg(long, default_value_t = 5)]
        blank_delay: u32,
    },
    /// Validate a synthetic delayed-cue instrument using fixed neural controls.
    /// Prints JSON; does not run an ecology or evaluate evolved cognition.
    DelayedCue {
        /// Number of all-zero blank ticks between one cue tick and one probe.
        #[arg(long, default_value_t = 5)]
        blank_delay: u32,
    },

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

    /// Print a fingerprint of the complete simulation state at chosen ticks.
    /// Equal fingerprints mean bit-identical state, so this measures whether two
    /// machines or builds follow the same trajectory. Writes no artifacts.
    Fingerprint {
        /// Path to the TOML config file.
        #[arg(short, long, default_value = "configs/default.toml")]
        config: PathBuf,

        /// Seed, overriding `world.seed` in the config.
        #[arg(long)]
        seed: Option<u64>,

        /// Fixed override, KEY=NUMBER (repeatable).
        #[arg(long = "set", value_name = "KEY=NUMBER")]
        set: Vec<String>,

        /// Comma-separated ascending ticks at which to print the fingerprint.
        #[arg(
            short,
            long,
            value_delimiter = ',',
            default_value = "1,10,100,1000,5000"
        )]
        ticks: Vec<u64>,

        /// Also write the full canonical state as `state_tick<N>.json` here, to
        /// find which values first differ between two platforms.
        #[arg(long)]
        dump_dir: Option<PathBuf>,
    },
}

fn fingerprint(
    config_path: &std::path::Path,
    seed: Option<u64>,
    overrides: &[String],
    ticks: &[u64],
    dump_dir: Option<&std::path::Path>,
) -> run::RunResult<()> {
    if ticks.is_empty() || ticks.windows(2).any(|w| w[0] >= w[1]) {
        return Err("--ticks must be a non-empty, strictly ascending list".into());
    }
    let mut config = sim_core::Config::from_file(config_path).map_err(|e| e.to_string())?;
    sweep::apply_overrides(&mut config, overrides)?;
    if let Some(seed) = seed {
        config.world.seed = seed;
    }
    config.validate().map_err(|e| e.to_string())?;
    let mut sim = sim_core::Simulation::new(config);
    for &target in ticks {
        while sim.tick < target && !sim.is_extinct() {
            sim.step();
        }
        let print = sim.state_fingerprint().map_err(|e| e.to_string())?;
        println!("tick={} fingerprint={print}", sim.tick);
        if let Some(dir) = dump_dir {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            let path = dir.join(format!("state_tick{}.json", sim.tick));
            std::fs::write(&path, sim.state_json().map_err(|e| e.to_string())?)
                .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
        if sim.is_extinct() {
            break;
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if cli.command.is_some()
        && (cli.resume.is_some()
            || cli.checkpoint_every.is_some()
            || cli.export_genomes_every.is_some()
            || cli.seed.is_some()
            || !cli.set.is_empty())
    {
        eprintln!(
            "Error: top-level resume/checkpoint/genome-export/--seed/--set options apply to single runs only; give a subcommand its own options after its name"
        );
        std::process::exit(1);
    }

    let result = match cli.command {
        Some(Commands::EvaluateDelayedCue {
            input,
            cue_channel,
            blank_delay,
        }) => exported_assay::run(&input, cue_channel, blank_delay),
        Some(Commands::DelayedCue { blank_delay }) => assay::delayed_cue(blank_delay),
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
        Some(Commands::Fingerprint {
            config,
            seed,
            set,
            ticks,
            dump_dir,
        }) => fingerprint(&config, seed, &set, &ticks, dump_dir.as_deref()),
        None => run::run_single(
            &cli.config,
            cli.ticks,
            cli.log_interval,
            &cli.output,
            &run::RunOptions {
                resume: cli.resume,
                checkpoint_every: cli.checkpoint_every,
                export_genomes_every: cli.export_genomes_every,
                seed: cli.seed,
                overrides: cli.set,
            },
        )
        .map(|_| ()),
    };
    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
