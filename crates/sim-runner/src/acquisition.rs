//! Isolated, serial task-directed genetic acquisition. Never runs the ecology.
use crate::delayed_choice::{self, ArenaConfig};
use crate::run::{self, RunResult};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sim_core::Config;
use sim_core::brain::MAX_NODES;
use sim_core::config::MutationConfig;
use sim_core::genome::{Genome, Innovations};
use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Protocol {
    population: usize,
    generations: u32,
    elites: usize,
    tournament: usize,
    seeds: Vec<u64>,
    hidden: usize,
    mutation: MutationConfig,
}

impl Protocol {
    fn validate(&self, arena: &ArenaConfig) -> RunResult<u64> {
        if !(2..=256).contains(&self.population)
            || !(1..=1000).contains(&self.generations)
            || self.elites == 0
            || self.elites >= self.population
            || self.tournament == 0
            || self.tournament > self.population
            || self.seeds.is_empty()
            || self.seeds.len() > 16
            || self.seeds.iter().copied().collect::<BTreeSet<_>>().len() != self.seeds.len()
            || self.population * self.seeds.len() > 1024
            || self.hidden == 0
            || self.hidden > MAX_NODES.saturating_sub(arena.input_count() + 8)
        {
            return Err("invalid acquisition population/generations/elites/tournament/seeds/hidden or population archive budget".into());
        }
        for p in [
            self.mutation.weight_probability,
            self.mutation.add_connection_probability,
            self.mutation.add_node_probability,
            self.mutation.toggle_probability,
            self.mutation.prune_probability,
            self.mutation.trait_probability,
        ] {
            if !p.is_finite() || !(0.0..=1.0).contains(&p) {
                return Err("mutation probabilities must be finite in [0,1]".into());
            }
        }
        if !self.mutation.weight_step.is_finite()
            || !(0.01..=2.0).contains(&self.mutation.weight_step)
            || self.mutation.add_recurrent_probability != 0.0
        {
            return Err(
                "visible acquisition requires finite weight_step in [0.01,2] and recurrence off"
                    .into(),
            );
        }
        if [
            self.mutation.add_connection_probability,
            self.mutation.add_node_probability,
            self.mutation.toggle_probability,
            self.mutation.prune_probability,
            self.mutation.trait_probability,
        ]
        .iter()
        .any(|p| *p != 0.0)
        {
            return Err(
                "first acquisition pilot requires topology and trait mutation off (weight-only)"
                    .into(),
            );
        }
        let evaluations =
            self.population as u64 * (u64::from(self.generations) + 1) * self.seeds.len() as u64;
        let trial_steps =
            u64::from(arena.cue_ticks) + u64::from(arena.blank_delay) + u64::from(arena.horizon);
        let extra_trials = 6 + 4 * self.seeds.len() as u64;
        if (trial_steps + 1) * extra_trials > 100_000 {
            return Err("acquisition exceeds 100000 retained trace samples".into());
        }
        let steps = (evaluations * 2 + extra_trials) * trial_steps;
        // A mutation adds at most one edge and a split adds two more. All genes,
        // including disabled genes, count in this conservative work bound.
        let edges = ((arena.input_count() + 1 + 7) * self.hidden + 7) as u64
            + 3 * u64::from(self.generations);
        if steps > 2_000_000
            || steps.saturating_mul(edges.min((MAX_NODES * MAX_NODES) as u64)) > 1_000_000_000
        {
            return Err(
                "acquisition exceeds 2000000 neural steps or 1000000000 edge-step budget".into(),
            );
        }
        Ok(evaluations)
    }
}

#[derive(Clone, Serialize)]
struct Candidate {
    id: u64,
    parent_id: Option<u64>,
    born_generation: u32,
    correct: u8,
    genome: Genome,
}

fn write_json(path: &Path, value: &impl Serialize) -> RunResult<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("create {}: {e}", path.display()))?;
    serde_json::to_writer(&mut file, value).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())
}

fn score(candidate: &mut Candidate, arena: &ArenaConfig) -> RunResult<()> {
    candidate.genome.validate().map_err(|e| e.to_string())?;
    if candidate.genome.connections.iter().any(|c| c.recurrent) {
        return Err("unexpected recurrent gene in feed-forward acquisition".into());
    }
    candidate.correct = delayed_choice::visible_score(&candidate.genome.build_brain(), arena)?;
    Ok(())
}

fn rank(population: &mut [Candidate]) {
    population.sort_by(|a, b| b.correct.cmp(&a.correct).then(a.id.cmp(&b.id)));
}

