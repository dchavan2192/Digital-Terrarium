//! # sim-core
//!
//! Core simulation engine for **Terrarium** — a digital world where creatures
//! with evolvable neural brains live, compete, cooperate, and evolve.
//!
//! This crate contains all simulation logic and **zero rendering code**.
//! It runs identically whether driven by the visual viewer or the headless runner.
//!
//! ## Architecture
//!
//! - [`config`] — Strongly-typed configuration deserialized from TOML.
//! - [`world`] — The 2D wraparound world, food grid, and spatial index.
//! - [`organism`] — Creature body, senses, energy, and age.
//! - [`genome`] — Heritable traits: body genes, brain weights, mutation rate.
//! - [`brain`] — Feed-forward neural network evaluated each tick.
//! - [`sim`] — The simulation state and deterministic tick loop.

pub mod brain;
pub mod config;
pub mod genome;
pub mod organism;
pub mod sim;
pub mod world;

pub use config::Config;
pub use sim::{Simulation, SpeciesInfo};
