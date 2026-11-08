//! Genome: the heritable blueprint for a creature.
//!
//! Contains body traits, brain weights, and a self-referential mutation rate
//! (so evolvability itself can evolve). Mutation is applied during reproduction.

use crate::brain::Brain;
use crate::config::{AgingConfig, BrainConfig, MutationConfig};
use rand::Rng;

/// The complete heritable state of a creature.
#[derive(Debug, Clone)]
pub struct Genome {
    /// Neural network weights (flat vector, layout defined in [`Brain`]).
    pub brain_weights: Vec<f64>,

    /// Number of hidden neurons in this creature's brain [MIN_HIDDEN, MAX_HIDDEN].
    /// Evolvable — topology can change across generations.
    /// Citation: Stanley & Miikkulainen (2002) NEAT showed topology evolution
    /// is critical for increasing complexity over generations.
    pub hidden_size: usize,

    /// Per-weight mutation probability. This is itself a gene that can drift.
    pub mutation_rate: f64,

    /// Per-weight mutation step size. Also evolvable.
    pub mutation_step: f64,

    /// Hue value [0, 360) used for lineage coloring and kin recognition.
    pub color_hue: f64,

    /// Gompertz aging rate (β). Higher = faster senescence. Evolvable.
    /// Citation: Williams (1957) antagonistic pleiotropy predicts selection
    /// can act on aging rate when it trades off against early-life fitness.
    pub aging_rate: f64,

    /// Relative body size [0.5, 3.0]. Affects speed, metabolism, combat.
    /// Citation: Kleiber (1932) — metabolic rate scales as mass^0.75.
    /// Larger creatures are slower but hit harder and survive hits better.
    pub body_size: f64,
}

/// Minimum hidden layer size (prevents degenerate zero-hidden brains).
pub const MIN_HIDDEN: usize = 4;

/// Compute weight count for a given topology.
pub fn weight_count_for(inputs: usize, hidden: usize, outputs: usize) -> usize {
    (inputs * hidden) + hidden + (hidden * outputs) + outputs
}

impl Genome {
    /// Create a random genome for the initial population.
    pub fn random(
        brain_config: &BrainConfig,
        mutation_config: &MutationConfig,
        aging_config: &AgingConfig,
        rng: &mut impl Rng,
    ) -> Self {
        let hidden_size = brain_config.hidden;
        let wc = weight_count_for(brain_config.inputs, hidden_size, brain_config.outputs);
        // Xavier-ish initialization: small random weights
        let scale = 1.0 / (brain_config.inputs as f64).sqrt();
        let brain_weights: Vec<f64> = (0..wc).map(|_| rng.gen_range(-scale..scale)).collect();

        Genome {
            brain_weights,
            hidden_size,
            mutation_rate: mutation_config.weight_probability,
            mutation_step: mutation_config.weight_step,
            color_hue: rng.gen_range(0.0..360.0),
            aging_rate: aging_config.aging_rate,
            body_size: 1.0,
        }
    }

    /// Produce a mutated offspring genome.
    ///
    /// The `inputs` and `outputs` parameters are needed to correctly resize the
    /// weight vector when hidden_size mutates (add/remove neuron).
    pub fn reproduce(&self, inputs: usize, outputs: usize, rng: &mut impl Rng) -> Self {
        let mut child = self.clone();

        // Mutate brain weights
        for w in &mut child.brain_weights {
            if rng.r#gen::<f64>() < self.mutation_rate {
                *w += rng.gen_range(-self.mutation_step..self.mutation_step);
            }
        }

