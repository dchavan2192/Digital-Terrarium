//! Synthetic input-clamp evaluation of every inherited genome in one export.
//! This never restores expressed living brains, ongoing neural state, or ecology.
use crate::run::RunResult;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sim_core::Config;
use sim_core::brain::{Brain, MAX_NODES};
use sim_core::genome::Genome;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_GENOMES: usize = 2000;
const MAX_GENES: usize = MAX_NODES * MAX_NODES;
const MAX_DELAY: u32 = 10_000;
const MAX_FORWARD_PASSES: u64 = 1_000_000;
const MAX_EDGE_STEPS: u64 = 100_000_000;

#[derive(Deserialize)]
struct Export {
    format: String,
    version: u32,
    core_build: String,
    target: String,
    tick: u64,
    config: Config,
    genomes: Vec<Record>,
}

#[derive(Deserialize)]
struct Record {
    id: u64,
    generation: u64,
    root_ancestor_id: u64,
    genome: Genome,
}

#[derive(Serialize, Debug)]
struct Trial {
    cue: i8,
    cue_response: f64,
    probe_response: f64,
    predicted_cue: i8,
    correct: bool,
}

#[derive(Serialize, Debug)]
struct Condition {
    clear_state_each_tick: bool,
    accuracy: f64,
    trials: Vec<Trial>,
}

fn condition(brain: &Brain, channel: usize, delay: u32, clear: bool) -> RunResult<Condition> {
    let mut trials = Vec::with_capacity(2);
    for cue in [-1_i8, 1] {
        let mut prev = [0.0; MAX_NODES];
        let mut act = [0.0; MAX_NODES];
        let mut inputs = [0.0; MAX_NODES];
        let mut step = |input: f64| -> RunResult<f64> {
            inputs[channel] = input;
            if clear {
                prev.fill(0.0);
            }
            let (out, activity) =
                brain.forward_with_memory(&inputs[..brain.inputs], &prev, &mut act);
            if !out.turn.is_finite() || !activity.is_finite() || act.iter().any(|x| !x.is_finite())
            {
                return Err("non-finite neural response during assay".into());
            }
            prev.copy_from_slice(&act);
            Ok(out.turn)
        };
        let cue_response = step(f64::from(cue))?;
        for _ in 0..delay {
            step(0.0)?;
        }
        let probe_response = step(0.0)?;
        let predicted_cue = if probe_response >= 0.0 { 1 } else { -1 };
        trials.push(Trial {
            cue,
            cue_response,
            probe_response,
            predicted_cue,
            correct: predicted_cue == cue,
        });
    }
    Ok(Condition {
        clear_state_each_tick: clear,
        accuracy: trials.iter().filter(|t| t.correct).count() as f64 / 2.0,
        trials,
    })
}

fn read_bounded(path: &Path) -> RunResult<Vec<u8>> {
    let mut file = File::open(path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err("input must be a regular genome-export file".into());
    }
    if metadata.len() > MAX_INPUT_BYTES {
        return Err("genome export exceeds 64 MiB limit".into());
    }
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err("genome export exceeds 64 MiB limit".into());
    }
    Ok(bytes)
}

