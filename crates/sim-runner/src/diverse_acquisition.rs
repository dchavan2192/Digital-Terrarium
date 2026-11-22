//! Isolated paired acquisition with matched founders and equal scheduled trial counts.
//! This family deliberately does not reuse or alter the legacy saved-selection format.
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
    version: u32,
    population: usize,
    generations: u32,
    elites: usize,
    tournament: usize,
    seeds: Vec<u64>,
    hidden: usize,
    mutation: MutationConfig,
    holdouts: Vec<Holdout>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Holdout {
    RelativeGoals {
        name: String,
        forward: f64,
        lateral: f64,
    },
    RigidRotation {
        name: String,
        radians: f64,
    },
}

fn frozen_holdouts() -> Vec<Holdout> {
    vec![
        Holdout::RelativeGoals {
            name: "geometry_forward_shift".into(),
            forward: 5.0,
            lateral: 3.5,
        },
        Holdout::RelativeGoals {
            name: "geometry_wide_shift".into(),
            forward: 3.5,
            lateral: 5.0,
        },
        Holdout::RelativeGoals {
            name: "geometry_near_narrow".into(),
            forward: 4.5,
            lateral: 3.0,
        },
        Holdout::RelativeGoals {
            name: "geometry_far_diagonal".into(),
            forward: 5.5,
            lateral: 4.5,
        },
        Holdout::RigidRotation {
            name: "heading_minus30".into(),
            radians: -std::f64::consts::PI / 6.0,
        },
        Holdout::RigidRotation {
            name: "heading_plus30".into(),
            radians: std::f64::consts::PI / 6.0,
        },
    ]
}

#[derive(Clone)]
struct Case {
    name: String,
    arena: ArenaConfig,
}

fn relative(base: &ArenaConfig, forward: f64, lateral: f64) -> RunResult<ArenaConfig> {
    if !forward.is_finite()
        || !lateral.is_finite()
        || !(0.0..=100.0).contains(&forward)
        || !(0.0..=100.0).contains(&lateral)
        || forward == 0.0
        || lateral == 0.0
    {
        return Err("relative goals require finite forward/lateral in (0,100]".into());
    }
    let mut arena = base.clone();
    let (sin, cos) = (
        sim_core::math::sin(base.start_heading),
        sim_core::math::cos(base.start_heading),
    );
    arena.goal_left_x = base.start_x + forward * cos - lateral * sin;
    arena.goal_left_y = base.start_y + forward * sin + lateral * cos;
    arena.goal_right_x = base.start_x + forward * cos + lateral * sin;
    arena.goal_right_y = base.start_y + forward * sin - lateral * cos;
    arena.validate()?;
    Ok(arena)
}

impl Holdout {
    fn resolve(&self, base: &ArenaConfig) -> RunResult<Case> {
        let (name, arena) = match self {
            Self::RelativeGoals {
                name,
                forward,
                lateral,
            } => (name, relative(base, *forward, *lateral)?),
            Self::RigidRotation { name, radians } => {
                if !radians.is_finite() || radians.abs() > std::f64::consts::PI {
                    return Err(
                        "holdout rotation must be finite and within half a revolution".into(),
                    );
                }
                let mut arena = base.clone();
                let (sin, cos) = (sim_core::math::sin(*radians), sim_core::math::cos(*radians));
                let rotate = |x, y| {
                    (
                        base.start_x + (x - base.start_x) * cos - (y - base.start_y) * sin,
                        base.start_y + (x - base.start_x) * sin + (y - base.start_y) * cos,
                    )
                };
                (arena.goal_left_x, arena.goal_left_y) = rotate(base.goal_left_x, base.goal_left_y);
                (arena.goal_right_x, arena.goal_right_y) =
                    rotate(base.goal_right_x, base.goal_right_y);
                arena.start_heading =
                    (base.start_heading + radians).rem_euclid(2.0 * std::f64::consts::PI);
                arena.validate()?;
                (name, arena)
            }
        };
        if name.is_empty()
            || name.len() > 64
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err("holdout names must be ASCII alphanumeric/underscore, length 1..64".into());
        }
        Ok(Case {
            name: name.clone(),
            arena,
        })
    }
}

