//! Complete-population export wrapper for the isolated delayed-choice arena.
use crate::delayed_choice::{self, ArenaConfig};
use crate::exported_assay;
use crate::run::RunResult;
use serde_json::{Value, json};
use std::path::Path;

const MAX_TRACE_SAMPLES: u64 = 100_000;

fn task_steps(config: &ArenaConfig) -> u64 {
    u64::from(config.cue_ticks) + u64::from(config.blank_delay) + u64::from(config.horizon)
}

fn trace_budget(config: &ArenaConfig, trials: u64) -> RunResult<()> {
    if (task_steps(config) + 1)
        .checked_mul(trials)
        .is_none_or(|n| n > MAX_TRACE_SAMPLES)
    {
        return Err("arena exceeds 100000 trace-sample budget; use a separately planned shorter task, not score-based subsampling".into());
    }
    Ok(())
}

pub fn controls(path: &Path, cue_visible: bool, delay_end_reset: bool) -> RunResult<()> {
    let config = delayed_choice::parse_config(path)?;
    trace_budget(&config, if delay_end_reset { 8 } else { 6 })?;
    let mut result = delayed_choice::validate_controls(&config, cue_visible)?;
    if delay_end_reset {
        let mut condition = delayed_choice::delay_end_reset(
            &delayed_choice::reference(&config, true),
            &config,
            cue_visible,
        )?;
        if condition["mean_reward"] != if cue_visible { 1.0 } else { 0.5 } {
            return Err("single delay-end reset reference control failed".into());
        }
        condition["name"] = json!("same_network_delay_end_reset");
        result["controls"].as_array_mut().unwrap().push(condition);
        result["checks"]["delay_end_reset_reward_expected"] = json!(true);
        result["delay_end_reset"] = json!(true);
    }
    result["arena_path"] = json!(path);
    result["protocol"] = protocol(&config, cue_visible);
    result["evaluator"] = evaluator_metadata();
    println!(
        "{}",
        serde_json::to_string(&result).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn protocol(config: &ArenaConfig, cue_visible: bool) -> Value {
    json!({"cue_channel_zero_based":2*config.ray_count+2,
        "pose_held_through_cue_and_delay":true,"hold_actions_discarded":true,
        "cue_visible":cue_visible,"cue_zero_after_release":!cue_visible,"correct_goal":{"positive_cue":"left","negative_cue":"right"},
        "reward":"one on correct first contact, zero on wrong contact or timeout; terminal only",
        "state_reset_each_trial":true,"body_scaling":false,"metabolism":false,
        "within_lifetime_updates":false,"training":false,
        "sensor_conventions":delayed_choice::SENSOR_CONVENTIONS,
        "motion_conventions":delayed_choice::MOTION_CONVENTIONS})
}

fn evaluator_metadata() -> Value {
    json!({"source_revision":env!("TERRARIUM_SOURCE_REVISION"),
        "source_dirty":env!("TERRARIUM_SOURCE_DIRTY"),
        "core_build":sim_core::checkpoint::CORE_BUILD_ID,
        "target":env!("TERRARIUM_TARGET"),
        "compiler":env!("TERRARIUM_COMPILER"),
        "build_profile":env!("TERRARIUM_BUILD_PROFILE")})
}

pub fn evaluate(
    path: &Path,
    arena_path: &Path,
    cue_visible: bool,
    delay_end_reset: bool,
) -> RunResult<()> {
    let config = delayed_choice::parse_config(arena_path)?;
    let loaded = exported_assay::load_validated(path)?;
    let required_inputs = 2 * config.ray_count + 10;
    if loaded.export.config.brain.inputs != required_inputs {
        return Err("export input layout differs from the arena ray count".into());
    }
    let trials = if delay_end_reset { 6 } else { 4 };
    trace_budget(&config, trials * loaded.export.genomes.len() as u64)?;
    exported_assay::validate_work(&loaded.export, trials * task_steps(&config))?;
    let mut results = Vec::with_capacity(loaded.export.genomes.len());
    for record in &loaded.export.genomes {
        let brain = record.genome.build_brain();
        let mut evaluation = delayed_choice::evaluate_brain(&brain, &config, cue_visible)?;
        if delay_end_reset {
            evaluation["same_network_delay_end_reset"] =
                delayed_choice::delay_end_reset(&brain, &config, cue_visible)?;
        }
        results.push(json!({"id":record.id,"generation":record.generation,
            "root_ancestor_id":record.root_ancestor_id,
            "enabled_recurrent_connections":brain.recurrent_count(),
            "evaluation":evaluation}));
    }
    let mut result = loaded.metadata();
    result["schema_version"] = json!(1);
    result["assay"] = json!("exported_genome_delayed_choice");
    result["cue_visible"] = json!(cue_visible);
    if delay_end_reset {
        result["delay_end_reset"] = json!(true);
    }
    result["interpretation"] = json!(if cue_visible {
        "visible-cue competence control; not memory use, training, natural ecology, evolutionary fitness, learned association or evidence of adaptive cognition"
    } else {
        "controlled delayed-choice movement task; not training, natural ecology, evolutionary fitness, learned association or evidence of adaptive cognition"
    });
    result["arena_path"] = json!(arena_path);
    result["arena"] = json!(config);
    result["protocol"] = protocol(&config, cue_visible);
    result["input_path"] = json!(path);
    result["brain_source"] = json!(
        "exported genotype weights times (1-snapshot methylation); expressed_brain and live memory ignored; no within-life updates"
    );
    result["sampling"] = json!({"evaluated":results.len(),
        "policy":"all exported genomes, in original order; no score-based selection",
        "independent_replicates":false,
        "unit_warning":"genomes from one donor and deterministic balanced cue trials are not independent evolutionary replicates"});
    result["results"] = json!(results);
    println!(
        "{}",
        serde_json::to_string(&result).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_excessive_trace_work_before_evaluating_a_population() {
        let config = delayed_choice::parse_config(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/delayed_choice.toml"),
        )
        .unwrap();
        assert!(trace_budget(&config, 4 * 898).is_ok());
        assert!(trace_budget(&config, 6 * 898).is_err());
        assert!(trace_budget(&config, u64::MAX).is_err());
    }
}
