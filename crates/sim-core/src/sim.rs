//! The simulation: state + deterministic tick loop.
//!
//! Each tick follows this pipeline:
//! 1. Rebuild spatial index
//! 2. For each organism: sense → think (parallel, read-only, no RNG)
//! 3. Act: turn, move, eat, metabolize, epigenetic methylation (+ brain refresh)
//!    3b. Pheromone deposit/decay; niche construction deposit/decay
//! 4. Combat
//! 5. Energy sharing
//! 6. Reproduce (asexual or sexual; both parents pay in sexual births)
//! 7. Die (energy ≤ 0, max age, Gompertz-Makeham)
//! 8. Droughts spawn/expire
//! 9. Regrow food (biome × season × drought × niche-construction multiplier)
//! 10. Stats (species, trophic, entropy, OEE activity, lineage diversity)
//!
//! The RNG is seeded, so identical config + seed = identical run.

use crate::config::Config;
use crate::genome::Genome;
use crate::organism::{Organism, OrganismId};
use crate::world::{
    ConstructionGrid, ElevationMap, FoodGrid, PheromoneGrid, SpatialGrid, wrap_coord,
    wraparound_delta,
};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Maximum number of vision rays (for stack allocation). Enforced by `Config::validate`.
use crate::config::MAX_RAYS;

/// Number of recent ticks to track per-species population for trend detection.
const SPECIES_HISTORY_LEN: usize = 50;

/// Complete simulation state. Owns all organisms, the world, and the RNG.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Simulation {
    pub config: Config,
    pub tick: u64,
    pub organisms: Vec<Organism>,
    pub food_grid: FoodGrid,
    /// Pheromone layer — stigmergic indirect communication (Wilson 1975).
    /// Only actively updated when `config.pheromones.enabled`.
    pub pheromone_grid: PheromoneGrid,
    /// Elevation map — slope movement penalty (MacArthur & Wilson 1967).
    /// Only active when `config.elevation.enabled`.
    pub elevation_map: ElevationMap,
    /// Niche construction grid — organisms modify local environment.
    /// Citation: Odling-Smee, Laland & Feldman (2003)
    pub construction_grid: ConstructionGrid,
    pub spatial_grid: SpatialGrid,
    pub rng: ChaCha8Rng,
    /// Random stream for the environment (drought timing and placement), kept
    /// apart from `rng` so that organism events cannot shift the schedule. Two
    /// runs with the same drought seed and settings see the same droughts even if
    /// a feature is switched off in one of them (Glasserman & Yao 1992).
    pub env_rng: ChaCha8Rng,
    /// Population-wide record of structural innovations (node identity for NEAT
    /// crossover). See [`crate::genome::Innovations`].
    pub innovations: crate::genome::Innovations,
    next_id: OrganismId,

    // Per-tick stats (updated at end of each tick)
    #[serde(with = "crate::checkpoint::tick_stats")]
    pub stats: TickStats,

    // Reusable buffers to avoid allocation in the hot loop
    positions: Vec<(f64, f64, bool)>,
    actions: Vec<Action>,

    /// Per-species population history ring buffer for trend detection.
    /// Key: species representative index, Value: recent population counts.
    species_history: Vec<Vec<usize>>,
    /// Representative genomes from last tick for species continuity.
    species_representatives: Vec<crate::genome::Genome>,
    /// Open-ended evolution activity tracker.
    /// Citation: Bedau & Packard (1991), Dolson et al. (2019)
    activity_tracker: crate::activity::ActivityTracker,
    /// Neutral shadow population for OEE measurement (Bedau, Snyder & Packard 1998).
    /// None when `activity_tracking.shadow = false`.
    shadow: Option<crate::shadow::ShadowPopulation>,
    /// Reusable per-tick buffer: organism already took part in a sexual mating this tick.
    mated_buf: Vec<bool>,
    /// Reusable per-tick buffer: this tick's brain activations, `MAX_NODES` per
    /// organism. The parallel think phase writes here (each organism its own
    /// chunk) so it can stay read-only on the organisms; the sequential phase then
    /// copies a chunk into `Organism::memory` for brains that have recurrent
    /// connections. Transient, so it is not part of a checkpoint.
    #[serde(skip)]
    activations: Vec<f64>,
    /// Lineage diversity measured at the most recent `lineage_window` checkpoint.
    lineage_diversity_cached: usize,
}

/// Per-species aggregate statistics.
///
/// Computed each tick via NEAT-style compatibility distance clustering.
/// Citation: Stanley & Miikkulainen (2002) "Evolving Neural Networks through
/// Augmenting Topologies" *Evolutionary Computation* 10(2):99-127
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SpeciesInfo {
    /// Species index (0-based, assigned during clustering).
    pub id: usize,
    /// Number of living members.
    pub population: usize,
    /// Mean hue of members — for color swatch display.
    pub mean_hue: f64,
    /// Mean energy across members.
    pub mean_energy: f64,
    /// Mean body size across members.
    pub mean_body_size: f64,
    /// Mean active brain connections.
    pub mean_connections: f64,
    /// Mean generation number.
    pub mean_generation: f64,
    /// Mean trophic level (1.0 = herbivore, 2.0 = predator).
    pub mean_trophic_level: f64,
    /// Fraction that are herbivores (predation < 25%).
    pub herbivore_fraction: f64,
    /// Fraction that are predators (predation > 50%).
    pub predator_fraction: f64,
    /// Population trend: positive = growing, negative = declining.
    /// Computed as (current - avg_recent) / avg_recent.
    pub trend: f64,
}

/// Summary statistics for the most recent tick.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TickStats {
    pub tick: u64,
    pub population: usize,
    pub births: usize,
    pub deaths: usize,
    pub total_births: u64,
    pub total_deaths: u64,
    pub attacks: usize,
    pub total_attacks: u64,
    pub mean_energy: f64,
    pub mean_brain_neurons: f64,
    pub species_count: usize,
    pub mean_generation: f64,
    pub shares: usize,
    pub total_shares: u64,
    pub cooperation_rate: f64,
    pub kin_share_ratio: f64,
    pub mean_trophic_level: f64,
    pub herbivore_count: usize,
    pub predator_count: usize,
    /// Mean brain output entropy [0, 1]. Higher = more varied behavior over time.
    /// Citation: Shannon (1948)
    pub mean_brain_entropy: f64,
    /// Behavioral diversity [0, 1]. Entropy of output distributions across the population.
    /// Higher = more diverse behavioral strategies in the population.
    pub behavioral_diversity: f64,
    /// Mean active connections per brain.
    pub mean_connections: f64,
    /// Mean number of nearby neighbors per organism (density pressure).
    /// Citation: Allee (1931)
    pub mean_crowding: f64,
    /// Current speciation compatibility distance threshold.
    /// Citation: Stanley & Miikkulainen (2002)
    pub speciation_threshold: f64,
    /// Mean activity-dependent brain metabolic cost per organism.
    /// Citation: Attwell & Laughlin (2001)
    pub mean_brain_activity: f64,
    /// Sexual reproduction births this tick (per-tick, not cumulative; see
    /// `total_sexual_births`). Citation: Maynard Smith (1978)
    pub sexual_births: u64,
    /// Asexual reproduction births this tick (per-tick; see `total_asexual_births`).
    pub asexual_births: u64,
    /// Number of active drought zones.
    pub active_droughts: usize,
    /// Novel structural topologies this window. Citation: Bedau & Packard (1991)
    pub novel_traits: u64,
    /// Distinct topologies currently alive.
    pub active_traits: u64,
    /// Mean cumulative activity Ā(t) over currently active (extant) topologies —
    /// Bedau, Snyder & Packard (1998). (Bedau's diversity D(t) is `active_traits`.)
    /// Updated every `activity_tracking.window_size` ticks.
    pub mean_cumulative_activity: f64,
    /// Maximum generation depth in the population.
    pub max_generation: u64,
    /// Windowed lineage diversity: of the organisms alive at the previous
    /// `lineage_window` checkpoint, how many had living descendants (or were alive)
    /// at the latest checkpoint. Before the first checkpoint equals `founder_lineages`.
    pub lineage_diversity: usize,
    /// Mean |construction level| across all cells (building and degrading both count).
    /// Citation: Odling-Smee, Laland & Feldman (2003)
    pub mean_construction_level: f64,
    /// Mean methylation level across all connections in the population.
    /// Citation: Jablonka & Lamb (2005)
    pub mean_methylation: f64,
    /// Cumulative sexual births since tick 0.
    pub total_sexual_births: u64,
    /// Cumulative asexual births since tick 0.
    pub total_asexual_births: u64,
    /// Distinct gen-0 root ancestors among living organisms (the old, monotonically
    /// non-increasing `lineage_diversity` definition).
    pub founder_lineages: usize,
    /// Signed mean construction level (net build minus degrade) across all cells.
    pub net_construction_level: f64,
    /// A_cum(t): total cumulative activity over currently active topologies.
    pub total_cumulative_activity: u64,
    /// Organisms ready to reproduce (energy >= threshold) this tick that were blocked
    /// only because the population was at `creature.hard_cap`. Should stay ~0 when the
    /// population is food-limited (Verhulst 1838 carrying capacity), not cap-limited.
    pub cap_blocked_births: u64,
    /// Cumulative `cap_blocked_births` since tick 0.
    pub total_cap_blocked_births: u64,
    // Organism-reserve accounting boundary: environmental regrowth is outside it.
    /// Total organism reserves at tick start; excludes environmental food.
    pub energy_before: f64,
    /// Total surviving organism reserves at tick end.
    pub energy_after: f64,
    /// Metabolizable algae energy harvested before reserve-cap clipping.
    pub energy_food_input: f64,
    /// Metabolizable fruit energy harvested before reserve-cap clipping.
    pub energy_fruit_input: f64,
    /// Size-scaled basal metabolic expenditure.
    pub energy_basal_cost: f64,
    /// Movement expenditure.
    pub energy_movement_cost: f64,
    /// Neuron and connection maintenance expenditure.
    pub energy_brain_structure_cost: f64,
    /// Activity-dependent neural expenditure.
    pub energy_brain_activity_cost: f64,
    /// Density-dependent expenditure.
    pub energy_crowding_cost: f64,
    /// Temperature mismatch expenditure.
    pub energy_temperature_cost: f64,
    /// Environmental construction expenditure.
    pub energy_construction_cost: f64,
    /// Strike plus idle attack expenditure.
    pub energy_attack_cost: f64,
    /// Prey reserve removed minus energy harvested by attackers.
    pub energy_combat_loss: f64,
    /// Donor reserve spent minus energy offered to recipients.
    pub energy_sharing_loss: f64,
    /// Reserve spent by initiating parents and sexual mates.
    pub energy_reproduction_cost: f64,
    /// Reserves introduced in newborns; balanced against reproductive expenditure.
    pub energy_offspring_input: f64,
    /// Signed reserves removed with dead organisms. Negative values remove overdrafts, not a biological energy source.
    pub energy_death_removal: f64,
    /// Food, combat and sharing gains discarded at the organism reserve cap.
    pub energy_cap_overflow: f64,
    /// Observed reserve change minus independently recorded net flows; should be near zero.
    pub energy_balance_residual: f64,
    /// Mean number of active recurrent connections per brain (0 without memory).
    pub mean_recurrent_connections: f64,
    /// Share of living creatures whose brain has at least one recurrent connection.
    pub recurrent_brain_fraction: f64,
    /// Mean diet_preference across the population [0=algae specialist, 1=fruit specialist].
    /// Tracks whether the population is diverging along the diet axis.
    /// Citation: Grant & Grant (2006); Schluter (2000) "Ecology of Adaptive Radiation"
    pub mean_diet_preference: f64,
    /// Population variance of diet_preference. Spread alone does not establish
    /// bimodality, character displacement or adaptive radiation.
    pub diet_variance: f64,
    /// Mean number of hidden neurons per brain (excludes fixed input/output nodes).
    /// Tracks how neural complexity evolves under different selection pressures.
    pub mean_hidden_nodes: f64,
    /// Median extant activity M(t) over currently alive topologies (Channon 2003).
    pub median_extant_activity: f64,
    // Shadow (neutral drift) OEE stats — None fields stay 0 when shadow is off.
    /// Shadow population size (should equal `population`; nonzero when shadow enabled).
    pub shadow_population: usize,
    /// Shadow novel traits this window (null model for `novel_traits`).
    pub shadow_novel_traits: u64,
    /// Shadow active traits right now (null model for `active_traits`).
    pub shadow_active_traits: u64,
    /// Shadow mean cumulative activity Ā(t) (null model for `mean_cumulative_activity`).
    pub shadow_mean_cumulative_activity: f64,
    /// Shadow median extant activity M(t) (null model for `median_extant_activity`).
    pub shadow_median_extant_activity: f64,
    /// Per-species breakdown with aggregate stats and trend.
    #[serde(skip)]
    pub species: Vec<SpeciesInfo>,
}

