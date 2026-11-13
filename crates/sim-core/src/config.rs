//! Strongly-typed simulation configuration.
//!
//! Every tunable parameter lives in a TOML file. This module deserializes it
//! into Rust structs so the rest of the crate never touches raw strings.
//!
//! # Example
//!
//! ```no_run
//! let cfg = sim_core::Config::from_file("configs/default.toml").unwrap();
//! assert_eq!(cfg.world.width, 1000.0);
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Top-level configuration. Mirrors the TOML structure exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub world: WorldConfig,
    pub food: FoodConfig,
    pub creature: CreatureConfig,
    pub energy: EnergyConfig,
    pub reproduction: ReproductionConfig,
    pub mutation: MutationConfig,
    pub brain: BrainConfig,
    pub aging: AgingConfig,
    pub combat: CombatConfig,
    pub sharing: SharingConfig,
    #[serde(default)]
    pub density_pressure: DensityPressureConfig,
    #[serde(default)]
    pub speciation: SpeciationConfig,
    #[serde(default)]
    pub temperature: TemperatureConfig,
    #[serde(default)]
    pub fruit: FruitConfig,
    #[serde(default)]
    pub pheromones: PheromoneConfig,
    #[serde(default)]
    pub daynight: DayNightConfig,
    #[serde(default)]
    pub elevation: ElevationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub width: f64,
    pub height: f64,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodConfig {
    pub cell_size: f64,
    pub max_per_cell: f64,
    pub regrowth_rate: f64,
    pub energy_per_food: f64,
    /// Spatial resource gradient zones. Each zone modifies the local regrowth rate.
    /// Creates ecological niches that drive adaptive radiation and speciation.
    ///
    /// Citation: Schluter, D. (2000) "The Ecology of Adaptive Radiation" Oxford UP
    #[serde(default)]
    pub biomes: Vec<BiomeZone>,
    /// Seasonal (cyclical) food variation. Regrowth oscillates sinusoidally over time.
    ///
    /// Citation: Botkin, D.B. (1990) "Forest Dynamics: An Ecological Model" Oxford UP
    #[serde(default)]
    pub seasons: SeasonsConfig,
    /// Stochastic localized droughts that temporarily kill food regrowth in a region.
    ///
    /// Creates scarcity pockets that force migration, predation, or cooperation.
    /// Citation: Holling (1973) "Resilience and Stability of Ecological Systems"
    /// Citation: Connell (1978) "Diversity in Tropical Rain Forests and Coral Reefs"
    /// (intermediate disturbance hypothesis)
    #[serde(default)]
    pub droughts: DroughtConfig,
    #[serde(default)]
    pub fruit: FruitConfig,
}

/// A circular zone with a modified food regrowth rate.
///
/// Regrowth at a cell = base_regrowth_rate × biome_multiplier (highest multiplier wins).
/// This creates spatial heterogeneity — lush oases vs barren deserts — driving
/// niche specialization and ultimately speciation.
///
/// Citation: Rosenzweig, M.L. (1995) "Species Diversity in Space and Time" Cambridge UP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeZone {
    /// Center X coordinate (world units).
    pub x: f64,
    /// Center Y coordinate (world units).
    pub y: f64,
    /// Radius of the zone (world units).
    pub radius: f64,
    /// Multiplier applied to the base regrowth rate within this zone.
    /// >1.0 = lush, <1.0 = barren, 0.0 = dead zone.
    pub regrowth_multiplier: f64,
}

/// Seasonal (cyclical) food regrowth variation.
///
/// Applies a sinusoidal multiplier to all regrowth rates: `1.0 + amplitude * sin(2π * tick / period)`.
/// This creates boom-bust resource cycles that select for energy-buffering strategies
/// and seasonal migration between biomes.
///
/// Citation: Botkin, D.B. (1990) "Forest Dynamics: An Ecological Model" Oxford UP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonsConfig {
    /// Amplitude of seasonal oscillation [0, 1). 0 = no seasons, 0.5 = regrowth swings 50%-150%.
    pub amplitude: f64,
    /// Period of one full seasonal cycle in ticks.
    pub period: f64,
}