fn training_cases(base: &ArenaConfig, arm: &str) -> RunResult<Vec<Case>> {
    match arm {
        "fixed" => Ok((1..=3)
            .map(|copy| Case {
                name: format!("reference_copy_{copy}"),
                arena: base.clone(),
            })
            .collect()),
        "varied" => Ok(vec![
            Case {
                name: "reference".into(),
                arena: base.clone(),
            },
            Case {
                name: "near".into(),
                arena: relative(base, 3.0, 3.0)?,
            },
            Case {
                name: "narrow".into(),
                arena: relative(base, 4.0, 2.5)?,
            },
        ]),
        _ => Err("unknown diversity arm".into()),
    }
}

fn arm_order(seed: u64) -> [&'static str; 2] {
    if seed.is_multiple_of(2) {
        ["fixed", "varied"]
    } else {
        ["varied", "fixed"]
    }
}

impl Protocol {
    fn validate(&self, base: &ArenaConfig) -> RunResult<(Vec<Case>, Value)> {
        base.validate()?;
        if self.version != 1
            || !(2..=64).contains(&self.population)
            || !(1..=25).contains(&self.generations)
            || self.elites == 0
            || self.elites >= self.population
            || self.tournament == 0
            || self.tournament > self.population
            || self.seeds.is_empty()
            || self.seeds.len() > 4
            || self.seeds.iter().copied().collect::<BTreeSet<_>>().len() != self.seeds.len()
            || !(1..=8).contains(&self.hidden)
            || self.hidden > MAX_NODES.saturating_sub(base.input_count() + 8)
            || self.holdouts.len() != 6
        {
            return Err("invalid diversity version/population/generations/elites/tournament/seeds/hidden or six-case holdout suite".into());
        }
        if self.holdouts != frozen_holdouts() {
            return Err(
                "diversity v1 requires the frozen six holdout definitions in their declared order"
                    .into(),
            );
        }
        if !self.mutation.weight_probability.is_finite()
            || !(0.0..=1.0).contains(&self.mutation.weight_probability)
            || !self.mutation.weight_step.is_finite()
            || !(0.01..=2.0).contains(&self.mutation.weight_step)
            || [
                self.mutation.add_connection_probability,
                self.mutation.add_node_probability,
                self.mutation.toggle_probability,
                self.mutation.prune_probability,
                self.mutation.add_recurrent_probability,
                self.mutation.trait_probability,
            ]
            .iter()
            .any(|p| *p != 0.0)
        {
            return Err("diversity acquisition requires finite bounded weight-only mutation, recurrence/topology/traits off".into());
        }
        let holdouts = self
            .holdouts
            .iter()
            .map(|case| case.resolve(base))
            .collect::<RunResult<Vec<_>>>()?;
        let mut names = BTreeSet::new();
        for case in &holdouts {
            if !names.insert(case.name.clone())
                || ["reference", "near", "narrow"].contains(&case.name.as_str())
            {
                return Err("holdout names must be unique and distinct from training cases".into());
            }
        }
        // Resolve every arena before writing artifacts; validate compatible inputs,
        // finite positive progress denominators, ray coverage and plant reachability.
        let training = training_cases(base, "varied")?;
        for case in training.iter().chain(&holdouts) {
            if case.arena.input_count() != base.input_count() {
                return Err("all diversity arenas must share input dimensions".into());
            }
            let checks = delayed_choice::geometry_preflight(&case.arena)?;
            let goals = checks["goals"]
                .as_array()
                .ok_or("missing geometry checks")?;
            if goals.iter().any(|g| {
                g["witness_within_horizon"] != true
                    || g["initial_ray_hits"].as_u64().unwrap_or(0) == 0
            }) {
                return Err(format!(
                    "case {} lacks initial goal coverage or a within-horizon plant witness",
                    case.name
                ));
            }
        }
        let evaluations = self.population as u64
            * (u64::from(self.generations) + 1)
            * self.seeds.len() as u64
            * 2;
        let training_trials = evaluations * 6;
        let trial_steps =
            u64::from(base.cue_ticks) + u64::from(base.blank_delay) + u64::from(base.horizon);
        // Each final champion is replayed intact+cleared on 3 training and 6 holdout
        // cases (four trials per case); six baseline control trials once overall.
        let diagnostic_trials =
            self.seeds.len() as u64 * 2 * (3 + self.holdouts.len() as u64) * 4 + 6;
        let trace_samples = (trial_steps + 1) * diagnostic_trials;
        let steps = (training_trials + diagnostic_trials) * trial_steps;
        let edges = ((base.input_count() + 1 + 7) * self.hidden + 7) as u64;
        if trace_samples > 100_000
            || steps > 3_000_000
            || steps.saturating_mul(edges) > 1_000_000_000
        {
            return Err("diversity budget exceeds 100000 trace samples, 3000000 neural steps or 1000000000 edge-steps".into());
        }
        Ok((
            holdouts,
            json!({"training_candidate_evaluations":evaluations,
            "scheduled_training_trials":training_trials,"training_trials_per_candidate":6,
            "diagnostic_trials":diagnostic_trials,"max_neural_steps":steps,
            "max_edge_steps":steps*edges,"retained_trace_samples_upper_bound":trace_samples,
            "note":"scheduled trials and conservative step bounds are matched; actual early-contact steps may differ"}),
        ))
    }
}

