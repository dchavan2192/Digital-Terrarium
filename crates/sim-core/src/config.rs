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
    /// Amount of energy transferred per share action.
    pub share_amount: f64,
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
