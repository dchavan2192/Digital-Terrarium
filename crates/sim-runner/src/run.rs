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

#[derive(Serialize)]
struct Manifest<'a> {
    schema_version: u32,
    engine_version: &'static str,
    source_revision: &'static str,
    source_dirty: &'static str,
    target: &'static str,
    build_profile: &'static str,
    compiler: &'static str,
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
) -> RunResult<(u64, bool)> {
    validate_request(ticks, log_interval)?;
    let config = Config::from_file(config_path).map_err(|e| e.to_string())?;
    let session = create_session(output)?;
    run_with_config(config, ticks, log_interval, &session, None)
}

/// Writes one run into a unique session. Called only after all sweep jobs validate.
pub fn run_with_config(
    config: Config,
    ticks: u64,
    log_interval: u64,
    output: &Path,
    label: Option<&str>,
) -> RunResult<(u64, bool)> {
    validate_request(ticks, log_interval)?;
    config.validate().map_err(|e| e.to_string())?;
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
        schema_version: 1,
        engine_version: env!("CARGO_PKG_VERSION"),
        source_revision: env!("TERRARIUM_SOURCE_REVISION"),
        source_dirty: env!("TERRARIUM_SOURCE_DIRTY"),
        target: env!("TERRARIUM_TARGET"),
        build_profile: env!("TERRARIUM_BUILD_PROFILE"),
        compiler: env!("TERRARIUM_COMPILER"),
        stats_schema_version: 2,
        topology_hash_version: config.activity_tracking.hash_version,
        environmental_rng: "shared_with_evolution; same seed does not synchronize droughts across feature arms",
        seed,
        requested_ticks: ticks,
        completed_ticks: 0,
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
    let mut completed_ticks = 0;
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
        let mut sim = Simulation::new(config.clone());
        println!("[seed={seed}] Starting {ticks} ticks");
        for t in 1..=ticks {
            sim.step();
            completed_ticks = sim.tick;
            survived = !sim.is_extinct();
            if t % log_interval == 0 || !survived || t == ticks {
                writer
                    .serialize(&sim.stats)
                    .map_err(|e| format!("write stats: {e}"))?;
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
