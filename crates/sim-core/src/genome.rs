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
}
