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

/// Maximum number of vision rays the tick loop supports (stack arrays).
pub const MAX_RAYS: usize = 16;
/// Scalar (non-ray) sensor inputs wired in sim.rs: energy, heading, heard signal,
/// body size, kin similarity, temperature, pheromone, light, niche level, food here.
pub const NUM_SCALAR_INPUTS: usize = 10;
/// Brain outputs read by sim.rs: turn, speed, eat, attack, signal, share, construct.
pub const NUM_BRAIN_OUTPUTS: usize = 7;

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
    pub pheromones: PheromoneConfig,
    #[serde(default)]
    pub daynight: DayNightConfig,
    #[serde(default)]
    pub elevation: ElevationConfig,
    #[serde(default)]
    pub activity_tracking: ActivityTrackingConfig,
    #[serde(default)]
    pub niche_construction: NicheConstructionConfig,
    #[serde(default)]
    pub epigenetics: EpigeneticsConfig,
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
    /// Seed of the drought schedule. Droughts draw from their own random stream,
    /// so when and where they fall depends only on this seed and the settings
    /// above, never on what the organisms do. Omit it to derive the schedule from
    /// `world.seed`. Set it to replay one schedule across different world seeds,
    /// or to vary the schedule while keeping the founding population fixed.
    pub schedule_seed: Option<u64>,
}

