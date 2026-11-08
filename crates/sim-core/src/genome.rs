//! Genome: the heritable blueprint for a creature.
//!
//! NEAT-style genome with connection genes and evolvable topology.
//! Each connection gene specifies a source node, target node, weight, and enabled flag.
//! Hidden nodes are added via node-splitting mutations (classic NEAT add-node).
//! Connections can be added between any valid feed-forward node pair.
//!
//! Citations:
//! - Stanley, K.O. & Miikkulainen, R. (2002) "Evolving Neural Networks through
//!   Augmenting Topologies" *Evolutionary Computation* 10(2):99-127
//! - Kassahun & Sommer (2005) "Efficient Reinforcement Learning through
//!   Evolutionary Acquisition of Neural Topologies" *ESANN*

use crate::brain::Brain;
use crate::config::{AgingConfig, BrainConfig, MutationConfig};
use rand::Rng;

/// A connection gene in the NEAT genome.
#[derive(Debug, Clone, Copy)]
pub struct ConnectionGene {
    /// Source node ID.
    pub from: u16,
    /// Target node ID.
    pub to: u16,
    /// Connection weight (evolvable).
    pub weight: f64,
    /// Whether this connection is active. Disabled connections can be re-enabled.
    pub enabled: bool,
}

/// A hidden node with its depth value for feed-forward ordering.
/// Depth 0.0 = input layer, 1.0 = output layer, 0 < depth < 1 = hidden.
#[derive(Debug, Clone, Copy)]
pub struct HiddenNode {
    pub id: u16,
    pub depth: f64,
}

/// Maximum connections per genome (prevents unbounded growth).
pub const MAX_CONNECTIONS: usize = 512;

/// The complete heritable state of a creature.
#[derive(Debug, Clone)]
pub struct Genome {
    /// NEAT connection genes.
    pub connections: Vec<ConnectionGene>,

    /// Hidden nodes with depth values for topological ordering.
    pub hidden_nodes: Vec<HiddenNode>,

    /// Next available node ID for add-node mutations.
    pub next_node_id: u16,

    /// Number of input nodes (fixed from config, stored for mutation logic).
    pub num_inputs: usize,

    /// Number of output nodes (fixed from config, stored for mutation logic).
    pub num_outputs: usize,

    /// Per-weight mutation probability. This is itself a gene that can drift.
    pub mutation_rate: f64,

    /// Per-weight mutation step size. Also evolvable.
    pub mutation_step: f64,

    /// Hue value [0, 360) used for lineage coloring and kin recognition.
    pub color_hue: f64,

    /// Gompertz aging rate (β). Higher = faster senescence. Evolvable.
    /// Citation: Williams (1957) antagonistic pleiotropy.
    pub aging_rate: f64,

    /// Relative body size [0.5, 3.0]. Affects speed, metabolism, combat.
    /// Citation: Kleiber (1932) metabolic allometry.
    pub body_size: f64,
}

impl Genome {
    /// Create a random genome with fully-connected initial topology.
    ///
    /// Initial network: bias + all inputs → all hidden → all outputs, plus bias → outputs.
    /// This matches the previous dense-matrix topology so behavior is continuous.
    pub fn random(
        brain_config: &BrainConfig,
        mutation_config: &MutationConfig,
        aging_config: &AgingConfig,
        rng: &mut impl Rng,
    ) -> Self {
        let inputs = brain_config.inputs;
        let outputs = brain_config.outputs;
        let hidden = brain_config.hidden;
        let first_output = (1 + inputs) as u16;
        let first_hidden = (1 + inputs + outputs) as u16;

        let scale = 1.0 / (inputs as f64).sqrt();

        // Create hidden nodes at depth 0.5
        let hidden_nodes: Vec<HiddenNode> = (0..hidden)
            .map(|i| HiddenNode {
                id: first_hidden + i as u16,
                depth: 0.5,
            })
            .collect();

        let mut connections = Vec::new();

        // Bias → hidden
        for h in 0..hidden {
            connections.push(ConnectionGene {
                from: 0,
                to: first_hidden + h as u16,
                weight: rng.gen_range(-scale..scale),
                enabled: true,
            });
        }

        // Input → hidden (fully connected)
        for i in 0..inputs {
            for h in 0..hidden {
                connections.push(ConnectionGene {
                    from: (1 + i) as u16,
                    to: first_hidden + h as u16,
                    weight: rng.gen_range(-scale..scale),
                    enabled: true,
                });
            }
        }

        // Bias → output
        for o in 0..outputs {
            connections.push(ConnectionGene {
                from: 0,
                to: first_output + o as u16,
                weight: rng.gen_range(-scale..scale),
                enabled: true,
            });
        }

        // Hidden → output (fully connected)
        for h in 0..hidden {
            for o in 0..outputs {
                connections.push(ConnectionGene {
                    from: first_hidden + h as u16,
                    to: first_output + o as u16,
                    weight: rng.gen_range(-scale..scale),
                    enabled: true,
                });
            }
        }

        // Sort by (from, to) for efficient genome_distance merge-join
        connections.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        Genome {
            connections,
            hidden_nodes,
            next_node_id: first_hidden + hidden as u16,
            num_inputs: inputs,
            num_outputs: outputs,
            mutation_rate: mutation_config.weight_probability,
            mutation_step: mutation_config.weight_step,
            color_hue: rng.gen_range(0.0..360.0),
            aging_rate: aging_config.aging_rate,
            body_size: 1.0,
        }
    }

