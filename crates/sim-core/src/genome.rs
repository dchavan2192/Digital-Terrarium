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

/// Population-wide identity of a node. Bias, input and output nodes use the fixed
/// ids `0..=inputs+outputs`; every hidden node has an id handed out by
/// [`Innovations`], so the same structural innovation carries the same id in every
/// lineage. Ids are historical markers, NOT positions in a brain's activation
/// buffer: [`Genome::build_brain`] maps them to local slots.
pub type NodeId = u32;

/// Historical record of structural innovations, shared by the whole population.
///
/// NEAT aligns the genes of two parents by historical origin, not by position
/// (Stanley & Miikkulainen 2002, sections 2.2 and 3.2). A connection gene is
/// identified here by its endpoints `(from, to)`, which is a valid historical
/// marker only if node ids themselves are historical. This registry makes them so:
/// splitting connection `A -> B` yields the same node id wherever and whenever it
/// happens. Before 2026-11-16 each genome numbered its own nodes, so two lineages
/// could give the same id to unrelated neurons and crossover paired genes that
/// were not homologous.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Innovations {
    next_node_id: NodeId,
    /// `(from << 32) | to` of a split connection -> the node that split created.
    /// A BTreeMap keeps serialization order deterministic.
    splits: std::collections::BTreeMap<u64, NodeId>,
}

impl Innovations {
    /// Registry for a population whose founders have `brain.hidden` hidden nodes.
    pub fn new(brain: &BrainConfig) -> Self {
        Innovations {
            next_node_id: (1 + brain.inputs + brain.outputs + brain.hidden) as NodeId,
            splits: std::collections::BTreeMap::new(),
        }
    }

    /// Node id for splitting connection `from -> to`.
    ///
    /// Returns the id recorded for that split, so independent lineages agree. If
    /// the genome doing the split already contains that node (it split this
    /// connection before, and the connection was later re-added), a fresh id is
    /// issued instead, because a genome cannot hold one node twice.
    pub fn node_for_split(
        &mut self,
        from: NodeId,
        to: NodeId,
        already_present: impl Fn(NodeId) -> bool,
    ) -> NodeId {
        let key = (u64::from(from) << 32) | u64::from(to);
        if let Some(&id) = self.splits.get(&key) {
            if !already_present(id) {
                return id;
            }
            return self.fresh();
        }
        let id = self.fresh();
        self.splits.insert(key, id);
        id
    }

    fn fresh(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id = self
            .next_node_id
            .checked_add(1)
            .expect("node id space exhausted");
        id
    }

    /// The next id that would be issued; every existing hidden node id is below it.
    pub fn next_node_id(&self) -> NodeId {
        self.next_node_id
    }

    /// Number of distinct connection splits recorded.
    pub fn recorded_splits(&self) -> usize {
        self.splits.len()
    }
}

/// A connection gene in the NEAT genome.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ConnectionGene {
    /// Source node id (see [`NodeId`]).
    pub from: NodeId,
    /// Target node id (see [`NodeId`]).
    pub to: NodeId,
    /// Connection weight (evolvable).
    pub weight: f64,
    /// Whether this connection is active. Disabled connections can be re-enabled.
    pub enabled: bool,
    /// Epigenetic methylation level [0, 1]. Suppresses effective weight.
    /// Effective weight = weight * (1 - methylation).
    /// Citation: Jablonka & Lamb (2005) "Evolution in Four Dimensions"
    pub methylation: f64,
    /// A recurrent connection feeds the target with the source's activation from
    /// the PREVIOUS tick, which gives the brain state (memory). Recurrent genes are
    /// exempt from the feed-forward depth rule: any non-bias source (including the
    /// target itself) to any hidden or output target. Feed-forward and recurrent
    /// genes on the same pair are distinct genes.
    /// Citation: Stanley & Miikkulainen (2002) — NEAT evolves recurrent connections.
    #[serde(default)]
    pub recurrent: bool,
}

impl ConnectionGene {
    /// Identity of a gene for sorting, crossover alignment and distance.
    #[inline]
    pub fn key(&self) -> (NodeId, NodeId, bool) {
        (self.from, self.to, self.recurrent)
    }
}