impl Default for DroughtConfig {
    fn default() -> Self {
        DroughtConfig {
            spawn_rate: 0.0,
            duration: 3000,
            radius: 150.0,
            severity: 0.05,
            max_active: 3,
            schedule_seed: None,
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
    /// Speed multiplier on ticks when the creature is in feeding mode (eat output > 0).
    /// Handling food and searching for it compete for the same time (Holling 1959;
    /// Stephens & Krebs 1986), so a creature that always tries to eat always moves
    /// slowly. 1.0 disables the trade-off (the pre-2026-11-15 rule).
    #[serde(default = "default_feeding_speed_factor")]
    pub feeding_speed_factor: f64,
    pub vision: VisionConfig,
}

fn default_feeding_speed_factor() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionConfig {
    pub ray_count: usize,
    pub fov: f64,
    pub range: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Energy parameters.
///
/// Brain cost is split into per-neuron and per-connection components following
/// the neuroscience finding that synaptic (connection) cost dominates soma
/// (neuron) cost by 50-85x in mammalian grey matter.
///
/// Citations:
/// - Attwell & Laughlin (2001) "An Energy Budget for Signaling in the Grey Matter"
///   *J. Cerebral Blood Flow & Metabolism* 21(10):1133-1145
/// - Laughlin et al. (1998) "The Metabolic Cost of Neural Information"
///   *Nature Neuroscience* 1(1):36-41
pub struct EnergyConfig {
    pub starting: f64,
    pub maximum: f64,
    pub base_metabolism: f64,
    pub movement_cost: f64,
    /// Per-neuron (soma) cost per tick. Small — cell bodies are metabolically cheap.
    pub brain_cost: f64,
    /// Per-connection (synapse) cost per tick. Should be ~5-10x brain_cost.
    /// This is the primary lever for constraining network complexity.
    /// Citation: Attwell & Laughlin (2001) — synapses dominate neural energy budget.
    #[serde(default = "default_connection_cost")]
    pub connection_cost: f64,
    /// Activity-dependent neural metabolism: cost per unit of |weight × activation|.
    /// Real brains spend ~80% of energy on active signaling, not maintenance.
    /// Selects for sparse coding — brains that activate few connections are cheaper.
    /// Citation: Attwell & Laughlin (2001), Niven & Laughlin (2008), Lennie (2003).
    #[serde(default)]
    pub activity_weight_cost: f64,
}

fn default_connection_cost() -> f64 {
    0.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproductionConfig {
    pub threshold: f64,
    pub parent_cost: f64,
    pub child_energy: f64,
    /// Enable sexual reproduction with NEAT crossover (Maynard Smith 1978).
    #[serde(default)]
    pub sexual: bool,
    /// Max distance to find a mate (world units).
    #[serde(default = "default_mate_range")]
    pub mate_range: f64,
    /// Probability of crossover vs asexual clone when mate is available.
    #[serde(default = "default_crossover_rate")]
    pub crossover_rate: f64,
    /// Fitter parent contributes disjoint/excess genes (NEAT §3.2).
    #[serde(default)]
    pub fitness_bias: bool,
    /// Energy the *mate* pays for a sexual birth. `None` (key omitted) = same as
    /// `parent_cost`, so a sexually produced offspring costs the pair twice what an
    /// asexual offspring costs its single parent while carrying only half of each
    /// parent's genes — the twofold cost of sex.
    /// Citation: Maynard Smith (1978) "The Evolution of Sex" Cambridge UP
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mate_cost: Option<f64>,
    /// A potential mate must hold at least `threshold × mate_energy_fraction` energy.
    /// (Was hard-coded 0.5 before Phase 7.5.)
    #[serde(default = "default_mate_energy_fraction")]
    pub mate_energy_fraction: f64,
}

impl ReproductionConfig {
    /// Effective energy cost paid by the mate in a sexual birth.
    pub fn effective_mate_cost(&self) -> f64 {
        self.mate_cost.unwrap_or(self.parent_cost)
    }
}

fn default_mate_energy_fraction() -> f64 {
    0.5
}

fn default_mate_range() -> f64 {
    15.0
}
fn default_crossover_rate() -> f64 {
    0.7
}

/// Mutation parameters.
///
/// Structural-mutation probabilities are per offspring (one Bernoulli draw each).
/// Defaults reproduce the pre-Phase-7.5 hard-coded values exactly.
/// Citation: Stanley & Miikkulainen (2002) — NEAT structural mutations;
/// Dolson & Park (2012) — prune rate should exceed add rate to curb bloat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationConfig {
    pub weight_probability: f64,
    pub weight_step: f64,
    /// Probability of an add-connection mutation per offspring.
    #[serde(default = "default_add_connection_probability")]
    pub add_connection_probability: f64,
    /// Probability of an add-node (connection split) mutation per offspring.
    #[serde(default = "default_add_node_probability")]
    pub add_node_probability: f64,
    /// Probability of toggling one connection's enabled flag per offspring.
    #[serde(default = "default_toggle_probability")]
    pub toggle_probability: f64,
    /// Probability of deleting one connection per offspring.
    #[serde(default = "default_prune_probability")]
    pub prune_probability: f64,
    /// Per-trait probability of mutating each scalar trait (mutation rate/step,
    /// aging rate, body size, preferred temperature, diet preference).
    #[serde(default = "default_trait_probability")]
    pub trait_probability: f64,
}

fn default_add_connection_probability() -> f64 {
    0.02
}
fn default_add_node_probability() -> f64 {
    0.01
}
fn default_toggle_probability() -> f64 {
    0.01
}
fn default_prune_probability() -> f64 {
    0.02
}
fn default_trait_probability() -> f64 {
    0.05
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
    /// Energy cost to the attacker per strike, paid only when a target is in range.
    /// The energetic cost of predation lies in the pursuit and strike, not in the
    /// readiness to attack (Holling 1959; Scantlebury et al. 2014).
    pub attack_cost: f64,
    /// Energy cost per tick of an attack attempt with no target in range.
    /// Default 0: an attack that never happens costs nothing. Set to `attack_cost`
    /// to reproduce the pre-2026-11-15 rule, under which predation went extinct.
    #[serde(default)]
    pub idle_attack_cost: f64,
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
    /// Fraction of the donated energy the recipient actually receives:
    /// recipient gets `share_amount * transfer_efficiency`. Must be in (0, 1] so
    /// sharing can never create energy (food is the only energy source).
    ///
    /// Hamilton's rB > C does NOT need energy to be created: B and C are fitness
    /// effects, and a unit of energy is worth more to a starving recipient (near
    /// death or below the reproduction threshold) than to a well-fed donor. The old
    /// `benefit_multiplier = 1.5` let two neighbours net +2.5 energy/tick each by
    /// sharing back and forth: a free-energy exploit, not kin selection.
    /// The legacy key `benefit_multiplier` is still accepted but must also be <= 1.
    /// Citation: Hamilton (1964) "The Genetical Evolution of Social Behaviour"
    #[serde(default = "default_transfer_efficiency", alias = "benefit_multiplier")]
    pub transfer_efficiency: f64,
}

fn default_transfer_efficiency() -> f64 {
    0.9
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

/// Open-ended evolution activity tracking — Bedau & Packard (1991), Dolson et al. (2019).
///
/// Tracks structural novelty rate and cumulative activity D(t).
/// If D(t) grows unboundedly, evolution is open-ended.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ActivityTrackingConfig {
    pub enabled: bool,
    /// Compute activity stats every N ticks (amortizes hashing cost).
    pub window_size: u64,
    /// Lineage-diversity window in ticks. Every `lineage_window` ticks the living
    /// organisms become clade founders; `lineage_diversity` reports how many of the
    /// organisms alive one window ago still have living descendants (or are alive).
    /// Unlike founder (gen-0) counts this does not trivially coalesce to 1.
    /// Always computed (independent of `enabled`); no RNG, no effect on dynamics.
    pub lineage_window: u64,
    /// Topology-hash version: 2 = order-independent commutative hash (default),
    /// 1 = legacy order-dependent FNV chain (for comparing with old runs).
    pub hash_version: u32,
}

/// Niche construction — organisms modify their local environment.
///
/// Positive construction deposits material that boosts local food regrowth;
/// negative construction degrades it. Material decays over time.
/// This creates ecological inheritance: offspring inherit a modified environment.
///
/// Citation: Odling-Smee, F.J., Laland, K.N. & Feldman, M.W. (2003)
///   "Niche Construction: The Neglected Process in Evolution" Princeton UP
/// Citation: Laland, K.N. et al. (1999) "Evolutionary consequences of niche
///   construction" PNAS 96:10242-10247
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NicheConstructionConfig {
    pub enabled: bool,
    /// Rate of material deposit/removal per tick when construct output is active.
    pub deposit_rate: f64,
    /// Multiplicative decay per tick (0.999 = slow decay, 0.99 = fast decay).
    pub decay_rate: f64,
    /// Maximum construction level per cell.
    pub max_level: f64,
    /// How much construction level boosts food regrowth (multiplier per unit level).
    pub regrowth_bonus: f64,
    /// Energy cost per unit of construction activity.
    pub energy_cost: f64,
    /// Dead zone: |construct output| must exceed this to build/degrade. Above it the
    /// intensity is rescaled to (|output| − threshold) / (1 − threshold), so effort and
    /// cost rise smoothly from zero. With the old value (0.01) a tanh output was almost
    /// never inside the dead zone, so every creature paid the construction cost on
    /// nearly every tick. The default (0.2) is a modelling choice: roughly two thirds
    /// of random initial brains abstain and one third build or degrade, so abstaining
    /// is free and there is still variation for selection to act on.
    pub activation_threshold: f64,
}

/// Epigenetic inheritance — heritable non-genetic methylation marks.
///
/// Under stress (low energy), active connections accumulate methylation that
/// partially silences them. Methylation is inherited with noise, creating a
/// Lamarckian channel that lets offspring pre-adapt to parental conditions.
///
/// Citation: Jablonka, E. & Lamb, M.J. (2005) "Evolution in Four Dimensions"
/// Citation: Heard, E. & Martienssen, R.A. (2014) "Transgenerational Epigenetic
///   Inheritance" Cell 157(1):95-109
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EpigeneticsConfig {
    pub enabled: bool,
    /// An organism is "stressed" when its energy is below
    /// `stress_repro_fraction × reproduction.threshold`, i.e. when its reserves are
    /// far below what it needs to breed (state-dependent life history). Must stay
    /// below `child_energy / threshold` or every newborn is born stressed.
    /// Replaces the old `stress_threshold` (fraction of energy.maximum), which
    /// classified every newborn as stressed; that key is now ignored if present.
    /// Citation: McNamara & Houston (1996) Nature 380:215-221;
    /// Wingfield et al. (1998) Am. Zool. 38:191-206 (emergency life-history stage).
    pub stress_repro_fraction: f64,
    /// DEPRECATED — ignored by the simulation since Phase 7.5 (it measured stress as
    /// energy / energy.maximum, which made every newborn "stressed"). Kept only so
    /// downstream code that still reads it compiles; use
    /// [`Config::epigenetic_stress_energy`] instead. Not serialized.
    #[serde(skip_serializing)]
    pub stress_threshold: f64,
    /// Maximum methylation gain per tick under stress.
    pub methylation_rate: f64,
    /// Passive demethylation per tick when not stressed.
    pub demethylation_rate: f64,
    /// Gaussian σ of epimutation noise added to *existing* marks (m > 0) at
    /// inheritance. Unmarked sites stay unmarked. 0 = faithful inheritance.
    pub inheritance_noise: f64,
    /// Re-apply effective weights `w·(1−m)` to the living brain every N ticks of
    /// age (1 = every tick). Within-lifetime plasticity: methylation silences
    /// expression in the soma, not only in offspring.
    /// Citation: Jablonka & Lamb (2005); Day & Sweatt (2010) Nat. Neurosci. 13:1319.
    pub brain_update_interval: u64,
}

impl Default for EpigeneticsConfig {
    fn default() -> Self {
        EpigeneticsConfig {
            enabled: false,
            stress_repro_fraction: 0.25,
            stress_threshold: 0.3,
            methylation_rate: 0.005,
            demethylation_rate: 0.001,
            inheritance_noise: 0.02,
            brain_update_interval: 1,
        }
    }
}

impl Default for NicheConstructionConfig {
    fn default() -> Self {
        NicheConstructionConfig {
            enabled: false,
            deposit_rate: 0.05,
            decay_rate: 0.999,
            max_level: 1.0,
            regrowth_bonus: 0.5,
            energy_cost: 0.5,
            activation_threshold: 0.2,
        }
    }
}

impl Default for ActivityTrackingConfig {
    fn default() -> Self {
        ActivityTrackingConfig {
            enabled: false,
            window_size: 100,
            lineage_window: 1000,
            hash_version: 2,
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
    /// Absolute energy below which an organism counts as epigenetically stressed:
    /// `epigenetics.stress_repro_fraction × reproduction.threshold`.
    pub fn epigenetic_stress_energy(&self) -> f64 {
        self.epigenetics.stress_repro_fraction * self.reproduction.threshold
    }

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
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate again after programmatic sweep overrides, before allocating a world.
        fn finite_tree(value: &toml::Value, path: &str) -> Result<(), ConfigError> {
            match value {
                toml::Value::Float(v) if !v.is_finite() => {
                    return Err(ConfigError::Invalid(format!("{path} must be finite")));
                }
                toml::Value::Table(fields) => {
                    for (key, value) in fields {
                        finite_tree(value, &format!("{path}.{key}"))?;
                    }
                }
                toml::Value::Array(values) => {
                    for (i, value) in values.iter().enumerate() {
                        finite_tree(value, &format!("{path}[{i}]"))?;
                    }
                }
                _ => {}
            }
            Ok(())
        }
        let serialized = toml::Value::try_from(self)
            .map_err(|e| ConfigError::Invalid(format!("cannot validate config: {e}")))?;
        finite_tree(&serialized, "config")?;
        for (name, value) in [
            ("food.max_per_cell", self.food.max_per_cell),
            ("food.regrowth_rate", self.food.regrowth_rate),
            ("food.energy_per_food", self.food.energy_per_food),
            (
                "food.fruit.energy_per_fruit",
                self.food.fruit.energy_per_fruit,
            ),
            (
                "food.fruit.biome_rate_multiplier",
                self.food.fruit.biome_rate_multiplier,
            ),
            ("creature.max_speed", self.creature.max_speed),
            ("creature.max_turn", self.creature.max_turn),
            ("energy.base_metabolism", self.energy.base_metabolism),
            ("energy.movement_cost", self.energy.movement_cost),
            ("energy.brain_cost", self.energy.brain_cost),
            ("energy.connection_cost", self.energy.connection_cost),
            (
                "energy.activity_weight_cost",
                self.energy.activity_weight_cost,
            ),
            ("reproduction.parent_cost", self.reproduction.parent_cost),
            ("reproduction.child_energy", self.reproduction.child_energy),
            ("reproduction.mate_range", self.reproduction.mate_range),
            ("aging.initial_senescence", self.aging.initial_senescence),
            ("aging.aging_rate", self.aging.aging_rate),
            ("combat.attack_range", self.combat.attack_range),
            ("combat.attack_damage", self.combat.attack_damage),
            ("combat.attack_cost", self.combat.attack_cost),
            ("combat.idle_attack_cost", self.combat.idle_attack_cost),
            ("sharing.share_range", self.sharing.share_range),
            ("sharing.share_amount", self.sharing.share_amount),
            (
                "density_pressure.cost_per_neighbor",
                self.density_pressure.cost_per_neighbor,
            ),
            ("temperature.cost_factor", self.temperature.cost_factor),
            ("pheromones.deposit_rate", self.pheromones.deposit_rate),
            ("elevation.slope_factor", self.elevation.slope_factor),
            (
                "niche_construction.deposit_rate",
                self.niche_construction.deposit_rate,
            ),
            (
                "niche_construction.regrowth_bonus",
                self.niche_construction.regrowth_bonus,
            ),
            (
                "niche_construction.energy_cost",
                self.niche_construction.energy_cost,
            ),
            (
                "epigenetics.inheritance_noise",
                self.epigenetics.inheritance_noise,
            ),
        ] {
            if value < 0.0 {
                return Err(ConfigError::Invalid(format!("{name} must be nonnegative")));
            }
        }
        for (name, value) in [
            ("world.width", self.world.width),
            ("world.height", self.world.height),
            ("food.cell_size", self.food.cell_size),
            ("creature.body_radius", self.creature.body_radius),
            ("creature.vision.range", self.creature.vision.range),
            ("creature.vision.fov", self.creature.vision.fov),
            ("energy.maximum", self.energy.maximum),
            ("reproduction.threshold", self.reproduction.threshold),
            ("mutation.weight_step", self.mutation.weight_step),
            ("food.seasons.period", self.food.seasons.period),
            ("food.droughts.radius", self.food.droughts.radius),
            ("daynight.period", self.daynight.period),
            ("pheromones.max_level", self.pheromones.max_level),
            (
                "niche_construction.max_level",
                self.niche_construction.max_level,
            ),
            (
                "speciation.fixed_threshold",
                self.speciation.fixed_threshold,
            ),
            ("speciation.min_threshold", self.speciation.min_threshold),
            ("speciation.max_threshold", self.speciation.max_threshold),
            ("speciation.factor", self.speciation.factor),
        ] {
            if value <= 0.0 {
                return Err(ConfigError::Invalid(format!("{name} must be positive")));
            }
        }
        for (name, value) in [
            (
                "mutation.weight_probability",
                self.mutation.weight_probability,
            ),
            (
                "mutation.add_connection_probability",
                self.mutation.add_connection_probability,
            ),
            (
                "mutation.add_node_probability",
                self.mutation.add_node_probability,
            ),
            (
                "mutation.toggle_probability",
                self.mutation.toggle_probability,
            ),
            (
                "mutation.prune_probability",
                self.mutation.prune_probability,
            ),
            (
                "mutation.trait_probability",
                self.mutation.trait_probability,
            ),
            (
                "reproduction.crossover_rate",
                self.reproduction.crossover_rate,
            ),
            (
                "reproduction.mate_energy_fraction",
                self.reproduction.mate_energy_fraction,
            ),
            ("aging.extrinsic_mortality", self.aging.extrinsic_mortality),
            ("food.seasons.amplitude", self.food.seasons.amplitude),
            ("food.droughts.spawn_rate", self.food.droughts.spawn_rate),
            ("food.droughts.severity", self.food.droughts.severity),
            ("pheromones.decay_rate", self.pheromones.decay_rate),
            (
                "niche_construction.decay_rate",
                self.niche_construction.decay_rate,
            ),
            (
                "epigenetics.stress_repro_fraction",
                self.epigenetics.stress_repro_fraction,
            ),
            (
                "epigenetics.methylation_rate",
                self.epigenetics.methylation_rate,
            ),
            (
                "epigenetics.demethylation_rate",
                self.epigenetics.demethylation_rate,
            ),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(ConfigError::Invalid(format!("{name} must be in [0, 1]")));
            }
        }
        if self.creature.hard_cap == 0 || self.creature.initial_count > self.creature.hard_cap {
            return Err(ConfigError::Invalid(
                "initial_count must not exceed a positive hard_cap".into(),
            ));
        }
        if self.creature.max_age == 0
            || self.food.droughts.duration == 0
            || self.activity_tracking.window_size == 0
            || self.activity_tracking.lineage_window == 0
            || self.epigenetics.brain_update_interval == 0
            || self.speciation.sample_size < 2
        {
            return Err(ConfigError::Invalid("age, duration and update intervals must be positive; speciation.sample_size must be >= 2".into()));
        }
        if self.energy.starting > self.energy.maximum
            || self.reproduction.child_energy > self.energy.maximum
            || self.reproduction.parent_cost > self.reproduction.threshold
        {
            return Err(ConfigError::Invalid("starting/child energy must not exceed maximum; parent_cost must not exceed threshold".into()));
        }
        if self.creature.max_turn > std::f64::consts::TAU
            || self.creature.vision.fov > std::f64::consts::TAU
        {
            return Err(ConfigError::Invalid(
                "max_turn and vision.fov must not exceed 2π".into(),
            ));
        }
        if self.speciation.min_threshold > self.speciation.max_threshold
            || !(1..=2).contains(&self.activity_tracking.hash_version)
        {
            return Err(ConfigError::Invalid(
                "invalid species threshold bounds or activity hash_version (expected 1 or 2)"
                    .into(),
            ));
        }
        let total_nodes = self
            .brain
            .inputs
            .checked_add(self.brain.outputs)
            .and_then(|n| n.checked_add(self.brain.hidden))
            .and_then(|n| n.checked_add(1));
        if total_nodes.is_none_or(|n| n > crate::brain::MAX_NODES)
            || self.brain.inputs > crate::brain::MAX_INPUTS
            || self.brain.outputs > crate::brain::MAX_OUTPUTS
        {
            return Err(ConfigError::Invalid(
                "initial brain exceeds fixed node/input/output capacity".into(),
            ));
        }
        for biome in &self.food.biomes {
            if biome.radius <= 0.0 || biome.regrowth_multiplier < 0.0 {
                return Err(ConfigError::Invalid(
                    "biome radius must be positive and regrowth_multiplier nonnegative".into(),
                ));
            }
        }

        if !(self.sharing.transfer_efficiency > 0.0 && self.sharing.transfer_efficiency <= 1.0) {
            return Err(ConfigError::Invalid(format!(
                "sharing.transfer_efficiency (or legacy benefit_multiplier) must be in (0, 1] \
                 so sharing conserves energy; got {}",
                self.sharing.transfer_efficiency
            )));
        }
        if self.reproduction.child_energy > self.reproduction.parent_cost {
            return Err(ConfigError::Invalid(format!(
                "reproduction.child_energy ({}) must not exceed parent_cost ({}): a birth \
                 cannot create energy",
                self.reproduction.child_energy, self.reproduction.parent_cost
            )));
        }
        if !(self.creature.feeding_speed_factor > 0.0 && self.creature.feeding_speed_factor <= 1.0)
        {
            return Err(ConfigError::Invalid(format!(
                "creature.feeding_speed_factor must be in (0, 1]; got {}",
                self.creature.feeding_speed_factor
            )));
        }
        if !(0.0..1.0).contains(&self.niche_construction.activation_threshold) {
            return Err(ConfigError::Invalid(format!(
                "niche_construction.activation_threshold must be in [0, 1); got {}",
                self.niche_construction.activation_threshold
            )));
        }
        if self.combat.idle_attack_cost < 0.0 || self.combat.attack_cost < 0.0 {
            return Err(ConfigError::Invalid(
                "combat.attack_cost and combat.idle_attack_cost must be >= 0".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.combat.energy_transfer_ratio) {
            return Err(ConfigError::Invalid(format!(
                "combat.energy_transfer_ratio must be in [0, 1] (a predator cannot gain more \
                 energy than the prey loses); got {}",
                self.combat.energy_transfer_ratio
            )));
        }
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
        // The sensor/actuator layout is fixed by sim.rs: 2 values per vision ray
        // + 9 scalar senses, and 7 actions. A mismatch silently miswires the brain
        // (e.g. an old 15/6 config reads a hidden node as `construct`).
        if self.creature.vision.ray_count == 0 || self.creature.vision.ray_count > MAX_RAYS {
            return Err(ConfigError::Invalid(format!(
                "creature.vision.ray_count must be in 1..={MAX_RAYS}"
            )));
        }
        let expected_inputs = 2 * self.creature.vision.ray_count + NUM_SCALAR_INPUTS;
        if self.brain.inputs != expected_inputs {
            return Err(ConfigError::Invalid(format!(
                "brain.inputs = {} but the simulation provides {} sensor inputs \
                 (2 × ray_count {} + {} scalar senses); set brain.inputs = {}",
                self.brain.inputs,
                expected_inputs,
                self.creature.vision.ray_count,
                NUM_SCALAR_INPUTS,
                expected_inputs
            )));
        }
        if self.brain.outputs != NUM_BRAIN_OUTPUTS {
            return Err(ConfigError::Invalid(format!(
                "brain.outputs = {} but the simulation reads {} actions \
                 (turn, speed, eat, attack, signal, share, construct); set brain.outputs = {}",
                self.brain.outputs, NUM_BRAIN_OUTPUTS, NUM_BRAIN_OUTPUTS
            )));
        }
        if self.epigenetics.enabled {
            let stress_level = self.epigenetic_stress_energy();
            if stress_level >= self.reproduction.child_energy {
                return Err(ConfigError::Invalid(format!(
                    "epigenetics stress level (stress_repro_fraction × threshold = {stress_level}) \
                     must be below reproduction.child_energy ({}) or every newborn is stressed",
                    self.reproduction.child_energy
                )));
            }
            if self.epigenetics.inheritance_noise < 0.0 {
                return Err(ConfigError::Invalid(
                    "epigenetics.inheritance_noise must be >= 0".into(),
                ));
            }
        }
        if let Some(mc) = self.reproduction.mate_cost
            && mc < 0.0
        {
            return Err(ConfigError::Invalid(
                "reproduction.mate_cost must be >= 0".into(),
            ));
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

#[cfg(test)]
mod validation_tests {
    use super::*;
    fn config() -> Config {
        Config::parse(include_str!("../../../configs/default.toml")).unwrap()
    }

    #[test]
    fn rejects_nonfinite_costs_and_oversized_brains() {
        let mut cfg = config();
        cfg.energy.connection_cost = f64::NAN;
        assert!(cfg.validate().unwrap_err().to_string().contains("finite"));
        cfg = config();
        cfg.brain.hidden = crate::brain::MAX_NODES;
        assert!(cfg.validate().unwrap_err().to_string().contains("capacity"));
    }

    #[test]
    fn rejects_invalid_resource_cost_probability_and_interval() {
        for field in ["cost", "probability", "interval", "capacity"] {
            let mut cfg = config();
            match field {
                "cost" => cfg.energy.base_metabolism = -1.0,
                "probability" => cfg.food.droughts.spawn_rate = 1.01,
                "interval" => cfg.epigenetics.brain_update_interval = 0,
                _ => cfg.creature.hard_cap = cfg.creature.initial_count - 1,
            }
            assert!(cfg.validate().is_err(), "{field}");
        }
    }

    #[test]
    fn allows_no_food_and_unreachable_reproduction_controls() {
        let mut cfg = config();
        cfg.food.max_per_cell = 0.0;
        cfg.food.regrowth_rate = 0.0;
        cfg.epigenetics.enabled = false;
        cfg.reproduction.threshold = cfg.energy.maximum * 10.0;
        assert!(cfg.validate().is_ok());
    }
}
