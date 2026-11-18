//! Memory: recurrent connections carry a node's previous-tick activation.
//!
//! Forced fixtures check the mechanism's causal pathway directly, and separate
//! tests check that with the mutation probability at 0 nothing of it exists
//! (the pattern used in `mechanism_effects.rs`). That a memory-off run follows
//! exactly the trajectory of the build before memory existed was checked against
//! saved reference states when the mechanism was added (docs/research/MEMORY_DESIGN.md).
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sim_core::brain::MAX_NODES;
use sim_core::genome::{ConnectionGene, Genome, Innovations, NodeId};
use sim_core::{Config, Simulation};

const DEFAULT_CONFIG: &str = include_str!("../../../configs/default.toml");
const OUT_TURN: usize = 0;

fn config() -> Config {
    Config::parse(DEFAULT_CONFIG).unwrap()
}

/// A genome with no hidden nodes and no connections, to be wired by hand.
fn blank() -> Genome {
    let config = config();
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let mut g = Genome::random(
        &config.brain,
        &config.mutation,
        &config.aging,
        &config.temperature,
        &config.food.fruit,
        &mut rng,
    );
    g.connections.clear();
    g.hidden_nodes.clear();
    g
}

fn gene(from: NodeId, to: NodeId, weight: f64, recurrent: bool) -> ConnectionGene {
    ConnectionGene {
        from,
        to,
        weight,
        enabled: true,
        methylation: 0.0,
        recurrent,
    }
}

fn wire(g: &mut Genome, genes: &[ConnectionGene]) {
    g.connections.extend_from_slice(genes);
    g.connections.sort_by_key(|c| c.key());
}

fn output_node(g: &Genome, out: usize) -> NodeId {
    (1 + g.num_inputs + out) as NodeId
}

/// Run `ticks` forward passes, feeding each tick's activations back as memory.
/// Returns the turn output of every tick.
fn run(g: &Genome, inputs_per_tick: &[Vec<f64>]) -> Vec<f64> {
    let brain = g.build_brain();
    let mut prev: Vec<f64> = Vec::new();
    let mut act = [0.0f64; MAX_NODES];
    inputs_per_tick
        .iter()
        .map(|inputs| {
            let (out, _) = brain.forward_with_memory(inputs, &prev, &mut act);
            prev = act[..brain.slot_count()].to_vec();
            out.turn
        })
        .collect()
}

#[test]
fn a_self_loop_carries_state_from_tick_to_tick() {
    let mut with_loop = blank();
    let turn = output_node(&with_loop, OUT_TURN);
    wire(
        &mut with_loop,
        &[gene(0, turn, 0.5, false), gene(turn, turn, 2.0, true)],
    );
    let mut without = blank();
    wire(&mut without, &[gene(0, turn, 0.5, false)]);

    let same_inputs = vec![vec![0.0; with_loop.num_inputs]; 4];
    let looped = run(&with_loop, &same_inputs);
    let plain = run(&without, &same_inputs);

    // Without memory, identical inputs give identical outputs on every tick.
    assert!(plain.windows(2).all(|w| w[0] == w[1]), "{plain:?}");
    // The first tick has no previous state, so both brains agree on it.
    assert_eq!(looped[0], plain[0]);
    // After that the loop feeds the output back into itself and it climbs.
    assert!(looped[1] > looped[0] + 0.1, "{looped:?}");
    assert!(looped[2] > looped[1], "{looped:?}");
    assert!(with_loop.build_brain().has_memory());
    assert!(!without.build_brain().has_memory());
}