/// A hidden node with its depth value for feed-forward ordering.
/// Depth 0.0 = input layer, 1.0 = output layer, 0 < depth < 1 = hidden.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct HiddenNode {
    pub id: NodeId,
    pub depth: f64,
}

/// Threshold that stops add-edge mutations. Dense founders and node-splitting
/// mutations may exceed it; it is not an absolute genome storage limit.
pub const MAX_CONNECTIONS: usize = 512;

/// The complete heritable state of a creature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Genome {
    /// NEAT connection genes.
    pub connections: Vec<ConnectionGene>,

    /// Hidden nodes with depth values for topological ordering.
    pub hidden_nodes: Vec<HiddenNode>,

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
    /// Digestion efficiency: energy * (1 - diet_slope * |food_type - diet_preference|)
    /// where diet_slope = config.food.fruit.diet_slope (default 1.0).
    /// Specialists get ~100% from their food; generalists get 50% from either (diet_slope=1).
    /// Grant & Grant (2006): beak size in Darwin's finches is the textbook example.
    pub diet_preference: f64,
}

impl Genome {
    /// Check externally loaded heritable state before building its brain.
    ///
    /// This uses the same structural, numeric and trait checks as checkpoints.
    /// Historical node ids may exceed the local activation-buffer capacity;
    /// only the number of nodes held at once is constrained. The caller must
    /// separately check that I/O counts match its intended sensor/action layout.
    /// Validation is read-only and does not repair, sort or otherwise change a
    /// genome. Untrusted genomes must pass this check before [`Self::build_brain`].
    pub fn validate(&self) -> Result<(), crate::checkpoint::CheckpointError> {
        crate::checkpoint::validate_genome(self)
    }

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
        let first_output = (1 + inputs) as NodeId;
        let first_hidden = (1 + inputs + outputs) as NodeId;

        let scale = 1.0 / (inputs as f64).sqrt();

        // Create hidden nodes at depth 0.5
        let hidden_nodes: Vec<HiddenNode> = (0..hidden)
            .map(|i| HiddenNode {
                id: first_hidden + i as NodeId,
                depth: 0.5,
            })
            .collect();

        let mut connections = Vec::new();

        // Bias → hidden
        for h in 0..hidden {
            connections.push(ConnectionGene {
                from: 0,
                to: first_hidden + h as NodeId,
                weight: rng.gen_range(-scale..scale),
                enabled: true,
                methylation: 0.0,
                recurrent: false,
            });
        }

        // Input → hidden (fully connected)
        for i in 0..inputs {
            for h in 0..hidden {
                connections.push(ConnectionGene {
                    from: (1 + i) as NodeId,
                    to: first_hidden + h as NodeId,
                    weight: rng.gen_range(-scale..scale),
                    enabled: true,
                    methylation: 0.0,
                    recurrent: false,
                });
            }
        }

        // Bias → output
        for o in 0..outputs {
            connections.push(ConnectionGene {
                from: 0,
                to: first_output + o as NodeId,
                weight: rng.gen_range(-scale..scale),
                enabled: true,
                methylation: 0.0,
                recurrent: false,
            });
        }

        // Hidden → output (fully connected)
        for h in 0..hidden {
            for o in 0..outputs {
                connections.push(ConnectionGene {
                    from: first_hidden + h as NodeId,
                    to: first_output + o as NodeId,
                    weight: rng.gen_range(-scale..scale),
                    enabled: true,
                    methylation: 0.0,
                    recurrent: false,
                });
            }
        }

        // Sort by (from, to) for efficient genome_distance merge-join
        connections.sort_by_key(|c| c.key());