/// Independent per-tick flows across the organism-reserve boundary.
/// This is not a whole-world energy/thermodynamics model: environmental food
/// regrowth and digestion losses are outside this boundary. Death removal is
/// signed because the existing tick order permits temporary negative reserves.
#[derive(Default)]
struct EnergyLedger {
    before: f64,
    food_input: f64,
    fruit_input: f64,
    basal_cost: f64,
    movement_cost: f64,
    brain_structure_cost: f64,
    brain_activity_cost: f64,
    crowding_cost: f64,
    temperature_cost: f64,
    construction_cost: f64,
    attack_cost: f64,
    combat_loss: f64,
    sharing_loss: f64,
    reproduction_cost: f64,
    offspring_input: f64,
    death_removal: f64,
    cap_overflow: f64,
}

impl EnergyLedger {
    fn net_flow(&self) -> f64 {
        self.food_input + self.fruit_input + self.offspring_input
            - self.basal_cost
            - self.movement_cost
            - self.brain_structure_cost
            - self.brain_activity_cost
            - self.crowding_cost
            - self.temperature_cost
            - self.construction_cost
            - self.attack_cost
            - self.combat_loss
            - self.sharing_loss
            - self.reproduction_cost
            - self.death_removal
            - self.cap_overflow
    }

    fn tolerance(&self, after: f64) -> f64 {
        // Scale with absolute flows (not their cancelling net), and reserve sums.
        let magnitude = self.before.abs()
            + after.abs()
            + self.food_input.abs()
            + self.fruit_input.abs()
            + self.offspring_input.abs()
            + self.basal_cost.abs()
            + self.movement_cost.abs()
            + self.brain_structure_cost.abs()
            + self.brain_activity_cost.abs()
            + self.crowding_cost.abs()
            + self.temperature_cost.abs()
            + self.construction_cost.abs()
            + self.attack_cost.abs()
            + self.combat_loss.abs()
            + self.sharing_loss.abs()
            + self.reproduction_cost.abs()
            + self.death_removal.abs()
            + self.cap_overflow.abs();
        1e-9 * (1.0 + magnitude)
    }
}

/// Compact action output — one per organism per tick.
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
struct Action {
    turn: f64,
    speed: f64,
    eat: f64,
    attack: f64,
    signal: f64,
    share: f64,
    construct: f64,
    alive: bool,
    neighbor_count: u32,
    /// Diet-similarity-weighted neighbor count for the MacArthur (1972) competition
    /// kernel. Equals `neighbor_count` when `diet_competition_sigma = 0` (kernel off).
    /// When the kernel is active: each neighbor j contributes
    ///   exp(-(diet_i - diet_j)² / (2σ²))
    /// so same-niche neighbors impose full pressure, diet-distant ones less.
    weighted_neighbors: f32,
    brain_activity: f64,
}

impl Default for Action {
    fn default() -> Self {
        Action {
            turn: 0.0,
            speed: 0.0,
            eat: 0.0,
            attack: 0.0,
            signal: 0.0,
            share: 0.0,
            construct: 0.0,
            alive: false,
            neighbor_count: 0,
            weighted_neighbors: 0.0,
            brain_activity: 0.0,
        }
    }
}

/// ChaCha stream id used by the environment's random draws.
const ENVIRONMENT_STREAM: u64 = 1;

impl Simulation {
    /// Create a new simulation from config. Spawns the initial population.
    pub fn new(config: Config) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(config.world.seed);
        // The environment gets ChaCha stream 1 of its own seed. With no explicit
        // schedule seed it shares the world seed but still a different stream, so
        // it never overlaps the organisms' draws (stream 0).
        let mut env_rng = ChaCha8Rng::seed_from_u64(
            config
                .food
                .droughts
                .schedule_seed
                .unwrap_or(config.world.seed),
        );
        env_rng.set_stream(ENVIRONMENT_STREAM);
        let food_grid = FoodGrid::new(&config.world, &config.food, &config.food.fruit);

        // Pheromone grid uses the same cell size as the food grid for cache coherence.
        // Citation: Wilson (1975), Deneubourg et al. (1990)
        let pheromone_grid =
            PheromoneGrid::new(&config.world, config.food.cell_size, &config.pheromones);

        let construction_grid = ConstructionGrid::new(
            &config.world,
            config.food.cell_size,
            &config.niche_construction,
        );

        // Elevation map — multi-octave sine terrain.
        // Citation: MacArthur & Wilson (1967), Wright (1943)
        let elevation_map = ElevationMap::new(&config.world, &config.elevation);

        // Spatial grid cell size = vision range so neighbor queries check 3×3 cells
        let spatial_cell_size = config.creature.vision.range;
        let spatial_grid = SpatialGrid::new(&config.world, spatial_cell_size);

        let cap = config.creature.hard_cap;
        let mut organisms = Vec::with_capacity(cap);
        let mut next_id: OrganismId = 0;

        for _ in 0..config.creature.initial_count {
            let genome = Genome::random(
                &config.brain,
                &config.mutation,
                &config.aging,
                &config.temperature,
                &config.food.fruit,
                &mut rng,
            );
            let brain = genome.build_brain();
            let org = Organism {
                id: next_id,
                pos_x: rng.gen_range(0.0..config.world.width),
                pos_y: rng.gen_range(0.0..config.world.height),
                heading: rng.gen_range(0.0..std::f64::consts::TAU),
                energy: config.energy.starting,
                age: 0,
                genome,
                brain,
                alive: true,
                parent_id: None,
                generation: 0,
                signal: 0.0,
                energy_from_food: 0.0,
                energy_from_attacks: 0.0,
                root_ancestor_id: next_id,
                birth_tick: 0,
                clade_id: next_id,
                output_bins: [[0.0; crate::organism::NUM_ENTROPY_BINS];
                    crate::organism::NUM_OUTPUTS],
                memory: Vec::new(),
            };
            organisms.push(org);
            next_id += 1;
        }

        let innovations = crate::genome::Innovations::new(&config.brain);
        let activity_tracker =
            crate::activity::ActivityTracker::new(config.activity_tracking.window_size);
        let shadow = if config.activity_tracking.shadow && config.activity_tracking.enabled {
            Some(crate::shadow::ShadowPopulation::new(
                organisms.iter().map(|o| o.genome.clone()),
                &config.activity_tracking,
                &config.brain,
                &config.mutation,
                &config.food.fruit,
                config.world.seed,
            ))
        } else {
            None
        };