#[derive(Clone, Serialize)]
struct CaseScore {
    case: String,
    correct: u8,
    terminal_progress: f64,
}

#[derive(Clone, Serialize)]
struct Candidate {
    id: u64,
    parent_id: Option<u64>,
    born_generation: u32,
    correct: u8,
    terminal_progress: f64,
    case_scores: Vec<CaseScore>,
    genome: Genome,
}

fn score(candidate: &mut Candidate, cases: &[Case]) -> RunResult<()> {
    candidate.genome.validate().map_err(|e| e.to_string())?;
    if candidate
        .genome
        .connections
        .iter()
        .any(|c| c.recurrent || c.methylation != 0.0)
    {
        return Err("unexpected recurrence or methylation in feed-forward diversity search".into());
    }
    let brain = candidate.genome.build_brain();
    let mut scores = Vec::with_capacity(3);
    for case in cases {
        // No caching of the reference copies: all six trials are actually run.
        let (correct, progress) = delayed_choice::visible_score_with_progress(&brain, &case.arena)?;
        if correct > 2 || !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
            return Err("invalid diversity training score".into());
        }
        scores.push(CaseScore {
            case: case.name.clone(),
            correct,
            terminal_progress: progress,
        });
    }
    candidate.correct = scores.iter().map(|s| s.correct).sum();
    candidate.terminal_progress = scores.iter().map(|s| s.terminal_progress).sum::<f64>() / 3.0;
    candidate.case_scores = scores;
    Ok(())
}

fn rank(population: &mut [Candidate]) {
    population.sort_by(|a, b| {
        b.correct
            .cmp(&a.correct)
            .then_with(|| b.terminal_progress.total_cmp(&a.terminal_progress))
            .then(a.id.cmp(&b.id))
    });
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

fn generation_log(file: &mut File, generation: u32, population: &[Candidate]) -> RunResult<()> {
    let scores: Vec<_> = population.iter().map(|c| json!({"id":c.id,"parent_id":c.parent_id,
        "born_generation":c.born_generation,"correct":c.correct,"terminal_progress":c.terminal_progress,
        "case_scores":c.case_scores})).collect();
    serde_json::to_writer(&mut *file, &json!({"generation":generation,"champion_id":population[0].id,
        "best_correct":population[0].correct,"champion_terminal_progress":population[0].terminal_progress,
        "all_six_correct":population.iter().filter(|c| c.correct==6).count(),"scores":scores}))
        .map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())
}

fn trace_cases(champion: &Candidate, cases: &[Case]) -> Vec<Value> {
    let brain = champion.genome.build_brain();
    cases
        .iter()
        .map(
            |case| match delayed_choice::evaluate_brain_with_progress(&brain, &case.arena) {
                Ok(evaluation) => {
                    json!({"case":case.name,"status":"completed","evaluation":evaluation})
                }
                Err(error) => json!({"case":case.name,"status":"failed","error":error}),
            },
        )
        .collect()
}

struct Trained {
    summary: Value,
    champion: Candidate,
    training_seconds: f64,
}