        Genome {
            connections,
            hidden_nodes,
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
    pub fn reproduce(
        &self,
        mcfg: &MutationConfig,
        fruit_config: &FruitConfig,
        innovations: &mut Innovations,
        rng: &mut impl Rng,
    ) -> Self {
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

        // 2b. Add a recurrent connection (memory). Off by default. The `> 0.0` guard
        // means no random number is drawn when it is off, so runs without memory
        // follow exactly the trajectory they did before this mutation existed.
        if mcfg.add_recurrent_probability > 0.0
            && rng.r#gen::<f64>() < mcfg.add_recurrent_probability
        {
            child.mutate_add_recurrent(rng);
        }

        // 3. Add node (default 1%) — split existing connection
        // Citation: Stanley & Miikkulainen (2002) — add-node splits A→B into A→C→B
        if rng.r#gen::<f64>() < mcfg.add_node_probability {
            child.mutate_add_node(innovations, rng);
        }

        // 4. Toggle connection (default 1%)
        if rng.r#gen::<f64>() < mcfg.toggle_probability && !child.connections.is_empty() {
            let idx = crate::math::random_index(rng, child.connections.len());
            child.connections[idx].enabled = !child.connections[idx].enabled;
        }

        // 4b. Prune connection (default 4% — 2x the add rate).
        // Dolson & Park (2012, Swarthmore) showed prune rate must exceed add rate
        // to prevent monotonic complexity growth in continuous evolution.
        // Combined with per-connection brain cost (Attwell & Laughlin 2001),
        // this creates genuine selection pressure for efficient brain topology.
        if rng.r#gen::<f64>() < mcfg.prune_probability && child.connections.len() > 1 {
            let idx = crate::math::random_index(rng, child.connections.len());
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
        // Uses dedicated probability/step (both doubled vs trait defaults) so diet
        // divergence can outrun neutral drift and reach stable specialist optima.
        // Citation: Schluter (2000) "The Ecology of Adaptive Radiation"
        let diet_step = fruit_config.diet_mutation_step;
        let diet_prob = fruit_config.diet_mutation_probability;
        if rng.r#gen::<f64>() < diet_prob {
            child.diet_preference += rng.gen_range(-diet_step..diet_step);
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

        // A hidden node left without any connection gene is gone for good: dropping
        // it frees its slot, so the node limit applies to the neurons a creature
        // has now, not to how many its lineage ever added.
        child.prune_orphan_nodes();

        // Re-sort connections for genome_distance efficiency
        child.connections.sort_by_key(|c| c.key());