fn generation_log(file: &mut File, generation: u32, population: &[Candidate]) -> RunResult<()> {
    let scores: Vec<_> = population
        .iter()
        .map(|c| json!({"id":c.id,"parent_id":c.parent_id,"correct":c.correct}))
        .collect();
    let mean = population.iter().map(|c| u64::from(c.correct)).sum::<u64>() as f64
        / (2 * population.len()) as f64;
    serde_json::to_writer(
        &mut *file,
        &json!({"generation":generation,"champion_id":population[0].id,
        "best_mean_reward":f64::from(population[0].correct)/2.0,"population_mean_reward":mean,
        "both_correct":population.iter().filter(|c| c.correct==2).count(),"scores":scores}),
    )
    .map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())
}

fn search(protocol: &Protocol, arena: &ArenaConfig, seed: u64, dir: &Path) -> RunResult<Value> {
    std::fs::create_dir(dir).map_err(|e| e.to_string())?;
    let timer = Instant::now();
    let mut config =
        Config::parse(include_str!("../../../configs/default.toml")).map_err(|e| e.to_string())?;
    config.brain.inputs = arena.input_count();
    config.brain.hidden = protocol.hidden;
    config.brain.outputs = 7;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut innovations = Innovations::new(&config.brain);
    let mut population = Vec::with_capacity(protocol.population);
    for id in 0..protocol.population as u64 {
        let genome = Genome::random(
            &config.brain,
            &protocol.mutation,
            &config.aging,
            &config.temperature,
            &config.food.fruit,
            &mut rng,
        );
        let mut candidate = Candidate {
            id,
            parent_id: None,
            born_generation: 0,
            correct: 0,
            genome,
        };
        score(&mut candidate, arena)?;
        population.push(candidate);
    }
    rank(&mut population);
    let founder_best = population[0].correct;
    write_json(&dir.join("founders.json"), &population)?;
    let mut curve = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("generations.jsonl"))
        .map_err(|e| e.to_string())?;
    generation_log(&mut curve, 0, &population)?;
    let mut next_id = protocol.population as u64;
    for generation in 1..=protocol.generations {
        let mut offspring = population[..protocol.elites].to_vec();
        while offspring.len() < protocol.population {
            // Population is ranked; tournament samples with replacement, lowest
            // rank wins. Ties therefore use the predeclared lower-ID rule.
            let parent = (0..protocol.tournament)
                .map(|_| sim_core::math::random_index(&mut rng, population.len()))
                .min()
                .unwrap();
            let mut child = Candidate {
                id: next_id,
                parent_id: Some(population[parent].id),
                born_generation: generation,
                correct: 0,
                genome: population[parent].genome.reproduce(
                    &protocol.mutation,
                    &mut innovations,
                    &mut rng,
                ),
            };
            next_id += 1;
            score(&mut child, arena)?;
            offspring.push(child);
        }
        // Elites are evaluated again: fixed population×(generations+1) budget.
        for elite in &mut offspring[..protocol.elites] {
            score(elite, arena)?;
        }
        rank(&mut offspring);
        population = offspring;
        generation_log(&mut curve, generation, &population)?;
    }
    write_json(&dir.join("final_population.json"), &population)?;
    write_json(&dir.join("innovations.json"), &innovations)?;
    let champion = &population[0];
    let summary = json!({"seed":seed,"status":"completed","evaluations":protocol.population as u64*(u64::from(protocol.generations)+1),
        "founder_best_mean_reward":f64::from(founder_best)/2.0,"final_best_mean_reward":f64::from(champion.correct)/2.0,
        "champion_id":champion.id,"both_correct_at_founding":founder_best==2,"both_correct_at_end":champion.correct==2,
        "improvement_over_founders":champion.correct>founder_best,
        "elapsed_seconds":timer.elapsed().as_secs_f64(),
        "champion_training_trials":delayed_choice::evaluate_brain(&champion.genome.build_brain(),arena,true)?});
    write_json(&dir.join("summary.json"), &summary)?;
    Ok(summary)
}

fn load_protocol(path: &Path) -> RunResult<Protocol> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(65_537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65_536 {
        return Err("protocol exceeds 64KiB".into());
    }
    let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    let raw: toml::Value = toml::from_str(text).map_err(|e| e.to_string())?;
    if let Some(mutation) = raw.get("mutation").and_then(toml::Value::as_table) {
        for key in mutation.keys() {
            if ![
                "weight_probability",
                "weight_step",
                "add_connection_probability",
                "add_node_probability",
                "toggle_probability",
                "prune_probability",
                "add_recurrent_probability",
                "trait_probability",
            ]
            .contains(&key.as_str())
            {
                return Err(format!("unknown acquisition mutation field: {key}"));
            }
        }
    }
    toml::from_str(text).map_err(|e| format!("invalid acquisition protocol: {e}"))
}

