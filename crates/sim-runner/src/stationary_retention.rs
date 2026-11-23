//! Frozen three-weight stationary retention pilot. See STATIONARY_RETENTION_PLAN.
use crate::run::{RunResult, create_session};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use serde_json::json;
use sim_core::brain::{Brain, MAX_NODES};
use std::{fs, path::Path};

#[derive(Clone, Serialize)]
struct Candidate {
    id: usize,
    parent: Option<usize>,
    weights: [f64; 3],
    responses: [f64; 2],
}

#[derive(Clone, Copy)]
pub(super) enum Reset {
    Intact,
    Boundary,
    EveryTick,
}

fn responses(weights: [f64; 3], visible: bool, reset: Reset) -> [f64; 2] {
    probe(weights, visible, reset, 5)
}
fn probe(
    [bias, cue_weight, recurrence]: [f64; 3],
    visible: bool,
    reset: Reset,
    delay: usize,
) -> [f64; 2] {
    let brain = Brain::from_neat_with_memory(
        1,
        7,
        &[],
        &[(0, 2, bias, true), (1, 2, cue_weight, true)],
        &[(2, 2, recurrence, 2)],
    );
    brain_probe(&brain, visible, reset, delay)
}

pub(super) fn brain_probe(brain: &Brain, visible: bool, reset: Reset, delay: usize) -> [f64; 2] {
    [-1.0, 1.0].map(|cue| {
        let mut prev = [0.0; MAX_NODES];
        let mut act = prev;
        let mut turn = 0.0;
        for tick in 0..=delay + 1 {
            if matches!(reset, Reset::EveryTick)
                || (tick == delay + 1 && matches!(reset, Reset::Boundary))
            {
                prev.fill(0.0);
            }
            let input = if tick == 0 || (tick == delay + 1 && visible) {
                cue
            } else {
                0.0
            };
            turn = brain.forward_with_memory(&[input], &prev, &mut act).0.turn;
            prev.copy_from_slice(&act);
        }
        turn
    })
}

pub(super) fn correct(r: [f64; 2]) -> u8 {
    u8::from(r[0] < 0.0) + u8::from(r[1] >= 0.0)
}
pub(super) fn margin(r: [f64; 2]) -> f64 {
    (-r[0]).min(r[1])
}
fn candidate(id: usize, parent: Option<usize>, weights: [f64; 3]) -> Candidate {
    Candidate {
        id,
        parent,
        weights,
        responses: responses(weights, false, Reset::Intact),
    }
}
pub(super) fn save(path: &Path, value: &impl Serialize) -> RunResult<()> {
    fs::write(path, serde_json::to_vec(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn rank(population: &mut [Candidate]) {
    population.sort_by(|a, b| {
        correct(b.responses)
            .cmp(&correct(a.responses))
            .then_with(|| margin(b.responses).total_cmp(&margin(a.responses)))
            .then(a.id.cmp(&b.id))
    });
}
pub fn run(output: &Path) -> RunResult<()> {
    let dir = create_session(output)?;
    let mut selected = Vec::new();
    for seed in 6000..6004 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut population: Vec<_> = (0..16)
            .map(|id| candidate(id, None, std::array::from_fn(|_| rng.gen_range(-2.0..2.0))))
            .collect();
        let mut history = Vec::new();
        let mut id = 16;
        for generation in 0..=16 {
            rank(&mut population);
            history.push(population.clone());
            if generation == 16 {
                break;
            }
            population.truncate(4);
            while population.len() < 16 {
                let parent = &population[sim_core::math::random_index(&mut rng, 4)];
                let child = candidate(
                    id,
                    Some(parent.id),
                    parent
                        .weights
                        .map(|w| (w + rng.gen_range(-0.3..0.3)).clamp(-4.0, 4.0)),
                );
                population.push(child);
                id += 1;
            }
        }
        save(&dir.join(format!("seed_{seed}.json")), &history)?;
        selected.push((seed, population[0].clone()));
    }
    save(&dir.join("selection.json"), &selected)?;
    let mut rows = Vec::new();
    for (seed, c) in selected {
        for visible in [false, true] {
            for (name, reset) in [
                ("intact", Reset::Intact),
                ("boundary", Reset::Boundary),
                ("every_tick", Reset::EveryTick),
            ] {
                let r = responses(c.weights, visible, reset);
                rows.push(json!({"seed":seed,"id":c.id,"visible":visible,"reset":name,"responses":r,"correct":correct(r)}));
            }
        }
    }
    save(
        &dir.join("diagnostics.json"),
        &json!({"schema":1,"cue_order":[-1,1],"compiler":env!("TERRARIUM_COMPILER"),"target":env!("TERRARIUM_TARGET"),"source":env!("TERRARIUM_SOURCE_REVISION"),"source_dirty":env!("TERRARIUM_SOURCE_DIRTY"),"core":sim_core::checkpoint::CORE_BUILD_ID,"rows":rows}),
    )
}

pub fn paired(output: &Path) -> RunResult<()> {
    use rand::seq::SliceRandom;
    let dir = create_session(output)?;
    let mut winners = Vec::new();
    for seed in 6100..6132 {
        let mut source = ChaCha8Rng::seed_from_u64(seed);
        let founders: Vec<_> = (0..16)
            .map(|id| {
                candidate(
                    id,
                    None,
                    std::array::from_fn(|_| source.gen_range(-2.0..2.0)),
                )
            })
            .collect();
        for arm in ["selected", "neutral"] {
            let mut rng = source.clone();
            let mut sampling = ChaCha8Rng::seed_from_u64(seed ^ (1 << 63));
            let mut pop = founders.clone();
            let mut history = Vec::new();
            for generation in 0..=16 {
                rank(&mut pop);
                history.push(pop.clone());
                if generation == 16 {
                    break;
                }
                if arm == "neutral" {
                    pop.sort_by_key(|c| c.id);
                    pop.shuffle(&mut sampling);
                }
                pop.truncate(4);
                for index in 0..12 {
                    let parent = &pop[sim_core::math::random_index(&mut rng, 4)];
                    let child = candidate(
                        16 + generation * 12 + index,
                        Some(parent.id),
                        parent
                            .weights
                            .map(|w| (w + rng.gen_range(-0.3..0.3)).clamp(-4.0, 4.0)),
                    );
                    pop.push(child);
                }
            }
            save(&dir.join(format!("{seed}_{arm}.json")), &history)?;
            winners.push((seed, arm, pop[0].clone()));
        }
    }
    save(&dir.join("selection.json"), &winners)?;
    let mut rows = Vec::new();
    for (seed, arm, c) in winners {
        for delay in [1, 5, 10, 20] {
            for visible in [false, true] {
                for (name, reset) in [
                    ("intact", Reset::Intact),
                    ("boundary", Reset::Boundary),
                    ("every_tick", Reset::EveryTick),
                ] {
                    let r = probe(c.weights, visible, reset, delay);
                    rows.push(json!({"seed":seed,"arm":arm,"id":c.id,"delay":delay,"visible":visible,"reset":name,"responses":r,"correct":correct(r)}));
                }
            }
        }
    }
    save(
        &dir.join("diagnostics.json"),
        &json!({"source":env!("TERRARIUM_SOURCE_REVISION"),"dirty":env!("TERRARIUM_SOURCE_DIRTY"),"core":sim_core::checkpoint::CORE_BUILD_ID,"compiler":env!("TERRARIUM_COMPILER"),"target":env!("TERRARIUM_TARGET"),"rows":rows}),
    )
}