// This function cannot access held-out arenas: every search finishes before the
// caller creates the campaign-wide selection lock and enters the evaluation phase.
fn search(
    protocol: &Protocol,
    base: &ArenaConfig,
    arm: &str,
    seed: u64,
    dir: &Path,
) -> RunResult<Trained> {
    std::fs::create_dir(dir).map_err(|e| e.to_string())?;
    let timer = Instant::now();
    let cases = training_cases(base, arm)?;
    let mut config =
        Config::parse(include_str!("../../../configs/default.toml")).map_err(|e| e.to_string())?;
    config.brain.inputs = base.input_count();
    config.brain.hidden = protocol.hidden;
    config.brain.outputs = 7;
    config.food.fruit.diet_mutation_probability = 0.0;
    // Reset this stream independently for each arm: founder genomes match by ID.
    // Selection may subsequently diverge; no claim of matched later histories.
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
        population.push(Candidate {
            id,
            parent_id: None,
            born_generation: 0,
            correct: 0,
            terminal_progress: 0.0,
            case_scores: Vec::new(),
            genome,
        });
    }
    // Persist full founder inputs before any score can fail.
    write_json(
        &dir.join("founder_genomes.json"),
        &population
            .iter()
            .map(|c| json!({"id":c.id,"genome":c.genome}))
            .collect::<Vec<_>>(),
    )?;
    for candidate in &mut population {
        score(candidate, &cases)?;
    }
    rank(&mut population);
    let founder_correct = population[0].correct;
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
            let parent = (0..protocol.tournament)
                .map(|_| sim_core::math::random_index(&mut rng, population.len()))
                .min()
                .unwrap();
            let mut child = Candidate {
                id: next_id,
                parent_id: Some(population[parent].id),
                born_generation: generation,
                correct: 0,
                terminal_progress: 0.0,
                case_scores: Vec::new(),
                genome: population[parent].genome.reproduce(
                    &protocol.mutation,
                    &config.food.fruit,
                    &mut innovations,
                    &mut rng,
                ),
            };
            next_id += 1;
            score(&mut child, &cases)?;
            offspring.push(child);
        }
        for elite in &mut offspring[..protocol.elites] {
            score(elite, &cases)?;
        }
        rank(&mut offspring);
        population = offspring;
        generation_log(&mut curve, generation, &population)?;
    }
    write_json(&dir.join("final_population.json"), &population)?;
    write_json(&dir.join("innovations.json"), &innovations)?;
    let champion = &population[0];
    // Commit the selected identity to disk before evaluating any held-out case.
    write_json(
        &dir.join("selection.json"),
        &json!({"seed":seed,"arm":arm,
        "champion_id":champion.id,"correct":champion.correct,"terminal_progress":champion.terminal_progress,
        "basis":"final training rank only; no holdout scores consulted"}),
    )?;
    let training_traces = trace_cases(champion, &cases);
    let summary = json!({"schema_version":1,"task":"visible_diversity_acquisition","seed":seed,"arm":arm,
        "status":"training_completed","evaluations":protocol.population as u64*(u64::from(protocol.generations)+1),
        "scheduled_training_trials":protocol.population as u64*(u64::from(protocol.generations)+1)*6,
        "champion_id":champion.id,"founder_best_correct":founder_correct,"founder_best_terminal_progress":founder_progress,
        "training_correct":champion.correct,"training_terminal_progress":champion.terminal_progress,
        "all_six_correct_at_end":champion.correct==6,"training_cases":training_traces});
    write_json(&dir.join("training_summary.json"), &summary)?;
    Ok(Trained {
        summary,
        champion: champion.clone(),
        training_seconds: timer.elapsed().as_secs_f64(),
    })
}

