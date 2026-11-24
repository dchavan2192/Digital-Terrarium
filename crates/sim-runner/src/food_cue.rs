//! Local controlled patch assay. Every energetic transaction and birth is native.
use crate::run::{RunResult, create_session};
use crate::stationary_retention::save;
use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use serde_json::json;
use sim_core::{
    Config, Simulation,
    genome::{ConnectionGene, Genome},
};
use std::{f64::consts::FRAC_PI_2, path::Path};

const FOOD_ROW_Y: f64 = 40.;
const DEFAULT_DISTANCE: f64 = 0.25;
const STEP_SPEED: f64 = 2.0;
const STEP_TURN: f64 = 0.6;

#[derive(Clone, Copy, Serialize, PartialEq, Debug)]
struct Pulse {
    delay: u32,
    food: i8,
    shuffled: i8,
}

fn schedule(seed: u64) -> Vec<Pulse> {
    let mut events = Vec::new();
    for delay in [2, 5, 8, 11] {
        for food in [-1, 1] {
            for shuffled in [-1, 1] {
                events.push(Pulse {
                    delay,
                    food,
                    shuffled,
                });
            }
        }
    }
    events.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));
    events
}

fn config() -> RunResult<Config> {
    let mut c =
        Config::parse(include_str!("../../../configs/default.toml")).map_err(|e| e.to_string())?;
    c.world.width = 100.0;
    c.world.height = 100.0;
    c.world.seed = 6400;
    c.creature.initial_count = 1;
    c.creature.hard_cap = 2;
    c.brain.hidden = 1; // The native config requires a hidden node; this control leaves it disconnected.
    c.creature.max_speed = 0.;
    c.creature.max_turn = 0.;
    c.creature.feeding_speed_factor = 1.;
    c.creature.vision.range = 20.;
    c.energy.starting = 45.;
    c.energy.base_metabolism = 0.05;
    c.food.cell_size = 10.;
    c.food.max_per_cell = 1.;
    c.food.energy_per_food = 15.;
    c.food.regrowth_rate = 0.;
    c.food.droughts.spawn_rate = 0.;
    c.food.seasons.amplitude = 0.;
    c.food.biomes.clear();
    c.food.fruit.enabled = false;
    c.temperature.enabled = false;
    c.elevation.enabled = false;
    c.pheromones.enabled = false;
    c.daynight.enabled = false;
    c.epigenetics.enabled = false;
    c.activity_tracking.enabled = false;
    c.niche_construction.enabled = false;
    c.aging.extrinsic_mortality = 0.;
    c.aging.initial_senescence = 0.;
    c.reproduction.sexual = false;
    c.reproduction.threshold = 60.;
    c.reproduction.parent_cost = 30.;
    c.reproduction.child_energy = 30.;
    c.validate().map_err(|e| e.to_string())?;
    Ok(c)
}

fn gene(from: u32, to: u32, weight: f64, recurrent: bool) -> ConnectionGene {
    ConnectionGene {
        from,
        to,
        weight,
        recurrent,
        enabled: true,
        methylation: 0.,
    }
}

#[derive(Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Policy {
    Recurrent,
    BoundaryReset,
    VisibleReset,
    FeedForward,
}

impl Policy {
    fn name(self) -> &'static str {
        match self {
            Self::Recurrent => "recurrent",
            Self::BoundaryReset => "boundary_reset",
            Self::VisibleReset => "visible_reset",
            Self::FeedForward => "feed_forward",
        }
    }
}

fn setup(c: &Config, policy: Policy, donor: Option<&Genome>) -> RunResult<Simulation> {
    let mut sim = Simulation::new(c.clone());
    sim.food_grid.cells.fill(0.);
    sim.food_grid.fruit_cells.fill(0.);
    let org = &mut sim.organisms[0];
    if let Some(donor) = donor {
        org.genome = donor.clone();
        org.brain = donor.build_brain();
        return Ok(sim);
    }
    let g = &mut org.genome;
    let turn = 1 + g.num_inputs as u32;
    let signed_cue = (2 * c.creature.vision.ray_count + 3) as u32;
    g.body_size = 1.;
    g.connections = vec![
        gene(0, turn, 0.15, false),
        gene(signed_cue, turn, 2., false),
        gene(0, turn + 1, 2., false),
        gene(0, turn + 2, 2., false),
    ];
    if policy != Policy::FeedForward {
        g.connections.push(gene(turn, turn, 2., true));
    }
    g.connections.sort_by_key(|edge| edge.key());
    g.validate().map_err(|e| e.to_string())?;
    org.brain = g.build_brain();
    Ok(sim)
}

