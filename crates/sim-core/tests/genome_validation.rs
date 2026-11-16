//! Malformed external genomes must be rejected before brain construction.
use sim_core::brain::MAX_NODES;
use sim_core::checkpoint::CheckpointError;
use sim_core::genome::{ConnectionGene, Genome, HiddenNode};

fn edge(from: u32, to: u32, recurrent: bool) -> ConnectionGene {
    ConnectionGene {
        from,
        to,
        weight: 2.0,
        enabled: true,
        methylation: 0.0,
        recurrent,
    }
}

fn genome() -> Genome {
    Genome {
        connections: vec![edge(1, 2, false)],
        hidden_nodes: vec![],
        num_inputs: 1,
        num_outputs: 7,
        mutation_rate: 0.1,
        mutation_step: 0.2,
        color_hue: 180.0,
        aging_rate: 0.001,
        body_size: 1.0,
        preferred_temp: 0.5,
        diet_preference: 0.5,
    }
}

fn rejected(g: &Genome) {
    assert!(matches!(
        g.validate(),
        Err(CheckpointError::InvalidState(_))
    ));
}

#[test]
fn crafted_maximum_dimensions_are_errors_not_overflow_panics() {
    for (inputs, outputs) in [
        (usize::MAX, 7),
        (1, usize::MAX),
        (usize::MAX, usize::MAX),
        (MAX_NODES, 7),
        (0, 7),
        (1, 6),
    ] {
        let mut g = genome();
        g.num_inputs = inputs;
        g.num_outputs = outputs;
        // Exercise the same typed deserialization path used for external exports.
        let g: Genome = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        rejected(&g);
    }
    let mut g = genome();
    g.hidden_nodes = vec![HiddenNode { id: 9, depth: 0.5 }; MAX_NODES];
    rejected(&g);
    g = genome();
    g.connections = vec![edge(1, 2, false); MAX_NODES * MAX_NODES + 1];
    rejected(&g);
}

#[test]
fn hidden_identity_depth_and_missing_nodes_are_checked() {
    for node in [
        HiddenNode { id: 1, depth: 0.5 },
        HiddenNode {
            id: 100,
            depth: 0.0,
        },
        HiddenNode {
            id: 100,
            depth: 1.0,
        },
        HiddenNode {
            id: 100,
            depth: f64::NAN,
        },
    ] {
        let mut g = genome();
        g.hidden_nodes.push(node);
        rejected(&g);
    }
    let mut g = genome();
    g.hidden_nodes = vec![
        HiddenNode {
            id: 100,
            depth: 0.5
        };
        2
    ];
    rejected(&g);
    g = genome();
    g.connections = vec![edge(1, 100, false)];
    rejected(&g);
    g.connections = vec![edge(100, 2, false)];
    rejected(&g);
}

#[test]
fn cycles_invalid_recurrence_and_gene_order_are_rejected_even_when_disabled() {
    for connections in [
        vec![edge(2, 2, false)],
        vec![edge(2, 1, false)],
        vec![edge(0, 2, true)],
        vec![edge(2, 1, true)],
        vec![edge(2, 0, true)],
        vec![edge(1, 3, false), edge(1, 2, false)],
        vec![edge(1, 2, false), edge(1, 2, false)],
        vec![edge(1, 2, true), edge(1, 2, false)],
    ] {
        for enabled in [true, false] {
            let mut g = genome();
            g.connections = connections.clone();
            g.connections.iter_mut().for_each(|c| c.enabled = enabled);
            rejected(&g);
        }
    }
    let mut g = genome();
    g.hidden_nodes = vec![
        HiddenNode {
            id: 100,
            depth: 0.25,
        },
        HiddenNode {
            id: 101,
            depth: 0.75,
        },
    ];
    g.connections = vec![edge(100, 101, false), edge(101, 100, false)];
    rejected(&g);
}

#[test]
fn nonfinite_weights_methylation_and_traits_are_rejected() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut g = genome();
        g.connections[0].weight = value;
        rejected(&g);
        g = genome();
        g.connections[0].methylation = value;
        rejected(&g);
        for property in 0..7 {
            let mut g = genome();
            match property {
                0 => g.mutation_rate = value,
                1 => g.mutation_step = value,
                2 => g.color_hue = value,
                3 => g.aging_rate = value,
                4 => g.body_size = value,
                5 => g.preferred_temp = value,
                _ => g.diet_preference = value,
            }
            rejected(&g);
        }
    }
    for value in [-0.1, 1.1] {
        let mut g = genome();
        g.connections[0].methylation = value;
        rejected(&g);
    }
}

#[test]
fn invalid_trait_bounds_are_rejected() {
    for property in 0..7 {
        let mut g = genome();
        match property {
            0 => g.mutation_rate = -0.1,
            1 => g.mutation_step = 0.0,
            2 => g.color_hue = 360.0,
            3 => g.aging_rate = -0.1,
            4 => g.body_size = 0.0,
            5 => g.preferred_temp = 1.1,
            _ => g.diet_preference = -0.1,
        }
        rejected(&g);
    }
}

#[test]
fn valid_dense_founders_above_mutation_add_edge_limit_remain_supported() {
    let mut g = genome();
    g.num_inputs = 20;
    g.hidden_nodes = (28..64).map(|id| HiddenNode { id, depth: 0.5 }).collect();
    g.connections.clear();
    for from in 0..=20 {
        for to in 28..64 {
            g.connections.push(edge(from, to, false));
        }
    }
    for from in 28..64 {
        for to in 21..=27 {
            g.connections.push(edge(from, to, false));
        }
    }
    g.connections.sort_by_key(ConnectionGene::key);
    assert!(g.connections.len() > sim_core::genome::MAX_CONNECTIONS);
    g.validate().unwrap();
    assert!(g.build_brain().forward(&[0.5; 20]).0.turn.is_finite());
}

#[test]
fn valid_historical_ids_and_recurrent_self_loops_preserve_brain_output() {
    let mut g = genome();
    g.hidden_nodes = vec![HiddenNode {
        id: u32::MAX,
        depth: 0.5,
    }];
    g.connections = vec![
        edge(1, u32::MAX, false),
        edge(u32::MAX, 2, false),
        edge(u32::MAX, u32::MAX, true),
    ];
    let before = serde_json::to_string(&g).unwrap();
    let first_output = g.build_brain().forward(&[1.0]).0.turn;
    g.validate().unwrap();
    assert_eq!(serde_json::to_string(&g).unwrap(), before);
    assert_eq!(g.build_brain().forward(&[1.0]).0.turn, first_output);
}
