//! Matched founder experiment specified in RETENTION_WIRING_PLAN.md.
use crate::run::{RunResult, create_session};
use crate::stationary_retention::{Reset, brain_probe, correct, margin, save};
use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use serde_json::json;
use sim_core::{
    Config,
    genome::{Genome, Innovations},
};
use std::path::Path;

#[derive(Serialize)]
struct Birth {
    parent: Option<usize>,
    genome: Genome,
    responses: [f64; 2],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    additional_responses: Vec<[f64; 2]>,
    #[serde(skip)]
    correct: u8,
    #[serde(skip)]
    margin: f64,
}
impl Birth {
    fn new(parent: Option<usize>, genome: Genome, delays: &[usize]) -> Self {
        let brain = genome.build_brain();
        let evaluate = |delay| brain_probe(&brain, false, Reset::Intact, delay);
        let responses = evaluate(delays[0]);
        let additional_responses: Vec<_> = delays[1..].iter().copied().map(evaluate).collect();
        let score = correct(responses)
            + additional_responses
                .iter()
                .copied()
                .map(correct)
                .sum::<u8>();
        let worst = additional_responses
            .iter()
            .copied()
            .map(margin)
            .fold(margin(responses), f64::min);
        Self {
            parent,
            genome,
            responses,
            additional_responses,
            correct: score,
            margin: worst,
        }
    }
}
pub fn run(output: &Path, delay_study: bool) -> RunResult<()> {
    let dir = create_session(output)?;
    let mut cfg =
        Config::parse(include_str!("../../../configs/default.toml")).map_err(|e| e.to_string())?;
    cfg.brain.inputs = 1;
    cfg.brain.hidden = 2;
    cfg.brain.outputs = 7;
    cfg.food.fruit.diet_mutation_probability = 0.0;
    let m = &mut cfg.mutation;
    m.weight_probability = 0.1;
    m.weight_step = 0.2;
    m.add_connection_probability = 0.0;
    m.add_node_probability = 0.0;
    m.toggle_probability = 0.0;
    m.prune_probability = 0.0;
    m.trait_probability = 0.0;
    let mut locks = Vec::new();
    let (seeds, arms, probes): (_, &[(&str, f64)], &[usize]) = if delay_study {
        (
            6300..6332,
            &[("fixed", 0.3), ("variable", 0.3)],
            &[8, 9, 18, 19],
        )
    } else {
        (
            6200..6232,
            &[("selected", 0.3), ("neutral", 0.3), ("feed_forward", 0.0)],
            &[4, 5, 6, 19, 20, 21],
        )
    };
    for seed in seeds {
        m.add_recurrent_probability = 0.0;
        let mut source = ChaCha8Rng::seed_from_u64(seed);
        let founders: Vec<_> = (0..16)
            .map(|_| {
                Genome::random(
                    &cfg.brain,
                    m,
                    &cfg.aging,
                    &cfg.temperature,
                    &cfg.food.fruit,
                    &mut source,
                )
            })
            .collect();
        for &(arm, rate) in arms {
            let delays: &[usize] = match arm {
                "fixed" => &[5, 5, 5, 5],
                "variable" => &[3, 4, 6, 7],
                _ => &[5],
            };
            m.add_recurrent_probability = rate;
            let mut rng = source.clone();
            let mut sampling = ChaCha8Rng::seed_from_u64(seed ^ (1 << 63));
            let mut innovations = Innovations::new(&cfg.brain);
            let mut births: Vec<_> = founders
                .iter()
                .cloned()
                .map(|g| Birth::new(None, g, delays))
                .collect();
            let mut pop: Vec<usize> = (0..16).collect();
            let mut generations = Vec::new();
            for generation in 0..=24 {
                pop.sort_by(|&a, &b| {
                    let x = &births[a];
                    let y = &births[b];
                    y.correct
                        .cmp(&x.correct)
                        .then_with(|| y.margin.total_cmp(&x.margin))
                        .then(a.cmp(&b))
                });
                generations.push(pop.clone());
                if generation == 24 {
                    break;
                }
                if arm == "neutral" {
                    pop.sort();
                    pop.shuffle(&mut sampling);
                }
                pop.truncate(4);
                for _ in 0..12 {
                    let parent = pop[sim_core::math::random_index(&mut rng, 4)];
                    let genome = births[parent].genome.reproduce(
                        m,
                        &cfg.food.fruit,
                        &mut innovations,
                        &mut rng,
                    );
                    pop.push(births.len());
                    births.push(Birth::new(Some(parent), genome, delays));
                }
            }
            locks.push((seed, arm, pop[0], births[pop[0]].genome.clone()));
            save(
                &dir.join(format!("{seed}_{arm}.json")),
                &json!({"births":births,"generations":generations}),
            )?;
        }
    }
    save(&dir.join("selection.json"), &locks)?;
    let mut rows = Vec::new();
    for (seed, arm, id, g) in locks {
        let brain = g.build_brain();
        for &delay in probes {
            rows.push(json!({"seed":seed,"arm":arm,"id":id,"delay":delay,"edges":brain.recurrent_count(),"hidden":brain_probe(&brain,false,Reset::Intact,delay),"hidden_reset":brain_probe(&brain,false,Reset::Boundary,delay),"visible_reset":brain_probe(&brain,true,Reset::Boundary,delay),"visible":brain_probe(&brain,true,Reset::Intact,delay)}));
        }
    }
    save(
        &dir.join("diagnostics.json"),
        &json!({"source":env!("TERRARIUM_SOURCE_REVISION"),"dirty":env!("TERRARIUM_SOURCE_DIRTY"),"compiler":env!("TERRARIUM_COMPILER"),"target":env!("TERRARIUM_TARGET"),"core":sim_core::checkpoint::CORE_BUILD_ID,"rows":rows}),
    )
}
