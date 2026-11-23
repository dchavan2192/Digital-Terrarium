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
}
impl Birth {
    fn new(parent: Option<usize>, genome: Genome) -> Self {
        let responses = brain_probe(&genome.build_brain(), false, Reset::Intact, 5);
        Self {
            parent,
            genome,
            responses,
        }
    }
}
pub fn run(output: &Path) -> RunResult<()> {
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
    for seed in 6200..6232 {
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
        for (arm, rate) in [("selected", 0.3), ("neutral", 0.3), ("feed_forward", 0.0)] {
            m.add_recurrent_probability = rate;
            let mut rng = source.clone();
            let mut sampling = ChaCha8Rng::seed_from_u64(seed ^ (1 << 63));
            let mut innovations = Innovations::new(&cfg.brain);
            let mut births: Vec<_> = founders
                .iter()
                .cloned()
                .map(|g| Birth::new(None, g))
                .collect();
            let mut pop: Vec<usize> = (0..16).collect();
            let mut generations = Vec::new();
            for generation in 0..=24 {
                pop.sort_by(|&a, &b| {
                    let x = births[a].responses;
                    let y = births[b].responses;
                    correct(y)
                        .cmp(&correct(x))
                        .then_with(|| margin(y).total_cmp(&margin(x)))
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
                    births.push(Birth::new(Some(parent), genome));
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
        for delay in [4, 5, 6, 19, 20, 21] {
            rows.push(json!({"seed":seed,"arm":arm,"id":id,"delay":delay,"edges":brain.recurrent_count(),"hidden":brain_probe(&brain,false,Reset::Intact,delay),"hidden_reset":brain_probe(&brain,false,Reset::Boundary,delay),"visible_reset":brain_probe(&brain,true,Reset::Boundary,delay),"visible":brain_probe(&brain,true,Reset::Intact,delay)}));
        }
    }
    save(
        &dir.join("diagnostics.json"),
        &json!({"source":env!("TERRARIUM_SOURCE_REVISION"),"dirty":env!("TERRARIUM_SOURCE_DIRTY"),"compiler":env!("TERRARIUM_COMPILER"),"target":env!("TERRARIUM_TARGET"),"core":sim_core::checkpoint::CORE_BUILD_ID,"rows":rows}),
    )
}