#[derive(Default)]
struct Ledger {
    basal: f64,
    neural: f64,
    movement: f64,
    algae: f64,
    newborns: Vec<Newborn>,
}

#[derive(Serialize)]
struct Newborn {
    id: u64,
    parent_id: u64,
    energy: f64,
    /// Completed simulation steps at observation. The native clock is zero-based.
    tick: u64,
    native_birth_tick: u64,
}

#[derive(Serialize)]
struct Event {
    index: usize,
    founder_id: u64,
    start_tick: u64,
    delay: u32,
    cue: i8,
    choice: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    choice_turn: Option<f64>,
    choice_x: f64,
    choice_y: f64,
    food: i8,
    pulse_tick: u64,
    pulse_cell: [usize; 2],
    food_units: f64,
    food_remaining: f64,
    algae_energy: f64,
    energy_before: f64,
    energy_after: f64,
    neural_cost: f64,
    basal_cost: f64,
    movement_cost: f64,
    newborns: Vec<Newborn>,
}

#[derive(Serialize)]
struct Lifetime {
    policy: Policy,
    arm: &'static str,
    hits: usize,
    offspring: usize,
    events: Vec<Event>,
    final_tick: u64,
}

fn tick(sim: &mut Simulation, cue: i8, ledger: &mut Ledger) -> RunResult<()> {
    let founder = sim.organisms.first().ok_or("founder absent")?.id;
    sim.step_with_external_cue(cue)?;
    let s = &sim.stats;
    if s.deaths != 0 || s.cap_blocked_births != 0 || s.energy_balance_residual.abs() > 1e-8 {
        return Err(format!(
            "invalid validation world at tick {}: deaths {}, cap-blocked {}, residual {}",
            sim.tick, s.deaths, s.cap_blocked_births, s.energy_balance_residual
        ));
    }
    ledger.basal += s.energy_basal_cost;
    ledger.neural += s.energy_brain_structure_cost + s.energy_brain_activity_cost;
    ledger.movement += s.energy_movement_cost;
    ledger.algae += s.energy_food_input;
    for child in sim.organisms.iter().skip(1) {
        if child.parent_id != Some(founder)
            || child.birth_tick + 1 != sim.tick
            || child.generation != 1
        {
            return Err("unexpected child identity".into());
        }
        ledger.newborns.push(Newborn {
            id: child.id,
            parent_id: founder,
            energy: child.energy,
            tick: sim.tick,
            native_birth_tick: child.birth_tick,
        });
    }
    if s.births != sim.organisms.len() - 1 {
        return Err("birth count disagrees with direct newborns".into());
    }
    sim.organisms.truncate(1);
    Ok(())
}

