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
    /// Eat intent — if > 0.5, the creature attempts to eat.
    pub eat: f64,
}

impl Brain {
    /// Create a new brain with the given weights. Panics if weight count is wrong.
    pub fn new(config: &BrainConfig, weights: Vec<f64>) -> Self {
        assert_eq!(
            weights.len(),
            config.total_weights(),
            "brain weight count mismatch: expected {}, got {}",
            config.total_weights(),
            weights.len()
        );
        Brain {
            inputs: config.inputs,
            hidden: config.hidden,
            outputs: config.outputs,
            weights,
        }
    }

    /// Run a forward pass. Returns the output actions.
    ///
    /// Activation: tanh for hidden, tanh for outputs.
    /// Outputs are then mapped: turn stays in [-1,1], speed is clamped to [0,1],
    /// eat stays as-is (thresholded externally at 0.0).
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

        // Hidden layer
        let mut hidden = vec![0.0; self.hidden];
        for h in 0..self.hidden {
            let mut sum = h_biases[h];
            for i in 0..self.inputs {
                sum += inputs[i] * ih_weights[i * self.hidden + h];
            }
            hidden[h] = sum.tanh();
        }

        // Output layer
        let mut output = vec![0.0; self.outputs];
        for o in 0..self.outputs {
            let mut sum = o_biases[o];
            for h in 0..self.hidden {
                sum += hidden[h] * ho_weights[h * self.outputs + o];
            }
            output[o] = sum.tanh();
        }

        BrainOutput {
            turn: output[0],                // [-1, 1]
            speed: (output[1] + 1.0) * 0.5, // tanh [-1,1] → [0, 1]
            eat: output[2],                 // > 0.0 means try to eat
        }
    }

    /// Total number of neurons (for energy cost calculation).
    pub fn neuron_count(&self) -> usize {
        self.hidden + self.outputs
    }
}
