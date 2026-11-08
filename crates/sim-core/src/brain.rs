//! Feed-forward neural network for creature brains.
//!
//! v1 uses a fixed topology: inputs → hidden → outputs. Only the weights
//! evolve. The network is intentionally tiny so thousands can run per tick.
//!
//! The weight layout in a flat `Vec<f64>` is:
//! 1. input→hidden weights  (inputs × hidden)
//! 2. hidden biases          (hidden)
//! 3. hidden→output weights  (hidden × outputs)
//! 4. output biases          (outputs)

use crate::config::BrainConfig;

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

/// Maximum hidden layer size supported (for stack allocation).
pub const MAX_HIDDEN: usize = 32;
/// Maximum output layer size supported.
pub const MAX_OUTPUTS: usize = 16;
/// Maximum input layer size supported.
pub const MAX_INPUTS: usize = 64;

/// A fixed-topology feed-forward neural network.
#[derive(Debug, Clone)]
pub struct Brain {
    pub inputs: usize,
    pub hidden: usize,
    pub outputs: usize,
    /// Flat weight vector. Layout described in module docs.
    pub weights: Vec<f64>,
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
    /// Create a new brain with the given weights. Panics if weight count is wrong.
    pub fn new(config: &BrainConfig, weights: Vec<f64>) -> Self {
        Self::from_genome(config.inputs, config.hidden, config.outputs, weights)
    }

    /// Create a brain with explicit layer sizes (used when hidden_size varies per organism).
    pub fn from_genome(inputs: usize, hidden: usize, outputs: usize, weights: Vec<f64>) -> Self {
        let expected = crate::genome::weight_count_for(inputs, hidden, outputs);
        assert_eq!(
            weights.len(),
            expected,
            "brain weight count mismatch: expected {}, got {} (topology {}→{}→{})",
            expected,
            weights.len(),
            inputs,
            hidden,
            outputs,
        );
        assert!(hidden <= MAX_HIDDEN);
        assert!(outputs <= MAX_OUTPUTS);
        assert!(inputs <= MAX_INPUTS);
        Brain {
            inputs,
            hidden,
            outputs,
            weights,
        }
    }

    /// Run a forward pass. Returns the output actions.
    /// Uses stack-allocated arrays — zero heap allocation.
    pub fn forward(&self, inputs: &[f64]) -> BrainOutput {
        debug_assert_eq!(inputs.len(), self.inputs);

        let ih_weights = &self.weights[..self.inputs * self.hidden];
        let h_biases_start = self.inputs * self.hidden;
        let h_biases = &self.weights[h_biases_start..h_biases_start + self.hidden];
        let ho_weights_start = h_biases_start + self.hidden;
        let ho_weights =
            &self.weights[ho_weights_start..ho_weights_start + self.hidden * self.outputs];
        let o_biases_start = ho_weights_start + self.hidden * self.outputs;
        let o_biases = &self.weights[o_biases_start..];

        // Stack-allocated buffers
        let mut hidden = [0.0f64; MAX_HIDDEN];
        let mut output = [0.0f64; MAX_OUTPUTS];

        // Hidden layer
        for h in 0..self.hidden {
            let mut sum = h_biases[h];
            let row_start = h; // weights laid out as input-major: ih_weights[i * hidden + h]
            for i in 0..self.inputs {
                sum += inputs[i] * ih_weights[i * self.hidden + row_start];
            }
            hidden[h] = fast_tanh(sum);
        }

        // Output layer
        for o in 0..self.outputs {
            let mut sum = o_biases[o];
            for h in 0..self.hidden {
                sum += hidden[h] * ho_weights[h * self.outputs + o];
            }
            output[o] = fast_tanh(sum);
        }

        BrainOutput {
            turn: output[0],                // [-1, 1]
            speed: (output[1] + 1.0) * 0.5, // tanh [-1,1] → [0, 1]
            eat: output[2],                 // > 0.0 means try to eat
            attack: output[3],              // > 0.0 means try to attack
            signal: output[4],              // [-1, 1] broadcast signal
            share: output[5],               // > 0.0 means share energy with nearest neighbor
        }
    }

    /// Total number of neurons (for energy cost calculation).
    pub fn neuron_count(&self) -> usize {
        self.hidden + self.outputs
    }
}