fn lifetime(
    c: &Config,
    plan: &[Pulse],
    policy: Policy,
    predictive: bool,
    donor: Option<&Genome>,
    start: [f64; 2],
) -> RunResult<Lifetime> {
    let mut sim = setup(c, policy, donor)?;
    let founder_id = sim.organisms[0].id;
    let mut events = Vec::new();
    let mut offspring = 0;
    let mut hits = 0;
    for (index, event) in plan.iter().enumerate() {
        let cue = if predictive {
            event.food
        } else {
            event.shuffled
        };
        let start_tick = sim.tick;
        let org = &mut sim.organisms[0];
        org.pos_x = start[0];
        org.pos_y = start[1];
        org.heading = FRAC_PI_2;
        org.memory.clear();
        let energy_before = org.energy;
        let mut ledger = Ledger::default();
        tick(&mut sim, cue, &mut ledger)?;
        for _ in 0..event.delay {
            tick(&mut sim, 0, &mut ledger)?;
        }
        let org = &mut sim.organisms[0];
        if [org.pos_x, org.pos_y] != start
            || org.heading != FRAC_PI_2
            || sim.food_grid.cells.iter().any(|&x| x != 0.)
        {
            return Err("pose or food leaked during hold".into());
        }
        if matches!(policy, Policy::BoundaryReset | Policy::VisibleReset) {
            org.memory.clear();
        }
        sim.config.creature.max_speed = STEP_SPEED;
        sim.config.creature.max_turn = STEP_TURN;
        tick(
            &mut sim,
            if policy == Policy::VisibleReset {
                cue
            } else {
                0
            },
            &mut ledger,
        )?;
        sim.config.creature.max_speed = 0.;
        sim.config.creature.max_turn = 0.;
        let org = &sim.organisms[0];
        let (x, y) = (org.pos_x, org.pos_y);
        let choice_turn = donor.map(|_| (org.heading - FRAC_PI_2) / STEP_TURN);
        let col = (x / sim.food_grid.cell_size) as usize;
        let row = (y / sim.food_grid.cell_size) as usize;
        let choice = match (col, row) {
            (3, 4) => 1,
            (4, 4) => -1,
            _ => 0,
        };
        let pulse_col = if event.food == 1 { 3 } else { 4 };
        let cell = 4 * sim.food_grid.cols + pulse_col;
        if ledger.algae != 0. || !ledger.newborns.is_empty() {
            return Err("unexpected harvest or birth before food pulse".into());
        }
        sim.food_grid.cells[cell] = 1.;
        tick(&mut sim, 0, &mut ledger)?;
        let remaining = sim.food_grid.cells[cell];
        let founder = &sim.organisms[0];
        if (x, y) != (founder.pos_x, founder.pos_y)
            || !(0. ..=1.).contains(&remaining)
            || (ledger.algae - (1. - remaining) * c.food.energy_per_food).abs() > 1e-8
            || (choice != event.food && ledger.algae != 0.)
            || (donor.is_none() && choice == event.food && remaining != 0.)
        {
            return Err("native feeding or meal pose disagrees with committed cell".into());
        }
        if (energy_before + ledger.algae
            - ledger.basal
            - ledger.movement
            - ledger.neural
            - ledger.newborns.len() as f64 * c.reproduction.parent_cost
            - founder.energy)
            .abs()
            > 1e-8
        {
            return Err("lifetime founder ledger disagrees with native reserve".into());
        }
        hits += usize::from(choice == event.food);
        offspring += ledger.newborns.len();
        events.push(Event {
            index,
            founder_id,
            start_tick,
            delay: event.delay,
            cue,
            choice,
            choice_turn,
            choice_x: x,
            choice_y: y,
            food: event.food,
            pulse_tick: sim.tick,
            pulse_cell: [pulse_col, 4],
            food_units: 1.,
            food_remaining: remaining,
            algae_energy: ledger.algae,
            energy_before,
            energy_after: founder.energy,
            neural_cost: ledger.neural,
            basal_cost: ledger.basal,
            movement_cost: ledger.movement,
            newborns: ledger.newborns,
        });
        sim.food_grid.cells[cell] = 0.;
    }
    Ok(Lifetime {
        policy,
        arm: if predictive { "predictive" } else { "shuffled" },
        hits,
        offspring,
        events,
        final_tick: sim.tick,
    })
}

pub fn run(seed: u64, output: &Path) -> RunResult<()> {
    experiment(seed, output, None, DEFAULT_DISTANCE)
}

pub fn run_transfer(seed: u64, path: &Path, output: &Path, distance: f64) -> RunResult<()> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|e| format!("open donor genome: {e}"))?;
    // The native controller is bounded by MAX_NODES. Reject accidentally supplied study archives.
    let mut bytes = Vec::new();
    let limit = 4 * 1024 * 1024;
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("donor genome exceeds 4 MiB".into());
    }
    let donor: Genome =
        serde_json::from_slice(&bytes).map_err(|e| format!("read donor genome: {e}"))?;
    experiment(seed, output, Some(&donor), distance)
}

