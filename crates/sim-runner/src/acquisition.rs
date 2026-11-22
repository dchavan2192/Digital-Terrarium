//! Isolated, serial task-directed genetic acquisition. Never runs the ecology.
use crate::delayed_choice::{self, ArenaConfig};
use crate::run::{self, RunResult};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sim_core::Config;
use sim_core::brain::MAX_NODES;
use sim_core::config::{FruitConfig, MutationConfig};
use sim_core::genome::{Genome, Innovations};
use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Fitness {
    #[default]
    TerminalOnly,
    TerminalProgress,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Protocol {
    population: usize,
    generations: u32,
    elites: usize,
    tournament: usize,
    seeds: Vec<u64>,
    hidden: usize,
    #[serde(default)]
    fitness: Fitness,
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

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    id: u64,
    parent_id: Option<u64>,
    born_generation: u32,
    correct: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    terminal_progress: Option<f64>,
    genome: Genome,
}

/// One training-ranked policy per declared seed, including technical failures.
#[derive(Debug)]
pub(crate) struct SavedPolicy {
    pub seed: u64,
    pub training: Value,
    pub genome: Option<Genome>,
}

#[derive(Debug)]
pub(crate) struct SavedSelection {
    pub metadata: Value,
    pub arena: ArenaConfig,
    pub policies: Vec<SavedPolicy>,
}

const SAVED_FILE_LIMIT: u64 = 64 * 1024 * 1024;
const SAVED_TOTAL_LIMIT: u64 = 128 * 1024 * 1024;

struct SavedFiles {
    session: std::path::PathBuf,
    used: u64,
    records: Vec<Value>,
}

impl SavedFiles {
    fn read(&mut self, relative: impl AsRef<Path>) -> RunResult<Value> {
        let path = self.session.join(relative);
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if !canonical.starts_with(&self.session) {
            return Err("saved training file escapes its session directory".into());
        }
        let mut file = File::open(&canonical).map_err(|e| e.to_string())?;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        let allowed = SAVED_FILE_LIMIT.min(SAVED_TOTAL_LIMIT.saturating_sub(self.used));
        if !metadata.is_file() || metadata.len() > allowed {
            return Err(
                "saved selection requires regular files <=64MiB each and <=128MiB total".into(),
            );
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(allowed + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > allowed {
            return Err("saved selection exceeds its file or aggregate input-byte budget".into());
        }
        self.used += bytes.len() as u64;
        let fingerprint = bytes.iter().fold(0xcbf29ce484222325_u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
        });
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|e| format!("invalid saved JSON {}: {e}", path.display()))?;
        self.records.push(
            json!({"path":canonical.to_string_lossy(),"bytes":bytes.len(),
            "fnv1a64_noncryptographic":format!("{fingerprint:016x}")}),
        );
        Ok(value)
    }
}

fn saved_require(ok: bool, reason: &str) -> RunResult<()> {
    if ok {
        Ok(())
    } else {
        Err(format!("invalid saved acquisition: {reason}"))
    }
}

fn finite_saved(value: &Value) -> bool {
    value.as_f64().is_some_and(f64::is_finite)
}

fn validate_saved_population(
    raw: Value,
    protocol: &Protocol,
    arena: &ArenaConfig,
) -> RunResult<Vec<Candidate>> {
    let records = raw
        .as_array()
        .ok_or("saved final population must be an array")?;
    saved_require(
        records.len() == protocol.population,
        "final population count differs",
    )?;
    let first_output = (arena.input_count() + 1) as u32;
    let first_hidden = first_output + 7;
    let mut edges = Vec::new();
    for hidden in first_hidden..first_hidden + protocol.hidden as u32 {
        for input in 0..=arena.input_count() as u32 {
            edges.push((input, hidden));
        }
        for output in first_output..first_output + 7 {
            edges.push((hidden, output));
        }
    }
    for output in first_output..first_output + 7 {
        edges.push((0, output));
    }
    edges.sort_unstable();
    for record in records {
        let genome = record.get("genome").ok_or("saved candidate lacks genome")?;
        saved_require(
            genome["connections"]
                .as_array()
                .is_some_and(|g| g.len() == edges.len())
                && genome["hidden_nodes"]
                    .as_array()
                    .is_some_and(|g| g.len() == protocol.hidden),
            "saved genome collection dimensions differ from the fixed founder topology",
        )?;
        saved_require(
            record["correct"].as_u64().is_some_and(|c| c <= 2),
            "correct count must be 0, 1 or 2",
        )?;
        match protocol.fitness {
            Fitness::TerminalOnly => saved_require(
                record.get("terminal_progress").is_none(),
                "progress field in terminal-only population",
            )?,
            Fitness::TerminalProgress => saved_require(
                record["terminal_progress"]
                    .as_f64()
                    .is_some_and(|p| p.is_finite() && (0.0..=1.0).contains(&p)),
                "missing/nonfinite/out-of-range progress score",
            )?,
        }
    }
    let mut population: Vec<Candidate> = serde_json::from_value(raw).map_err(|e| e.to_string())?;
    let mut ids = BTreeSet::new();
    let max_id = protocol.population as u64
        + (protocol.population - protocol.elites) as u64 * u64::from(protocol.generations);
    for candidate in &population {
        saved_require(
            ids.insert(candidate.id)
                && candidate.id < max_id
                && candidate.born_generation <= protocol.generations,
            "duplicate/out-of-range candidate ID or generation",
        )?;
        if candidate.id < protocol.population as u64 {
            saved_require(
                candidate.parent_id.is_none() && candidate.born_generation == 0,
                "founder identity has a parent or nonzero birth generation",
            )?;
        } else {
            let birth = 1
                + (candidate.id - protocol.population as u64)
                    / (protocol.population - protocol.elites) as u64;
            saved_require(
                candidate.born_generation as u64 == birth
                    && candidate.parent_id.is_some_and(|p| p < candidate.id),
                "offspring identity/birth/parent is inconsistent",
            )?;
        }
        let genome = &candidate.genome;
        genome
            .validate()
            .map_err(|e| format!("saved genome {}: {e}", candidate.id))?;
        saved_require(
            genome.num_inputs == arena.input_count()
                && genome.num_outputs == 7
                && genome.hidden_nodes.len() == protocol.hidden
                && genome.hidden_nodes.iter().enumerate().all(|(index, node)| {
                    node.id == first_hidden + index as u32 && node.depth == 0.5
                })
                && genome
                    .connections
                    .iter()
                    .map(|c| (c.from, c.to))
                    .eq(edges.iter().copied())
                && genome
                    .connections
                    .iter()
                    .all(|c| c.enabled && !c.recurrent && c.methylation == 0.0)
                && genome.mutation_rate == protocol.mutation.weight_probability
                && genome.mutation_step == protocol.mutation.weight_step,
            "saved genome does not match fixed feed-forward, zero-mark founder topology/mutation",
        )?;
    }
    let order: Vec<_> = population.iter().map(|c| c.id).collect();
    rank(&mut population, protocol.fitness);
    saved_require(
        population.iter().map(|c| c.id).eq(order),
        "archived final population is not in declared training rank",
    )?;
    Ok(population)
}

fn validate_saved_champion(
    summary: &Value,
    champion: &Candidate,
    protocol: &Protocol,
    seed: u64,
) -> RunResult<()> {
    let founder = summary["founder_best_mean_reward"].as_f64();
    let final_reward = f64::from(champion.correct) / 2.0;
    saved_require(
        summary["status"] == "completed"
            && summary["seed"].as_u64() == Some(seed)
            && summary["evaluations"].as_u64()
                == Some(protocol.population as u64 * (u64::from(protocol.generations) + 1))
            && summary["champion_id"].as_u64() == Some(champion.id)
            && summary["final_best_mean_reward"].as_f64() == Some(final_reward)
            && founder.is_some_and(|v| [0.0, 0.5, 1.0].contains(&v) && v <= final_reward)
            && finite_saved(&summary["elapsed_seconds"])
            && summary["elapsed_seconds"].as_f64().unwrap() >= 0.0,
        "champion/score/evaluation/time summary disagrees with final training population",
    )?;
    let founder = founder.unwrap();
    saved_require(
        summary["both_correct_at_founding"] == (founder == 1.0)
            && summary["both_correct_at_end"] == (champion.correct == 2)
            && summary["improvement_over_founders"] == (final_reward > founder),
        "summary acquisition flags disagree with recorded terminal scores",
    )?;
    match protocol.fitness {
        Fitness::TerminalOnly => saved_require(
            summary.get("founder_best_terminal_progress").is_none()
                && summary.get("final_best_terminal_progress").is_none(),
            "terminal-only summary contains progress",
        )?,
        Fitness::TerminalProgress => saved_require(
            summary["founder_best_terminal_progress"]
                .as_f64()
                .is_some_and(|p| p.is_finite() && (0.0..=1.0).contains(&p))
                && summary["final_best_terminal_progress"].as_f64() == champion.terminal_progress,
            "selected champion progress summary differs",
        )?,
    }
    let replay = &summary["champion_training_trials"];
    saved_require(
        replay["cue_visible"] == true,
        "champion replay is not the declared visible task",
    )?;
    for (name, clear) in [("intact", false), ("same_network_state_cleared", true)] {
        let condition = &replay[name];
        let trials = condition["trials"]
            .as_array()
            .ok_or("saved champion lacks balanced training trials")?;
        saved_require(
            condition["clear_state_each_tick"] == clear
                && trials.len() == 2
                && trials[0]["cue"].as_i64() == Some(-1)
                && trials[1]["cue"].as_i64() == Some(1)
                && condition["mean_reward"].as_f64() == Some(final_reward),
            "champion replay score/order differs",
        )?;
        let mut correct = 0_u64;
        for trial in trials {
            let target = if trial["cue"] == 1 { "left" } else { "right" };
            let choice = trial["choice"]
                .as_str()
                .ok_or("invalid saved training choice")?;
            saved_require(
                ["left", "right", "timeout"].contains(&choice)
                    && trial["target"] == target
                    && trial["reward"].as_u64() == Some(u64::from(choice == target)),
                "training trial target/reward differs",
            )?;
            correct += trial["reward"].as_u64().unwrap();
            match protocol.fitness {
                Fitness::TerminalOnly => saved_require(
                    trial.get("terminal_progress").is_none(),
                    "terminal-only training replay contains progress",
                )?,
                Fitness::TerminalProgress => saved_require(
                    trial["terminal_progress"].as_f64().is_some_and(|p| {
                        p.is_finite()
                            && (0.0..=1.0).contains(&p)
                            && (trial["reward"] != 1 || p == 1.0)
                    }),
                    "invalid per-cue training progress",
                )?,
            }
        }
        saved_require(
            correct == u64::from(champion.correct),
            "champion replay correct count differs",
        )?;
        if protocol.fitness == Fitness::TerminalProgress {
            let mean = trials
                .iter()
                .map(|t| t["terminal_progress"].as_f64().unwrap())
                .sum::<f64>()
                / 2.0;
            saved_require(
                condition["mean_terminal_progress"].as_f64() == Some(mean)
                    && Some(mean) == champion.terminal_progress,
                "training replay progress mean differs",
            )?;
        } else {
            saved_require(
                condition.get("mean_terminal_progress").is_none(),
                "terminal-only replay contains mean progress",
            )?;
        }
    }
    saved_require(
        replay["intact"]["trials"] == replay["same_network_state_cleared"]["trials"],
        "feed-forward archived training replays disagree",
    )
}

/// Select only the archived training champion, never a policy chosen by new tests.
/// Does not evaluate a controller or load any partial population from failed seeds.
pub(crate) fn load_saved_selection(session: &Path) -> RunResult<SavedSelection> {
    let session = session.canonicalize().map_err(|e| e.to_string())?;
    saved_require(
        session.is_dir(),
        "input is not a saved acquisition session directory",
    )?;
    let mut files = SavedFiles {
        session: session.clone(),
        used: 0,
        records: Vec::new(),
    };
    let manifest = files.read("manifest.json")?;
    let root = files.read("summary.json")?;
    saved_require(
        manifest["schema_version"].as_u64() == Some(1)
            && manifest["task"] == "visible_cue_genetic_acquisition"
            && manifest["status"] == "started"
            && root["schema_version"].as_u64() == Some(1),
        "wrong manifest/task/schema/launch status",
    )?;
    let protocol: Protocol = serde_json::from_value(manifest["protocol"].clone())
        .map_err(|e| format!("saved protocol: {e}"))?;
    let arena: ArenaConfig = serde_json::from_value(manifest["arena"].clone())
        .map_err(|e| format!("saved arena: {e}"))?;
    arena.validate()?;
    let evaluations = protocol.validate(&arena)?;
    saved_require(
        manifest["total_evaluations"].as_u64() == Some(evaluations),
        "manifest work budget differs",
    )?;
    let mutation_keys = |value: &Value| {
        value
            .as_object()
            .map(|o| o.keys().cloned().collect::<BTreeSet<_>>())
    };
    saved_require(
        mutation_keys(&manifest["protocol"]["mutation"])
            == mutation_keys(&serde_json::to_value(&protocol.mutation).map_err(|e| e.to_string())?),
        "saved mutation table is incomplete or has unknown keys",
    )?;
    let provenance_keys = [
        "source_revision",
        "source_dirty",
        "core_build",
        "compiler",
        "target",
        "build_profile",
    ];
    for key in provenance_keys {
        saved_require(
            manifest[key].as_str().is_some_and(|v| !v.is_empty()),
            "missing saved build/source provenance",
        )?;
    }
    saved_require(
        ["true", "false", "unknown"].contains(&manifest["source_dirty"].as_str().unwrap()),
        "invalid saved dirty-source flag",
    )?;
    let runs = root["runs"]
        .as_array()
        .ok_or("saved root summary lacks runs")?;
    saved_require(
        runs.len() == protocol.seeds.len()
            && runs
                .iter()
                .zip(&protocol.seeds)
                .all(|(run, seed)| run["seed"].as_u64() == Some(*seed)),
        "root summary does not preserve every declared seed exactly once and in order",
    )?;
    let failures = runs.iter().filter(|run| run["status"] == "failed").count();
    saved_require(
        root["technical_failures"].as_u64() == Some(failures as u64)
            && root["status"]
                == if failures == 0 {
                    "completed"
                } else {
                    "completed_with_failures"
                },
        "root completion status/failure count disagrees with runs",
    )?;
    let mut policies = Vec::with_capacity(runs.len());
    for (training, seed) in runs.iter().zip(&protocol.seeds) {
        let genome = if training["status"] == "failed" {
            saved_require(
                training["error"].as_str().is_some_and(|e| !e.is_empty()),
                "failed seed lacks its error record",
            )?;
            None
        } else {
            saved_require(
                training["status"] == "completed",
                "partial/unknown seed cannot become a saved policy",
            )?;
            let summary = files.read(format!("seed_{seed}/summary.json"))?;
            saved_require(
                &summary == training,
                "root and per-seed completion summaries disagree",
            )?;
            let population = validate_saved_population(
                files.read(format!("seed_{seed}/final_population.json"))?,
                &protocol,
                &arena,
            )?;
            let champion = &population[0];
            validate_saved_champion(&summary, champion, &protocol, *seed)?;
            Some(champion.genome.clone())
        };
        policies.push(SavedPolicy {
            seed: *seed,
            training: training.clone(),
            genome,
        });
    }
    let provenance: serde_json::Map<String, Value> = provenance_keys
        .into_iter()
        .map(|key| (key.to_owned(), manifest[key].clone()))
        .collect();
    let metadata = json!({"schema_version":1,"source_kind":"saved_acquisition_training_selection",
        "session_path":session.to_string_lossy(),"training_manifest":manifest,
        "source_provenance":provenance,"resolved_protocol":protocol,
        "training_completion_status":root["status"],"technical_failures":failures,
        "selection_basis":"archived final training rank only; no new evaluation or test scores used for policy selection",
        "input_files":files.records,"total_input_bytes":files.used,
        "input_identity":"FNV1a64 over exact bytes is noncryptographic; archive SHA-256 externally",
        "limits":{"per_file_bytes":SAVED_FILE_LIMIT,"aggregate_bytes":SAVED_TOTAL_LIMIT}});
    Ok(SavedSelection {
        metadata,
        arena,
        policies,
    })
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

fn score(candidate: &mut Candidate, arena: &ArenaConfig, fitness: Fitness) -> RunResult<()> {
    candidate.genome.validate().map_err(|e| e.to_string())?;
    if candidate.genome.connections.iter().any(|c| c.recurrent) {
        return Err("unexpected recurrent gene in feed-forward acquisition".into());
    }
    let brain = candidate.genome.build_brain();
    match fitness {
        Fitness::TerminalOnly => {
            candidate.correct = delayed_choice::visible_score(&brain, arena)?;
            candidate.terminal_progress = None;
        }
        Fitness::TerminalProgress => {
            let (correct, progress) = delayed_choice::visible_score_with_progress(&brain, arena)?;
            if !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
                return Err("invalid terminal progress".into());
            }
            candidate.correct = correct;
            candidate.terminal_progress = Some(progress);
        }
    }
    Ok(())
}

fn rank(population: &mut [Candidate], fitness: Fitness) {
    population.sort_by(|a, b| {
        b.correct
            .cmp(&a.correct)
            .then_with(|| {
                if fitness == Fitness::TerminalProgress {
                    b.terminal_progress
                        .unwrap()
                        .total_cmp(&a.terminal_progress.unwrap())
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .then(a.id.cmp(&b.id))
    });
}

fn generation_log(file: &mut File, generation: u32, population: &[Candidate]) -> RunResult<()> {
    let scores: Vec<_> = population
        .iter()
        .map(|c| {
            let mut row = json!({"id":c.id,"parent_id":c.parent_id,"correct":c.correct});
            if let Some(progress) = c.terminal_progress {
                row["terminal_progress"] = json!(progress);
            }
            row
        })
        .collect();
    let mean = population.iter().map(|c| u64::from(c.correct)).sum::<u64>() as f64
        / (2 * population.len()) as f64;
    let mut record = json!({"generation":generation,"champion_id":population[0].id,
        "best_mean_reward":f64::from(population[0].correct)/2.0,"population_mean_reward":mean,
        "both_correct":population.iter().filter(|c| c.correct==2).count(),"scores":scores});
    if let Some(progress) = population[0].terminal_progress {
        record["champion_terminal_progress"] = json!(progress);
    }
    serde_json::to_writer(&mut *file, &record).map_err(|e| e.to_string())?;
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
    // Ecological diet mutation has its own probability after the diet revision.
    // This task's weight-only protocol disables that independent trait path too.
    config.food.fruit.diet_mutation_probability = 0.0;
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
            terminal_progress: None,
            genome,
        };
        score(&mut candidate, arena, protocol.fitness)?;
        population.push(candidate);
    }
    rank(&mut population, protocol.fitness);
    let founder_best = population[0].correct;
    let founder_progress = population[0].terminal_progress;
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
                terminal_progress: None,
                genome: population[parent].genome.reproduce(
                    &protocol.mutation,
                    &FruitConfig::default(),
                    &mut innovations,
                    &mut rng,
                ),
            };
            next_id += 1;
            score(&mut child, arena, protocol.fitness)?;
            offspring.push(child);
        }
        // Elites are evaluated again: fixed population×(generations+1) budget.
        for elite in &mut offspring[..protocol.elites] {
            score(elite, arena, protocol.fitness)?;
        }
        rank(&mut offspring, protocol.fitness);
        population = offspring;
        generation_log(&mut curve, generation, &population)?;
    }
    write_json(&dir.join("final_population.json"), &population)?;
    write_json(&dir.join("innovations.json"), &innovations)?;
    let champion = &population[0];
    let mut summary = json!({"seed":seed,"status":"completed","evaluations":protocol.population as u64*(u64::from(protocol.generations)+1),
        "founder_best_mean_reward":f64::from(founder_best)/2.0,"final_best_mean_reward":f64::from(champion.correct)/2.0,
        "champion_id":champion.id,"both_correct_at_founding":founder_best==2,"both_correct_at_end":champion.correct==2,
        "improvement_over_founders":champion.correct>founder_best,
        "elapsed_seconds":timer.elapsed().as_secs_f64(),
        "champion_training_trials":if protocol.fitness == Fitness::TerminalProgress {
            delayed_choice::evaluate_brain_with_progress(&champion.genome.build_brain(), arena)?
        } else { delayed_choice::evaluate_brain(&champion.genome.build_brain(),arena,true)? }});
    if let Some(progress) = champion.terminal_progress {
        summary["founder_best_terminal_progress"] = json!(founder_progress.unwrap());
        summary["final_best_terminal_progress"] = json!(progress);
    }
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
    let mut manifest = json!({"schema_version":1,"task":"visible_cue_genetic_acquisition","status":"started",
        "protocol":protocol,"arena":arena,"protocol_path":protocol_path,"arena_path":arena_path,
        "total_evaluations":evaluations,"execution":"serial; no ecology or Rayon pool",
        "fitness":"number correct across cues [-1,+1]; terminal first contact only; wrong/timeout zero",
        "selection":"asexual elitism; tournament with replacement; descending correct then ascending persistent candidate ID; no crossover/speciation",
        "founders":"Genome::random with declared dimensions/mutation; no injected reference weights",
        "embedded_founder_defaults":Config::parse(include_str!("../../../configs/default.toml")).map_err(|e|e.to_string())?,
        "isolated_diet_mutation_probability":0.0,
        "phenotype":"20-style arena inputs (dimensions in arena), declared hidden, seven outputs; methylation zero and fixed; no body scaling, metabolism, energy selection or lifetime updates; neutral hue drift from native reproduce retained",
        "champion_rule":"best member of final population by training score then lowest ID; fixed budget, no early stopping",
        "validation":"none; champion trials replay training conditions only; not a held-out evaluation",
        "unit":"independent founder-population seed; cues/candidates/generations are not independent replicates",
        "source_revision":env!("TERRARIUM_SOURCE_REVISION"),"source_dirty":env!("TERRARIUM_SOURCE_DIRTY"),
        "core_build":sim_core::checkpoint::CORE_BUILD_ID,"compiler":env!("TERRARIUM_COMPILER"),
        "target":env!("TERRARIUM_TARGET"),"build_profile":env!("TERRARIUM_BUILD_PROFILE"),"rng":"ChaCha8Rng seed_from_u64; fixed-width random_index",
        "interpretation":"engineering feasibility pilot; genetic optimization across generations, not memory, within-life learning, ecological adaptation or generalization"});
    if protocol.fitness == Fitness::TerminalProgress {
        manifest["fitness"] = json!(
            "correct-count primary, then mean clipped terminal surface-distance progress toward assigned goal; teacher target consulted only after termination"
        );
        manifest["selection"] = json!(
            "asexual elitism; tournament with replacement; descending correct, descending terminal_progress, ascending persistent candidate ID; no crossover/speciation"
        );
        manifest["champion_rule"] = json!(
            "best final population member by correct count, terminal progress, then ID; fixed budget, no early stopping"
        );
        manifest["progress_definition"] = json!(
            "mean across cues of clamp((start_surface_distance-terminal_surface_distance)/start_surface_distance,0,1); confirmed correct contact=1; terminal means first contact or horizon, never closest approach"
        );
    }
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

    fn saved_fixture(progress: bool, failed_seed: bool) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NUMBER: AtomicU64 = AtomicU64::new(0);
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../output/saved-selection-tests")
            .join(format!(
                "{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NUMBER.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&path).unwrap();
        let arena: ArenaConfig =
            toml::from_str(include_str!("../../../configs/delayed_choice.toml")).unwrap();
        let mut protocol: Protocol =
            toml::from_str(include_str!("../../../configs/visible_acquisition.toml")).unwrap();
        protocol.population = 2;
        protocol.generations = 1;
        protocol.elites = 1;
        protocol.tournament = 1;
        protocol.seeds = vec![700, 701];
        protocol.fitness = if progress {
            Fitness::TerminalProgress
        } else {
            Fitness::TerminalOnly
        };
        let manifest = json!({"schema_version":1,"task":"visible_cue_genetic_acquisition","status":"started",
            "protocol":protocol,"arena":arena,"total_evaluations":8,
            "source_revision":"test-source","source_dirty":"true","core_build":"test-build",
            "compiler":"test-compiler","target":"test-target","build_profile":"debug"});
        write_json(&path.join("manifest.json"), &manifest).unwrap();
        let cfg = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
        let mut runs = Vec::new();
        for seed in [700, 701] {
            let dir = path.join(format!("seed_{seed}"));
            std::fs::create_dir(&dir).unwrap();
            if failed_seed && seed == 701 {
                runs.push(
                    json!({"seed":seed,"status":"failed","error":"recorded fixture failure"}),
                );
                // The loader must never inspect partial data for this failed seed.
                std::fs::write(dir.join("final_population.json"), b"invalid partial JSON").unwrap();
                continue;
            }
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut make = |id, parent_id, born_generation, correct, terminal_progress| Candidate {
                id,
                parent_id,
                born_generation,
                correct,
                terminal_progress,
                genome: Genome::random(
                    &cfg.brain,
                    &protocol.mutation,
                    &cfg.aging,
                    &cfg.temperature,
                    &cfg.food.fruit,
                    &mut rng,
                ),
            };
            let population = vec![
                make(2, Some(0), 1, 2, progress.then_some(1.0)),
                make(0, None, 0, 1, progress.then_some(0.5)),
            ];
            write_json(&dir.join("final_population.json"), &population).unwrap();
            let mut negative = json!({"cue":-1,"target":"right","choice":"right","reward":1});
            let mut positive = json!({"cue":1,"target":"left","choice":"left","reward":1});
            if progress {
                negative["terminal_progress"] = json!(1.0);
                positive["terminal_progress"] = json!(1.0);
            }
            let mut intact = json!({"clear_state_each_tick":false,"mean_reward":1.0,"trials":[negative,positive]});
            if progress {
                intact["mean_terminal_progress"] = json!(1.0);
            }
            let mut cleared = intact.clone();
            cleared["clear_state_each_tick"] = json!(true);
            let mut summary = json!({"seed":seed,"status":"completed","evaluations":4,
                "founder_best_mean_reward":0.5,"final_best_mean_reward":1.0,"champion_id":2,
                "both_correct_at_founding":false,"both_correct_at_end":true,"improvement_over_founders":true,
                "elapsed_seconds":0.1,"champion_training_trials":{"cue_visible":true,"intact":intact,"same_network_state_cleared":cleared}});
            if progress {
                summary["founder_best_terminal_progress"] = json!(0.5);
                summary["final_best_terminal_progress"] = json!(1.0);
            }
            write_json(&dir.join("summary.json"), &summary).unwrap();
            runs.push(summary);
        }
        write_json(
            &path.join("summary.json"),
            &json!({"schema_version":1,
            "status":if failed_seed {"completed_with_failures"} else {"completed"},
            "technical_failures":usize::from(failed_seed),"runs":runs}),
        )
        .unwrap();
        path
    }

    fn alter_saved(path: &Path, change: impl FnOnce(&mut Value)) {
        let mut value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        change(&mut value);
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }

    #[test]
    fn saved_loader_selects_only_training_rank_and_retains_failed_seed() {
        for progress in [false, true] {
            let session = saved_fixture(progress, true);
            let selection = load_saved_selection(&session).unwrap();
            assert_eq!(
                selection
                    .policies
                    .iter()
                    .map(|p| p.seed)
                    .collect::<Vec<_>>(),
                vec![700, 701]
            );
            assert!(selection.policies[0].genome.is_some());
            assert!(selection.policies[1].genome.is_none());
            assert_eq!(selection.policies[0].training["champion_id"], 2);
            assert_eq!(selection.policies[1].training["status"], "failed");
            assert_eq!(selection.arena.input_count(), 20);
            assert_eq!(
                selection.metadata["input_files"].as_array().unwrap().len(),
                4
            );
            assert!(
                selection.metadata["input_files"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|f| f["fnv1a64_noncryptographic"].as_str().unwrap().len() == 16)
            );
            assert_eq!(selection.metadata["technical_failures"], 1);
        }
    }

    #[test]
    fn saved_loader_rejects_nonchampion_corruption_rank_ids_and_summary_mismatches() {
        for mutation in 0..5 {
            let session = saved_fixture(true, false);
            let file = session.join("seed_700/final_population.json");
            alter_saved(&file, |pop| match mutation {
                0 => pop[1]["genome"]["connections"][0]["methylation"] = json!(0.5),
                1 => pop.as_array_mut().unwrap().reverse(),
                2 => {
                    let first = pop[0]["id"].clone();
                    pop[1]["id"] = first;
                }
                3 => pop[1]["terminal_progress"] = Value::Null,
                _ => pop[1]["genome"]["num_inputs"] = json!(usize::MAX),
            });
            assert!(load_saved_selection(&session).is_err());
        }
        let session = saved_fixture(false, false);
        alter_saved(&session.join("seed_700/summary.json"), |s| {
            s["champion_id"] = json!(0)
        });
        assert!(load_saved_selection(&session).is_err());
    }

    #[test]
    fn saved_loader_rejects_partial_status_missing_seeds_and_failure_count_mismatch() {
        for mutation in 0..3 {
            let session = saved_fixture(false, false);
            alter_saved(&session.join("summary.json"), |root| match mutation {
                0 => root["status"] = json!("running"),
                1 => {
                    root["runs"].as_array_mut().unwrap().pop();
                }
                _ => root["technical_failures"] = json!(1),
            });
            assert!(load_saved_selection(&session).is_err());
        }
    }

    #[test]
    fn saved_loader_enforces_individual_and_aggregate_read_budgets() {
        let session = saved_fixture(false, false);
        OpenOptions::new()
            .write(true)
            .open(session.join("manifest.json"))
            .unwrap()
            .set_len(SAVED_FILE_LIMIT + 1)
            .unwrap();
        assert!(
            load_saved_selection(&session)
                .unwrap_err()
                .contains("64MiB")
        );
        let session = saved_fixture(false, false).canonicalize().unwrap();
        let mut files = SavedFiles {
            session,
            used: SAVED_TOTAL_LIMIT,
            records: Vec::new(),
        };
        assert!(files.read("manifest.json").is_err());
    }
    #[test]
    fn progress_breaks_only_correctness_ties_then_uses_id() {
        let cfg = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
        let mut rng = ChaCha8Rng::seed_from_u64(99);
        let genome = Genome::random(
            &cfg.brain,
            &cfg.mutation,
            &cfg.aging,
            &cfg.temperature,
            &cfg.food.fruit,
            &mut rng,
        );
        let make = |id, correct, progress| Candidate {
            id,
            parent_id: None,
            born_generation: 0,
            correct,
            terminal_progress: Some(progress),
            genome: genome.clone(),
        };
        let population = vec![
            make(1, 1, 0.5),
            make(9, 1, 0.8),
            make(3, 2, 0.0),
            make(2, 1, 0.8),
        ];
        let mut progressed = population.clone();
        rank(&mut progressed, Fitness::TerminalProgress);
        assert_eq!(
            progressed.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![3, 2, 9, 1]
        );
        let mut legacy = population;
        rank(&mut legacy, Fitness::TerminalOnly);
        assert_eq!(
            legacy.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![3, 1, 2, 9]
        );
        let omitted: Protocol =
            toml::from_str(include_str!("../../../configs/visible_acquisition.toml")).unwrap();
        assert_eq!(omitted.fitness, Fitness::TerminalOnly);
    }

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