pub fn run(protocol_path: &Path, arena_path: &Path, output: &Path) -> RunResult<()> {
    let protocol = load_protocol(protocol_path)?;
    let arena = delayed_choice::parse_config(arena_path)?;
    let evaluations = protocol.validate(&arena)?;
    // Validate everything before creating artifacts, and pin the positive controls.
    let controls = delayed_choice::validate_controls(&arena, true)?;
    let session = run::create_session(output)?;
    let manifest = json!({"schema_version":1,"task":"visible_cue_genetic_acquisition","status":"started",
        "protocol":protocol,"arena":arena,"protocol_path":protocol_path,"arena_path":arena_path,
        "total_evaluations":evaluations,"execution":"serial; no ecology or Rayon pool",
        "fitness":"number correct across cues [-1,+1]; terminal first contact only; wrong/timeout zero",
        "selection":"asexual elitism; tournament with replacement; descending correct then ascending persistent candidate ID; no crossover/speciation",
        "founders":"Genome::random with declared dimensions/mutation; no injected reference weights",
        "embedded_founder_defaults":Config::parse(include_str!("../../../configs/default.toml")).map_err(|e|e.to_string())?,
        "phenotype":"20-style arena inputs (dimensions in arena), declared hidden, seven outputs; methylation zero and fixed; no body scaling, metabolism, energy selection or lifetime updates; neutral hue drift from native reproduce retained",
        "champion_rule":"best member of final population by training score then lowest ID; fixed budget, no early stopping",
        "validation":"none; champion trials replay training conditions only; not a held-out evaluation",
        "unit":"independent founder-population seed; cues/candidates/generations are not independent replicates",
        "source_revision":env!("TERRARIUM_SOURCE_REVISION"),"source_dirty":env!("TERRARIUM_SOURCE_DIRTY"),
        "core_build":sim_core::checkpoint::CORE_BUILD_ID,"compiler":env!("TERRARIUM_COMPILER"),
        "target":env!("TERRARIUM_TARGET"),"build_profile":env!("TERRARIUM_BUILD_PROFILE"),"rng":"ChaCha8Rng seed_from_u64; fixed-width random_index",
        "interpretation":"engineering feasibility pilot; genetic optimization across generations, not memory, within-life learning, ecological adaptation or generalization"});
    write_json(&session.join("manifest.json"), &manifest)?;
    write_json(&session.join("controls.json"), &controls)?;
    let mut results = Vec::new();
    for seed in &protocol.seeds {
        match search(
            &protocol,
            &arena,
            *seed,
            &session.join(format!("seed_{seed}")),
        ) {
            Ok(summary) => results.push(summary),
            Err(error) => {
                results.push(json!({"seed":seed,"status":"failed","error":error}));
                // Technical failures are retained; finish remaining declared seeds.
            }
        }
    }
    let failures = results.iter().filter(|r| r["status"] == "failed").count();
    write_json(
        &session.join("summary.json"),
        &json!({"schema_version":1,
        "status":if failures==0 {"completed"} else {"completed_with_failures"},"technical_failures":failures,"runs":results}),
    )?;
    if failures > 0 {
        return Err(format!(
            "{failures} acquisition runs failed; see {}",
            session.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_rejects_work_overflow_and_invalid_designs() {
        let mut p: Protocol =
            toml::from_str(include_str!("../../../configs/visible_acquisition.toml")).unwrap();
        let a: ArenaConfig =
            toml::from_str(include_str!("../../../configs/delayed_choice.toml")).unwrap();
        assert_eq!(p.validate(&a).unwrap(), 4992);
        p.seeds.push(p.seeds[0]);
        assert!(p.validate(&a).is_err());
        p.seeds.pop();
        p.generations = u32::MAX;
        assert!(p.validate(&a).is_err());
        p.generations = 25;
        let mut small = p.clone();
        small.population = 2;
        small.generations = 1;
        small.elites = 1;
        small.tournament = 1;
        small.seeds = vec![1];
        let mut long = a.clone();
        long.cue_ticks = 10000;
        long.blank_delay = 10000;
        long.horizon = 10000;
        assert!(small.validate(&long).unwrap_err().contains("trace samples"));
        p.mutation.add_recurrent_probability = f64::NAN;
        assert!(p.validate(&a).is_err());
        p.mutation.add_recurrent_probability = 0.1;
        assert!(p.validate(&a).is_err());
    }
}
