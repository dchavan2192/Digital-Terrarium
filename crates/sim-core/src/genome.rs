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
use crate::config::{AgingConfig, BrainConfig, FruitConfig, MutationConfig, TemperatureConfig};
use rand::Rng;

/// A connection gene in the NEAT genome.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ConnectionGene {
    /// Source node ID.
    pub from: u16,
    /// Target node ID.
    pub to: u16,
    /// Connection weight (evolvable).
    pub weight: f64,
    /// Whether this connection is active. Disabled connections can be re-enabled.
    pub enabled: bool,
    /// Epigenetic methylation level [0, 1]. Suppresses effective weight.
    /// Effective weight = weight * (1 - methylation).
    /// Citation: Jablonka & Lamb (2005) "Evolution in Four Dimensions"
    pub methylation: f64,
}

/// A hidden node with its depth value for feed-forward ordering.
/// Depth 0.0 = input layer, 1.0 = output layer, 0 < depth < 1 = hidden.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct HiddenNode {
    pub id: u16,
    pub depth: f64,
}

/// Maximum connections per genome (prevents unbounded growth).
pub const MAX_CONNECTIONS: usize = 512;

/// The complete heritable state of a creature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

    /// Preferred temperature [0, 1]. 0 = cold (north), 1 = warm (south).
    /// Metabolism penalty scales with mismatch to local temperature.
    /// Bergmann (1847) / Allen (1877): temperature drives body-plan adaptation.
    pub preferred_temp: f64,

    /// Diet preference [0, 1]. 0 = algae specialist, 1 = fruit specialist.
    /// Digestion efficiency: energy * (1.0 - 0.7 * |food_type - diet_preference|)
    /// Specialists get ~100% from their food; generalists get ~65% from either.
    /// Grant & Grant (2006): beak size in Darwin's finches is the textbook example.
    pub diet_preference: f64,
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
        _temperature_config: &TemperatureConfig,
        _fruit_config: &FruitConfig,
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
                methylation: 0.0,
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
                    methylation: 0.0,
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
                methylation: 0.0,
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
                    methylation: 0.0,
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
            preferred_temp: rng.gen_range(0.0..1.0),
            diet_preference: rng.gen_range(0.0..1.0),
        }
    }

    /// Produce a mutated offspring genome.
    ///
    /// NEAT mutations applied (probabilities from `[mutation]`, defaults in parens):
    /// 1. Weight perturbation (per-connection, at the evolvable mutation_rate)
    /// 2. Add connection (`add_connection_probability`, 2%)
    /// 3. Add node (`add_node_probability`, 1%) — split a connection with a hidden node
    /// 4. Toggle connection (`toggle_probability`, 1%) — flip enabled/disabled
    /// 5. Prune connection (`prune_probability`, 4%) — delete a random connection
    /// 6. Scalar trait mutations (`trait_probability`, 5% each) + color drift
    pub fn reproduce(&self, mcfg: &MutationConfig, rng: &mut impl Rng) -> Self {
        let mut child = self.clone();

        // 1. Mutate connection weights
        for c in &mut child.connections {
            if rng.r#gen::<f64>() < self.mutation_rate {
                c.weight += rng.gen_range(-self.mutation_step..self.mutation_step);
            }
        }

        // 2. Add connection (default 2% — balanced against 4% pruning)
        // Citation: Stanley & Miikkulainen (2002) — structural mutation adds connections
        // Citation: Dolson & Park (2012) — prune rate should be 1.5-2x add rate
        if rng.r#gen::<f64>() < mcfg.add_connection_probability {
            child.mutate_add_connection(rng);
        }

        // 3. Add node (default 1%) — split existing connection
        // Citation: Stanley & Miikkulainen (2002) — add-node splits A→B into A→C→B
        if rng.r#gen::<f64>() < mcfg.add_node_probability {
            child.mutate_add_node(rng);
        }

        // 4. Toggle connection (default 1%)
        if rng.r#gen::<f64>() < mcfg.toggle_probability && !child.connections.is_empty() {
            let idx = rng.gen_range(0..child.connections.len());
            child.connections[idx].enabled = !child.connections[idx].enabled;
        }

        // 4b. Prune connection (default 4% — 2x the add rate).
        // Dolson & Park (2012, Swarthmore) showed prune rate must exceed add rate
        // to prevent monotonic complexity growth in continuous evolution.
        // Combined with per-connection brain cost (Attwell & Laughlin 2001),
        // this creates genuine selection pressure for efficient brain topology.
        if rng.r#gen::<f64>() < mcfg.prune_probability && child.connections.len() > 1 {
            let idx = rng.gen_range(0..child.connections.len());
            child.connections.remove(idx);
        }

        // 5. Mutate the mutation rate itself (meta-mutation)
        if rng.r#gen::<f64>() < mcfg.trait_probability {
            child.mutation_rate *= rng.gen_range(0.8..1.2);
            child.mutation_rate = child.mutation_rate.clamp(0.001, 1.0);
        }

        // 6. Mutate the step size
        if rng.r#gen::<f64>() < mcfg.trait_probability {
            child.mutation_step *= rng.gen_range(0.8..1.2);
            child.mutation_step = child.mutation_step.clamp(0.01, 2.0);
        }

        // 7. Mutate aging rate
        if rng.r#gen::<f64>() < mcfg.trait_probability {
            child.aging_rate *= rng.gen_range(0.8..1.2);
            child.aging_rate = child.aging_rate.clamp(0.0001, 0.01);
        }

        // 8. Mutate body size
        if rng.r#gen::<f64>() < mcfg.trait_probability {
            child.body_size *= rng.gen_range(0.9..1.1);
            child.body_size = child.body_size.clamp(0.5, 3.0);
        }

        // 9. Mutate preferred temperature — Bergmann (1847): temperature drives adaptation
        // Selection pressure from the N-S gradient causes directional evolution
        if rng.r#gen::<f64>() < mcfg.trait_probability {
            child.preferred_temp += rng.gen_range(-0.1..0.1);
            child.preferred_temp = child.preferred_temp.clamp(0.0, 1.0);
        }

        // 10. Mutate diet preference — Grant & Grant (2006): character displacement
        // Resource partitioning between algae and fruit specialists drives speciation
        if rng.r#gen::<f64>() < mcfg.trait_probability {
            child.diet_preference += rng.gen_range(-0.1..0.1);
            child.diet_preference = child.diet_preference.clamp(0.0, 1.0);
        }

        // 11. Small drift in color (kin marker)
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
                methylation: 0.0,
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
            methylation: old.methylation,
        });

        // C→B with original weight (preserves output contribution)
        self.connections.push(ConnectionGene {
            from: new_id,
            to: old.to,
            weight: old.weight,
            enabled: true,
            methylation: old.methylation,
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
        // Effective weight = weight * (1 - methylation) — epigenetic suppression.
        // Citation: Jablonka & Lamb (2005) "Evolution in Four Dimensions"
        let conns: Vec<(u16, u16, f64, bool)> = self
            .connections
            .iter()
            .map(|c| (c.from, c.to, c.weight * (1.0 - c.methylation), c.enabled))
            .collect();
        Brain::from_neat(self.num_inputs, self.num_outputs, &hidden, &conns)
    }

    /// Effective (methylation-suppressed) weight of connection gene `idx`.
    /// Must match the formula in [`Genome::build_brain`].
    #[inline]
    pub fn effective_weight(&self, idx: usize) -> f64 {
        let c = &self.connections[idx];
        c.weight * (1.0 - c.methylation)
    }

    /// NEAT crossover: produce offspring from two parents with potentially different topologies.
    ///
    /// - Matching genes (same from,to): randomly inherit from either parent (50/50)
    /// - Disjoint/excess genes: inherit from the fitter parent only
    /// - Hidden nodes: union of both parents
    /// - Scalar traits: average of both parents
    /// - Standard mutations applied to offspring
    ///
    /// Citation: Stanley & Miikkulainen (2002) §3.2, Maynard Smith (1978)
    pub fn crossover(
        parent_a: &Genome,
        parent_b: &Genome,
        a_is_fitter: bool,
        mcfg: &MutationConfig,
        rng: &mut impl Rng,
    ) -> Self {
        let (fitter, weaker) = if a_is_fitter {
            (parent_a, parent_b)
        } else {
            (parent_b, parent_a)
        };

        let mut connections = Vec::new();
        let ac = &parent_a.connections;
        let bc = &parent_b.connections;
        let mut i = 0;
        let mut j = 0;

        while i < ac.len() && j < bc.len() {
            let a_key = (ac[i].from, ac[i].to);
            let b_key = (bc[j].from, bc[j].to);

            if a_key == b_key {
                // Matching gene: 50/50 from either parent
                if rng.r#gen::<bool>() {
                    connections.push(ac[i]);
                } else {
                    connections.push(bc[j]);
                }
                i += 1;
                j += 1;
            } else if a_key < b_key {
                // Disjoint from A: include only if A is fitter
                if a_is_fitter {
                    connections.push(ac[i]);
                }
                i += 1;
            } else {
                // Disjoint from B: include only if B is fitter
                if !a_is_fitter {
                    connections.push(bc[j]);
                }
                j += 1;
            }
        }
        // Excess genes from the fitter parent
        if a_is_fitter {
            while i < ac.len() {
                connections.push(ac[i]);
                i += 1;
            }
        } else {
            while j < bc.len() {
                connections.push(bc[j]);
                j += 1;
            }
        }

        // Hidden nodes: union with dedup
        let mut hidden_nodes = fitter.hidden_nodes.clone();
        for node in &weaker.hidden_nodes {
            if !hidden_nodes.iter().any(|n| n.id == node.id) {
                hidden_nodes.push(*node);
            }
        }
        // Prune orphan hidden nodes not referenced by any connection
        hidden_nodes.retain(|n| connections.iter().any(|c| c.from == n.id || c.to == n.id));

        let next_node_id = hidden_nodes
            .iter()
            .map(|n| n.id + 1)
            .max()
            .unwrap_or(fitter.next_node_id)
            .max(fitter.next_node_id)
            .max(weaker.next_node_id);

        // Blend scalar traits
        let mut child = Genome {
            connections,
            hidden_nodes,
            next_node_id,
            num_inputs: fitter.num_inputs,
            num_outputs: fitter.num_outputs,
            mutation_rate: (parent_a.mutation_rate + parent_b.mutation_rate) / 2.0,
            mutation_step: (parent_a.mutation_step + parent_b.mutation_step) / 2.0,
            color_hue: {
                // Wraparound-aware hue blending
                let diff = parent_b.color_hue - parent_a.color_hue;
                if diff.abs() <= 180.0 {
                    (parent_a.color_hue + parent_b.color_hue) / 2.0
                } else if diff > 0.0 {
                    ((parent_a.color_hue + parent_b.color_hue + 360.0) / 2.0) % 360.0
                } else {
                    ((parent_a.color_hue + parent_b.color_hue - 360.0) / 2.0 + 360.0) % 360.0
                }
            },
            aging_rate: (parent_a.aging_rate + parent_b.aging_rate) / 2.0,
            body_size: (parent_a.body_size + parent_b.body_size) / 2.0,
            preferred_temp: (parent_a.preferred_temp + parent_b.preferred_temp) / 2.0,
            diet_preference: (parent_a.diet_preference + parent_b.diet_preference) / 2.0,
        };

        // Sort connections for genome_distance efficiency
        child
            .connections
            .sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        // Apply standard mutations
        child = child.reproduce(mcfg, rng);

        child
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
        let temp_dist = (self.preferred_temp - other.preferred_temp).abs();
        let diet_dist = (self.diet_preference - other.diet_preference).abs();

        // NEAT-style weighted combination:
        // Structure dominates (different wiring = likely different species)
        // Weight similarity matters for closely-related organisms
        0.8 * structure_dist
            + 0.5 * weight_dist
            + 0.3 * hidden_dist
            + 0.3 * size_dist
            + 0.3 * aging_dist
            + 0.2 * mutation_dist
            + 0.2 * temp_dist
            + 0.2 * diet_dist
    }
}