#[test]
fn a_recurrent_edge_from_an_input_is_a_one_tick_delay() {
    let mut g = blank();
    let turn = output_node(&g, OUT_TURN);
    let sensor: NodeId = 3; // any input node
    wire(&mut g, &[gene(sensor, turn, 3.0, true)]);
    let n = g.num_inputs;
    let pulse = |on: bool| {
        let mut v = vec![0.0; n];
        v[sensor as usize - 1] = if on { 1.0 } else { 0.0 };
        v
    };
    // The sensor fires on tick 1 only.
    let turns = run(&g, &[pulse(false), pulse(true), pulse(false), pulse(false)]);
    assert_eq!(turns[0], 0.0);
    assert_eq!(
        turns[1], 0.0,
        "the pulse is not visible on the tick it happens"
    );
    assert!(turns[2] > 0.9, "it arrives one tick later: {turns:?}");
    assert_eq!(turns[3], 0.0, "and is gone again after that");
}

#[test]
fn feed_forward_and_recurrent_genes_on_one_pair_are_distinct_and_inherited() {
    let config = config();
    let mut g = blank();
    let turn = output_node(&g, OUT_TURN);
    wire(
        &mut g,
        &[
            gene(2, turn, 0.4, false),
            gene(2, turn, -0.7, true),
            gene(turn, turn, 0.3, true),
        ],
    );
    assert_eq!(g.connections.len(), 3);
    assert!(g.connections.windows(2).all(|w| w[0].key() < w[1].key()));
    let brain = g.build_brain();
    assert_eq!((brain.connections.len(), brain.recurrent_count()), (1, 2));
    assert_eq!(
        brain.connection_count(),
        3,
        "upkeep counts recurrent edges too"
    );

    // Crossing a genome with itself, with mutation off, must return the same genes.
    let mut mutation = config.mutation.clone();
    mutation.add_connection_probability = 0.0;
    mutation.add_node_probability = 0.0;
    mutation.toggle_probability = 0.0;
    mutation.prune_probability = 0.0;
    mutation.trait_probability = 0.0;
    g.mutation_rate = 0.0;
    let mut innovations = Innovations::new(&config.brain);
    let mut rng = ChaCha8Rng::seed_from_u64(4);
    let child = Genome::crossover(
        &g,
        &g,
        true,
        &mutation,
        &config.food.fruit,
        &mut innovations,
        &mut rng,
    );
    let keys = |x: &Genome| x.connections.iter().map(|c| c.key()).collect::<Vec<_>>();
    assert_eq!(keys(&child), keys(&g));

    // A disabled recurrent gene is not part of the brain.
    let mut off = g.clone();
    for c in &mut off.connections {
        if c.recurrent {
            c.enabled = false;
        }
    }
    assert!(!off.build_brain().has_memory());
}

#[test]
fn the_mutation_adds_valid_recurrent_genes_and_never_splits_them() {
    let config = config();
    let mut rng = ChaCha8Rng::seed_from_u64(7);
    let mut g = Genome::random(
        &config.brain,
        &config.mutation,
        &config.aging,
        &config.temperature,
        &config.food.fruit,
        &mut rng,
    );
    let mut mutation = config.mutation.clone();
    mutation.add_recurrent_probability = 1.0;
    mutation.add_node_probability = 1.0; // add-node must only split feed-forward genes
    mutation.prune_probability = 0.0;
    let mut innovations = Innovations::new(&config.brain);
    for generation in 1..=20 {
        g = g.reproduce(&mutation, &config.food.fruit, &mut innovations, &mut rng);
        let recurrent: Vec<_> = g.connections.iter().filter(|c| c.recurrent).collect();
        assert_eq!(
            recurrent.len(),
            generation,
            "one new recurrent gene per birth"
        );
        for c in &recurrent {
            assert_ne!(c.from, 0, "the bias is not a source");
            assert!(
                c.to as usize > g.num_inputs,
                "targets are hidden or output nodes"
            );
            assert!(g.slot_of(c.from).is_some() && g.slot_of(c.to).is_some());
        }
        assert!(g.connections.windows(2).all(|w| w[0].key() < w[1].key()));
        // Every hidden node sits strictly between an input and an output in depth,
        // which would not hold if a recurrent edge had been split.
        assert!(
            g.hidden_nodes
                .iter()
                .all(|n| n.depth > 0.0 && n.depth < 1.0)
        );
    }
    assert!(g.build_brain().has_memory());
}