/// Streaming exact-byte identity, explicitly noncryptographic. The archived
/// campaign must additionally record external SHA-256 before scientific use.
fn file_identity(session: &Path, relative: &Path) -> RunResult<Value> {
    let mut file = File::open(session.join(relative)).map_err(|e| e.to_string())?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();
    if size > 64 * 1024 * 1024 {
        return Err("training lock artifact exceeds 64MiB".into());
    }
    let mut hash = 0xcbf29ce484222325u64;
    let mut count = 0u64;
    let mut buffer = [0u8; 65_536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > 64 * 1024 * 1024 {
            return Err("training lock artifact grew beyond 64MiB".into());
        }
        for byte in &buffer[..n] {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
    }
    if count != size {
        return Err("training artifact size changed during selection lock".into());
    }
    Ok(json!({"path":relative,"bytes":count,"fnv1a64_noncryptographic":format!("{hash:016x}")}))
}

fn finish_evaluation(
    mut trained: Trained,
    holdouts: &[Case],
    lock: &Value,
    dir: &Path,
) -> RunResult<Value> {
    let timer = Instant::now();
    let held_out = trace_cases(&trained.champion, holdouts);
    let training_traces = trained.summary["training_cases"]
        .as_array()
        .ok_or("missing training traces")?;
    let failures = training_traces
        .iter()
        .chain(&held_out)
        .filter(|c| c["status"] != "completed")
        .count();
    trained.summary["holdout_cases"] = json!(held_out);
    trained.summary["technical_case_failures"] = json!(failures);
    trained.summary["status"] = json!(if failures == 0 {
        "completed"
    } else {
        "completed_with_failures"
    });
    trained.summary["selection_lock"] = lock.clone();
    let evaluation_seconds = timer.elapsed().as_secs_f64();
    trained.summary["training_elapsed_seconds"] = json!(trained.training_seconds);
    trained.summary["evaluation_elapsed_seconds"] = json!(evaluation_seconds);
    trained.summary["elapsed_seconds"] = json!(trained.training_seconds + evaluation_seconds);
    trained.summary["elapsed_definition"] = json!(
        "active training plus held-out evaluation wall time; excludes the intervening work on other arm/seed runs and selection-lock hashing"
    );
    write_json(&dir.join("summary.json"), &trained.summary)?;
    Ok(trained.summary)
}

fn load_protocol(path: &Path) -> RunResult<(Protocol, String)> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(65_537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65_536 {
        return Err("diversity protocol exceeds 64KiB".into());
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
                return Err(format!("unknown diversity mutation field: {key}"));
            }
        }
    }
    let fingerprint = bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    Ok((
        toml::from_str(text).map_err(|e| format!("invalid diversity protocol: {e}"))?,
        format!("{fingerprint:016x}"),
    ))
}