        // Mutate hidden layer size (topology evolution, 2% chance)
        // Citation: Stanley & Miikkulainen (2002) — add/remove node mutations.
        // Clamped to [MIN_HIDDEN, MAX_HIDDEN] for stack-allocation safety.
        if rng.r#gen::<f64>() < 0.02 {
            let old_hidden = child.hidden_size;
            if rng.r#gen::<bool>() {
                child.hidden_size = (old_hidden + 1).min(crate::brain::MAX_HIDDEN);
            } else {
                child.hidden_size = old_hidden.saturating_sub(1).max(MIN_HIDDEN);
            }
            if child.hidden_size != old_hidden {
                child.resize_weights(inputs, outputs, old_hidden, rng);
            }
        }

        // Mutate the mutation rate itself (meta-mutation)
        if rng.r#gen::<f64>() < 0.05 {
            child.mutation_rate *= rng.gen_range(0.8..1.2);
            child.mutation_rate = child.mutation_rate.clamp(0.001, 1.0);
        }

        // Mutate the step size
        if rng.r#gen::<f64>() < 0.05 {
            child.mutation_step *= rng.gen_range(0.8..1.2);
            child.mutation_step = child.mutation_step.clamp(0.01, 2.0);
        }

        // Mutate aging rate (evolvable longevity gene)
        // Williams (1957): selection can act on senescence rate.
        // Clamped to [0.0001, 0.01] to prevent immortality or instant death.
        if rng.r#gen::<f64>() < 0.05 {
            child.aging_rate *= rng.gen_range(0.8..1.2);
            child.aging_rate = child.aging_rate.clamp(0.0001, 0.01);
        }

        // Mutate body size
        // Kleiber (1932): body size is the key trait determining metabolic strategy.
        // Clamped to [0.5, 3.0] — half-size to triple-size relative to default.
        if rng.r#gen::<f64>() < 0.05 {
            child.body_size *= rng.gen_range(0.9..1.1);
            child.body_size = child.body_size.clamp(0.5, 3.0);
        }

        // Small drift in color — offspring look similar to parents (kin marker)
        child.color_hue += rng.gen_range(-5.0..5.0);
        if child.color_hue < 0.0 {
            child.color_hue += 360.0;
        }
        if child.color_hue >= 360.0 {
            child.color_hue -= 360.0;
        }

        child
    }

    /// Resize the weight vector after a hidden_size mutation.
    ///
    /// Weight layout: [ih_weights(inputs*hidden), h_biases(hidden), ho_weights(hidden*outputs), o_biases(outputs)]
    ///
    /// When adding a neuron: insert zero-initialized weights for the new neuron
    /// (new connections start silent, must be activated by subsequent weight mutations).
    /// When removing a neuron: drop the last hidden neuron's weights.
    ///
    /// Xavier scale used for new weights so they're not dead-on-arrival.
    fn resize_weights(
        &mut self,
        inputs: usize,
        outputs: usize,
        old_hidden: usize,
        rng: &mut impl Rng,
    ) {
        let new_hidden = self.hidden_size;
        let scale = 1.0 / (inputs as f64).sqrt();

        // Decode old layout
        let old_ih_end = inputs * old_hidden;
        let old_hb_end = old_ih_end + old_hidden;
        let old_ho_end = old_hb_end + old_hidden * outputs;
        // old_ob_end = old_ho_end + outputs (rest of vec)

        let old_ih = &self.brain_weights[..old_ih_end];
        let old_hb = &self.brain_weights[old_ih_end..old_hb_end];
        let old_ho = &self.brain_weights[old_hb_end..old_ho_end];
        let old_ob = &self.brain_weights[old_ho_end..];

        let copy_h = old_hidden.min(new_hidden);

        // Build new ih weights: for each input, copy existing hidden connections, pad/truncate
        let mut new_weights = Vec::with_capacity(weight_count_for(inputs, new_hidden, outputs));
        for i in 0..inputs {
            let src_start = i * old_hidden;
            for h in 0..copy_h {
                new_weights.push(old_ih[src_start + h]);
            }
            // New neurons get Xavier-scale random weights
            for _ in copy_h..new_hidden {
                new_weights.push(rng.gen_range(-scale..scale));
            }
        }

        // Hidden biases
        new_weights.extend_from_slice(&old_hb[..copy_h]);
        new_weights.extend(std::iter::repeat_n(0.0, new_hidden - copy_h));

        // ho weights: for each hidden, copy existing output connections, pad/truncate
        for h in 0..copy_h {
            let src_start = h * outputs;
            for o in 0..outputs {
                new_weights.push(old_ho[src_start + o]);
            }
        }
        // New neurons → output connections start near zero
        for _ in copy_h..new_hidden {
            for _ in 0..outputs {
                new_weights.push(rng.gen_range(-scale * 0.1..scale * 0.1));
            }
        }

        // Output biases (unchanged)
        new_weights.extend_from_slice(old_ob);

        self.brain_weights = new_weights;
    }

    /// Build a [`Brain`] from this genome's weights and hidden_size.
    pub fn build_brain(&self, config: &BrainConfig) -> Brain {
        Brain::from_genome(
            config.inputs,
            self.hidden_size,
            config.outputs,
            self.brain_weights.clone(),
        )
    }

    /// Compute compatibility distance between two genomes.
    ///
    /// Inspired by NEAT (Stanley & Miikkulainen, 2002) compatibility distance,
    /// adapted for variable-topology networks. When genomes have different hidden sizes,
    /// we compare overlapping weights and add a penalty for the size difference.
    ///
    /// Citation: Stanley, K.O. & Miikkulainen, R. (2002) "Evolving Neural Networks
    /// through Augmenting Topologies" *Evolutionary Computation* 10(2):99-127
    pub fn genome_distance(&self, other: &Genome) -> f64 {
        // Weight distance: compare overlapping weights, normalize by max weight count
        let weight_dist = if !self.brain_weights.is_empty() && !other.brain_weights.is_empty() {
            let min_len = self.brain_weights.len().min(other.brain_weights.len());
            let max_len = self.brain_weights.len().max(other.brain_weights.len());
            let sum: f64 = self.brain_weights[..min_len]
                .iter()
                .zip(other.brain_weights[..min_len].iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            sum / max_len as f64
        } else {
            0.0
        };

        // Hidden size difference (normalized by MAX_HIDDEN range)
        let hidden_dist = (self.hidden_size as f64 - other.hidden_size as f64).abs()
            / crate::brain::MAX_HIDDEN as f64;

        // Trait distances (normalized to roughly [0, 1] range)
        let size_dist = (self.body_size - other.body_size).abs() / 2.5; // range is 2.5
        let aging_dist = (self.aging_rate - other.aging_rate).abs() / 0.01; // range is ~0.01
        let mutation_dist = (self.mutation_rate - other.mutation_rate).abs(); // already [0, 1]

        // Weighted combination — brain differences dominate, topology difference
        // is a strong speciation signal (different brain architecture = different species)
        1.0 * weight_dist
            + 0.8 * hidden_dist
            + 0.5 * size_dist
            + 0.5 * aging_dist
            + 0.3 * mutation_dist
    }
}