    /// Produce a mutated offspring genome.
    ///
    /// NEAT mutations applied:
    /// 1. Weight perturbation (per-connection, at mutation_rate)
    /// 2. Add connection (3%) — new edge between unconnected feed-forward pair
    /// 3. Add node (1%) — split existing connection with a new hidden node
    /// 4. Toggle connection (1%) — flip enabled/disabled
    /// 5. Body/aging/color trait mutations (unchanged from previous)
    pub fn reproduce(&self, rng: &mut impl Rng) -> Self {
        let mut child = self.clone();

        // 1. Mutate connection weights
        for c in &mut child.connections {
            if rng.r#gen::<f64>() < self.mutation_rate {
                c.weight += rng.gen_range(-self.mutation_step..self.mutation_step);
            }
        }

        // 2. Add connection (3% chance)
        // Citation: Stanley & Miikkulainen (2002) — structural mutation adds connections
        if rng.r#gen::<f64>() < 0.03 {
            child.mutate_add_connection(rng);
        }

        // 3. Add node (1% chance) — split existing connection
        // Citation: Stanley & Miikkulainen (2002) — add-node splits A→B into A→C→B
        if rng.r#gen::<f64>() < 0.01 {
            child.mutate_add_node(rng);
        }

        // 4. Toggle connection (1% chance)
        if rng.r#gen::<f64>() < 0.01 && !child.connections.is_empty() {
            let idx = rng.gen_range(0..child.connections.len());
            child.connections[idx].enabled = !child.connections[idx].enabled;
        }

        // 5. Mutate the mutation rate itself (meta-mutation)
        if rng.r#gen::<f64>() < 0.05 {
            child.mutation_rate *= rng.gen_range(0.8..1.2);
            child.mutation_rate = child.mutation_rate.clamp(0.001, 1.0);
        }

        // 6. Mutate the step size
        if rng.r#gen::<f64>() < 0.05 {
            child.mutation_step *= rng.gen_range(0.8..1.2);
            child.mutation_step = child.mutation_step.clamp(0.01, 2.0);
        }

        // 7. Mutate aging rate
        if rng.r#gen::<f64>() < 0.05 {
            child.aging_rate *= rng.gen_range(0.8..1.2);
            child.aging_rate = child.aging_rate.clamp(0.0001, 0.01);
        }

        // 8. Mutate body size
        if rng.r#gen::<f64>() < 0.05 {
            child.body_size *= rng.gen_range(0.9..1.1);
            child.body_size = child.body_size.clamp(0.5, 3.0);
        }

        // 9. Small drift in color (kin marker)
        child.color_hue += rng.gen_range(-5.0..5.0);
        if child.color_hue < 0.0 {
            child.color_hue += 360.0;
        }
        if child.color_hue >= 360.0 {
            child.color_hue -= 360.0;
        }

        // Re-sort connections for genome_distance efficiency
        child
            .connections
            .sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        child
    }

    /// Add a new connection between two unconnected nodes.
    ///
    /// A valid connection goes from a node with lower depth to one with higher depth
    /// (feed-forward constraint). Tries up to 20 random pairs before giving up.
    fn mutate_add_connection(&mut self, rng: &mut impl Rng) {
        if self.connections.len() >= MAX_CONNECTIONS {
            return;
        }

        let all_nodes = self.all_nodes_with_depth();

        for _ in 0..20 {
            let from_idx = rng.gen_range(0..all_nodes.len());
            let to_idx = rng.gen_range(0..all_nodes.len());
            let (from_id, from_depth) = all_nodes[from_idx];
            let (to_id, to_depth) = all_nodes[to_idx];

            // Feed-forward: source depth must be strictly less than target depth
            if from_depth >= to_depth {
                continue;
            }

            // No duplicate connections
            if self
                .connections
                .iter()
                .any(|c| c.from == from_id && c.to == to_id)
            {
                continue;
            }

            let scale = 1.0 / (self.num_inputs as f64).sqrt();
            self.connections.push(ConnectionGene {
                from: from_id,
                to: to_id,
                weight: rng.gen_range(-scale..scale),
                enabled: true,
            });
            return;
        }
    }

    /// Split an existing connection with a new hidden node.
    ///
    /// Classic NEAT add-node mutation: connection A→B is disabled, and replaced
    /// by A→C (weight 1.0) and C→B (original weight). This preserves the network's
    /// behavior while adding structural complexity.
    ///
    /// The new node C gets depth = (depth_A + depth_B) / 2.
    fn mutate_add_node(&mut self, rng: &mut impl Rng) {
        if self.next_node_id as usize >= crate::brain::MAX_NODES {
            return; // hit max nodes
        }

        // Pick a random enabled connection
        let enabled_indices: Vec<usize> = self
            .connections
            .iter()
            .enumerate()
            .filter(|(_, c)| c.enabled)
            .map(|(i, _)| i)
            .collect();
        if enabled_indices.is_empty() {
            return;
        }

        let idx = enabled_indices[rng.gen_range(0..enabled_indices.len())];
        let old = self.connections[idx];

        // Disable the old connection
        self.connections[idx].enabled = false;

        // Create new hidden node at midpoint depth
        let from_depth = self.node_depth(old.from);
        let to_depth = self.node_depth(old.to);
        let new_id = self.next_node_id;
        let new_depth = (from_depth + to_depth) / 2.0;

        self.hidden_nodes.push(HiddenNode {
            id: new_id,
            depth: new_depth,
        });
        self.next_node_id += 1;

        // A→C with weight 1.0 (preserves signal magnitude)
        self.connections.push(ConnectionGene {
            from: old.from,
            to: new_id,
            weight: 1.0,
            enabled: true,
        });

        // C→B with original weight (preserves output contribution)
        self.connections.push(ConnectionGene {
            from: new_id,
            to: old.to,
            weight: old.weight,
            enabled: true,
        });
    }

    /// Get depth of any node by ID.
    fn node_depth(&self, id: u16) -> f64 {
        if id == 0 || (id as usize) <= self.num_inputs {
            0.0 // bias or input
        } else if (id as usize) <= self.num_inputs + self.num_outputs {
            1.0 // output
        } else {
            self.hidden_nodes
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.depth)
                .unwrap_or(0.5)
        }
    }

    /// Get all node IDs with their depth values.
    fn all_nodes_with_depth(&self) -> Vec<(u16, f64)> {
        let mut nodes =
            Vec::with_capacity(1 + self.num_inputs + self.num_outputs + self.hidden_nodes.len());
        nodes.push((0, 0.0)); // bias
        for i in 1..=self.num_inputs {
            nodes.push((i as u16, 0.0));
        }
        for o in 0..self.num_outputs {
            nodes.push(((1 + self.num_inputs + o) as u16, 1.0));
        }
        for n in &self.hidden_nodes {
            nodes.push((n.id, n.depth));
        }
        nodes
    }

    /// Build a [`Brain`] from this genome's connection genes and hidden nodes.
    pub fn build_brain(&self) -> Brain {
        let hidden: Vec<(u16, f64)> = self.hidden_nodes.iter().map(|n| (n.id, n.depth)).collect();
        let conns: Vec<(u16, u16, f64, bool)> = self
            .connections
            .iter()
            .map(|c| (c.from, c.to, c.weight, c.enabled))
            .collect();
        Brain::from_neat(self.num_inputs, self.num_outputs, &hidden, &conns)
    }

    /// Compute compatibility distance between two NEAT genomes.
    ///
    /// Uses a merge-join on connections sorted by (from, to) to compute:
    /// - Structural distance: fraction of disjoint/excess connection genes
    /// - Weight distance: mean weight difference of matching connections
    /// - Trait distances: body size, aging rate, hidden count
    ///
    /// Citation: Stanley & Miikkulainen (2002) — compatibility distance with
    /// coefficients c1 (structure), c2 (weight), c3 (traits).
    pub fn genome_distance(&self, other: &Genome) -> f64 {
        // Connections are pre-sorted by (from, to). Merge-join — zero allocation.
        let sc = &self.connections;
        let oc = &other.connections;
        let mut i = 0;
        let mut j = 0;
        let mut matching_weight_diff = 0.0;
        let mut matching_count = 0usize;
        let mut disjoint_count = 0usize;

        while i < sc.len() && j < oc.len() {
            let a = (sc[i].from, sc[i].to);
            let b = (oc[j].from, oc[j].to);

            if a == b {
                matching_weight_diff += (sc[i].weight - oc[j].weight).abs();
                matching_count += 1;
                i += 1;
                j += 1;
            } else if a < b {
                disjoint_count += 1;
                i += 1;
            } else {
                disjoint_count += 1;
                j += 1;
            }
        }
        disjoint_count += (sc.len() - i) + (oc.len() - j);

        let max_conns = sc.len().max(oc.len()).max(1);
        let weight_dist = if matching_count > 0 {
            matching_weight_diff / matching_count as f64
        } else {
            0.0
        };
        let structure_dist = disjoint_count as f64 / max_conns as f64;

        // Topology distance
        let hidden_dist = (self.hidden_nodes.len() as f64 - other.hidden_nodes.len() as f64).abs()
            / crate::brain::MAX_NODES as f64;

        // Trait distances (normalized to roughly [0, 1])
        let size_dist = (self.body_size - other.body_size).abs() / 2.5;
        let aging_dist = (self.aging_rate - other.aging_rate).abs() / 0.01;
        let mutation_dist = (self.mutation_rate - other.mutation_rate).abs();

        // NEAT-style weighted combination:
        // Structure dominates (different wiring = likely different species)
        // Weight similarity matters for closely-related organisms
        0.8 * structure_dist
            + 0.5 * weight_dist
            + 0.3 * hidden_dist
            + 0.3 * size_dist
            + 0.3 * aging_dist
            + 0.2 * mutation_dist
    }
}
