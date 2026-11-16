//! Historical node identity for NEAT crossover, and the node limit.
//!
//! Before 2026-11-16 each genome numbered its own hidden nodes from a private
//! counter that only went up. Two consequences:
//! - Two lineages could give the same id to unrelated neurons (and different ids
//!   to the same innovation), so crossover matched genes that were not homologous.
//!   NEAT requires alignment by historical origin (Stanley & Miikkulainen 2002).
//! - Ids doubled as positions in the 64-slot activation buffer, so a lineage could
//!   add at most 64 − 35 = 29 neurons in its whole history, even if earlier ones
//!   had long been pruned away.
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sim_core::brain::MAX_NODES;
use sim_core::genome::{Genome, Innovations};
use sim_core::{Config, Simulation};

const DEFAULT_CONFIG: &str = include_str!("../../../configs/default.toml");

fn config() -> Config {
    Config::parse(DEFAULT_CONFIG).unwrap()
}

/// A founder genome and the registry of its population. Mutation is switched
/// off, so `reproduce` and `crossover` copy without random structural change.
fn founder(seed: u64) -> (Genome, Innovations, Config) {
    let mut config = config();
    config.mutation.add_connection_probability = 0.0;
    config.mutation.add_node_probability = 0.0;
    config.mutation.toggle_probability = 0.0;
    config.mutation.prune_probability = 0.0;
    config.mutation.trait_probability = 0.0;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut genome = Genome::random(
        &config.brain,
        &config.mutation,
        &config.aging,
        &config.temperature,
        &config.food.fruit,
        &mut rng,
    );
    genome.mutation_rate = 0.0; // no weight perturbation either
    let innovations = Innovations::new(&config.brain);
    (genome, innovations, config)
}

fn hidden_ids(g: &Genome) -> Vec<u32> {
    let mut ids: Vec<u32> = g.hidden_nodes.iter().map(|n| n.id).collect();
    ids.sort_unstable();
    ids
}

fn outputs(g: &Genome) -> [f64; 7] {
    let brain = g.build_brain();
    let inputs: Vec<f64> = (0..g.num_inputs)
        .map(|i| ((i as f64) * 0.37).sin())
        .collect();
    let (o, _) = brain.forward(&inputs);
    [
        o.turn,
        o.speed,
        o.eat,
        o.attack,
        o.signal,
        o.share,
        o.construct,
    ]
}

#[test]
fn the_same_split_gets_the_same_node_in_independent_lineages() {
    let (base, mut innovations, _) = founder(1);
    let mut a = base.clone();
    let mut b = base.clone();
    let id_a = a.split_connection(10, &mut innovations).unwrap();
    // Lineage b makes an unrelated innovation first, then the same split as a.
    let other = b.split_connection(50, &mut innovations).unwrap();
    let id_b = b.split_connection(10, &mut innovations).unwrap();
    assert_eq!(id_a, id_b, "one innovation, one identity");
    assert_ne!(id_a, other, "different splits are different innovations");
    assert_eq!(innovations.recorded_splits(), 2);
}

#[test]
fn splitting_again_what_a_genome_already_has_issues_a_fresh_node() {
    let (mut g, mut innovations, _) = founder(2);
    let first = g.split_connection(10, &mut innovations).unwrap();
    // Gene 10 is disabled but still present; splitting it again must not reuse
    // the node the genome already holds.
    let second = g.split_connection(10, &mut innovations).unwrap();
    assert_ne!(first, second);
    let ids = hidden_ids(&g);
    assert_eq!(ids.len(), 10, "8 founders + 2 new nodes, no duplicates");
    assert!(ids.windows(2).all(|w| w[0] != w[1]));
}

#[test]
fn crossover_recognises_homologous_innovations() {
    let (base, mut innovations, config) = founder(3);
    let mut rng = ChaCha8Rng::seed_from_u64(30);
    // Both parents independently split gene 10; only b also splits gene 50.
    let mut a = base.clone();
    let shared = a.split_connection(10, &mut innovations).unwrap();
    let mut b = base.clone();
    let private_b = b.split_connection(50, &mut innovations).unwrap();
    assert_eq!(b.split_connection(10, &mut innovations).unwrap(), shared);
    // split_connection appends genes; reproduce() restores the sorted order that
    // crossover's merge-join needs.
    let a = a.reproduce(&config.mutation, &mut innovations, &mut rng);
    let b = b.reproduce(&config.mutation, &mut innovations, &mut rng);

    // a is fitter: the child takes a's wiring, and the shared node appears once.
    let child = Genome::crossover(&a, &b, true, &config.mutation, &mut innovations, &mut rng);
    assert_eq!(hidden_ids(&child), hidden_ids(&a));
    assert!(hidden_ids(&child).contains(&shared));
    assert!(!hidden_ids(&child).contains(&private_b));
    assert_eq!(child.connections.len(), a.connections.len());
    let into_shared = child.connections.iter().filter(|c| c.to == shared).count();
    let out_of_shared = child
        .connections
        .iter()
        .filter(|c| c.from == shared)
        .count();
    assert_eq!(
        (into_shared, out_of_shared),
        (1, 1),
        "the shared split is inherited once"
    );

    // b is fitter: the child takes b's wiring, including b's private innovation.
    let child = Genome::crossover(&a, &b, false, &config.mutation, &mut innovations, &mut rng);
    assert_eq!(hidden_ids(&child), hidden_ids(&b));
    // Every gene's endpoints exist, so the child builds a working brain.
    assert!(
        child
            .connections
            .iter()
            .all(|c| child.slot_of(c.from).is_some() && child.slot_of(c.to).is_some())
    );
    assert!(outputs(&child).iter().all(|v| v.is_finite()));
}

