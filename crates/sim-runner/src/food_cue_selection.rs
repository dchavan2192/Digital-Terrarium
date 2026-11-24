//! Compare selection for signed turns or direct offspring on matched native lifetimes.
use crate::food_cue::training_scores;
use crate::run::{RunResult, create_session};
use crate::stationary_retention::save;
use crate::wiring::search_config;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use serde_json::json;
use sim_core::{
    Config,
    genome::{Genome, Innovations},
};
use std::path::Path;

const SEEDS: std::ops::Range<u64> = 6700..6708;
const POPULATION: usize = 16;
const ELITES: usize = 4;
const GENERATIONS: usize = 12;
const REQUIRED_SEEDS: usize = 4;
const ARMS: [(&str, bool, bool); 4] = [
    ("fixed_turn", false, false),
    ("varied_turn", true, false),
    ("fixed_births", false, true),
    ("varied_births", true, true),
];

#[derive(Clone, Serialize)]
struct Candidate {
    parent: Option<usize>,
    genome: Genome,
    turn_correct: usize,
    min_signed_margin: f64,
    native_births: usize,
}

impl Candidate {
    fn score(genome: Genome, parent: Option<usize>, varied: bool) -> RunResult<Self> {
        let (turn_correct, min_signed_margin, native_births) = training_scores(&genome, varied)?;
        if !min_signed_margin.is_finite() || turn_correct > 16 {
            return Err("invalid scores on the 16-pulse training schedule".into());
        }
        Ok(Self {
            parent,
            genome,
            turn_correct,
            min_signed_margin,
            native_births,
        })
    }
}

struct Founders {
    seed: u64,
    rng: ChaCha8Rng,
    fixed: Vec<Candidate>,
    varied: Vec<Candidate>,
}

impl Founders {
    fn generate(seed: u64, config: &Config) -> RunResult<Self> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut fixed = Vec::with_capacity(POPULATION);
        let mut varied = Vec::with_capacity(POPULATION);
        for _ in 0..POPULATION {
            let genome = Genome::random(
                &config.brain,
                &config.mutation,
                &config.aging,
                &config.temperature,
                &config.food.fruit,
                &mut rng,
            );
            fixed.push(Candidate::score(genome.clone(), None, false)?);
            varied.push(Candidate::score(genome, None, true)?);
        }
        Ok(Self {
            seed,
            rng,
            fixed,
            varied,
        })
    }
}

fn rank(ids: &mut [usize], births: &[Candidate], by_births: bool) {
    ids.sort_by(|&a, &b| {
        let (x, y) = (&births[a], &births[b]);
        let score_order = if by_births {
            y.native_births.cmp(&x.native_births)
        } else {
            y.turn_correct.cmp(&x.turn_correct).then_with(|| {
                y.min_signed_margin
                    .partial_cmp(&x.min_signed_margin)
                    .expect("candidate margins have been validated")
            })
        };
        score_order.then(a.cmp(&b))
    });
}

fn evolve(
    founders: &Founders,
    config: &Config,
    arm: &str,
    varied: bool,
    by_births: bool,
    dir: &Path,
) -> RunResult<(usize, Genome)> {
    let mut rng = founders.rng.clone();
    let mut innovations = Innovations::new(&config.brain);
    let mut births = if varied {
        founders.varied.clone()
    } else {
        founders.fixed.clone()
    };
    let mut pop: Vec<_> = (0..POPULATION).collect();
    let mut generations = Vec::with_capacity(GENERATIONS + 1);
    for generation in 0..=GENERATIONS {
        rank(&mut pop, &births, by_births);
        generations.push(pop.clone());
        if generation == GENERATIONS {
            break;
        }
        pop.truncate(ELITES);
        for _ in ELITES..POPULATION {
            let parent = pop[sim_core::math::random_index(&mut rng, ELITES)];
            let genome = births[parent].genome.reproduce(
                &config.mutation,
                &config.food.fruit,
                &mut innovations,
                &mut rng,
            );
            pop.push(births.len());
            births.push(Candidate::score(genome, Some(parent), varied)?);
        }
    }
    let winner = (pop[0], births[pop[0]].genome.clone());
    save(
        &dir.join(format!("{}_{arm}.json", founders.seed)),
        &json!({"births":births,"generations":generations}),
    )?;
    Ok(winner)
}