#[test]
fn with_the_probability_at_zero_memory_does_not_exist() {
    let config = config();
    assert_eq!(
        config.mutation.add_recurrent_probability, 0.0,
        "off by default"
    );
    let mut sim = Simulation::new(config);
    for _ in 0..400 {
        sim.step();
        assert_eq!(sim.stats.mean_recurrent_connections, 0.0);
        assert_eq!(sim.stats.recurrent_brain_fraction, 0.0);
    }
    assert!(
        sim.stats.total_births > 50,
        "the run must include many births"
    );
    for o in &sim.organisms {
        assert!(o.memory.is_empty());
        assert!(!o.brain.has_memory());
        assert!(o.genome.connections.iter().all(|c| !c.recurrent));
    }
}

fn memory_config() -> Config {
    let mut config = config();
    config.mutation.add_recurrent_probability = 0.5;
    config.food.regrowth_rate = 0.006; // many births, so memory spreads quickly
    config
}

#[test]
fn memory_spreads_is_stored_per_creature_and_keeps_the_energy_books_balanced() {
    let mut sim = Simulation::new(memory_config());
    let mut worst_residual = 0.0f64;
    for _ in 0..400 {
        sim.step();
        worst_residual = worst_residual.max(sim.stats.energy_balance_residual.abs());
    }
    assert!(
        sim.stats.recurrent_brain_fraction > 0.3,
        "{}",
        sim.stats.recurrent_brain_fraction
    );
    assert!(sim.stats.mean_recurrent_connections > 0.3);
    assert!(
        worst_residual < 1e-6,
        "energy ledger residual {worst_residual}"
    );
    let mut remembered = 0;
    for o in &sim.organisms {
        if o.brain.has_memory() && o.age > 0 {
            assert_eq!(o.memory.len(), o.brain.slot_count());
            assert_eq!(o.memory[0], 1.0, "slot 0 is the bias node");
            assert!(
                o.memory
                    .iter()
                    .all(|v| v.is_finite() && v.abs() <= 1.0 + 1e-12)
            );
            remembered += 1;
        } else if !o.brain.has_memory() {
            assert!(
                o.memory.is_empty(),
                "brains without recurrence keep no state"
            );
        }
    }
    assert!(remembered > 10);
}

#[test]
fn a_checkpoint_in_mid_run_continues_exactly_with_memory() {
    let mut straight = Simulation::new(memory_config());
    for _ in 0..300 {
        straight.step();
    }

    let mut first = Simulation::new(memory_config());
    for _ in 0..150 {
        first.step();
    }
    assert!(
        first.organisms.iter().any(|o| !o.memory.is_empty()),
        "the checkpoint must contain remembered state, or the test proves nothing"
    );
    let bytes = first
        .to_checkpoint_bytes()
        .expect("state with memory must validate");
    let mut resumed = Simulation::from_checkpoint_bytes(&bytes).unwrap();
    for _ in 0..150 {
        resumed.step();
    }
    assert_eq!(
        resumed.state_fingerprint().unwrap(),
        straight.state_fingerprint().unwrap()
    );
}

#[test]
fn losing_the_remembered_state_changes_what_happens_next() {
    // Guard for the test above: if memory were ignored, wiping it would not matter.
    let mut a = Simulation::new(memory_config());
    for _ in 0..150 {
        a.step();
    }
    let bytes = a.to_checkpoint_bytes().unwrap();
    let mut wiped = Simulation::from_checkpoint_bytes(&bytes).unwrap();
    for o in &mut wiped.organisms {
        o.memory.clear();
    }
    for _ in 0..50 {
        a.step();
        wiped.step();
    }
    assert_ne!(
        a.state_fingerprint().unwrap(),
        wiped.state_fingerprint().unwrap()
    );
}