        child
    }

    /// Remove hidden nodes that no connection gene (enabled or disabled) refers to.
    pub fn prune_orphan_nodes(&mut self) {
        let connections = &self.connections;
        self.hidden_nodes
            .retain(|n| connections.iter().any(|c| c.from == n.id || c.to == n.id));
    }

    /// Most hidden nodes one genome can hold: the brain's activation buffer has
    /// `MAX_NODES` slots, and bias, inputs and outputs take the first ones.
    pub fn max_hidden_nodes(&self) -> usize {
        crate::brain::MAX_NODES.saturating_sub(1 + self.num_inputs + self.num_outputs)
    }

    /// Local slot of a node in this genome's brain, or `None` if the genome has no
    /// such node. Fixed nodes keep their ids as slots; hidden nodes follow, in the
    /// order of `hidden_nodes`.
    pub fn slot_of(&self, id: NodeId) -> Option<usize> {
        let fixed = self.num_inputs + self.num_outputs;
        if id as usize <= fixed {
            return Some(id as usize);
        }
        self.hidden_nodes
            .iter()
            .position(|n| n.id == id)
            .map(|pos| fixed + 1 + pos)
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
            let from_idx = crate::math::random_index(rng, all_nodes.len());
            let to_idx = crate::math::random_index(rng, all_nodes.len());
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
                .any(|c| c.from == from_id && c.to == to_id && !c.recurrent)
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
                recurrent: false,
            });
            return;
        }
    }

    /// Add a recurrent connection: `to` receives `from`'s activation of the previous
    /// tick. Source: any input, hidden or output node (the bias is constant, so it
    /// is excluded). Target: any hidden or output node, including the source itself.
    /// Tries up to 20 random pairs before giving up.
    fn mutate_add_recurrent(&mut self, rng: &mut impl Rng) {
        if self.connections.len() >= MAX_CONNECTIONS {
            return;
        }
        let fixed = self.num_inputs + self.num_outputs;
        // Sources: ids 1..=fixed, then hidden nodes. Targets: outputs, then hidden.
        let sources = fixed + self.hidden_nodes.len();
        let targets = self.num_outputs + self.hidden_nodes.len();
        if targets == 0 {
            return;
        }
        for _ in 0..20 {
            let s = crate::math::random_index(rng, sources);
            let t = crate::math::random_index(rng, targets);
            let from = if s < fixed {
                (1 + s) as NodeId
            } else {
                self.hidden_nodes[s - fixed].id
            };
            let to = if t < self.num_outputs {
                (1 + self.num_inputs + t) as NodeId
            } else {
                self.hidden_nodes[t - self.num_outputs].id
            };
            if self
                .connections
                .iter()
                .any(|c| c.from == from && c.to == to && c.recurrent)
            {
                continue;
            }
            let scale = 1.0 / (self.num_inputs as f64).sqrt();
            self.connections.push(ConnectionGene {
                from,
                to,
                weight: rng.gen_range(-scale..scale),
                enabled: true,
                methylation: 0.0,
                recurrent: true,
            });
            return;
        }
    }

    /// Split a random enabled connection with a new hidden node (NEAT add-node).
    fn mutate_add_node(&mut self, innovations: &mut Innovations, rng: &mut impl Rng) {
        if self.hidden_nodes.len() >= self.max_hidden_nodes() {
            return; // every brain slot is in use
        }
        let enabled_indices: Vec<usize> = self
            .connections
            .iter()
            .enumerate()
            .filter(|(_, c)| c.enabled && !c.recurrent)
            .map(|(i, _)| i)
            .collect();
        if enabled_indices.is_empty() {
            return;
        }
        let idx = enabled_indices[crate::math::random_index(rng, enabled_indices.len())];
        self.split_connection(idx, innovations);
    }

    /// Split connection gene `idx` with a new hidden node.
    ///
    /// Classic NEAT add-node: connection A→B is disabled and replaced by A→C
    /// (weight 1.0) and C→B (original weight). C's id comes from the shared
    /// [`Innovations`] registry, so the same split made in another lineage yields
    /// the same node and the two are recognised as homologous in crossover. C's
    /// depth is the midpoint of A's and B's, which makes depth a function of the
    /// node id alone. Returns the new node's id, or `None` if the genome is full.
    pub fn split_connection(
        &mut self,
        idx: usize,
        innovations: &mut Innovations,
    ) -> Option<NodeId> {
        if self.hidden_nodes.len() >= self.max_hidden_nodes() {
            return None;
        }
        let old = self.connections[idx];
        if old.recurrent {
            // A node placed on a recurrent edge has no well-defined depth.
            return None;
        }
        let (from_depth, to_depth) = (self.node_depth(old.from), self.node_depth(old.to));
        let new_depth = (from_depth + to_depth) / 2.0;
        if !(from_depth < new_depth && new_depth < to_depth) {
            // The two depths are adjacent floating-point values: there is no room
            // for a node between them, and evaluation order would be ambiguous.
            return None;
        }
        let new_id = {
            let hidden = &self.hidden_nodes;
            innovations.node_for_split(old.from, old.to, |id| hidden.iter().any(|n| n.id == id))
        };
        self.connections[idx].enabled = false;
        self.hidden_nodes.push(HiddenNode {
            id: new_id,
            depth: new_depth,
        });
        // A→C with weight 1.0 (preserves signal magnitude)
        self.connections.push(ConnectionGene {
            from: old.from,
            to: new_id,
            weight: 1.0,
            enabled: true,
            methylation: old.methylation,
            recurrent: false,
        });
        // C→B with original weight (preserves output contribution)
        self.connections.push(ConnectionGene {
            from: new_id,
            to: old.to,
            weight: old.weight,
            enabled: true,
            methylation: old.methylation,
            recurrent: false,
        });
        Some(new_id)
    }

    /// Get depth of any node by ID.
    fn node_depth(&self, id: NodeId) -> f64 {
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
    fn all_nodes_with_depth(&self) -> Vec<(NodeId, f64)> {
        let mut nodes =
            Vec::with_capacity(1 + self.num_inputs + self.num_outputs + self.hidden_nodes.len());
        nodes.push((0, 0.0)); // bias
        for i in 1..=self.num_inputs {
            nodes.push((i as NodeId, 0.0));
        }
        for o in 0..self.num_outputs {
            nodes.push(((1 + self.num_inputs + o) as NodeId, 1.0));
        }
        for n in &self.hidden_nodes {
            nodes.push((n.id, n.depth));
        }
        nodes
    }

    /// Build a [`Brain`] from this genome's connection genes and hidden nodes.
    ///
    /// Node ids are population-wide and unbounded, so they are mapped to local
    /// slots here (see [`Genome::slot_of`]). The brain's `gene_index` still refers
    /// to positions in `self.connections`.
    pub fn build_brain(&self) -> Brain {
        let first_hidden_slot = 1 + self.num_inputs + self.num_outputs;
        let hidden: Vec<(u16, f64)> = self
            .hidden_nodes
            .iter()
            .enumerate()
            .map(|(pos, n)| ((first_hidden_slot + pos) as u16, n.depth))
            .collect();
        // Effective weight = weight * (1 - methylation) — epigenetic suppression.
        // Citation: Jablonka & Lamb (2005) "Evolution in Four Dimensions"
        // A gene whose endpoint is missing would be a broken invariant (orphan
        // pruning only removes unreferenced nodes); it is kept in place, disabled,
        // so gene positions stay aligned with `gene_index`.
        // Recurrent genes are passed separately; in the feed-forward list they stay
        // in place but disabled, which keeps gene positions aligned with gene_index.
        let conns: Vec<(u16, u16, f64, bool)> = self
            .connections
            .iter()
            .map(|c| match (self.slot_of(c.from), self.slot_of(c.to)) {
                (Some(from), Some(to)) => (
                    from as u16,
                    to as u16,
                    c.weight * (1.0 - c.methylation),
                    c.enabled && !c.recurrent,
                ),
                _ => {
                    debug_assert!(false, "connection gene refers to a node the genome lacks");
                    (0, 0, 0.0, false)
                }
            })
            .collect();
        let recurrent: Vec<(u16, u16, f64, u16)> = self
            .connections
            .iter()
            .enumerate()
            .filter(|(_, c)| c.enabled && c.recurrent)
            .filter_map(|(gi, c)| match (self.slot_of(c.from), self.slot_of(c.to)) {
                (Some(from), Some(to)) => Some((
                    from as u16,
                    to as u16,
                    c.weight * (1.0 - c.methylation),
                    gi as u16,
                )),
                _ => None,
            })
            .collect();
        Brain::from_neat_with_memory(
            self.num_inputs,
            self.num_outputs,
            &hidden,
            &conns,
            &recurrent,
        )
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
    /// - Matching genes (same from,to): randomly inherit from either parent (50/50).
    ///   Node ids are historical markers shared by the population ([`Innovations`]),
    ///   so equal endpoints mean the two genes descend from the same innovation.
    /// - Disjoint/excess genes: inherit from the fitter parent only. This is the
    ///   NEAT rule; the child's wiring is the fitter parent's, with weights of the
    ///   shared genes drawn from both.
    /// - Hidden nodes: those referenced by the inherited genes
    /// - Scalar traits: average of both parents
    /// - Standard mutations applied to offspring
    ///
    /// Citation: Stanley & Miikkulainen (2002) §3.2, Maynard Smith (1978)
    pub fn crossover(
        parent_a: &Genome,
        parent_b: &Genome,
        a_is_fitter: bool,
        mcfg: &MutationConfig,
        fruit_config: &FruitConfig,
        innovations: &mut Innovations,
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
            let a_key = ac[i].key();
            let b_key = bc[j].key();

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

        // Blend scalar traits
        let mut child = Genome {
            connections,
            hidden_nodes,
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
        child.connections.sort_by_key(|c| c.key());

        // Apply standard mutations
        child = child.reproduce(mcfg, fruit_config, innovations, rng);

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
            let a = sc[i].key();
            let b = oc[j].key();

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
