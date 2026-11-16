//! Single-run execution with non-overwriting artifacts and explicit provenance.

use serde::Serialize;
use sim_core::{Config, Simulation};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub type RunResult<T> = Result<T, String>;
static NEXT_SESSION: AtomicU64 = AtomicU64::new(0);

/// One unique output root per invocation. Existing artifacts are never reused.
pub fn create_session(output: &Path) -> RunResult<PathBuf> {
    std::fs::create_dir_all(output).map_err(|e| format!("create output: {e}"))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let id = NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
    let path = output.join(format!("session_{stamp}_{}_{}", std::process::id(), id));
    std::fs::create_dir(&path).map_err(|e| format!("create unique session: {e}"))?;
    println!("Session: {}", path.display());
    Ok(path)
}

pub fn validate_request(ticks: u64, log_interval: u64) -> RunResult<()> {
    if ticks == 0 || log_interval == 0 {
        return Err("--ticks and --log-interval must be positive".into());
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct RunOptions {
    pub resume: Option<PathBuf>,
    pub checkpoint_every: Option<u64>,
    pub export_genomes_every: Option<u64>,
    /// Seed override for a fresh single run (not valid with `resume`).
    pub seed: Option<u64>,
    /// `KEY=NUMBER` overrides for a fresh single run (not valid with `resume`).
    pub overrides: Vec<String>,
}

impl RunOptions {
    fn validate(&self) -> RunResult<()> {
        if self.checkpoint_every == Some(0) || self.export_genomes_every == Some(0) {
            return Err("checkpoint/export intervals must be positive".into());
        }
        if self.resume.is_some() && (self.seed.is_some() || !self.overrides.is_empty()) {
            return Err(
                "--seed/--set cannot change a resumed run: it continues the checkpoint's own configuration"
                    .into(),
            );
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct Manifest<'a> {
    schema_version: u32,
    engine_version: &'static str,
    source_revision: &'static str,
    source_dirty: &'static str,
    target: &'static str,
    build_profile: &'static str,
    compiler: &'static str,
    core_build_id: &'static str,
    checkpoint_version: u32,
    resumed_from: Option<String>,
    starting_tick: u64,
    simulated_ticks: u64,
    checkpoint_every: Option<u64>,
    export_genomes_every: Option<u64>,
    stats_schema_version: u32,
    topology_hash_version: u32,
    environmental_rng: &'static str,
    seed: u64,
    requested_ticks: u64,
    completed_ticks: u64,
    log_interval: u64,
    status: &'a str,
    survived: bool,
    elapsed_seconds: f64,
    error: &'a str,
    config: &'a Config,
}

pub fn run_single(
    config_path: &Path,
    ticks: u64,
    log_interval: u64,
    output: &Path,
    options: &RunOptions,
) -> RunResult<(u64, bool)> {
    options.validate()?;
    validate_request(ticks, log_interval)?;
    let resumed = options
        .resume
        .as_ref()
        .map(|path| {
            Simulation::load_checkpoint(path)
                .map_err(|e| format!("cannot resume {}: {e}", path.display()))
        })
        .transpose()?;
    if let Some(sim) = &resumed {
        if ticks <= sim.tick {
            return Err(format!(
                "--ticks is an absolute target: {ticks} must exceed checkpoint tick {}",
                sim.tick
            ));
        }
        if sim.is_extinct() {
            return Err("cannot resume an extinct population".into());
        }
    }
    let config = match &resumed {
        Some(sim) => sim.config.clone(),
        None => {
            let mut config = Config::from_file(config_path).map_err(|e| e.to_string())?;
            crate::sweep::apply_overrides(&mut config, &options.overrides)?;
            if let Some(seed) = options.seed {
                config.world.seed = seed;
            }
            // Validate before any artifact exists, as sweeps do for generated jobs.
            config
                .validate()
                .map_err(|e| format!("invalid configuration after overrides: {e}"))?;
            config
        }
    };
    let session = create_session(output)?;
    execute(
        config,
        ticks,
        log_interval,
        &session,
        None,
        options,
        resumed,
    )
}

/// Writes one run into a unique session. Called only after all sweep jobs validate.
pub fn run_with_config(
    config: Config,
    ticks: u64,
    log_interval: u64,
    output: &Path,
    label: Option<&str>,
) -> RunResult<(u64, bool)> {
    execute(
        config,
        ticks,
        log_interval,
        output,
        label,
        &RunOptions::default(),
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute(
    config: Config,
    ticks: u64,
    log_interval: u64,
    output: &Path,
    label: Option<&str>,
    options: &RunOptions,
    resumed: Option<Simulation>,
) -> RunResult<(u64, bool)> {
    validate_request(ticks, log_interval)?;
    options.validate()?;
    config.validate().map_err(|e| e.to_string())?;
    let starting_tick = resumed.as_ref().map_or(0, |sim| sim.tick);
    std::fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let seed = config.world.seed;
    let suffix = label
        .map(|l| format!("seed{seed}_{l}"))
        .unwrap_or_else(|| format!("seed{seed}"));
    let csv_path = output.join(format!("stats_{suffix}.csv"));
    let config_path = output.join(format!("config_{suffix}.toml"));
    let manifest_path = output.join(format!("manifest_{suffix}.toml"));
    // Reserve the identity first; never truncate a previous run or append mixed data.
    let mut manifest_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&manifest_path)
        .map_err(|e| format!("reserve {}: {e}", manifest_path.display()))?;
    let started = Instant::now();
    let mut manifest = Manifest {
        schema_version: 2,
        engine_version: env!("CARGO_PKG_VERSION"),
        source_revision: env!("TERRARIUM_SOURCE_REVISION"),
        source_dirty: env!("TERRARIUM_SOURCE_DIRTY"),
        target: env!("TERRARIUM_TARGET"),
        build_profile: env!("TERRARIUM_BUILD_PROFILE"),
        compiler: env!("TERRARIUM_COMPILER"),
        core_build_id: sim_core::checkpoint::CORE_BUILD_ID,
        checkpoint_version: sim_core::checkpoint::CHECKPOINT_VERSION,
        resumed_from: options.resume.as_ref().map(|p| p.display().to_string()),
        starting_tick,
        simulated_ticks: 0,
        checkpoint_every: options.checkpoint_every,
        export_genomes_every: options.export_genomes_every,
        stats_schema_version: 2,
        topology_hash_version: config.activity_tracking.hash_version,
        environmental_rng: "shared_with_evolution; same seed does not synchronize droughts across feature arms",
        seed,
        requested_ticks: ticks,
        completed_ticks: starting_tick,
        log_interval,
        status: "running",
        survived: true,
        elapsed_seconds: 0.0,
        error: "",
        config: &config,
    };
    use std::io::{Seek, Write};
    let write_manifest = |file: &mut std::fs::File, value: &Manifest<'_>| -> RunResult<()> {
        let text = toml::to_string_pretty(value).map_err(|e| e.to_string())?;
        file.rewind().map_err(|e| e.to_string())?;
        file.set_len(0).map_err(|e| e.to_string())?;
        file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())
    };
    write_manifest(&mut manifest_file, &manifest)?;
    let mut completed_ticks = starting_tick;
    let mut survived = true;
    // Catch simulation panics so artifacts say failed, not complete. A killed process
    // leaves status=running, which is explicitly incomplete and must not enter analysis.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> RunResult<()> {
        let config_text = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
        let mut config_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&config_path)
            .map_err(|e| e.to_string())?;
        config_file
            .write_all(config_text.as_bytes())
            .map_err(|e| e.to_string())?;
        let csv_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&csv_path)
            .map_err(|e| e.to_string())?;
        let mut writer = csv::Writer::from_writer(csv_file);
        let mut sim = resumed.unwrap_or_else(|| Simulation::new(config.clone()));
        println!("[seed={seed}] Starting at tick {starting_tick}, absolute target {ticks}");
        for t in starting_tick + 1..=ticks {
            sim.step();
            completed_ticks = sim.tick;
            survived = !sim.is_extinct();
            if t % log_interval == 0 || !survived || t == ticks {
                writer
                    .serialize(&sim.stats)
                    .map_err(|e| format!("write stats: {e}"))?;
            }
            let final_step = !survived || t == ticks;
            if options
                .checkpoint_every
                .is_some_and(|interval| t % interval == 0 || final_step)
            {
                let path = output.join(format!("checkpoint_{suffix}_tick{t}.json"));
                sim.save_checkpoint(&path)
                    .map_err(|e| format!("checkpoint: {e}"))?;
                println!("Checkpoint: {}", path.display());
            }
            if options
                .export_genomes_every
                .is_some_and(|interval| t % interval == 0 || final_step)
            {
                let path = output.join(format!("genomes_{suffix}_tick{t}.json"));
                sim.export_genomes(&path)
                    .map_err(|e| format!("genome export: {e}"))?;
                println!("Genomes: {}", path.display());
            }
            if !survived {
                break;
            }
        }
        writer.flush().map_err(|e| format!("flush stats: {e}"))?;
        Ok(())
    }));
    let error = match outcome {
        Ok(Ok(())) => None,
        Ok(Err(e)) => Some(e),
        Err(payload) => Some(
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "simulation panicked".into()),
        ),
    };
    manifest.completed_ticks = completed_ticks;
    manifest.simulated_ticks = completed_ticks - starting_tick;
    manifest.survived = survived;
    manifest.elapsed_seconds = started.elapsed().as_secs_f64();
    manifest.status = if error.is_some() {
        "failed"
    } else if survived {
        "completed"
    } else {
        "extinct"
    };
    manifest.error = error.as_deref().unwrap_or("");
    write_manifest(&mut manifest_file, &manifest)?;
    if let Some(error) = error {
        return Err(format!("seed {seed} failed: {error}"));
    }
    println!(
        "[seed={seed}] {} at tick {completed_ticks}; CSV: {}",
        manifest.status,
        csv_path.display()
    );
    Ok((completed_ticks, survived))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_artifact_write_marks_run_failed_and_refuses_retry() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/runner-tests/failure");
        let session = create_session(&root).unwrap();
        let config = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
        let config_path = session.join("config_seed42.toml");
        std::fs::write(&config_path, "preserve-existing-data").unwrap();
        assert!(run_with_config(config.clone(), 1, 1, &session, None).is_err());
        let path = session.join("manifest_seed42.toml");
        let manifest: toml::Value =
            toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(manifest["status"].as_str(), Some("failed"));
        assert_eq!(manifest["completed_ticks"].as_integer(), Some(0));
        assert_eq!(
            std::fs::read_to_string(config_path).unwrap(),
            "preserve-existing-data"
        );
        let original = std::fs::read_to_string(&path).unwrap();
        assert!(run_with_config(config, 1, 1, &session, None).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), original);
    }
}
