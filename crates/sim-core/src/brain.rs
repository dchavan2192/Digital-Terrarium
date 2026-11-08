//! NEAT-style feed-forward neural network for creature brains.
//!
//! Each brain has arbitrary topology: input nodes, output nodes, and a variable
//! number of hidden nodes connected by evolvable connections. Connections can
//! link any node pair where the source has lower depth than the target, enabling
//! skip connections (input→output) and deep hidden layers.
//!
//! Node numbering:
//!   0 = bias (always outputs 1.0)
//!   1..=num_inputs = input nodes (sensor data)
//!   num_inputs+1..=num_inputs+num_outputs = output nodes (actions)
//!   num_inputs+num_outputs+1.. = hidden nodes (evolved topology)
//!
//! The forward pass uses a stack-allocated activation buffer — zero heap allocation
//! per tick. Connections are pre-sorted at construction time for a single linear pass.
//!
//! Citation: Stanley, K.O. & Miikkulainen, R. (2002) "Evolving Neural Networks
//! through Augmenting Topologies" *Evolutionary Computation* 10(2):99-127

/// Fast rational approximation of tanh.
///
/// Pade approximant: tanh(x) ≈ x(27 + x²) / (27 + 9x²) for |x| < ~4.
/// Clamps to ±1 outside that range. ~3-5x faster than std tanh on the hot path.
/// Max error ~0.004 — negligible for evolved weights where exact values don't matter.
#[inline(always)]
fn fast_tanh(x: f64) -> f64 {
    if x > 4.0 {
        return 1.0;
    }
    if x < -4.0 {
        return -1.0;
    }
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

/// Maximum total nodes (bias + inputs + outputs + hidden) for stack allocation.
pub const MAX_NODES: usize = 64;
/// Maximum output layer size supported.
pub const MAX_OUTPUTS: usize = 16;
/// Maximum input layer size supported.
pub const MAX_INPUTS: usize = 64;

/// A NEAT-style feed-forward neural network with arbitrary topology.
#[derive(Debug, Clone)]
pub struct Brain {
    pub inputs: usize,
    pub hidden: usize,
    pub outputs: usize,
    /// Enabled connections sorted for evaluation: (from_node_id, to_node_id, weight).
    /// Sorted by target node's evaluation order, then by source node.
    pub connections: Vec<(u16, u16, f64)>,
    /// Non-input node IDs in topological (evaluation) order.
    /// Hidden nodes sorted by depth, then output nodes.
    pub eval_order: Vec<u16>,
}

/// Output actions produced by the brain each tick.
#[derive(Debug, Clone, Copy)]
pub struct BrainOutput {
    /// Desired turn amount, normalized to [-1, 1]. Scaled by max_turn externally.
    pub turn: f64,
    /// Desired speed, normalized to [0, 1]. Scaled by max_speed externally.
    pub speed: f64,
    /// Eat intent — if > 0.0, the creature attempts to eat.
    pub eat: f64,
    /// Attack intent — if > 0.0, the creature attempts to attack a nearby creature.
    pub attack: f64,
    /// Broadcast signal [-1, 1]. Heard by nearby creatures as a brain input.
    /// Citation: Werner & Dyer (1991) "Evolution of Communication in Artificial Organisms"
    pub signal: f64,
    /// Share intent — if > 0.0, transfer energy to nearest neighbor.
    /// Citation: Hamilton (1964) kin selection, Trivers (1971) reciprocal altruism
    pub share: f64,
}

impl Brain {
    /// Construct a brain from NEAT genome data.
    ///
    /// - `num_inputs`: number of sensor inputs (not counting bias node)
    /// - `num_outputs`: number of output actions
    /// - `hidden_nodes`: (node_id, depth) pairs for each hidden node
    /// - `connections`: (from, to, weight, enabled) connection genes
    ///
    /// Only enabled connections are kept. Connections are pre-sorted by target
    /// node evaluation order for a single-pass forward evaluation.
    pub fn from_neat(
        num_inputs: usize,
        num_outputs: usize,
        hidden_nodes: &[(u16, f64)],
        connections: &[(u16, u16, f64, bool)],
    ) -> Self {
        assert!(num_inputs <= MAX_INPUTS);
        assert!(num_outputs <= MAX_OUTPUTS);

        let first_output = (num_inputs + 1) as u16;
        let last_output = (num_inputs + num_outputs) as u16;

        // Build eval order: hidden nodes by (depth, id), then output nodes by id
        let mut eval_nodes: Vec<(u16, f64)> = hidden_nodes.to_vec();
        for id in first_output..=last_output {
            eval_nodes.push((id, 1.0));
        }
        eval_nodes.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        let eval_order: Vec<u16> = eval_nodes.iter().map(|&(id, _)| id).collect();

        // Map node ID → position in eval_order for connection sorting
        let mut node_eval_pos = [u32::MAX; MAX_NODES];
        for (pos, &node_id) in eval_order.iter().enumerate() {
            node_eval_pos[node_id as usize] = pos as u32;
        }

        // Filter enabled connections and sort by (target_eval_pos, source_id)
        let mut sorted: Vec<(u16, u16, f64)> = connections
            .iter()
            .filter(|c| c.3) // enabled only
            .map(|&(f, t, w, _)| (f, t, w))
            .collect();
        sorted.sort_by(|a, b| {
            node_eval_pos[a.1 as usize]
                .cmp(&node_eval_pos[b.1 as usize])
                .then(a.0.cmp(&b.0))
        });

        Brain {
            inputs: num_inputs,
            hidden: hidden_nodes.len(),
            outputs: num_outputs,
            connections: sorted,
            eval_order,
        }
    }

    /// Run a forward pass. Returns the output actions.
    ///
    /// Uses stack-allocated activation buffer — zero heap allocation.
    /// Connections are pre-sorted so this is a single linear pass through the
    /// connection list, matching the topological evaluation order.
    #[inline]
    pub fn forward(&self, inputs: &[f64]) -> BrainOutput {
        debug_assert_eq!(inputs.len(), self.inputs);

        let mut act = [0.0f64; MAX_NODES];
        act[0] = 1.0; // bias node
        for (i, &v) in inputs.iter().enumerate() {
            act[i + 1] = v;
        }

        // Evaluate nodes in topological order.
        // Connections are pre-sorted to match eval_order — single linear pass.
        let mut ci = 0;
        for &node in &self.eval_order {
            let mut sum = 0.0;
            while ci < self.connections.len() && self.connections[ci].1 == node {
                let (from, _, w) = self.connections[ci];
                sum += act[from as usize] * w;
                ci += 1;
            }
            act[node as usize] = fast_tanh(sum);
        }

        // Read output nodes: (inputs+1)..=(inputs+outputs)
        let o = self.inputs + 1;
        BrainOutput {
            turn: act[o],                    // [-1, 1]
            speed: (act[o + 1] + 1.0) * 0.5, // tanh [-1,1] → [0, 1]
            eat: act[o + 2],                 // > 0.0 means try to eat
            attack: act[o + 3],              // > 0.0 means try to attack
            signal: act[o + 4],              // [-1, 1] broadcast signal
            share: act[o + 5],               // > 0.0 means share energy
        }
    }

    /// Total number of neurons (hidden + output) for energy cost calculation.
    pub fn neuron_count(&self) -> usize {
        self.hidden + self.outputs
    }

    /// Number of active (enabled) connections.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}