#[test]
fn a_lineage_can_keep_innovating_past_the_old_lifetime_cap() {
    let (mut g, mut innovations, _) = founder(4);
    let founders = g.hidden_nodes.len();
    let mut issued = std::collections::BTreeSet::new();
    for round in 0..200 {
        // Split a founder gene, then lose the new neuron again (as pruning would).
        let gene = round % 100;
        g.connections[gene].enabled = true;
        let id = g
            .split_connection(gene, &mut innovations)
            .unwrap_or_else(|| panic!("round {round}: adding a neuron was refused"));
        issued.insert(id);
        g.connections.retain(|c| c.from != id && c.to != id);
        g.prune_orphan_nodes();
        assert_eq!(g.hidden_nodes.len(), founders);
    }
    // 100 distinct splits were made twice each: the second time reuses the id.
    assert_eq!(issued.len(), 100);
    assert!(
        *issued.iter().max().unwrap() as usize > MAX_NODES,
        "node ids are no longer limited by the activation buffer"
    );
}

#[test]
fn the_limit_is_on_neurons_held_at_once_and_big_ids_still_run() {
    let (mut g, mut innovations, _) = founder(5);
    // Another lineage first makes 30 innovations on OTHER connections, so the ids
    // issued to this genome's own splits lie beyond the activation buffer size.
    // (Splitting the same connections would, correctly, reuse the same ids.)
    let (mut spare, _, _) = founder(6);
    for gene in 100..130 {
        spare.split_connection(gene, &mut innovations);
    }
    let mut gene = 0;
    while g.split_connection(gene, &mut innovations).is_some() {
        gene += 1;
    }
    assert_eq!(g.hidden_nodes.len(), g.max_hidden_nodes());
    assert_eq!(
        g.max_hidden_nodes(),
        MAX_NODES - (1 + g.num_inputs + g.num_outputs)
    );
    assert!(g.hidden_nodes.iter().any(|n| n.id as usize > MAX_NODES));
    // Slots are dense and distinct even though ids are sparse and large.
    let mut slots: Vec<usize> = g
        .hidden_nodes
        .iter()
        .map(|n| g.slot_of(n.id).unwrap())
        .collect();
    slots.sort_unstable();
    assert_eq!(slots.first(), Some(&(1 + g.num_inputs + g.num_outputs)));
    assert_eq!(slots.last(), Some(&(MAX_NODES - 1)));
    assert!(outputs(&g).iter().all(|v| v.is_finite()));
    assert_eq!(g.slot_of(4_000_000_000), None);
}

#[test]
fn a_brain_depends_on_wiring_not_on_the_numbers_used_as_ids() {
    let (mut g, mut innovations, _) = founder(7);
    let node = g.split_connection(10, &mut innovations).unwrap();
    let before = outputs(&g);
    // Give that neuron a different (huge) identity everywhere it appears.
    let renamed = 3_000_000_000u32;
    for n in &mut g.hidden_nodes {
        if n.id == node {
            n.id = renamed;
        }
    }
    for c in &mut g.connections {
        if c.from == node {
            c.from = renamed;
        }
        if c.to == node {
            c.to = renamed;
        }
    }
    assert_eq!(before, outputs(&g));
}

#[test]
fn heavy_structural_turnover_keeps_every_brain_valid() {
    // Many additions and deletions per birth: under the old scheme lineages hit
    // the lifetime cap within a few hundred ticks and add-node silently stopped.
    let mut config = config();
    config.mutation.add_node_probability = 0.6;
    config.mutation.prune_probability = 0.6;
    config.mutation.add_connection_probability = 0.3;
    config.food.regrowth_rate = 0.01; // plenty of births
    let mut sim = Simulation::new(config);
    let first_id = sim.innovations.next_node_id();
    for _ in 0..1500 {
        sim.step();
    }
    assert!(!sim.is_extinct());
    let issued = sim.innovations.next_node_id() - first_id;
    assert!(issued > 200, "expected many innovations, got {issued}");
    let cap = sim.organisms[0].genome.max_hidden_nodes();
    assert!(
        sim.organisms
            .iter()
            .all(|o| o.genome.hidden_nodes.len() <= cap)
    );
    assert!(
        sim.organisms.iter().any(|o| o
            .genome
            .hidden_nodes
            .iter()
            .any(|n| n.id as usize > MAX_NODES)),
        "living creatures should carry neurons numbered beyond the old cap"
    );
    // Full structural validation of every genome and brain, and an exact round trip
    // including the innovation registry.
    let bytes = sim.to_checkpoint_bytes().expect("state must validate");
    let restored = Simulation::from_checkpoint_bytes(&bytes).unwrap();
    assert_eq!(
        restored.innovations.next_node_id(),
        sim.innovations.next_node_id()
    );
    assert_eq!(
        restored.innovations.recorded_splits(),
        sim.innovations.recorded_splits()
    );
    assert_eq!(
        restored.state_fingerprint().unwrap(),
        sim.state_fingerprint().unwrap()
    );
}
