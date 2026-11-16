//! The environment has its own random stream.
//!
//! Before 2026-11-16 droughts drew from the same stream as evolution, so any
//! change in organism behaviour (for example switching a feature off) shifted when
//! and where every later drought fell. A comparison of two feature arms then
//! compared two different environments. These tests pin the repaired behaviour:
//! the drought schedule depends only on its seed and the drought settings.
use sim_core::{Config, Simulation};

const DEFAULT_CONFIG: &str = include_str!("../../../configs/default.toml");

fn config() -> Config {
    let mut config = Config::parse(DEFAULT_CONFIG).unwrap();
    // Frequent short droughts so several start and end within a short test.
    config.food.droughts.spawn_rate = 0.03;
    config.food.droughts.duration = 60;
    config.food.droughts.max_active = 3;
    config.creature.initial_count = 300;
    config
}

/// (x, y, ticks_remaining) of every active drought after each of `ticks` steps.
fn schedule(config: Config, ticks: u64) -> Vec<Vec<(u64, u64, u64)>> {
    let mut sim = Simulation::new(config);
    (0..ticks)
        .map(|_| {
            sim.step();
            sim.food_grid
                .droughts
                .iter()
                .map(|d| (d.x.to_bits(), d.y.to_bits(), d.ticks_remaining))
                .collect()
        })
        .collect()
}

fn drought_count(schedule: &[Vec<(u64, u64, u64)>]) -> usize {
    schedule.iter().map(Vec::len).sum()
}

#[test]
fn drought_schedule_is_identical_across_feature_arms() {
    let base = schedule(config(), 400);
    assert!(
        drought_count(&base) > 100,
        "the test must actually exercise droughts"
    );

    // Each arm changes how many random draws the organisms make.
    let mut asexual = config();
    asexual.reproduction.sexual = false;
    let mut no_epigenetics = config();
    no_epigenetics.epigenetics.enabled = false;
    let mut no_niche = config();
    no_niche.niche_construction.enabled = false;
    let mut hungry = config();
    hungry.food.regrowth_rate *= 0.25;

    for (name, arm) in [
        ("asexual", asexual),
        ("no epigenetics", no_epigenetics),
        ("no niche construction", no_niche),
        ("less food", hungry),
    ] {
        assert_eq!(
            base,
            schedule(arm, 400),
            "drought schedule shifted in the '{name}' arm"
        );
    }
}

/// Guard against a vacuous pass above: the arms must change what the organisms
/// do, otherwise an unchanged schedule would prove nothing.
#[test]
fn organisms_do_diverge_between_those_arms() {
    let outcome = |change: fn(&mut Config)| {
        let mut c = config();
        c.creature.initial_count = 300; // dense enough for mates to be found
        change(&mut c);
        let mut sim = Simulation::new(c);
        for _ in 0..400 {
            sim.step();
        }
        let energy: f64 = sim.organisms.iter().map(|o| o.energy).sum();
        (
            sim.stats.total_births,
            sim.organisms.len(),
            energy.to_bits(),
        )
    };
    let base = outcome(|_| {});
    assert_ne!(
        base,
        outcome(|c| c.reproduction.sexual = false),
        "asexual arm"
    );
    assert_ne!(
        base,
        outcome(|c| c.epigenetics.enabled = false),
        "no-epigenetics arm"
    );
    assert_ne!(
        base,
        outcome(|c| c.food.regrowth_rate *= 0.25),
        "less-food arm"
    );
}

#[test]
fn schedule_seed_replays_one_schedule_across_world_seeds() {
    let with = |world_seed: u64, schedule_seed: Option<u64>| {
        let mut c = config();
        c.world.seed = world_seed;
        c.food.droughts.schedule_seed = schedule_seed;
        c
    };
    // Same schedule seed, different founders: identical droughts.
    assert_eq!(
        schedule(with(1, Some(77)), 300),
        schedule(with(2, Some(77)), 300)
    );
    // Different schedule seed: a different schedule.
    assert_ne!(
        schedule(with(1, Some(77)), 300),
        schedule(with(1, Some(78)), 300)
    );
    // Without a schedule seed the world seed decides it.
    assert_ne!(schedule(with(1, None), 300), schedule(with(2, None), 300));
    assert_eq!(schedule(with(1, None), 300), schedule(with(1, None), 300));
}

#[test]
fn schedule_seed_does_not_change_the_founding_population() {
    let founders = |schedule_seed: Option<u64>| {
        let mut c = config();
        c.food.droughts.schedule_seed = schedule_seed;
        let sim = Simulation::new(c);
        sim.organisms
            .iter()
            .map(|o| {
                (
                    o.pos_x.to_bits(),
                    o.pos_y.to_bits(),
                    o.genome.color_hue.to_bits(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(founders(None), founders(Some(5)));
    assert_eq!(founders(Some(5)), founders(Some(6)));
}

#[test]
fn schedule_seed_round_trips_through_toml() {
    let text = DEFAULT_CONFIG.replace("# schedule_seed = 7", "schedule_seed = 7");
    assert_ne!(text, DEFAULT_CONFIG);
    let config = Config::parse(&text).unwrap();
    assert_eq!(config.food.droughts.schedule_seed, Some(7));
    assert_eq!(
        Config::parse(DEFAULT_CONFIG)
            .unwrap()
            .food
            .droughts
            .schedule_seed,
        None
    );
}
