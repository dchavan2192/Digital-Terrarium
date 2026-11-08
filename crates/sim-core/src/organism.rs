//! Organism: a creature living in the terrarium.
//!
//! Each organism has a physical body (position, heading, energy, age),
//! a genome (heritable traits), and a brain (neural network built from the genome).

use crate::brain::Brain;
use crate::genome::Genome;

/// Unique identifier for an organism, monotonically increasing.
pub type OrganismId = u64;

/// A living creature in the simulation.
#[derive(Debug, Clone)]
pub struct Organism {
    pub id: OrganismId,
    pub pos_x: f64,
    pub pos_y: f64,
    /// Heading in radians [0, 2π).
    pub heading: f64,
    pub energy: f64,
    pub age: u64,
    pub genome: Genome,
    pub brain: Brain,
    /// Whether this organism is still alive. Dead organisms are cleaned up at end of tick.
    pub alive: bool,
    /// Parent organism ID (None for initial population).
    pub parent_id: Option<OrganismId>,
    /// Generation count (0 for initial population, parent's generation + 1 for offspring).
    pub generation: u64,
    /// Current broadcast signal [-1, 1]. Updated each tick from brain output.
    pub signal: f64,
    /// Cumulative energy gained from eating food (lifetime).
    pub energy_from_food: f64,
    /// Cumulative energy gained from attacking other creatures (lifetime).
    pub energy_from_attacks: f64,

    /// Running activation histograms for brain output entropy.
    /// 6 outputs × 8 bins over [-1, 1], with exponential decay (τ ≈ 100 ticks).
    /// Citation: Shannon (1948) "A Mathematical Theory of Communication"
    pub output_bins: [[f64; NUM_ENTROPY_BINS]; NUM_OUTPUTS],
}

/// Number of histogram bins for entropy computation.
pub const NUM_ENTROPY_BINS: usize = 8;
/// Number of brain outputs tracked for entropy.
pub const NUM_OUTPUTS: usize = 6;
/// Exponential decay factor for activation histograms (~100 tick memory).
pub const ENTROPY_DECAY: f64 = 0.99;

/// Compute normalized Shannon entropy from a histogram.
/// Returns value in [0, 1] where 0 = deterministic, 1 = maximum entropy.
///
/// H = -Σ(p·ln(p)) / ln(num_bins)
///
/// Citation: Shannon, C.E. (1948) "A Mathematical Theory of Communication"
/// *Bell System Technical Journal* 27(3):379-423
pub fn shannon_entropy(bins: &[f64; NUM_ENTROPY_BINS]) -> f64 {
    let total: f64 = bins.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }
    let mut h = 0.0;
    for &count in bins {
        if count > 0.0 {
            let p = count / total;
            h -= p * p.ln();
        }
    }
    // Normalize by max entropy (ln(NUM_BINS)) to get [0, 1]
    h / (NUM_ENTROPY_BINS as f64).ln()
}

/// Map a value in [-1, 1] to a histogram bin index.
#[inline]
pub fn activation_to_bin(value: f64) -> usize {
    let normalized = (value + 1.0) * 0.5; // [-1,1] → [0,1]
    let idx = (normalized * NUM_ENTROPY_BINS as f64) as usize;
    idx.min(NUM_ENTROPY_BINS - 1)
}

impl Organism {
    /// Build the sensor input vector for the brain.
    ///
    /// v3 layout (15 inputs):
    /// - 5 vision rays × 2 values each (distance normalized [0,1], object type [-1,1])
    ///   type encoding: -1 = nothing, 0 = food, 1 = creature
    /// - own energy normalized to [0,1] (energy / max_energy)
    /// - own heading normalized to [-1,1] (heading / π - 1)
    /// - nearby signal (distance-weighted average of broadcasts)
    /// - own body_size normalized to [-1,1]
    /// - kin similarity to nearest creature [-1,1] (1=same hue, -1=opposite)
    pub fn build_sensor_inputs(
        &self,
        ray_distances: &[f64],
        ray_types: &[f64],
        max_energy: f64,
        signal: f64,
    ) -> Vec<f64> {
        let mut inputs = Vec::with_capacity(self.brain.inputs);

        // Vision rays: interleave distance and type
        for i in 0..ray_distances.len() {
            inputs.push(ray_distances[i]); // [0, 1] where 1 = max range, nothing seen
            inputs.push(ray_types[i]); // -1 = nothing, 0 = food, 1 = creature
        }

        // Own energy normalized
        inputs.push(self.energy / max_energy);

        // Own heading normalized to [-1, 1]
        inputs.push(self.heading / std::f64::consts::PI - 1.0);

        // Nearby signal
        inputs.push(signal);

        inputs
    }
}