/// Stochastic localized droughts — temporary regrowth-killing zones.
///
/// Droughts spawn randomly, last for a duration, then fade. Inside a drought
/// zone, food regrowth is multiplied by `severity` (typically ~0.05-0.1),
/// effectively killing the food supply in that region.
///
/// This creates the scarcity pockets needed for predation and cooperation
/// to become viable strategies, and prevents any single location from being
/// a permanent safe haven.
///
/// Citations:
/// - Holling, C.S. (1973) "Resilience and Stability of Ecological Systems"
///   *Annual Review of Ecology and Systematics* 4:1-23
/// - Connell, J.H. (1978) "Diversity in Tropical Rain Forests and Coral Reefs"
///   *Science* 199(4335):1302-1310 (intermediate disturbance hypothesis)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DroughtConfig {
    /// Probability of a new drought spawning per tick. ~0.0002 = one every ~5000 ticks.
    pub spawn_rate: f64,
    /// How long a drought lasts in ticks.
    pub duration: u64,
    /// Radius of the drought zone in world units.
    pub radius: f64,
    /// Regrowth multiplier inside drought (0.05 = 5% of normal). Lower = more severe.
    pub severity: f64,
    /// Maximum number of simultaneous droughts.
    pub max_active: usize,
}

impl Default for DroughtConfig {
    fn default() -> Self {
        DroughtConfig {
            spawn_rate: 0.0,
            duration: 3000,
            radius: 150.0,
            severity: 0.05,
            max_active: 3,
        }
    }
}

