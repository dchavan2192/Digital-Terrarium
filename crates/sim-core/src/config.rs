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
            return Err(ConfigError::Invalid("world dimensions must be positive".into()));
        }
        if self.food.cell_size <= 0.0 {
            return Err(ConfigError::Invalid("food cell_size must be positive".into()));
        }
        if self.creature.initial_count == 0 {
            return Err(ConfigError::Invalid("initial_count must be > 0".into()));
        }
        if self.energy.starting <= 0.0 {
            return Err(ConfigError::Invalid("starting energy must be positive".into()));
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