        Simulation {
            config,
            tick: 0,
            organisms,
            food_grid,
            pheromone_grid,
            construction_grid,
            elevation_map,
            spatial_grid,
            rng,
            env_rng,
            innovations,
            next_id,
            stats: TickStats::default(),
            positions: Vec::with_capacity(cap),
            actions: Vec::with_capacity(cap),
            species_history: Vec::new(),
            species_representatives: Vec::new(),
            activity_tracker,
            shadow,
            mated_buf: Vec::with_capacity(cap),
            activations: Vec::new(),
            lineage_diversity_cached: 0,
        }
    }

    /// Advance the simulation by one tick.
    pub fn step(&mut self) {
        let mut ledger = EnergyLedger {
            before: self.organisms.iter().map(|o| o.energy).sum(),
            ..EnergyLedger::default()
        };
        let mut births = 0usize;
        let mut deaths = 0usize;

        let organism_count = self.organisms.len();

        // 1. Rebuild spatial index from position snapshot
        self.positions.clear();
        self.positions
            .extend(self.organisms.iter().map(|o| (o.pos_x, o.pos_y, o.alive)));
        self.spatial_grid.rebuild(&self.positions);

        // Pre-extract config values used in the inner loop to avoid repeated field access
        let ray_count = self.config.creature.vision.ray_count;
        let fov = self.config.creature.vision.fov;
        let range = self.config.creature.vision.range;
        let food_cell_size = self.config.food.cell_size;
        let world_w = self.config.world.width;
        let world_h = self.config.world.height;
        let max_energy = self.config.energy.maximum;
        let max_turn = self.config.creature.max_turn;
        let max_speed = self.config.creature.max_speed;
        let energy_per_food = self.config.food.energy_per_food;
        let feeding_speed_factor = self.config.creature.feeding_speed_factor;
        let base_metabolism = self.config.energy.base_metabolism;
        let movement_cost_factor = self.config.energy.movement_cost;
        let brain_cost_per_neuron = self.config.energy.brain_cost;
        let brain_cost_per_connection = self.config.energy.connection_cost;
        let activity_weight_cost = self.config.energy.activity_weight_cost;
        let density_cost_per_neighbor = self.config.density_pressure.cost_per_neighbor;
        let diet_competition_sigma = self.config.density_pressure.diet_competition_sigma;
        let temp_enabled = self.config.temperature.enabled;
        let temp_cost_factor = self.config.temperature.cost_factor;
        let fruit_enabled = self.config.food.fruit.enabled;
        let energy_per_fruit = self.config.food.fruit.energy_per_fruit;
        let diet_slope = self.config.food.fruit.diet_slope;
        let epigenetics_enabled = self.config.epigenetics.enabled;
        // Stress = reserves far below what is needed to breed (see EpigeneticsConfig).
        let epigenetics_stress_energy = self.config.epigenetic_stress_energy();
        let epigenetics_brain_interval = self.config.epigenetics.brain_update_interval.max(1);
        let epigenetics_methylation_rate = self.config.epigenetics.methylation_rate;
        let epigenetics_demethylation_rate = self.config.epigenetics.demethylation_rate;

        // Day/night light level — sinusoidal oscillation [0, 1].
        // L < 0.5 = night: vision range halved.
        // Brain input (input 18) wired in FOURTH step.
        // Citation: Kronfeld-Schor & Dayan (2003)
        let elevation_enabled = self.config.elevation.enabled;
        let daynight_enabled = self.config.daynight.enabled;
        let light_level = if daynight_enabled && self.config.daynight.period > 0.0 {
            let phase = std::f64::consts::TAU * self.tick as f64 / self.config.daynight.period;
            (1.0 + crate::math::sin(phase)) * 0.5
        } else {
            1.0
        };
        // Halve vision range at night; brain input deferred to FOURTH step
        let effective_range = if daynight_enabled && light_level < 0.5 {
            range * 0.5
        } else {
            range
        };
        let effective_range_inv = 1.0 / effective_range;

        let angle_step = if ray_count > 1 {
            fov / (ray_count - 1) as f64
        } else {
            0.0
        };
        let acceptance = if ray_count > 1 {
            (angle_step / 2.0).max(0.1)
        } else {
            0.2
        };

        // Precompute local ray direction unit vectors (rotated per organism's heading later)
        let mut ray_local = [(0.0f64, 0.0f64); MAX_RAYS];
        let half_fov = fov / 2.0;
        for (r, slot) in ray_local.iter_mut().enumerate().take(ray_count) {
            let local_angle = -half_fov + r as f64 * angle_step;
            *slot = (crate::math::cos(local_angle), crate::math::sin(local_angle));
        }
        // Precompute acceptance threshold as cosine for dot-product comparison
        let cos_acceptance = crate::math::cos(acceptance);

        // Capture effective vision range so the closure uses the day/night-adjusted value
        let vision_range = effective_range;
        let vision_range_inv = effective_range_inv;

        // 2. Sense → Think → Act (parallelized with rayon)
        //
        // This phase is read-only on organisms, food_grid, spatial_grid, and pheromone_grid.
        // No RNG is used, so parallel execution preserves determinism.
        // Each organism independently: cast vision rays → run brain forward pass → produce Action.
        let organisms = &self.organisms;
        let food_grid = &self.food_grid;
        let food_max_per_cell = self.food_grid.max_per_cell.max(f64::MIN_POSITIVE);
        let pheromone_grid = &self.pheromone_grid;
        let construction_grid = &self.construction_grid;
        let spatial_grid = &self.spatial_grid;

        // One activation chunk per organism; the buffer is reused across ticks.
        self.activations
            .resize(organism_count * crate::brain::MAX_NODES, 0.0);
        let activations = &mut self.activations;
        #[cfg(feature = "parallel")]
        let iter = activations
            .par_chunks_mut(crate::brain::MAX_NODES)
            .enumerate();
        #[cfg(not(feature = "parallel"))]
        let iter = activations.chunks_mut(crate::brain::MAX_NODES).enumerate();
        self.actions = iter
            .map(|(i, act)| {
                if !organisms[i].alive {
                    return Action::default();
                }

                let heading = organisms[i].heading;
                let ox = organisms[i].pos_x;
                let oy = organisms[i].pos_y;
                let cos_h = crate::math::cos(heading);
                let sin_h = crate::math::sin(heading);

                // Stack-allocated ray results
                let mut ray_types = [-1.0f64; MAX_RAYS];
                let mut closest_dists = [vision_range; MAX_RAYS];

                // Precompute world-space ray direction unit vectors for this organism
                let mut ray_dirs = [(0.0f64, 0.0f64); MAX_RAYS];
                for r in 0..ray_count {
                    let (lcos, lsin) = ray_local[r];
                    ray_dirs[r] = (cos_h * lcos - sin_h * lsin, sin_h * lcos + cos_h * lsin);
                }

                // Check food along rays (sample at cell_size intervals)
                // When fruit is enabled, distinguish types:
                //   0.0 = algae, -0.5 = fruit, 1.0 = creature, -1.0 = nothing
                for r in 0..ray_count {
                    let (ray_dx, ray_dy) = ray_dirs[r];
                    let mut d = food_cell_size;
                    while d < closest_dists[r] {
                        let sx = wrap_coord(ox + ray_dx * d, world_w);
                        let sy = wrap_coord(oy + ray_dy * d, world_h);
                        if food_grid.food_at(sx, sy) > 0.1 {
                            closest_dists[r] = d;
                            ray_types[r] = 0.0; // algae
                            break;
                        }
                        // Check fruit layer when enabled
                        if fruit_enabled && food_grid.fruit_at(sx, sy) > 0.1 {
                            closest_dists[r] = d;
                            ray_types[r] = -0.5; // fruit (distinct from algae and creature)
                            break;
                        }
                        d += food_cell_size;
                    }
                }

                // Check creatures — dot-product ray test (no atan2)
                // Also accumulate nearby signals and track nearest creature for kin recognition
                // Count neighbors for density pressure — Allee (1931)
                // MacArthur (1972) competition kernel: weight by diet similarity when σ > 0
                let range_sq = vision_range * vision_range;
                let mut signal_sum = 0.0f64;
                let mut signal_weight = 0.0f64;
                let mut nearest_dist_sq = f64::MAX;
                let mut nearest_hue = -1.0f64; // sentinel: no creature found
                let mut neighbor_count: u32 = 0;
                let mut weighted_neighbors: f32 = 0.0;
                let my_diet = organisms[i].genome.diet_preference;
                let kernel_inv_2sigma2 = if diet_competition_sigma > 0.0 {
                    1.0 / (2.0 * diet_competition_sigma * diet_competition_sigma)
                } else {
                    0.0
                };
                spatial_grid.for_each_nearby(ox, oy, |ni| {
                    if ni == i || !organisms[ni].alive {
                        return;
                    }
                    neighbor_count += 1;
                    let dx = wraparound_delta(ox, organisms[ni].pos_x, world_w);
                    let dy = wraparound_delta(oy, organisms[ni].pos_y, world_h);
                    let dist_sq = dx * dx + dy * dy;
                    // Diet-similarity weight for the competition kernel.
                    // When σ=0, kernel is off; all neighbors contribute equally (weight=1).
                    // Citation: MacArthur (1972); Chesson (2000)
                    let diet_weight = if kernel_inv_2sigma2 > 0.0 {
                        let d = my_diet - organisms[ni].genome.diet_preference;
                        crate::math::exp(-(d * d) * kernel_inv_2sigma2) as f32
                    } else {
                        1.0f32
                    };
                    weighted_neighbors += diet_weight;
                    if dist_sq > range_sq || dist_sq < 0.000001 {
                        return;
                    }
                    let dist = dist_sq.sqrt();
                    let inv_dist = 1.0 / dist;
                    let nx = dx * inv_dist;
                    let ny = dy * inv_dist;

                    // Accumulate signal: closer creatures have stronger influence
                    // Weight = 1/dist so nearby signals dominate
                    signal_sum += organisms[ni].signal * inv_dist;
                    signal_weight += inv_dist;

                    // Track nearest creature for kin recognition
                    if dist_sq < nearest_dist_sq {
                        nearest_dist_sq = dist_sq;
                        nearest_hue = organisms[ni].genome.color_hue;
                    }

                    for r in 0..ray_count {
                        if dist >= closest_dists[r] {
                            continue;
                        }
                        let (rdx, rdy) = ray_dirs[r];
                        let dot = rdx * nx + rdy * ny;
                        if dot >= cos_acceptance {
                            closest_dists[r] = dist;
                            ray_types[r] = 1.0; // creature
                        }
                    }
                });

                // Compute heard signal: distance-weighted average of nearby broadcasts
                let heard_signal = if signal_weight > 0.0 {
                    (signal_sum / signal_weight).clamp(-1.0, 1.0)
                } else {
                    0.0
                };

                // Kin similarity: how close is nearest creature's hue to ours?
                // Hue wraps around 360, so min angular distance ∈ [0, 180].
                // Normalized to [-1, 1]: 1 = identical hue (kin), -1 = opposite hue.
                // Hamilton (1964): kin recognition enables rB > C altruism.
                let kin_similarity = if nearest_hue >= 0.0 {
                    let own_hue = organisms[i].genome.color_hue;
                    let mut diff = (own_hue - nearest_hue).abs();
                    if diff > 180.0 {
                        diff = 360.0 - diff;
                    }
                    // diff ∈ [0, 180] → similarity ∈ [1, -1]
                    1.0 - diff / 90.0
                } else {
                    0.0 // no creature nearby
                };

                // Normalize ray distances
                let mut ray_distances = [0.0f64; MAX_RAYS];
                for r in 0..ray_count {
                    ray_distances[r] = closest_dists[r] * vision_range_inv;
                }

                // Build sensor inputs on the stack
                let mut inputs = [0.0f64; 64]; // MAX_INPUTS
                let mut idx = 0;
                for r in 0..ray_count {
                    inputs[idx] = ray_distances[r];
                    idx += 1;
                    inputs[idx] = ray_types[r];
                    idx += 1;
                }
                inputs[idx] = organisms[i].energy / max_energy;
                idx += 1;
                inputs[idx] = organisms[i].heading / std::f64::consts::PI - 1.0;
                idx += 1;
                inputs[idx] = heard_signal;
                idx += 1;
                // Body size normalized to [-1, 1]: (size - 1.75) / 1.25
                // Maps [0.5, 3.0] -> [-1, 1]
                inputs[idx] = (organisms[i].genome.body_size - 1.75) / 1.25;
                idx += 1;
                // Kin similarity to nearest creature [-1, 1]
                inputs[idx] = kin_similarity;
                idx += 1;

                // Input 16: local temperature [0, 1] — y/height (north=cold, south=warm)
                // 0.0 = cold (north), 1.0 = warm (south). Active regardless of temp_enabled
                // so the brain always has positional latitude information.
                // Citation: Bergmann (1847), Allen (1877)
                inputs[idx] = oy / world_h;
                idx += 1;

                // Input 17: local pheromone concentration [0, 1], normalized by max_level.
                // 0.0 when pheromone system is disabled (grid stays zeroed).
                // Citation: Wilson (1975), Deneubourg et al. (1990)
                inputs[idx] = pheromone_grid.level_at(ox, oy);
                idx += 1;

                // Input 18: current light level [0, 1] — 1.0 = full day, 0.0 = darkest night.
                // 1.0 when day/night system is disabled (always day).
                // Citation: Kronfeld-Schor & Dayan (2003)
                inputs[idx] = light_level;
                idx += 1;

                // Input 19: local niche construction level [-1, 1].
                // 0.0 when niche construction is disabled (grid stays zeroed).
                // Citation: Odling-Smee, Laland & Feldman (2003)
                inputs[idx] = construction_grid.level_at(ox, oy);
                idx += 1;

                // Input 20: food in the creature's own cell [0, 1] (algae + fruit, as a
                // fraction of one full cell). Vision rays start one cell out, so without
                // this the creature cannot tell whether it is standing on food. Models
                // contact chemoreception (taste on contact), e.g. Dethier (1976).
                let here = food_grid.food_at(ox, oy)
                    + if fruit_enabled {
                        food_grid.fruit_at(ox, oy)
                    } else {
                        0.0
                    };
                inputs[idx] = (here / food_max_per_cell).min(1.0);

                let brain_inputs = &inputs[..organisms[i].brain.inputs];
                // Recurrent connections read the organism's previous activations;
                // this tick's go into its own chunk of the shared buffer.
                let (output, brain_activity) =
                    organisms[i]
                        .brain
                        .forward_with_memory(brain_inputs, &organisms[i].memory, act);

                Action {
                    turn: output.turn,
                    speed: output.speed,
                    eat: output.eat,
                    attack: output.attack,
                    signal: output.signal,
                    share: output.share,
                    construct: output.construct,
                    alive: true,
                    neighbor_count,
                    weighted_neighbors,
                    brain_activity,
                }
            })
            .collect();

        // Compute mean crowding from neighbor counts (for stats)
        let mean_crowding = {
            let alive_count = self.actions.iter().filter(|a| a.alive).count();
            if alive_count > 0 {
                let total: u64 = self
                    .actions
                    .iter()
                    .filter(|a| a.alive)
                    .map(|a| a.neighbor_count as u64)
                    .sum();
                total as f64 / alive_count as f64
            } else {
                0.0
            }
        };

        // Compute mean brain activity (for stats)
        let mean_brain_activity = {
            let alive_count = self.actions.iter().filter(|a| a.alive).count();
            if alive_count > 0 {
                let total: f64 = self
                    .actions
                    .iter()
                    .filter(|a| a.alive)
                    .map(|a| a.brain_activity)
                    .sum();
                total / alive_count as f64
            } else {
                0.0
            }
        };

        // 3. Apply actions: movement, eating, metabolism
        for i in 0..organism_count {
            let action = self.actions[i];
            if !action.alive {
                continue;
            }
            let org = &mut self.organisms[i];

            // Memory: keep this tick's activations as the previous state for the
            // next one. Only brains with recurrent connections need it; the vector
            // is sized once, on the creature's first tick.
            if org.brain.has_memory() {
                let slots = org.brain.slot_count();
                let start = i * crate::brain::MAX_NODES;
                if org.memory.len() != slots {
                    org.memory.resize(slots, 0.0);
                }
                org.memory
                    .copy_from_slice(&self.activations[start..start + slots]);
            }

            // Turn
            org.heading += action.turn * max_turn;
            // Fast angle wrap (no fmod, just conditional)
            if org.heading >= std::f64::consts::TAU {
                org.heading -= std::f64::consts::TAU;
            } else if org.heading < 0.0 {
                org.heading += std::f64::consts::TAU;
            }

            // Update broadcast signal
            org.signal = action.signal;

            // Update activation histograms for brain entropy
            // Citation: Shannon (1948) — information-theoretic measure of brain activity
            {
                use crate::organism::{
                    ENTROPY_DECAY, NUM_ENTROPY_BINS, NUM_OUTPUTS, activation_to_bin,
                };
                let raw_outputs = [
                    action.turn,
                    action.speed * 2.0 - 1.0, // undo [0,1] → [-1,1] for binning
                    action.eat,
                    action.attack,
                    action.signal,
                    action.share,
                    action.construct,
                ];
                for (o, &raw_val) in raw_outputs.iter().enumerate().take(NUM_OUTPUTS) {
                    // Exponential decay of existing bin counts
                    for bin_val in org.output_bins[o].iter_mut().take(NUM_ENTROPY_BINS) {
                        *bin_val *= ENTROPY_DECAY;
                    }
                    // Increment current bin
                    let bin = activation_to_bin(raw_val);
                    org.output_bins[o][bin] += 1.0;
                }
            }

            // Move — larger creatures are slower (speed / body_size).
            // Elevation slope penalty: steep terrain reduces effective speed.
            // Citation: MacArthur & Wilson (1967), Wright (1943)
            // Feeding mode (eat output > 0) slows travel: handling food and searching
            // for it compete for the same time (Holling 1959; Stephens & Krebs 1986).
            let body_size = org.genome.body_size;
            let feeding_factor = if action.eat > 0.0 {
                feeding_speed_factor
            } else {
                1.0
            };
            let base_speed = action.speed * max_speed / body_size * feeding_factor;
            let cos_h = crate::math::cos(org.heading);
            let sin_h = crate::math::sin(org.heading);
            let next_x = wrap_coord(org.pos_x + cos_h * base_speed, world_w);
            let next_y = wrap_coord(org.pos_y + sin_h * base_speed, world_h);
            let actual_speed = if elevation_enabled {
                let slope_mult = self
                    .elevation_map
                    .speed_multiplier(org.pos_x, org.pos_y, next_x, next_y);
                base_speed * slope_mult
            } else {
                base_speed
            };
            org.pos_x = wrap_coord(org.pos_x + cos_h * actual_speed, world_w);
            org.pos_y = wrap_coord(org.pos_y + sin_h * actual_speed, world_h);

            // Eat — algae (base food) + optional fruit layer
            // R* resource competition (Tilman 1982): specialists consume proportional to
            // their efficiency, so a fruit specialist (algae_efficiency=0) removes nothing
            // from the algae pool. This creates genuine niche separation by resource
            // exhaustion — specialists have lower R* for their own resource.
            // Citation: Tilman, D. (1982) "Resource Competition and Community Structure"
            // Citation: Grant & Grant (2006) Darwin's finches character displacement
            if action.eat > 0.0 {
                let algae_efficiency = if fruit_enabled {
                    // diet_slope=1.0: algae specialist (diet=0) gets 100%, generalist 50%,
                    // fruit specialist 0% (can't digest algae efficiently).
                    // Citation: Grant & Grant (2006); Schluter (2000)
                    (1.0 - diet_slope * org.genome.diet_preference).max(0.0)
                } else {
                    1.0
                };
                // Consume only as much algae as the organism can digest — specialists
                // deplete their preferred resource; mismatched food passes uneaten.
                let algae = self
                    .food_grid
                    .eat_at(org.pos_x, org.pos_y, algae_efficiency);
                let algae_energy = algae * energy_per_food;

                // Eat fruit if enabled — symmetric: algae specialists consume 0 fruit
                let fruit_energy = if fruit_enabled {
                    let fruit_efficiency =
                        (1.0 - diet_slope * (1.0 - org.genome.diet_preference)).max(0.0);
                    let fruit = self
                        .food_grid
                        .eat_fruit_at(org.pos_x, org.pos_y, fruit_efficiency);
                    fruit * energy_per_fruit
                } else {
                    0.0
                };

                ledger.food_input += algae_energy;
                ledger.fruit_input += fruit_energy;
                let food_energy = algae_energy + fruit_energy;
                org.energy_from_food += food_energy;
                org.energy += food_energy;
                if org.energy > max_energy {
                    ledger.cap_overflow += org.energy - max_energy;
                    org.energy = max_energy;
                }
            }

            // Metabolize — Kleiber's law: metabolism scales as body_size^0.75
            // Density pressure: crowding adds per-neighbor cost — Allee (1931)
            // Temperature mismatch: extra cost when local temp deviates from preference
            // Citation: Bergmann (1847), Allen (1877) — thermoregulation drives adaptation
            let movement_cost = movement_cost_factor * actual_speed * actual_speed;
            // Brain cost: soma + synapse maintenance + activity-dependent signaling
            // Attwell & Laughlin (2001): ~80% of neural energy is active signaling
            let brain_cost = brain_cost_per_neuron * org.brain.neuron_count() as f64
                + brain_cost_per_connection * org.brain.connection_count() as f64
                + activity_weight_cost * action.brain_activity;
            let size_metabolism = base_metabolism * crate::math::powf(body_size, 0.75);
            // Use diet-similarity-weighted neighbor count when the competition kernel is
            // active; falls back to raw count when sigma=0 (weighted_neighbors == neighbor_count).
            let effective_neighbors = if diet_competition_sigma > 0.0 {
                action.weighted_neighbors as f64
            } else {
                action.neighbor_count as f64
            };
            let crowding_cost = density_cost_per_neighbor * effective_neighbors;
            let temp_cost = if temp_enabled {
                // Temperature = y/height: 0=north/cold, 1=south/warm
                let local_temp = org.pos_y / world_h;
                let mismatch = (local_temp - org.genome.preferred_temp).abs();
                base_metabolism * temp_cost_factor * mismatch
            } else {
                0.0
            };
            ledger.basal_cost += size_metabolism;
            ledger.movement_cost += movement_cost;
            ledger.brain_structure_cost += brain_cost_per_neuron * org.brain.neuron_count() as f64
                + brain_cost_per_connection * org.brain.connection_count() as f64;
            ledger.brain_activity_cost += activity_weight_cost * action.brain_activity;
            ledger.crowding_cost += crowding_cost;
            ledger.temperature_cost += temp_cost;
            org.energy -= size_metabolism + movement_cost + brain_cost + crowding_cost + temp_cost;

            org.age += 1;

            // Epigenetic methylation update — stress-responsive gene silencing.
            // Under stress, active connections accumulate methylation (suppression).
            // When not stressed, methylation slowly decays (demethylation).
            // Stress is measured relative to the reproduction threshold, not to
            // energy.maximum: an organism is stressed when its reserves are far below
            // what it needs to breed — the state that triggers the "emergency
            // life-history stage" (Wingfield et al. 1998) and drives state-dependent
            // life-history decisions (McNamara & Houston 1996). Newborns
            // (child_energy) start above this level, so they are not born stressed.
            // Citation: Jablonka & Lamb (2005), Heard & Martienssen (2014)
            if epigenetics_enabled {
                let stressed = org.energy < epigenetics_stress_energy;
                for conn in &mut org.genome.connections {
                    if stressed && conn.enabled {
                        conn.methylation =
                            (conn.methylation + epigenetics_methylation_rate).min(1.0);
                    } else if conn.methylation > 0.0 {
                        conn.methylation =
                            (conn.methylation - epigenetics_demethylation_rate).max(0.0);
                    }
                }
                // Within-lifetime plasticity: methylation silences expression in the
                // living soma too, not only in offspring. Patch effective weights
                // w·(1−m) into the existing brain in place (no allocation, no RNG).
                // Citation: Day & Sweatt (2010) Nat. Neurosci. 13:1319-1323
                if org.age.is_multiple_of(epigenetics_brain_interval) {
                    let genome = &org.genome;
                    org.brain.refresh_weights(|gi| genome.effective_weight(gi));
                }
            }
        }

        // 3b. Pheromone update — deposit + decay
        //
        // Each living creature deposits pheromone at its current cell, then
        // the entire grid decays multiplicatively. Net effect: concentration peaks
        // where many creatures have recently been, fading within ~100 ticks.
        // Brain input (input 17) wired in FOURTH step.
        //
        // Self-organization (Deneubourg 1990): a trail reinforcement loop emerges
        // when creatures evolve to follow high-concentration paths.
        // Citation: Wilson (1975), Deneubourg et al. (1990)
        if self.config.pheromones.enabled {
            for org in &self.organisms {
                self.pheromone_grid.deposit(org.pos_x, org.pos_y);
            }
            self.pheromone_grid.decay();
        }

        // Niche construction — organisms deposit/remove environmental material.
        // Citation: Odling-Smee, Laland & Feldman (2003)
        if self.config.niche_construction.enabled {
            let construct_cost = self.config.niche_construction.energy_cost;
            let construct_dead_zone = self.config.niche_construction.activation_threshold;
            for (i, org) in self.organisms.iter_mut().enumerate() {
                if !org.alive {
                    continue;
                }
                // Outputs inside the dead zone mean "do nothing" and cost nothing.
                // Above it, effort rises smoothly from zero to full intensity, and the
                // cost is proportional to the work done (Odling-Smee et al. 2003).
                let raw = self.actions[i].construct;
                if raw.abs() > construct_dead_zone {
                    let effort = (raw.abs() - construct_dead_zone) / (1.0 - construct_dead_zone);
                    let intensity = effort * raw.signum();
                    self.construction_grid
                        .modify(org.pos_x, org.pos_y, intensity);
                    ledger.construction_cost += effort * construct_cost;
                    org.energy -= effort * construct_cost;
                }
            }
            self.construction_grid.decay();
        }

        // 4. Combat — predator-prey energy transfer
        //
        // Lotka-Volterra dynamics: predators gain energy from prey, but at a cost.
        // Attack has an energy cost (discourages universal aggression) and energy
        // transfer is lossy (thermodynamic inefficiency in trophic transfer).
        //
        // Citations:
        //   Lotka (1925) "Elements of Physical Biology"
        //   Volterra (1926) "Fluctuations in the Abundance of a Species"
        //   Yaeger (1994) "Computational Genetics, Physiology, Metabolism, Neural Systems..."
        let mut attacks = 0usize;
        let attack_range = self.config.combat.attack_range;
        let attack_cost = self.config.combat.attack_cost;
        let idle_attack_cost = self.config.combat.idle_attack_cost;
        let attack_damage = self.config.combat.attack_damage;
        let transfer_ratio = self.config.combat.energy_transfer_ratio;
        let attack_range_sq = attack_range * attack_range;

        // Rebuild spatial grid with post-movement positions for attack range checks
        self.positions.clear();
        self.positions
            .extend(self.organisms.iter().map(|o| (o.pos_x, o.pos_y, o.alive)));
        self.spatial_grid.rebuild(&self.positions);

        // Process attacks in index order for determinism.
        // Each attacker picks the nearest living neighbor within attack_range.
        for i in 0..organism_count {
            let action = self.actions[i];
            if !action.alive || action.attack <= 0.0 {
                continue;
            }
            let org_x = self.organisms[i].pos_x;
            let org_y = self.organisms[i].pos_y;

            // Find nearest living neighbor within attack range
            let mut best_target: Option<usize> = None;
            let mut best_dist_sq = attack_range_sq;

            self.spatial_grid.for_each_nearby(org_x, org_y, |ni| {
                if ni == i || !self.organisms[ni].alive {
                    return;
                }
                let dx = wraparound_delta(org_x, self.organisms[ni].pos_x, world_w);
                let dy = wraparound_delta(org_y, self.organisms[ni].pos_y, world_h);
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < best_dist_sq {
                    best_dist_sq = dist_sq;
                    best_target = Some(ni);
                }
            });

            if let Some(target) = best_target {
                // The strike itself is what costs energy (Holling 1959; Scantlebury
                // et al. 2014), so the cost is paid only when there is a target.
                ledger.attack_cost += attack_cost;
                self.organisms[i].energy -= attack_cost;
                // Damage scales with attacker size, reduced by victim size
                // Bigger creatures hit harder and absorb hits better
                let attacker_size = self.organisms[i].genome.body_size;
                let victim_size = self.organisms[target].genome.body_size;
                let scaled_damage = attack_damage * attacker_size / victim_size;
                // Never negative: a victim already below zero energy has nothing to lose.
                let actual_damage = scaled_damage.min(self.organisms[target].energy).max(0.0);
                self.organisms[target].energy -= actual_damage;
                let gained = actual_damage * transfer_ratio;
                ledger.combat_loss += actual_damage - gained;
                ledger.cap_overflow += (self.organisms[i].energy + gained - max_energy).max(0.0);
                self.organisms[i].energy_from_attacks += gained;
                self.organisms[i].energy = (self.organisms[i].energy + gained).min(max_energy);
                attacks += 1;
            } else {
                // No target in range: nothing was struck (cost defaults to 0).
                ledger.attack_cost += idle_attack_cost;
                self.organisms[i].energy -= idle_attack_cost;
            }
        }

        // 5. Energy sharing — cooperative energy transfer
        //
        // Creatures can voluntarily transfer energy to their nearest neighbor.
        // Whether this behavior evolves depends on kin selection (Hamilton, 1964):
        // rB > C, where r = relatedness, B = benefit to recipient, C = cost to donor.
        // Since offspring spawn near parents and inherit similar color hue,
        // related creatures cluster spatially — enabling kin-selected altruism.
        //
        // Citations:
        //   Hamilton (1964) "The Genetical Evolution of Social Behaviour"
        //   Trivers (1971) "The Evolution of Reciprocal Altruism"
        let mut shares = 0usize;
        let mut kin_shares = 0usize;
        let share_range = self.config.sharing.share_range;
        let share_amount = self.config.sharing.share_amount;
        let transfer_efficiency = self.config.sharing.transfer_efficiency;
        let share_range_sq = share_range * share_range;

        for i in 0..self.organisms.len() {
            let action = self.actions[i];
            if !action.alive || action.share <= 0.0 {
                continue;
            }
            // Don't share if it would kill the donor
            if self.organisms[i].energy <= share_amount {
                continue;
            }

            let org_x = self.organisms[i].pos_x;
            let org_y = self.organisms[i].pos_y;

            // Find nearest living neighbor within share range
            let mut best_target: Option<usize> = None;
            let mut best_dist_sq = share_range_sq;

            self.spatial_grid.for_each_nearby(org_x, org_y, |ni| {
                if ni == i || !self.organisms[ni].alive {
                    return;
                }
                let dx = wraparound_delta(org_x, self.organisms[ni].pos_x, world_w);
                let dy = wraparound_delta(org_y, self.organisms[ni].pos_y, world_h);
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < best_dist_sq {
                    best_dist_sq = dist_sq;
                    best_target = Some(ni);
                }
            });

            if let Some(target) = best_target {
                // Donor pays share_amount; recipient gets share_amount * transfer_efficiency
                // (<= 1, validated), so sharing never creates energy. Any benefit to kin
                // must come from energy being worth more to the recipient (Hamilton 1964).
                self.organisms[i].energy -= share_amount;
                let received = share_amount * transfer_efficiency;
                ledger.sharing_loss += share_amount - received;
                ledger.cap_overflow +=
                    (self.organisms[target].energy + received - max_energy).max(0.0);
                self.organisms[target].energy =
                    (self.organisms[target].energy + received).min(max_energy);
                shares += 1;

                // Track kin-directed sharing (hue diff < 30 degrees = kin)
                let donor_hue = self.organisms[i].genome.color_hue;
                let recipient_hue = self.organisms[target].genome.color_hue;
                let mut hue_diff = (donor_hue - recipient_hue).abs();
                if hue_diff > 180.0 {
                    hue_diff = 360.0 - hue_diff;
                }
                if hue_diff < 30.0 {
                    kin_shares += 1;
                }
            }
        }

        // 6. Reproduce — asexual or sexual (NEAT crossover)
        // Citation: Maynard Smith (1978), Muller (1932), Stanley & Miikkulainen (2002) §3.2
        let mut offspring: Vec<Organism> = Vec::new();
        let current_alive = self.organisms.iter().filter(|o| o.alive).count();
        let repro_threshold = self.config.reproduction.threshold;
        let parent_cost = self.config.reproduction.parent_cost;
        let child_energy = self.config.reproduction.child_energy;
        let hard_cap = self.config.creature.hard_cap;
        let body_radius = self.config.creature.body_radius;
        let sexual = self.config.reproduction.sexual;
        let mate_range = self.config.reproduction.mate_range;
        let crossover_rate = self.config.reproduction.crossover_rate;
        let fitness_bias = self.config.reproduction.fitness_bias;
        // Twofold cost of sex: the mate pays too (default = parent_cost), so a pair
        // spends twice the energy per offspring while each transmits half its genes.
        // Citation: Maynard Smith (1978) "The Evolution of Sex"
        let mate_cost = self.config.reproduction.effective_mate_cost();
        let mate_min_energy = repro_threshold * self.config.reproduction.mate_energy_fraction;
        let mutation_cfg = &self.config.mutation;
        let fruit_cfg = &self.config.food.fruit;
        let mut sexual_births = 0u64;
        let mut asexual_births = 0u64;
        let mut cap_blocked_births = 0u64;

        // Each organism takes part in at most one sexual mating per tick (as
        // initiator or as mate). Reused buffer — no per-tick allocation after warm-up.
        self.mated_buf.clear();
        self.mated_buf.resize(self.organisms.len(), false);

        for parent_idx in 0..self.organisms.len() {
            let org = &self.organisms[parent_idx];
            if !org.alive {
                continue;
            }
            if org.energy < repro_threshold {
                continue;
            }
            // Ready to reproduce but the population cap is full. Counted so experiments can
            // verify the population is food-limited, not cap-limited (Phase 9a).
            if (current_alive + offspring.len()) >= hard_cap {
                cap_blocked_births += 1;
                continue;
            }
            // Already mated this tick (as someone's mate): no second birth this tick.
            if self.mated_buf[parent_idx] {
                continue;
            }
            let mut paid_mate: Option<usize> = None;

            let mut child_genome = if sexual {
                // Try to find a mate nearby. Eligibility uses *live* energy so a mate
                // that already paid for a birth this tick is judged on what it has left.
                let mut best_mate: Option<usize> = None;
                let mut best_dist_sq = mate_range * mate_range;
                let px = org.pos_x;
                let py = org.pos_y;
                let organisms = &self.organisms;
                let mated = &self.mated_buf;
                self.spatial_grid.for_each_nearby(px, py, |j| {
                    if j == parent_idx || mated[j] {
                        return;
                    }
                    let mate = &organisms[j];
                    if !mate.alive || mate.energy < mate_min_energy || mate.energy <= mate_cost {
                        return;
                    }
                    let (mx, my) = (self.positions[j].0, self.positions[j].1);
                    let dx = wraparound_delta(px, mx, world_w);
                    let dy = wraparound_delta(py, my, world_h);
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq < best_dist_sq {
                        best_dist_sq = dist_sq;
                        best_mate = Some(j);
                    }
                });

                if let Some(mate_idx) = best_mate {
                    if self.rng.r#gen::<f64>() < crossover_rate {
                        // Sexual reproduction with NEAT crossover
                        let mate = &self.organisms[mate_idx];
                        let a_is_fitter = if fitness_bias {
                            org.energy >= mate.energy
                        } else {
                            self.rng.r#gen::<bool>()
                        };
                        sexual_births += 1;
                        paid_mate = Some(mate_idx);
                        Genome::crossover(
                            &org.genome,
                            &mate.genome,
                            a_is_fitter,
                            mutation_cfg,
                            fruit_cfg,
                            &mut self.innovations,
                            &mut self.rng,
                        )
                    } else {
                        asexual_births += 1;
                        org.genome.reproduce(
                            mutation_cfg,
                            fruit_cfg,
                            &mut self.innovations,
                            &mut self.rng,
                        )
                    }
                } else {
                    // No mate found — asexual fallback
                    asexual_births += 1;
                    org.genome.reproduce(
                        mutation_cfg,
                        fruit_cfg,
                        &mut self.innovations,
                        &mut self.rng,
                    )
                }
            } else {
                asexual_births += 1;
                org.genome.reproduce(
                    mutation_cfg,
                    fruit_cfg,
                    &mut self.innovations,
                    &mut self.rng,
                )
            };

            // Epigenetic inheritance: child inherits methylation + Gaussian
            // epimutation noise N(0, σ²) via Box-Muller from two uniform draws
            // (rand 0.8 without rand_distr; deterministic on the sim RNG).
            // Only existing marks (m > 0) are perturbed: unmarked sites stay
            // unmarked, avoiding the upward drift that clamping symmetric noise at
            // 0 would cause on the (majority) zero-methylation sites.
            // σ = 0 → faithful inheritance with no RNG draws.
            // Citation: Jablonka & Lamb (2005), Heard & Martienssen (2014),
            //           Box & Muller (1958) Ann. Math. Stat. 29:610-611
            if epigenetics_enabled {
                let noise_sigma = self.config.epigenetics.inheritance_noise;
                if noise_sigma > 0.0 {
                    for conn in &mut child_genome.connections {
                        if conn.methylation <= 0.0 {
                            continue;
                        }
                        // u1 ∈ (0, 1] so ln(u1) is finite.
                        let u1 = 1.0 - self.rng.r#gen::<f64>();
                        let u2 = self.rng.r#gen::<f64>();
                        let z = (-2.0 * crate::math::ln(u1)).sqrt()
                            * crate::math::cos(std::f64::consts::TAU * u2);
                        conn.methylation = (conn.methylation + noise_sigma * z).clamp(0.0, 1.0);
                    }
                }
            }

            let child_brain = child_genome.build_brain();
            let offset_angle = self.rng.gen_range(0.0..std::f64::consts::TAU);
            let offset_dist = body_radius * 2.0;
            let org = &self.organisms[parent_idx]; // reborrow after rng usage
            let child = Organism {
                id: self.next_id,
                pos_x: wrap_coord(
                    org.pos_x + crate::math::cos(offset_angle) * offset_dist,
                    world_w,
                ),
                pos_y: wrap_coord(
                    org.pos_y + crate::math::sin(offset_angle) * offset_dist,
                    world_h,
                ),
                heading: self.rng.gen_range(0.0..std::f64::consts::TAU),
                energy: child_energy,
                age: 0,
                genome: child_genome,
                brain: child_brain,
                alive: true,
                parent_id: Some(org.id),
                generation: org.generation + 1,
                signal: 0.0,
                energy_from_food: 0.0,
                energy_from_attacks: 0.0,
                root_ancestor_id: org.root_ancestor_id,
                birth_tick: self.tick,
                clade_id: org.clade_id,
                output_bins: [[0.0; crate::organism::NUM_ENTROPY_BINS];
                    crate::organism::NUM_OUTPUTS],
                memory: Vec::new(),
            };
            self.next_id += 1;
            ledger.reproduction_cost += parent_cost;
            ledger.offspring_input += child_energy;
            self.organisms[parent_idx].energy -= parent_cost;
            if let Some(m) = paid_mate {
                ledger.reproduction_cost += mate_cost;
                self.organisms[m].energy -= mate_cost;
                self.mated_buf[m] = true;
                self.mated_buf[parent_idx] = true;
            }
            births += 1;
            offspring.push(child);
        }
        self.organisms.extend(offspring);

        // 6. Die (energy depleted or Gompertz-Makeham probabilistic mortality)
        //
        // Gompertz-Makeham law: h(age) = λ + α·exp(β·age)
        //   λ = extrinsic_mortality (accidents, background hazard)
        //   α = initial_senescence (tiny at birth)
        //   β = aging_rate (from genome — evolvable)
        //
        // Citations:
        //   Gompertz (1825) "On the Nature of the Function Expressive of the Law of Human Mortality"
        //   Williams (1957) "Pleiotropy, Natural Selection, and the Evolution of Senescence"
        //   Kirkwood (1977) "Evolution of ageing"
        let max_age = self.config.creature.max_age;
        let lambda = self.config.aging.extrinsic_mortality;
        let alpha = self.config.aging.initial_senescence;
        for org in &mut self.organisms {
            if !org.alive {
                continue;
            }
            if org.energy <= 0.0 || org.age >= max_age {
                org.alive = false;
                deaths += 1;
            } else {
                // Gompertz-Makeham: per-tick death probability increases exponentially with age
                let beta = org.genome.aging_rate;
                let hazard = lambda + alpha * crate::math::exp(beta * org.age as f64);
                // Clamp to [0, 1] — at very old ages exp() can overflow
                let death_prob = hazard.min(1.0);
                if self.rng.r#gen::<f64>() < death_prob {
                    org.alive = false;
                    deaths += 1;
                }
            }
        }

        ledger.death_removal = self
            .organisms
            .iter()
            .filter(|o| !o.alive)
            .map(|o| o.energy)
            .sum();

        // Remove dead organisms to keep memory bounded
        self.organisms.retain(|o| o.alive);

        // Mirror demographic events into the neutral shadow population (OEE null model).
        // The shadow uses its own RNG stream and never touches self.rng.
        if let Some(ref mut shadow) = self.shadow {
            shadow.mirror(asexual_births, sexual_births, deaths);
        }

        // 7. Droughts: stochastic localized food crashes
        // Citation: Holling (1973) "Resilience and Stability of Ecological Systems"
        // Citation: Connell (1978) intermediate disturbance hypothesis
        self.food_grid
            .maybe_spawn_drought(&self.config.food.droughts, &mut self.env_rng);

        // 8. Regrow food (with seasonal modulation + drought suppression).
        // Niche construction multiplies each cell's *actual* algae regrowth by
        // (1 + level × regrowth_bonus), so it respects biome, season and drought.
        // Citation: Botkin (1990) "Forest Dynamics" — sinusoidal resource oscillation
        // Citation: Odling-Smee, Laland & Feldman (2003) — niche construction
        let construction = if self.config.niche_construction.enabled {
            Some(&self.construction_grid)
        } else {
            None
        };
        self.food_grid.regrow_with_construction(
            self.tick,
            &self.config.food.seasons,
            fruit_enabled,
            construction,
        );

        // Update stats
        let population = self.organisms.len();
        let (total_energy, total_neurons, total_generation) =
            self.organisms
                .iter()
                .fold((0.0, 0.0, 0u64), |(e, n, g), o| {
                    (
                        e + o.energy,
                        n + o.brain.neuron_count() as f64,
                        g + o.generation,
                    )
                });
        let mean_energy = if population > 0 {
            total_energy / population as f64
        } else {
            0.0
        };
        let mean_brain_neurons = if population > 0 {
            total_neurons / population as f64
        } else {
            0.0
        };
        let mean_generation = if population > 0 {
            total_generation as f64 / population as f64
        } else {
            0.0
        };

        // Species clustering — greedy threshold-based assignment
        //
        // Inspired by NEAT compatibility distance (Stanley & Miikkulainen, 2002).
        // Each organism is assigned to the first species whose representative
        // genome is within `species_threshold` distance. If no match, a new species
        // is created. This is O(organisms * species) — fast when species count is small.
        //
        // Dynamic threshold: sample pairwise genome distances, set threshold =
        // factor × mean_distance. Adapts to actual population diversity so species
        // count is meaningful regardless of how divergent genomes become over time.
        //
        // Citation: Stanley & Miikkulainen (2002) — dynamic compatibility threshold
        let species_threshold = if self.config.speciation.dynamic && population > 1 {
            let sample_size = self.config.speciation.sample_size.min(population);
            let stride = if population > sample_size {
                population / sample_size
            } else {
                1
            };
            let mut total_distance = 0.0;
            let mut pair_count = 0usize;
            for a in 0..sample_size {
                let ai = a * stride;
                for b in (a + 1)..sample_size {
                    let bi = b * stride;
                    total_distance += self.organisms[ai]
                        .genome
                        .genome_distance(&self.organisms[bi].genome);
                    pair_count += 1;
                }
            }
            if pair_count > 0 {
                let mean_distance = total_distance / pair_count as f64;
                (mean_distance * self.config.speciation.factor).clamp(
                    self.config.speciation.min_threshold,
                    self.config.speciation.max_threshold,
                )
            } else {
                self.config.speciation.fixed_threshold
            }
        } else {
            self.config.speciation.fixed_threshold
        };
        // Assign each organism to a species and collect per-species stats.
        // Use previous tick's representatives for continuity when possible.
        let (species_count, species_infos) = if population > 0 {
            let mut representatives: Vec<Genome> = Vec::new();
            let mut species_assignments: Vec<usize> = Vec::with_capacity(population);

            for org in &self.organisms {
                let mut assigned = None;
                // Try previous representatives first for temporal continuity
                for (si, rep) in self.species_representatives.iter().enumerate() {
                    if org.genome.genome_distance(rep) < species_threshold {
                        // Ensure this species index exists in our current list
                        if si < representatives.len() {
                            assigned = Some(si);
                        } else if representatives.len() == si {
                            // Extend to match old rep
                            representatives.push(org.genome.clone());
                            assigned = Some(si);
                        }
                        break;
                    }
                }
                // Fall back to current representatives
                if assigned.is_none() {
                    for (si, rep) in representatives.iter().enumerate() {
                        if org.genome.genome_distance(rep) < species_threshold {
                            assigned = Some(si);
                            break;
                        }
                    }
                }
                // New species
                if assigned.is_none() {
                    assigned = Some(representatives.len());
                    representatives.push(org.genome.clone());
                }
                species_assignments.push(assigned.unwrap());
            }

            let num_species = representatives.len();

            // Collect per-species aggregates
            let mut infos: Vec<SpeciesInfo> = (0..num_species)
                .map(|id| SpeciesInfo {
                    id,
                    ..Default::default()
                })
                .collect();

            for (oi, &si) in species_assignments.iter().enumerate() {
                let org = &self.organisms[oi];
                let info = &mut infos[si];
                info.population += 1;
                info.mean_hue += org.genome.color_hue;
                info.mean_energy += org.energy;
                info.mean_body_size += org.genome.body_size;
                info.mean_connections += org.brain.connection_count() as f64;
                info.mean_generation += org.generation as f64;

                let total_intake = org.energy_from_food + org.energy_from_attacks;
                let pred_frac = if total_intake > 0.0 {
                    org.energy_from_attacks / total_intake
                } else {
                    0.0
                };
                info.mean_trophic_level += 1.0 + pred_frac;
                if pred_frac < 0.25 {
                    info.herbivore_fraction += 1.0;
                } else if pred_frac > 0.5 {
                    info.predator_fraction += 1.0;
                }
            }

            // Normalize and compute trends
            // Resize history buffer to match current species count
            if self.species_history.len() < num_species {
                self.species_history.resize(num_species, Vec::new());
            }

            for info in &mut infos {
                let n = info.population as f64;
                if n > 0.0 {
                    info.mean_hue /= n;
                    info.mean_energy /= n;
                    info.mean_body_size /= n;
                    info.mean_connections /= n;
                    info.mean_generation /= n;
                    info.mean_trophic_level /= n;
                    info.herbivore_fraction /= n;
                    info.predator_fraction /= n;
                }

                // Update history and compute trend
                if info.id < self.species_history.len() {
                    let hist = &mut self.species_history[info.id];
                    hist.push(info.population);
                    if hist.len() > SPECIES_HISTORY_LEN {
                        hist.remove(0);
                    }
                    if hist.len() >= 2 {
                        let avg: f64 = hist.iter().sum::<usize>() as f64 / hist.len() as f64;
                        if avg > 0.0 {
                            info.trend = (info.population as f64 - avg) / avg;
                        }
                    }
                }
            }

            // Sort species by population (largest first) for UI display
            infos.sort_by_key(|a| std::cmp::Reverse(a.population));

            // Update representatives for next tick continuity
            self.species_representatives = representatives;

            (num_species, infos)
        } else {
            self.species_representatives.clear();
            self.species_history.clear();
            (0, Vec::new())
        };

        // Trophic level analysis
        //
        // Trophic level = 1 + fraction of energy from predation.
        // Pure herbivores (all energy from food) = level 1.0
        // Pure predators (all energy from attacks) = level 2.0
        // Omnivores fall in between.
        //
        // Citation: Lindeman, R.L. (1942) "The Trophic-Dynamic Aspect of Ecology"
        // *Ecology* 23(4):399-417
        let mut total_trophic = 0.0;
        let mut herbivore_count = 0usize;
        let mut predator_count = 0usize;
        for org in &self.organisms {
            let total_intake = org.energy_from_food + org.energy_from_attacks;
            if total_intake > 0.0 {
                let predation_fraction = org.energy_from_attacks / total_intake;
                let trophic_level = 1.0 + predation_fraction;
                total_trophic += trophic_level;
                // Classify: >75% from food = herbivore, >50% from attacks = predator
                if predation_fraction < 0.25 {
                    herbivore_count += 1;
                } else if predation_fraction > 0.5 {
                    predator_count += 1;
                }
            } else {
                // No energy intake yet — default to herbivore (level 1.0)
                total_trophic += 1.0;
                herbivore_count += 1;
            }
        }
        let mean_trophic_level = if population > 0 {
            total_trophic / population as f64
        } else {
            0.0
        };

        self.tick += 1;
        let total_births = self.stats.total_births + births as u64;
        let total_deaths = self.stats.total_deaths + deaths as u64;
        let total_attacks = self.stats.total_attacks + attacks as u64;
        let total_shares = self.stats.total_shares + shares as u64;

        // Cooperation metrics
        let cooperation_rate = if population > 0 {
            shares as f64 / population as f64
        } else {
            0.0
        };
        let kin_share_ratio = if shares > 0 {
            kin_shares as f64 / shares as f64
        } else {
            0.0
        };

        // Brain entropy — information-theoretic analysis
        //
        // Per-organism: mean Shannon entropy of output activation histograms.
        // Measures how varied each brain's behavior is over recent history.
        // Population-level: entropy of output distributions across all organisms.
        // Measures behavioral diversity — are creatures doing different things?
        //
        // Citation: Shannon, C.E. (1948) "A Mathematical Theory of Communication"
        // Also: Beer & Williams (2015) "Information Processing and Dynamics in
        // Minimally Cognitive Agents" — entropy analysis of evolved neural controllers
        let (mean_brain_entropy, behavioral_diversity, mean_connections) = if population > 0 {
            use crate::organism::{NUM_ENTROPY_BINS, NUM_OUTPUTS, shannon_entropy};

            // Per-organism entropy: average over outputs, then over population
            let mut total_entropy = 0.0;
            let mut total_connections = 0.0;
            for org in &self.organisms {
                let mut org_entropy = 0.0;
                for bins in org.output_bins.iter().take(NUM_OUTPUTS) {
                    org_entropy += shannon_entropy(bins);
                }
                total_entropy += org_entropy / NUM_OUTPUTS as f64;
                total_connections += org.brain.connection_count() as f64;
            }

            // Population behavioral diversity: bin each output across all organisms
            let mut pop_bins = [[0.0f64; NUM_ENTROPY_BINS]; NUM_OUTPUTS];
            for org in &self.organisms {
                // Use current action outputs (from the Action stored this tick)
                // We need the raw outputs... but they're not stored after the loop.
                // Instead, use the histogram bins as a proxy — the most recent bin
                // has the highest weight. This is a reasonable approximation.
                // Actually, let's just look at the organism's signal and reconstruct
                // from the action buffer.
                // Simpler: bin the current output_bins peak for each organism.
                for (o, pop_bin) in pop_bins.iter_mut().enumerate().take(NUM_OUTPUTS) {
                    // Find the bin with highest count (most frequent recent output)
                    let peak_bin = org.output_bins[o]
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    pop_bin[peak_bin] += 1.0;
                }
            }
            let mut pop_diversity = 0.0;
            for pop_bin in pop_bins.iter().take(NUM_OUTPUTS) {
                pop_diversity += shannon_entropy(pop_bin);
            }
            pop_diversity /= NUM_OUTPUTS as f64;

            (
                total_entropy / population as f64,
                pop_diversity,
                total_connections / population as f64,
            )
        } else {
            (0.0, 0.0, 0.0)
        };

        // OEE activity tracking — Bedau & Packard (1991), Dolson et al. (2019)
        if self.config.activity_tracking.enabled {
            self.activity_tracker.update(
                self.tick,
                self.organisms.iter().filter(|o| o.alive).map(|o| {
                    crate::activity::topology_hash_versioned(
                        &o.genome,
                        self.config.activity_tracking.hash_version,
                    )
                }),
            );
            if let Some(ref mut shadow) = self.shadow {
                shadow.update_activity(self.tick);
            }
        }
        let activity_stats = &self.activity_tracker.stats;
        let shadow_stats_snapshot = self.shadow.as_ref().map(|s| (s.len(), s.stats().clone()));

        let max_generation = self
            .organisms
            .iter()
            .filter(|o| o.alive)
            .map(|o| o.generation)
            .max()
            .unwrap_or(0);

        let founder_lineages = {
            let mut roots: Vec<u64> = self
                .organisms
                .iter()
                .filter(|o| o.alive)
                .map(|o| o.root_ancestor_id)
                .collect();
            roots.sort_unstable();
            roots.dedup();
            roots.len()
        };

        // Windowed lineage diversity (coalescent-style): at each checkpoint, count
        // how many clade founders from the previous checkpoint still have living
        // descendants, then make every living organism a founder of its own clade.
        // Founder (gen-0) counts inevitably coalesce to 1; this measure stays
        // informative for the whole run. No RNG; does not affect dynamics.
        // Citation: Dolson et al. (2019) "The MODES Toolbox" (lineage/persistence)
        let lineage_window = self.config.activity_tracking.lineage_window.max(1);
        if self.tick.is_multiple_of(lineage_window) {
            let mut clades: Vec<u64> = self
                .organisms
                .iter()
                .filter(|o| o.alive)
                .map(|o| o.clade_id)
                .collect();
            clades.sort_unstable();
            clades.dedup();
            self.lineage_diversity_cached = clades.len();
            for o in &mut self.organisms {
                o.clade_id = o.id;
            }
        }
        let lineage_diversity = if self.tick < lineage_window {
            founder_lineages
        } else {
            self.lineage_diversity_cached
        };

        let energy_balance_residual = total_energy - ledger.before - ledger.net_flow();
        debug_assert!(
            energy_balance_residual.is_finite()
                && energy_balance_residual.abs() <= ledger.tolerance(total_energy),
            "tick {} organism reserve balance residual {} exceeds tolerance {}",
            self.tick,
            energy_balance_residual,
            ledger.tolerance(total_energy)
        );
        // Memory: how much of the population carries recurrent connections.
        let (total_recurrent, brains_with_memory) =
            self.organisms.iter().fold((0usize, 0usize), |(r, m), o| {
                (
                    r + o.brain.recurrent_count(),
                    m + usize::from(o.brain.has_memory()),
                )
            });
        let (mean_recurrent_connections, recurrent_brain_fraction) = if population > 0 {
            (
                total_recurrent as f64 / population as f64,
                brains_with_memory as f64 / population as f64,
            )
        } else {
            (0.0, 0.0)
        };

        // Diet diversity — measures adaptive radiation along the diet axis.
        // High diet_variance = bimodal population (two specialists); low = generalists.
        // Citation: Grant & Grant (2006); Schluter (2000)
        let (mean_diet_preference, diet_variance, mean_hidden_nodes) = if population > 0 {
            let (sum_diet, sum_diet_sq, sum_hidden) =
                self.organisms
                    .iter()
                    .fold((0.0f64, 0.0f64, 0.0f64), |(sd, sd2, sh), o| {
                        let d = o.genome.diet_preference;
                        (sd + d, sd2 + d * d, sh + o.brain.hidden as f64)
                    });
            let mean_d = sum_diet / population as f64;
            let variance = (sum_diet_sq / population as f64) - (mean_d * mean_d);
            (mean_d, variance.max(0.0), sum_hidden / population as f64)
        } else {
            (0.0, 0.0, 0.0)
        };

        self.stats = TickStats {
            mean_recurrent_connections,
            recurrent_brain_fraction,
            mean_diet_preference,
            diet_variance,
            mean_hidden_nodes,
            energy_before: ledger.before,
            energy_after: total_energy,
            energy_food_input: ledger.food_input,
            energy_fruit_input: ledger.fruit_input,
            energy_basal_cost: ledger.basal_cost,
            energy_movement_cost: ledger.movement_cost,
            energy_brain_structure_cost: ledger.brain_structure_cost,
            energy_brain_activity_cost: ledger.brain_activity_cost,
            energy_crowding_cost: ledger.crowding_cost,
            energy_temperature_cost: ledger.temperature_cost,
            energy_construction_cost: ledger.construction_cost,
            energy_attack_cost: ledger.attack_cost,
            energy_combat_loss: ledger.combat_loss,
            energy_sharing_loss: ledger.sharing_loss,
            energy_reproduction_cost: ledger.reproduction_cost,
            energy_offspring_input: ledger.offspring_input,
            energy_death_removal: ledger.death_removal,
            energy_cap_overflow: ledger.cap_overflow,
            energy_balance_residual,
            tick: self.tick,
            population,
            births,
            deaths,
            total_births,
            total_deaths,
            attacks,
            total_attacks,
            shares,
            total_shares,
            cooperation_rate,
            kin_share_ratio,
            mean_energy,
            mean_brain_neurons,
            species_count,
            mean_generation,
            mean_trophic_level,
            herbivore_count,
            predator_count,
            mean_brain_entropy,
            behavioral_diversity,
            mean_connections,
            mean_crowding,
            speciation_threshold: species_threshold,
            mean_brain_activity,
            sexual_births,
            asexual_births,
            active_droughts: self.food_grid.droughts.len(),
            novel_traits: activity_stats.novel_traits,
            active_traits: activity_stats.active_traits,
            mean_cumulative_activity: activity_stats.mean_cumulative_activity,
            max_generation,
            lineage_diversity,
            mean_construction_level: if self.config.niche_construction.enabled {
                let sum: f64 = self.construction_grid.cells.iter().map(|c| c.abs()).sum();
                sum / self.construction_grid.cells.len() as f64
            } else {
                0.0
            },
            net_construction_level: if self.config.niche_construction.enabled {
                let sum: f64 = self.construction_grid.cells.iter().sum();
                sum / self.construction_grid.cells.len() as f64
            } else {
                0.0
            },
            total_sexual_births: self.stats.total_sexual_births + sexual_births,
            total_asexual_births: self.stats.total_asexual_births + asexual_births,
            cap_blocked_births,
            total_cap_blocked_births: self.stats.total_cap_blocked_births + cap_blocked_births,
            founder_lineages,
            total_cumulative_activity: activity_stats.total_cumulative_activity,
            median_extant_activity: activity_stats.median_extant_activity,
            shadow_population: shadow_stats_snapshot.as_ref().map_or(0, |(len, _)| *len),
            shadow_novel_traits: shadow_stats_snapshot
                .as_ref()
                .map_or(0, |(_, s)| s.novel_traits),
            shadow_active_traits: shadow_stats_snapshot
                .as_ref()
                .map_or(0, |(_, s)| s.active_traits),
            shadow_mean_cumulative_activity: shadow_stats_snapshot
                .as_ref()
                .map_or(0.0, |(_, s)| s.mean_cumulative_activity),
            shadow_median_extant_activity: shadow_stats_snapshot
                .as_ref()
                .map_or(0.0, |(_, s)| s.median_extant_activity),
            mean_methylation: if self.config.epigenetics.enabled && population > 0 {
                let (total_meth, total_conns) =
                    self.organisms
                        .iter()
                        .filter(|o| o.alive)
                        .fold((0.0f64, 0u64), |(m, c), o| {
                            let conn_meth: f64 =
                                o.genome.connections.iter().map(|c| c.methylation).sum();
                            (m + conn_meth, c + o.genome.connections.len() as u64)
                        });
                if total_conns > 0 {
                    total_meth / total_conns as f64
                } else {
                    0.0
                }
            } else {
                0.0
            },
            species: species_infos,
        };
    }

    pub(crate) fn validate_checkpoint_internal(&self) -> Result<(), String> {
        if self.organisms.len() > self.config.creature.hard_cap
            || self.organisms.iter().any(|o| o.id >= self.next_id)
            || self.stats.tick != self.tick
            || (self.tick > 0 && self.stats.population != self.organisms.len())
            || self.spatial_grid.world_width != self.config.world.width
            || self.spatial_grid.world_height != self.config.world.height
            || self.spatial_grid.cell_size != self.config.creature.vision.range
        {
            return Err("checkpoint population, IDs, stats or spatial dimensions disagree".into());
        }
        let mut ids = std::collections::HashSet::with_capacity(self.organisms.len());
        if self.organisms.iter().any(|o| !ids.insert(o.id)) {
            return Err("duplicate checkpoint organism ID".into());
        }
        if self
            .positions
            .iter()
            .any(|&(x, y, _)| !x.is_finite() || !y.is_finite())
            || self.actions.iter().any(|a| {
                [
                    a.turn,
                    a.speed,
                    a.eat,
                    a.attack,
                    a.signal,
                    a.share,
                    a.construct,
                    a.brain_activity,
                ]
                .iter()
                .any(|v| !v.is_finite())
            })
            || self
                .species_history
                .iter()
                .any(|h| h.len() > SPECIES_HISTORY_LEN)
        {
            return Err("invalid checkpoint cached state".into());
        }
        for g in &self.species_representatives {
            crate::checkpoint::validate_genome(g).map_err(|e| e.to_string())?;
            if g.num_inputs != self.config.brain.inputs
                || g.num_outputs != self.config.brain.outputs
            {
                return Err("species representative I/O mismatch".into());
            }
        }
        self.activity_tracker
            .validate_checkpoint(self.tick, self.config.activity_tracking.window_size)?;
        Ok(())
    }

    /// Check if the population has gone extinct.
    pub fn is_extinct(&self) -> bool {
        self.organisms.is_empty()
    }
}
