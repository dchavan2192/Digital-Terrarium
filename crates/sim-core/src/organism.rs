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
    /// Cumulative energy gained from eating food (lifetime).
    pub energy_from_food: f64,
    /// Cumulative energy gained from attacking other creatures (lifetime).
    pub energy_from_attacks: f64,
}

impl Organism {
    /// Build the sensor input vector for the brain.
    ///
    /// v2 layout (14 inputs):
    /// - 5 vision rays × 2 values each (distance normalized [0,1], object type [-1,1])
    ///   type encoding: -1 = nothing, 0 = food, 1 = creature
    /// - own energy normalized to [0,1] (energy / max_energy)
    /// - own heading normalized to [-1,1] (heading / π - 1)
    /// - nearby signal (0 if none heard)
    /// - own body_size normalized to [-1,1]
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
