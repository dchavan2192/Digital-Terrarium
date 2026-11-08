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

impl Genome {
    /// Create a random genome for the initial population.
    pub fn random(
        brain_config: &BrainConfig,
        mutation_config: &MutationConfig,
        aging_config: &AgingConfig,
        rng: &mut impl Rng,
    ) -> Self {
        let weight_count = brain_config.total_weights();
        // Xavier-ish initialization: small random weights
        let scale = 1.0 / (brain_config.inputs as f64).sqrt();
        let brain_weights: Vec<f64> = (0..weight_count)
            .map(|_| rng.gen_range(-scale..scale))
            .collect();

        Genome {
            brain_weights,
            mutation_rate: mutation_config.weight_probability,
            mutation_step: mutation_config.weight_step,
            color_hue: rng.gen_range(0.0..360.0),
            aging_rate: aging_config.aging_rate,
            body_size: 1.0,
        }
    }

    /// Produce a mutated offspring genome.
    pub fn reproduce(&self, rng: &mut impl Rng) -> Self {
        let mut child = self.clone();

        // Mutate brain weights
        for w in &mut child.brain_weights {
            if rng.r#gen::<f64>() < self.mutation_rate {
                *w += rng.gen_range(-self.mutation_step..self.mutation_step);
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

    /// Build a [`Brain`] from this genome's weights.
    pub fn build_brain(&self, config: &BrainConfig) -> Brain {
        Brain::new(config, self.brain_weights.clone())
    }

    /// Compute compatibility distance between two genomes.
    ///
    /// Inspired by NEAT (Stanley & Miikkulainen, 2002) compatibility distance,
    /// but adapted for our fixed-topology networks. Distance is a weighted sum of:
    /// - Mean absolute weight difference (normalized by weight count)
    /// - Trait differences (body_size, aging_rate, mutation_rate)
    ///
    /// Citation: Stanley, K.O. & Miikkulainen, R. (2002) "Evolving Neural Networks
    /// through Augmenting Topologies" *Evolutionary Computation* 10(2):99-127
    pub fn genome_distance(&self, other: &Genome) -> f64 {
        // Weight distance: mean absolute difference
        let weight_dist = if !self.brain_weights.is_empty() {
            let sum: f64 = self
                .brain_weights
                .iter()
                .zip(other.brain_weights.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            sum / self.brain_weights.len() as f64
        } else {
            0.0
        };

        // Trait distances (normalized to roughly [0, 1] range)
        let size_dist = (self.body_size - other.body_size).abs() / 2.5; // range is 2.5
        let aging_dist = (self.aging_rate - other.aging_rate).abs() / 0.01; // range is ~0.01
        let mutation_dist = (self.mutation_rate - other.mutation_rate).abs(); // already [0, 1]

        // Weighted combination — weights tuned so brain differences dominate
        // but body plan differences also contribute to speciation
        1.0 * weight_dist + 0.5 * size_dist + 0.5 * aging_dist + 0.3 * mutation_dist
    }
}