fn has_variation(pop: &[Candidate]) -> bool {
    pop.iter()
        .any(|candidate| candidate.native_births != pop[0].native_births)
}

pub fn run(output: &Path) -> RunResult<()> {
    let dir = create_session(output)?;
    let mut cfg = search_config()?;
    cfg.mutation.add_recurrent_probability = 0.;
    let mut mutation = cfg.mutation.clone();
    mutation.add_recurrent_probability = 0.3;
    save(
        &dir.join("manifest.json"),
        &json!({"schema":1,"task":"food_cue_selection","seeds":SEEDS.collect::<Vec<_>>(),
                "arms":ARMS.map(|a| a.0),"training_schedule_seed":6800,"generations":GENERATIONS,
                "population":POPULATION,"elites":ELITES,"children":POPULATION-ELITES,
                "brain":cfg.brain,"founder_mutation":cfg.mutation,"mutation":mutation,
                "compiler":env!("TERRARIUM_COMPILER"),"target":env!("TERRARIUM_TARGET"),
                "source":env!("TERRARIUM_SOURCE_REVISION"),"dirty":env!("TERRARIUM_SOURCE_DIRTY"),
                "core":sim_core::checkpoint::CORE_BUILD_ID}),
    )?;
    let cohorts = SEEDS
        .map(|seed| Founders::generate(seed, &cfg))
        .collect::<RunResult<Vec<_>>>()?;
    let fixed = cohorts.iter().filter(|c| has_variation(&c.fixed)).count();
    let varied = cohorts.iter().filter(|c| has_variation(&c.varied)).count();
    let passed = fixed >= REQUIRED_SEEDS && varied >= REQUIRED_SEEDS;
    let birth_counts = |candidates: &[Candidate]| {
        candidates
            .iter()
            .map(|c| c.native_births)
            .collect::<Vec<_>>()
    };
    let rows: Vec<_> = cohorts
        .iter()
        .map(|c| {
            json!({"seed":c.seed,"fixed":birth_counts(&c.fixed),
                                                    "varied":birth_counts(&c.varied)})
        })
        .collect();
    save(
        &dir.join("founder_gate.json"),
        &json!({"required_seeds":REQUIRED_SEEDS,"passed":passed,"fixed_seeds_with_variation":fixed,
                 "varied_seeds_with_variation":varied,"rows":rows}),
    )?;
    if !passed {
        return Err(format!(
            "insufficient founder birth variation: fixed {fixed}, varied {varied}, each requires {REQUIRED_SEEDS}; see {}",
            dir.display()
        ));
    }
    cfg.mutation = mutation;
    let mut winners = Vec::new();
    for cohort in &cohorts {
        for (arm, varied, by_births) in ARMS {
            let (id, genome) = evolve(cohort, &cfg, arm, varied, by_births, &dir)?;
            winners.push((cohort.seed, arm, id, genome));
        }
    }
    save(&dir.join("selection.json"), &winners)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranking_uses_only_the_assigned_objective_and_breaks_ties_by_id() {
        let config = search_config().unwrap();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let genome = Genome::random(
            &config.brain,
            &config.mutation,
            &config.aging,
            &config.temperature,
            &config.food.fruit,
            &mut rng,
        );
        let scores = [(8, 0., 1), (16, 0.2, 0), (8, -0., 2), (16, 0.4, 0)];
        let candidates: Vec<_> = scores
            .into_iter()
            .map(
                |(turn_correct, min_signed_margin, native_births)| Candidate {
                    genome: genome.clone(),
                    parent: None,
                    turn_correct,
                    min_signed_margin,
                    native_births,
                },
            )
            .collect();
        let mut ids = [2, 1, 3, 0];
        rank(&mut ids, &candidates, false);
        assert_eq!(ids, [3, 1, 0, 2]);
        rank(&mut ids, &candidates, true);
        assert_eq!(ids, [2, 0, 1, 3]);
    }
}