fn evaluate(bytes: &[u8], channel: usize, delay: u32) -> RunResult<Value> {
    if bytes.len() as u64 > MAX_INPUT_BYTES || delay > MAX_DELAY {
        return Err("require input <=64 MiB and --blank-delay <=10000".into());
    }
    // Inspect collection sizes before deserializing typed genomes or building brains.
    // The bounded JSON input also limits allocation in this first parsing stage.
    let raw: Value =
        serde_json::from_slice(bytes).map_err(|e| format!("invalid export JSON: {e}"))?;
    let array = raw
        .get("genomes")
        .and_then(Value::as_array)
        .ok_or("missing genomes array")?;
    if array.is_empty() || array.len() > MAX_GENOMES {
        return Err("require between 1 and 2000 genomes; no implicit subsampling".into());
    }
    for record in array {
        let genome = record.get("genome").ok_or("record lacks genome")?;
        let genes = genome
            .get("connections")
            .and_then(Value::as_array)
            .ok_or("genome lacks connections")?;
        let hidden = genome
            .get("hidden_nodes")
            .and_then(Value::as_array)
            .ok_or("genome lacks hidden_nodes")?;
        if genes.len() > MAX_GENES || hidden.len() > MAX_NODES {
            return Err("genome collection size exceeds supported bounds".into());
        }
    }
    let donor_config = raw
        .get("config")
        .cloned()
        .ok_or("export lacks full config")?;
    let export: Export =
        serde_json::from_value(raw).map_err(|e| format!("invalid genome export: {e}"))?;
    if export.format != "terrarium-genomes" || export.version != 1 {
        return Err("only terrarium-genomes version 1 is supported".into());
    }
    if export.core_build.is_empty() || export.target.is_empty() {
        return Err("export lacks source core_build/target metadata".into());
    }
    export
        .config
        .validate()
        .map_err(|e| format!("invalid donor config: {e}"))?;
    if channel >= export.config.brain.inputs {
        return Err("--cue-channel must be a zero-based configured input index".into());
    }
    if export.genomes.len() > export.config.creature.hard_cap {
        return Err("export population exceeds its configured hard cap".into());
    }
    let mut ids = HashSet::new();
    let mut edge_steps = 0_u64;
    let passes_per_genome = 4 * (u64::from(delay) + 2);
    if passes_per_genome * export.genomes.len() as u64 > MAX_FORWARD_PASSES {
        return Err(
            "assay exceeds 1000000 forward-pass budget; reduce delay, not population".into(),
        );
    }
    // Validate the complete donor population before evaluating any record.
    for record in &export.genomes {
        if !ids.insert(record.id) {
            return Err(format!("duplicate organism id {}", record.id));
        }
        record
            .genome
            .validate()
            .map_err(|e| format!("invalid genome {}: {e}", record.id))?;
        if record.genome.num_inputs != export.config.brain.inputs
            || record.genome.num_outputs != export.config.brain.outputs
        {
            return Err(format!(
                "genome {} I/O differs from donor config",
                record.id
            ));
        }
        edge_steps += passes_per_genome * record.genome.connections.len() as u64;
    }
    if edge_steps > MAX_EDGE_STEPS {
        return Err(
            "assay exceeds 100000000 edge-step budget; reduce delay, not population".into(),
        );
    }
    let mut results = Vec::with_capacity(export.genomes.len());
    for record in export.genomes {
        let brain = record.genome.build_brain();
        results.push(json!({
            "id":record.id, "generation":record.generation,
            "root_ancestor_id":record.root_ancestor_id,
            "enabled_recurrent_connections":brain.recurrent_count(),
            "intact":condition(&brain, channel, delay, false)?,
            "same_network_state_cleared":condition(&brain, channel, delay, true)?
        }));
    }
    // An explicitly noncryptographic identity over exact input bytes. Archive a
    // separate SHA-256 externally; this is never called a cryptographic checksum.
    let fingerprint = bytes.iter().fold(0xcbf29ce484222325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    Ok(json!({
        "schema_version":1, "assay":"exported_genome_delayed_cue",
        "interpretation":"synthetic input clamp; not learned association, natural behaviour, fitness or evidence of evolved cognition",
        "input_bytes":bytes.len(), "input_fnv1a64_noncryptographic":format!("{fingerprint:016x}"),
        "source_export":{"format":export.format,"version":export.version,"core_build":export.core_build,"target":export.target,"tick":export.tick,"config":donor_config},
        "evaluator":{"source_revision":env!("TERRARIUM_SOURCE_REVISION"),"source_dirty":env!("TERRARIUM_SOURCE_DIRTY"),"core_build":sim_core::checkpoint::CORE_BUILD_ID,"target":env!("TERRARIUM_TARGET")},
        "protocol":{"cue_channel_zero_based":channel,"cue_ticks":1,"blank_delay_ticks":delay,"probe_ticks":1,"other_inputs":0.0,"probe_inputs":vec![0.0;export.config.brain.inputs],"readout":"turn","zero_tie_prediction":1,"state_reset_each_trial":true,"brain_source":"exported genotype weights times (1-snapshot methylation); expressed_brain and live memory ignored","within_lifetime_updates":false},
        "sampling":{"evaluated":results.len(),"policy":"all exported genomes, in original order; no score-based selection","independent_replicates":false,"unit_warning":"genomes within one donor population are not independent evolutionary replicates; two balanced deterministic cues are not trial replicates"},
        "results":results
    }))
}

pub fn run(path: &Path, channel: usize, delay: u32) -> RunResult<()> {
    let bytes = read_bounded(path)?;
    let mut result = evaluate(&bytes, channel, delay)?;
    result["input_path"] = json!(path);
    println!(
        "{}",
        serde_json::to_string(&result).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::{Simulation, genome::ConnectionGene};

    fn fixture() -> Value {
        let mut config = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
        config.creature.initial_count = 1;
        let sim = Simulation::new(config.clone());
        let mut genome = sim.organisms[0].genome.clone();
        genome.hidden_nodes.clear();
        let turn = (genome.num_inputs + 1) as u32;
        genome.connections = vec![
            ConnectionGene {
                from: 1,
                to: turn,
                weight: 2.0,
                enabled: true,
                methylation: 0.0,
                recurrent: false,
            },
            ConnectionGene {
                from: turn,
                to: turn,
                weight: 2.0,
                enabled: true,
                methylation: 0.0,
                recurrent: true,
            },
        ];
        let mut config = serde_json::to_value(config).unwrap();
        config["epigenetics"]["stress_threshold"] = json!(0.3);
        json!({"format":"terrarium-genomes","version":1,"core_build":"donor-build","target":"donor-target","tick":100,"config":config,
            "genomes":[{"id":7,"generation":3,"root_ancestor_id":2,"genome":genome,"expressed_brain":{"deliberately":"not a brain"},"memory":[999.0]}]})
    }

    fn eval(value: &Value) -> RunResult<Value> {
        evaluate(&serde_json::to_vec(value).unwrap(), 0, 5)
    }

    #[test]
    fn rebuilds_inherited_brain_preserves_provenance_and_clears_state() {
        let input = fixture();
        let out = eval(&input).unwrap();
        assert_eq!(out["source_export"]["config"], input["config"]);
        let result = &out["results"][0];
        assert_eq!(result["id"], 7);
        assert_eq!(result["generation"], 3);
        assert_eq!(result["root_ancestor_id"], 2);
        assert_eq!(result["intact"]["accuracy"], 1.0);
        assert_eq!(result["same_network_state_cleared"]["accuracy"], 0.5);
        assert_eq!(out["sampling"]["evaluated"], 1);
    }

    #[test]
    fn inherited_methylation_is_applied_and_feed_forward_cannot_read_removed_cue() {
        let mut input = fixture();
        input["genomes"][0]["genome"]["connections"][1]["methylation"] = json!(1.0);
        assert_eq!(
            eval(&input).unwrap()["results"][0]["intact"]["accuracy"],
            0.5
        );
        input["genomes"][0]["genome"]["connections"][1]["enabled"] = json!(false);
        let result = eval(&input).unwrap();
        assert_eq!(result["results"][0]["enabled_recurrent_connections"], 0);
        assert_eq!(result["results"][0]["intact"]["accuracy"], 0.5);
    }

    #[test]
    fn rejects_format_empty_duplicates_invalid_config_and_malformed_genomes() {
        let base = fixture();
        let mut cases = Vec::new();
        let mut x = base.clone();
        x["format"] = json!("other");
        cases.push(x);
        let mut x = base.clone();
        x["version"] = json!(2);
        cases.push(x);
        let mut x = base.clone();
        x["genomes"][0]["id"] = json!(-1);
        cases.push(x);
        let mut x = base.clone();
        x["genomes"] = json!([]);
        cases.push(x);
        let mut x = base.clone();
        let record = x["genomes"][0].clone();
        x["genomes"].as_array_mut().unwrap().push(record);
        cases.push(x);
        let mut x = base.clone();
        x["config"]["world"]["width"] = json!(0.0);
        cases.push(x);
        let mut x = base.clone();
        x["genomes"][0]["genome"]["num_inputs"] = json!(usize::MAX);
        cases.push(x);
        let mut x = base.clone();
        x["genomes"][0]["genome"]["num_inputs"] = json!(19);
        cases.push(x);
        let mut x = base.clone();
        x["genomes"][0]["genome"]["connections"][0]["to"] = json!(u32::MAX);
        cases.push(x);
        for value in cases {
            assert!(eval(&value).is_err(), "{value}");
        }
        let bytes = serde_json::to_vec(&base).unwrap();
        assert!(evaluate(&bytes, usize::MAX, 5).is_err());
        assert!(evaluate(&bytes, 0, MAX_DELAY + 1).is_err());
        assert!(evaluate(b"not json", 0, 5).is_err());
    }

    #[test]
    fn rejects_oversized_collections_and_work_without_subsampling() {
        let base = fixture();
        let mut input = base.clone();
        input["genomes"] = json!(vec![base["genomes"][0].clone(); MAX_GENOMES + 1]);
        assert!(eval(&input).unwrap_err().contains("between 1 and 2000"));
        let mut input = base.clone();
        input["genomes"][0]["genome"]["connections"] = json!(vec![
                base["genomes"][0]["genome"]["connections"][0].clone();
                MAX_GENES + 1
            ]);
        assert!(eval(&input).unwrap_err().contains("collection size"));
        let mut input = base.clone();
        input["genomes"] = json!(
            (0..26)
                .map(|id| {
                    let mut record = base["genomes"][0].clone();
                    record["id"] = json!(id);
                    record
                })
                .collect::<Vec<_>>()
        );
        let bytes = serde_json::to_vec(&input).unwrap();
        assert!(
            evaluate(&bytes, 0, MAX_DELAY)
                .unwrap_err()
                .contains("forward-pass budget")
        );
    }
}