fn experiment(seed: u64, output: &Path, donor: Option<&Genome>, distance: f64) -> RunResult<()> {
    let start = [40., FOOD_ROW_Y - distance];
    if !(0. ..FOOD_ROW_Y).contains(&start[1]) {
        return Err(
            "distance must place the founder south of the food row and inside the world".into(),
        );
    }
    let mut c = config()?;
    let transferred = if let Some(original) = donor {
        let genome = crate::food_cue_transfer::transfer(original, &c)?;
        c.brain.hidden = genome.hidden_nodes.len().max(1);
        c.validate().map_err(|e| e.to_string())?;
        crate::food_cue_transfer::verify(original, &genome, &c)?;
        Some(genome)
    } else {
        None
    };
    let plan = schedule(seed);
    let dir = create_session(output)?;
    let mut runs = Vec::new();
    let mut manifest = json!({"task":"food_cue_payoff","schema":1,"config":c,"schedule_seed":seed,
        "start":start,"start_heading":FRAC_PI_2,"choice_speed_limit":STEP_SPEED,"choice_turn_limit":STEP_TURN,
        "positive_patch":[3,4],"negative_patch":[4,4],"sign_convention":"+1 means left",
        "phases":["one cue at fixed pose","blank delay at fixed pose","one movement tick on empty grid","one feeding tick with fixed pose"],
        "reference_weights":{"bias_to_turn":0.15,"cue_to_turn":2.0,"turn_self_loop":2.0,"bias_to_eat":2.0,"bias_to_speed":2.0},
        "source":env!("TERRARIUM_SOURCE_REVISION"),"dirty":env!("TERRARIUM_SOURCE_DIRTY"),"core":sim_core::checkpoint::CORE_BUILD_ID,
        "compiler":env!("TERRARIUM_COMPILER"),"target":env!("TERRARIUM_TARGET")});
    if let Some(original) = donor {
        manifest["task"] = json!("food_cue_transfer");
        manifest["donor"] = json!(original);
        manifest["transferred"] = json!(&transferred);
        manifest
            .as_object_mut()
            .unwrap()
            .remove("reference_weights");
    }
    save(&dir.join("manifest.json"), &manifest)?;
    save(&dir.join("schedule.json"), &plan)?;
    for policy in [
        Policy::Recurrent,
        Policy::BoundaryReset,
        Policy::VisibleReset,
        Policy::FeedForward,
    ] {
        if transferred.is_some() && policy == Policy::FeedForward {
            continue;
        }
        for predictive in [true, false] {
            let run = lifetime(&c, &plan, policy, predictive, transferred.as_ref(), start)?;
            println!(
                "{} {}: {}/{} correct landings, {} direct births",
                run.policy.name(),
                run.arm,
                run.hits,
                run.events.len(),
                run.offspring
            );
            runs.push(run);
        }
    }
    save(&dir.join("result.json"), &json!({"runs":runs}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_external_channel_preserves_the_native_solo_trajectory() {
        let c = config().unwrap();
        let mut ordinary = setup(&c, Policy::Recurrent, None).unwrap();
        let mut external = setup(&c, Policy::Recurrent, None).unwrap();
        let initial = external.state_fingerprint().unwrap();
        assert!(external.step_with_external_cue(2).is_err());
        assert_eq!(initial, external.state_fingerprint().unwrap());
        for _ in 0..10 {
            ordinary.step();
            external.step_with_external_cue(0).unwrap();
        }
        assert_eq!(
            ordinary.state_fingerprint().unwrap(),
            external.state_fingerprint().unwrap()
        );
    }

    #[test]
    fn schedule_is_seeded_and_independent_within_each_delay() {
        let plan = schedule(6400);
        assert_eq!(plan, schedule(6400));
        assert_ne!(plan, schedule(6401));
        for delay in [2, 5, 8, 11] {
            for food in [-1, 1] {
                for shuffled in [-1, 1] {
                    assert_eq!(
                        plan.iter()
                            .filter(|&&e| e
                                == Pulse {
                                    delay,
                                    food,
                                    shuffled
                                })
                            .count(),
                        1
                    );
                }
            }
        }
    }
}