impl Default for SeasonsConfig {
    fn default() -> Self {
        SeasonsConfig {
            amplitude: 0.0,
            period: 10000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureConfig {
    pub initial_count: usize,
    pub hard_cap: usize,
    pub body_radius: f64,
    pub max_speed: f64,
    pub max_turn: f64,
    pub max_age: u64,
    pub vision: VisionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionConfig {
    pub ray_count: usize,
    pub fov: f64,
    pub range: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConfig {
    pub starting: f64,
    pub maximum: f64,
    pub base_metabolism: f64,
    pub movement_cost: f64,
    pub brain_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproductionConfig {
    pub threshold: f64,
    pub parent_cost: f64,
    pub child_energy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationConfig {
    pub weight_probability: f64,
    pub weight_step: f64,
}

/// Gompertz-Makeham aging parameters.
///
/// Per-tick mortality probability: h(age) = extrinsic_mortality + initial_senescence * exp(aging_rate * age)
///
/// Based on the Gompertz-Makeham law of mortality (Gompertz, 1825; Makeham, 1860).
/// - `extrinsic_mortality` (λ): age-independent death rate (accidents, starvation background)
/// - `initial_senescence` (α): baseline senescence rate at birth
/// - `aging_rate` (β): how fast mortality accelerates with age — this is the key evolvable parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgingConfig {
    /// Age-independent mortality per tick (λ in Gompertz-Makeham). Accidents, background hazard.
    pub extrinsic_mortality: f64,
    /// Initial senescence rate (α). Tiny at birth, amplified exponentially by aging_rate.
    pub initial_senescence: f64,
    /// Default aging rate (β) for initial population. Encoded in genome; can evolve.
    pub aging_rate: f64,
}

/// Combat parameters for predator-prey dynamics.
///
/// Inspired by Lotka-Volterra predator-prey models (Lotka, 1925; Volterra, 1926)
/// and Yaeger's Polyworld (1994) where creatures could fight for energy.
///
/// Key design: attacking has a cost, and energy transfer is lossy (thermodynamic
/// inefficiency). This ensures that predation is only viable when prey is abundant
/// and creates selective pressure for both predator and prey strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatConfig {
    /// Maximum range at which an attack can land (world units).
    pub attack_range: f64,
    /// Energy cost to the attacker per attack attempt (win or lose).
    pub attack_cost: f64,
    /// Base damage dealt to the victim per successful attack.
    pub attack_damage: f64,
    /// Fraction of damage dealt that the attacker absorbs as energy [0, 1].
    /// < 1.0 models thermodynamic loss in trophic transfer (~10% in nature).
    pub energy_transfer_ratio: f64,
}

/// Energy sharing parameters.
///
/// Enables cooperative behavior: creatures can voluntarily transfer energy
/// to nearby creatures. Whether this evolves depends on kin selection
/// (Hamilton, 1964) or reciprocal altruism (Trivers, 1971).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConfig {
    /// Maximum range for energy sharing (world units).
    pub share_range: f64,
    /// Amount of energy the donor pays per share action.
    pub share_amount: f64,
    /// Multiplier applied to received energy: recipient gets share_amount * benefit_multiplier.
    /// Models synergistic group benefits (e.g., shared warmth, group defense).
    /// Must be >= 1.0 for Hamilton's rB > C to be satisfiable.
    /// Citation: Hamilton (1964) "The Genetical Evolution of Social Behaviour"
    #[serde(default = "default_benefit_multiplier")]
    pub benefit_multiplier: f64,
}

fn default_benefit_multiplier() -> f64 {
    1.5
}

/// Population density pressure — crowding increases metabolism.
///
/// Creatures in dense local neighborhoods pay an extra metabolic cost per
/// nearby neighbor, modeling negative density dependence (interference
/// competition). At high local densities, organisms expend more energy
/// competing for space and resources.
///
/// Citation: Allee, W.C. (1931) "Animal Aggregations: A Study in General Sociology"
/// University of Chicago Press
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DensityPressureConfig {
    /// Extra energy cost per tick per nearby neighbor. 0.0 = disabled.
    pub cost_per_neighbor: f64,
}

impl Default for DensityPressureConfig {
    fn default() -> Self {
        DensityPressureConfig {
            cost_per_neighbor: 0.0,
        }
    }
}

/// Dynamic speciation threshold configuration.
///
/// Computes the compatibility distance threshold from population genome variance
/// so species count adapts to actual diversity. When the population is genetically
/// diverse, the threshold rises (keeping species meaningful). When homogeneous,
/// it drops (detecting finer distinctions).
///
/// Citation: Stanley, K.O. & Miikkulainen, R. (2002) "Evolving Neural Networks
/// through Augmenting Topologies" — dynamic compatibility threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpeciationConfig {
    /// Use dynamic threshold computed from genome variance (true) or fixed value (false).
    pub dynamic: bool,
    /// Fixed compatibility distance threshold (used when dynamic=false, or as initial fallback).
    pub fixed_threshold: f64,
    /// Number of organisms to sample for mean distance estimation (O(n²) pairs among sample).
    pub sample_size: usize,
    /// Threshold = factor × mean pairwise genome distance among sample.
    pub factor: f64,
    /// Floor: prevents over-splitting into trivially small species.
    pub min_threshold: f64,
    /// Ceiling: prevents collapsing all organisms into one species.
    pub max_threshold: f64,
}

impl Default for SpeciationConfig {
    fn default() -> Self {
        SpeciationConfig {
            dynamic: false,
            fixed_threshold: 0.5,
            sample_size: 50,
            factor: 0.75,
            min_threshold: 0.1,
            max_threshold: 2.0,
        }
    }
}

/// Per-cell temperature gradient — cold in the north (y=0), warm in the south.
///
/// Each creature has an evolvable `preferred_temp` gene. When local temperature
/// deviates from the preference, extra metabolic cost is incurred.
///
/// Bergmann's rule (1847): larger body = more thermal mass in cold climates.
/// Allen's rule (1877): shorter extremities in cold = less surface area.
/// Here those effects emerge from selection on body_size + preferred_temp combos.
///
/// Citation: Bergmann, C. (1847) "Über die Verhältnisse der Wärmeökonomie der Thiere
///   zu ihrer Größe" *Göttinger Studien* 3(1):595-708
/// Citation: Allen, J.A. (1877) "The influence of Physical Conditions in the
///   Genesis of Species" *Radical Review* 1:108-140
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TemperatureConfig {
    /// Whether the temperature gradient is active.
    pub enabled: bool,
    /// Extra metabolic multiplier per unit temperature mismatch.
    /// temp_cost = base_metabolism * cost_factor * |local_temp - preferred_temp|
    pub cost_factor: f64,
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        TemperatureConfig {
            enabled: false,
            cost_factor: 0.5,
        }
    }
}

/// Two-food-type system: algae (base food) + fruit (biome-only).
///
/// Algae grows everywhere at the base regrowth rate. Fruit grows only inside
/// biome zones at a multiplied rate, creating a resource that only specialists
/// can exploit efficiently. Each creature has an evolvable diet_preference gene.
///
/// Digestion efficiency: energy * (1.0 - 0.7 * |food_type - diet_preference|)
///   - Algae specialist (diet_preference=0): 100% algae, 30% fruit
///   - Fruit specialist (diet_preference=1): 100% fruit, 30% algae
///   - Generalist (diet_preference=0.5): 65% from either
///
/// This models character displacement and adaptive radiation, where population
/// divergence along a resource axis reduces interspecific competition.
///
/// Citation: Grant & Grant (2006) "Evolution of Character Displacement in
///   Darwin's Finches" *Science* 313(5784):224-226
/// Citation: Schluter, D. (2000) "The Ecology of Adaptive Radiation" Oxford UP
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FruitConfig {
    /// Whether the fruit food type is active.
    pub enabled: bool,
    /// Energy per unit of fruit consumed (before diet efficiency adjustment).
    pub energy_per_fruit: f64,
    /// Fruit regrowth rate multiplier applied on top of the biome's normal rate.
    /// Fruit only appears inside biome zones (outside biomes: no fruit).
    pub biome_rate_multiplier: f64,
}

impl Default for FruitConfig {
    fn default() -> Self {
        FruitConfig {
            enabled: false,
            energy_per_fruit: 20.0,
            biome_rate_multiplier: 2.0,
        }
    }
}

/// Per-cell pheromone layer enabling stigmergic communication.
///
/// Each creature deposits pheromone at its current cell each tick. Pheromone
/// decays multiplicatively, creating concentration gradients that encode
/// past creature activity. Brains that attend to local pheromone level can
/// evolve trail-following, territory marking, and predator-avoidance.
///
/// Self-organizing trail dynamics: a positive feedback loop emerges when
/// creatures preferentially follow high-pheromone paths, reinforcing those
/// paths — the classic Deneubourg mechanism.
///
/// Citation: Wilson, E.O. (1975) "Sociobiology: The New Synthesis" Harvard UP
/// Citation: Deneubourg et al. (1990) "The Self-Organizing Exploratory Pattern
///   of the Argentine Ant" *J. Insect Behavior* 3(2):159-168
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PheromoneConfig {
    /// Whether the pheromone system is active.
    pub enabled: bool,
    /// Amount of pheromone deposited per creature per tick.
    pub deposit_rate: f64,
    /// Multiplicative decay per tick. 0.99 ≈ 100-tick half-life.
    pub decay_rate: f64,
    /// Maximum pheromone level per cell (caps deposit to prevent overflow).
    pub max_level: f64,
}

impl Default for PheromoneConfig {
    fn default() -> Self {
        PheromoneConfig {
            enabled: false,
            deposit_rate: 0.1,
            decay_rate: 0.99,
            max_level: 10.0,
        }
    }
}

/// Day/night cycle — sinusoidal light level modulates vision range.
///
/// Light level: `L = (1 + sin(2π * tick / period)) / 2`, oscillating [0, 1].
/// During night (`L < 0.5`), effective vision range is halved, forcing
/// creatures to rely on other senses (pheromones, signals) or stay put.
///
/// Ecological consequences:
/// - Selection for temporal niche partitioning: diurnal vs. nocturnal strategies
/// - Nocturnal predators may ambush sight-dependent prey
/// - Pheromone trails become more valuable at night than visual scanning
/// - Brain input (input 18) wired in FOURTH step
///
/// Citation: Kronfeld-Schor, N. & Dayan, T. (2003) "Partitioning of Time as
///   an Ecological Resource" *Annual Review of Ecology, Evolution, and Systematics*
///   34:153-181
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DayNightConfig {
    /// Whether the day/night cycle is active.
    pub enabled: bool,
    /// Number of ticks per full day/night cycle. 1000 ≈ 1 tick per minute equivalent.
    pub period: f64,
}

impl Default for DayNightConfig {
    fn default() -> Self {
        DayNightConfig {
            enabled: false,
            period: 1000.0,
        }
    }
}

/// Elevation map — multi-octave sine height field, no external crates.
///
/// Height is computed as the sum of N sine waves at increasing frequencies and
/// decreasing amplitudes (standard fBm-style terrain synthesis):
///   h(x, y) = Σ_k (1/2^k) * sin(2^k * fx * x + φ_kx) * sin(2^k * fy * y + φ_ky)
/// where fx, fy are base spatial frequencies derived from `seed`.
///
/// Movement penalty: each step the creature's effective speed is multiplied by
///   1 / (1 + slope_factor * |height(to) - height(from)|)
/// so organisms moving uphill (or downhill on steep terrain) are slower.
/// This creates impassable ridges, sheltered valleys, and island-like patches
/// that isolate subpopulations — enabling allopatric speciation.
///
/// No brain input is added for elevation (organisms feel it only through movement
/// efficiency, selecting for topography-aware route planning indirectly).
///
/// Citation: MacArthur, R.H. & Wilson, E.O. (1967) "The Theory of Island
///   Biogeography" Princeton UP — habitat fragmentation drives speciation
/// Citation: Wright, S. (1943) "Isolation by Distance" *Genetics* 28(2):114-138
/// Citation: Musgrave (1994) "The Synthesis and Rendering of Eroded Fractal Terrains"
///   fBm (fractional Brownian motion) multi-octave noise method
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ElevationConfig {
    /// Whether elevation affects movement speed.
    pub enabled: bool,
    /// Speed multiplier denominator per unit height difference.
    /// effective_speed = base_speed / (1 + slope_factor * |Δh|)
    /// At slope_factor=5.0, a height diff of 0.2 (steep) halves speed.
    pub slope_factor: f64,
    /// Number of octaves in the height field (more = more detail, slower init).
    pub octaves: usize,
    /// Seed offset for the height field frequencies and phases. Independent of
    /// the main simulation seed so elevation is reproducible but separate.
    pub seed: u64,
}

impl Default for ElevationConfig {
    fn default() -> Self {
        ElevationConfig {
            enabled: false,
            slope_factor: 5.0,
            octaves: 4,
            seed: 7777,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfig {
    pub inputs: usize,
    pub hidden: usize,
    pub outputs: usize,
}

impl BrainConfig {
    /// Total number of weights in the feed-forward network.
    /// (inputs → hidden) + (hidden → outputs) + hidden biases + output biases.
    pub fn total_weights(&self) -> usize {
        (self.inputs * self.hidden) + self.hidden + (self.hidden * self.outputs) + self.outputs
    }

    /// Total number of neurons (hidden + output) for brain-cost accounting.
    pub fn total_neurons(&self) -> usize {
        self.hidden + self.outputs
    }
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::Io(e, path.as_ref().to_path_buf()))?;
        let config: Config = toml::from_str(&text)
            .map_err(|e| ConfigError::Parse(e, path.as_ref().to_path_buf()))?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a TOML string (useful for tests and embedding).
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let config: Config = toml::from_str(text).map_err(ConfigError::ParseStr)?;
        config.validate()?;
        Ok(config)
    }

    /// Sanity-check values that would cause panics or nonsense if wrong.
    fn validate(&self) -> Result<(), ConfigError> {
        if self.world.width <= 0.0 || self.world.height <= 0.0 {
            return Err(ConfigError::Invalid(
                "world dimensions must be positive".into(),
            ));
        }
        if self.food.cell_size <= 0.0 {
            return Err(ConfigError::Invalid(
                "food cell_size must be positive".into(),
            ));
        }
        if self.creature.initial_count == 0 {
            return Err(ConfigError::Invalid("initial_count must be > 0".into()));
        }
        if self.energy.starting <= 0.0 {
            return Err(ConfigError::Invalid(
                "starting energy must be positive".into(),
            ));
        }
        if self.reproduction.parent_cost >= self.energy.maximum {
            return Err(ConfigError::Invalid(
                "parent_cost must be less than max energy".into(),
            ));
        }
        if self.brain.inputs == 0 || self.brain.hidden == 0 || self.brain.outputs == 0 {
            return Err(ConfigError::Invalid("brain layer sizes must be > 0".into()));
        }
        Ok(())
    }
}

/// Errors that can occur when loading configuration.
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error, std::path::PathBuf),
    Parse(toml::de::Error, std::path::PathBuf),
    ParseStr(toml::de::Error),
    Invalid(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e, path) => write!(f, "failed to read {}: {}", path.display(), e),
            ConfigError::Parse(e, path) => write!(f, "failed to parse {}: {}", path.display(), e),
            ConfigError::ParseStr(e) => write!(f, "failed to parse config string: {}", e),
            ConfigError::Invalid(msg) => write!(f, "invalid config: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}