pub fn run(protocol_path: &Path, arena_path: &Path, output: &Path) -> RunResult<()> {
    let (protocol, fingerprint) = load_protocol(protocol_path)?;
    let base = delayed_choice::parse_config(arena_path)?;
    let (holdouts, budget) = protocol.validate(&base)?;
    let controls = delayed_choice::validate_controls(&base, true)?;
    let fixed = training_cases(&base, "fixed")?;
    let varied = training_cases(&base, "varied")?;
    let describe = |cases: &[Case]| -> RunResult<Vec<Value>> {
        cases
            .iter()
            .map(|case| {
                Ok(json!({"case":case.name,"arena":case.arena,
            "geometry_preflight":delayed_choice::geometry_preflight(&case.arena)?}))
            })
            .collect()
    };
    let resolved_fixed = describe(&fixed)?;
    let resolved_varied = describe(&varied)?;
    let resolved_holdouts = describe(&holdouts)?;
    let session = run::create_session(output)?;
    let schedule: Vec<_> = protocol
        .seeds
        .iter()
        .map(|seed| json!({"seed":seed,"arms":arm_order(*seed)}))
        .collect();
    write_json(
        &session.join("manifest.json"),
        &json!({"schema_version":1,"task":"visible_diversity_acquisition",
        "status":"started","protocol":protocol,"arena":base,"protocol_path":protocol_path,"arena_path":arena_path,
        "protocol_fnv1a64_noncryptographic":fingerprint,"budget":budget,"schedule":schedule,
        "training_cases":{"fixed":resolved_fixed,"varied":resolved_varied},"holdout_cases":resolved_holdouts,
        "execution":"serial native; no ecology or Rayon pool; arm order alternates by seed parity",
        "selection":"descending correct across six training trials, then descending mean terminal progress across three cases, then ascending persistent ID",
        "progress_definition":"mean of per-case balanced-cue clamp((start_surface_distance-terminal_surface_distance)/start_surface_distance,0,1); confirmed correct contact=1; never closest approach",
        "founders":"independent ChaCha8Rng(seed) reset for each arm; Genome::random; full founder inputs archived before scoring; native tournament/reproduce mechanics retained",
        "embedded_founder_defaults":Config::parse(include_str!("../../../configs/default.toml")).map_err(|e|e.to_string())?,
        "isolated_diet_mutation_probability":0.0,
        "phenotype":"fixed feed-forward topology, no recurrence/plasticity/methylation/within-life learning; neutral hue drift from native reproduce retained",
        "champion_rule":"final training rank only; all training attempts finish and campaign-wide selection_lock.json is written and fingerprinted before ANY held-out neural evaluation; all seeds and partial outcomes retained; no early stopping or holdout selection",
        "unit":"paired independent founder-population seed; arms/cues/cases/candidates/generations are not additional independent replicates",
        "interpretation":"small engineering comparison of fixed versus varied task-directed genetic training; no memory or ecological-adaptation claim",
        "source_revision":env!("TERRARIUM_SOURCE_REVISION"),"source_dirty":env!("TERRARIUM_SOURCE_DIRTY"),
        "core_build":sim_core::checkpoint::CORE_BUILD_ID,"compiler":env!("TERRARIUM_COMPILER"),
        "target":env!("TERRARIUM_TARGET"),"build_profile":env!("TERRARIUM_BUILD_PROFILE")}),
    )?;
    write_json(&session.join("controls.json"), &controls)?;
    for arm in ["fixed", "varied"] {
        std::fs::create_dir(session.join(arm)).map_err(|e| e.to_string())?;
    }
    let mut searches = Vec::new();
    for seed in &protocol.seeds {
        for arm in arm_order(*seed) {
            let dir = session.join(arm).join(format!("seed_{seed}"));
            let result = match search(&protocol, &base, arm, *seed, &dir) {
                Ok(trained) => Ok(trained),
                Err(error) => {
                    let failed = json!({"seed":seed,"arm":arm,"status":"failed","error":error});
                    if dir.is_dir() && !dir.join("summary.json").exists() {
                        let _ = write_json(&dir.join("summary.json"), &failed);
                    }
                    Err(failed)
                }
            };
            searches.push((*seed, arm, result));
        }
    }
    let mut locked = Vec::new();
    for (seed, arm, result) in &searches {
        let record = match result {
            Ok(trained) => {
                let relative = Path::new(arm).join(format!("seed_{seed}"));
                let artifacts = [
                    "founder_genomes.json",
                    "founders.json",
                    "generations.jsonl",
                    "final_population.json",
                    "innovations.json",
                    "selection.json",
                    "training_summary.json",
                ]
                .iter()
                .map(|file| file_identity(&session, &relative.join(file)))
                .collect::<RunResult<Vec<_>>>()?;
                json!({"seed":seed,"arm":arm,"status":"training_completed",
                    "champion_id":trained.champion.id,"correct":trained.champion.correct,
                    "terminal_progress":trained.champion.terminal_progress,"artifacts":artifacts})
            }
            Err(failed) => failed.clone(),
        };
        locked.push(record);
    }
    let selected = locked
        .iter()
        .filter(|r| r["status"] == "training_completed")
        .count();
    write_json(
        &session.join("selection_lock.json"),
        &json!({"schema_version":1,
        "task":"visible_diversity_acquisition","status":if selected==searches.len() {"locked"} else {"locked_with_failures"},
        "expected_searches":searches.len(),"selected_champions":selected,"failed_searches":searches.len()-selected,
        "basis":"all declared training attempts completed before any held-out neural evaluation; champion identity selected solely by final training rank",
        "hash_note":"exact-byte FNV1a64 identities are noncryptographic; archive external SHA-256 separately",
        "runs":locked}),
    )?;
    let lock = file_identity(&session, Path::new("selection_lock.json"))?;
    // Phase two begins only after the complete selection lock exists on disk.
    let mut results = Vec::new();
    for (seed, arm, result) in searches {
        let summary = match result {
            Ok(trained) => {
                let dir = session.join(arm).join(format!("seed_{seed}"));
                match finish_evaluation(trained, &holdouts, &lock, &dir) {
                    Ok(summary) => summary,
                    Err(error) => {
                        json!({"seed":seed,"arm":arm,"status":"failed","error":error,"phase":"evaluation"})
                    }
                }
            }
            Err(failed) => failed,
        };
        results.push(summary);
    }
    let failures = results
        .iter()
        .filter(|r| r["status"] != "completed")
        .count();
    write_json(
        &session.join("summary.json"),
        &json!({"schema_version":1,"task":"visible_diversity_acquisition",
        "status":if failures==0 {"completed"} else {"completed_with_failures"},
        "technical_failures":failures,"selection_lock":lock,"runs":results}),
    )?;
    if failures > 0 {
        return Err(format!(
            "{failures} diversity arm/seed runs had technical failures; see {}",
            session.display()
        ));
    }
    Ok(())
}
