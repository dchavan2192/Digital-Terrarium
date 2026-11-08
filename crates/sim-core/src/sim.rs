//! The simulation: state + deterministic tick loop.
//!
//! Each tick follows this pipeline:
//! 1. Rebuild spatial index
//! 2. For each organism: sense → think → act
//! 3. Eat (consume food from the grid)
//! 4. Metabolize (deduct energy costs)
//! 5. Reproduce (if energy > threshold and population < cap)
//! 6. Die (old age or energy ≤ 0)
//! 7. Regrow food
//!
//! The RNG is seeded, so identical config + seed = identical run.

use crate::config::Config;
use crate::genome::Genome;
use crate::organism::{Organism, OrganismId};
use crate::world::{FoodGrid, SpatialGrid, wrap_coord, wraparound_delta};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

/// Maximum number of vision rays (for stack allocation).
const MAX_RAYS: usize = 16;

/// Complete simulation state. Owns all organisms, the world, and the RNG.
pub struct Simulation {
    pub config: Config,
    pub tick: u64,
    pub organisms: Vec<Organism>,
    pub food_grid: FoodGrid,
    pub spatial_grid: SpatialGrid,
    pub rng: ChaCha8Rng,
    next_id: OrganismId,

    // Per-tick stats (updated at end of each tick)
    pub stats: TickStats,

    // Reusable buffers to avoid allocation in the hot loop
    positions: Vec<(f64, f64, bool)>,
    actions: Vec<Action>,
}

/// Summary statistics for the most recent tick.
#[derive(Debug, Clone, Default, serde::Serialize)]
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
}

/// Compact action output — one per organism per tick.
#[derive(Clone, Copy)]
struct Action {
    turn: f64,
    speed: f64,
    eat: f64,
    attack: f64,
    signal: f64,
    share: f64,
    alive: bool,
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
            alive: false,
        }
    }
}

impl Simulation {
    /// Create a new simulation from config. Spawns the initial population.
    pub fn new(config: Config) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(config.world.seed);
        let food_grid = FoodGrid::new(&config.world, &config.food);

        // Spatial grid cell size = vision range so neighbor queries check 3×3 cells
        let spatial_cell_size = config.creature.vision.range;
        let spatial_grid = SpatialGrid::new(&config.world, spatial_cell_size);

        let cap = config.creature.hard_cap;
        let mut organisms = Vec::with_capacity(cap);
        let mut next_id: OrganismId = 0;

        for _ in 0..config.creature.initial_count {
            let genome = Genome::random(&config.brain, &config.mutation, &config.aging, &mut rng);
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
                output_bins: [[0.0; crate::organism::NUM_ENTROPY_BINS];
                    crate::organism::NUM_OUTPUTS],
            };
            organisms.push(org);
            next_id += 1;
        }

        Simulation {
            config,
            tick: 0,
            organisms,
            food_grid,
            spatial_grid,
            rng,
            next_id,
            stats: TickStats::default(),
            positions: Vec::with_capacity(cap),
            actions: Vec::with_capacity(cap),
        }
    }

    /// Advance the simulation by one tick.
    pub fn step(&mut self) {
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
        let range_inv = 1.0 / range;
        let food_cell_size = self.config.food.cell_size;
        let world_w = self.config.world.width;
        let world_h = self.config.world.height;
        let max_energy = self.config.energy.maximum;
        let max_turn = self.config.creature.max_turn;
        let max_speed = self.config.creature.max_speed;
        let energy_per_food = self.config.food.energy_per_food;
        let base_metabolism = self.config.energy.base_metabolism;
        let movement_cost_factor = self.config.energy.movement_cost;
        let brain_cost_per_neuron = self.config.energy.brain_cost;

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
            *slot = (local_angle.cos(), local_angle.sin());
        }
        // Precompute acceptance threshold as cosine for dot-product comparison
        let cos_acceptance = acceptance.cos();

        // 2. Sense → Think → Act (parallelized with rayon)
        //
        // This phase is read-only on organisms, food_grid, and spatial_grid.
        // No RNG is used, so parallel execution preserves determinism.
        // Each organism independently: cast vision rays → run brain forward pass → produce Action.
        let organisms = &self.organisms;
        let food_grid = &self.food_grid;
        let spatial_grid = &self.spatial_grid;

        self.actions = (0..organism_count)
            .into_par_iter()
            .map(|i| {
                if !organisms[i].alive {
                    return Action::default();
                }

                let heading = organisms[i].heading;
                let ox = organisms[i].pos_x;
                let oy = organisms[i].pos_y;
                let cos_h = heading.cos();
                let sin_h = heading.sin();

                // Stack-allocated ray results
                let mut ray_types = [-1.0f64; MAX_RAYS];
                let mut closest_dists = [range; MAX_RAYS];

                // Precompute world-space ray direction unit vectors for this organism
                let mut ray_dirs = [(0.0f64, 0.0f64); MAX_RAYS];
                for r in 0..ray_count {
                    let (lcos, lsin) = ray_local[r];
                    ray_dirs[r] = (cos_h * lcos - sin_h * lsin, sin_h * lcos + cos_h * lsin);
                }

                // Check food along rays (sample at cell_size intervals)
                for r in 0..ray_count {
                    let (ray_dx, ray_dy) = ray_dirs[r];
                    let mut d = food_cell_size;
                    while d < closest_dists[r] {
                        let sx = wrap_coord(ox + ray_dx * d, world_w);
                        let sy = wrap_coord(oy + ray_dy * d, world_h);
                        if food_grid.food_at(sx, sy) > 0.1 {
                            closest_dists[r] = d;
                            ray_types[r] = 0.0; // food
                            break;
                        }
                        d += food_cell_size;
                    }
                }

                // Check creatures — dot-product ray test (no atan2)
                // Also accumulate nearby signals and track nearest creature for kin recognition
                let range_sq = range * range;
                let mut signal_sum = 0.0f64;
                let mut signal_weight = 0.0f64;
                let mut nearest_dist_sq = f64::MAX;
                let mut nearest_hue = -1.0f64; // sentinel: no creature found
                spatial_grid.for_each_nearby(ox, oy, |ni| {
                    if ni == i || !organisms[ni].alive {
                        return;
                    }
                    let dx = wraparound_delta(ox, organisms[ni].pos_x, world_w);
                    let dy = wraparound_delta(oy, organisms[ni].pos_y, world_h);
                    let dist_sq = dx * dx + dy * dy;
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
                    ray_distances[r] = closest_dists[r] * range_inv;
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

                let brain_inputs = &inputs[..organisms[i].brain.inputs];
                let output = organisms[i].brain.forward(brain_inputs);

                Action {
                    turn: output.turn,
                    speed: output.speed,
                    eat: output.eat,
                    attack: output.attack,
                    signal: output.signal,
                    share: output.share,
                    alive: true,
                }
            })
            .collect();

        // 3. Apply actions: movement, eating, metabolism
        for i in 0..organism_count {
            let action = self.actions[i];
            if !action.alive {
                continue;
            }
            let org = &mut self.organisms[i];

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
                ];
                for (o, &raw_val) in raw_outputs.iter().enumerate().take(NUM_OUTPUTS) {
                    // Exponential decay of existing bin counts
                    for b in 0..NUM_ENTROPY_BINS {
                        org.output_bins[o][b] *= ENTROPY_DECAY;
                    }
                    // Increment current bin
                    let bin = activation_to_bin(raw_val);
                    org.output_bins[o][bin] += 1.0;
                }
            }

            // Move — larger creatures are slower (speed / body_size)
            let body_size = org.genome.body_size;
            let actual_speed = action.speed * max_speed / body_size;
            org.pos_x += org.heading.cos() * actual_speed;
            org.pos_y += org.heading.sin() * actual_speed;
            org.pos_x = wrap_coord(org.pos_x, world_w);
            org.pos_y = wrap_coord(org.pos_y, world_h);

            // Eat
            if action.eat > 0.0 {
                let consumed = self.food_grid.eat_at(org.pos_x, org.pos_y, 1.0);
                let food_energy = consumed * energy_per_food;
                org.energy_from_food += food_energy;
                org.energy += food_energy;
                if org.energy > max_energy {
                    org.energy = max_energy;
                }
            }

            // Metabolize — Kleiber's law: metabolism scales as body_size^0.75
            let movement_cost = movement_cost_factor * actual_speed * actual_speed;
            let brain_cost = brain_cost_per_neuron * org.brain.neuron_count() as f64;
            let size_metabolism = base_metabolism * body_size.powf(0.75);
            org.energy -= size_metabolism + movement_cost + brain_cost;

            org.age += 1;
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

            // Pay the attack cost regardless of hitting anything
            self.organisms[i].energy -= attack_cost;

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
                // Damage scales with attacker size, reduced by victim size
                // Bigger creatures hit harder and absorb hits better
                let attacker_size = self.organisms[i].genome.body_size;
                let victim_size = self.organisms[target].genome.body_size;
                let scaled_damage = attack_damage * attacker_size / victim_size;
                let actual_damage = scaled_damage.min(self.organisms[target].energy);
                self.organisms[target].energy -= actual_damage;
                let gained = actual_damage * transfer_ratio;
                self.organisms[i].energy_from_attacks += gained;
                self.organisms[i].energy = (self.organisms[i].energy + gained).min(max_energy);
                attacks += 1;
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
                self.organisms[i].energy -= share_amount;
                self.organisms[target].energy =
                    (self.organisms[target].energy + share_amount).min(max_energy);
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

        // 6. Reproduce
        let mut offspring: Vec<Organism> = Vec::new();
        let current_alive = self.organisms.iter().filter(|o| o.alive).count();
        let repro_threshold = self.config.reproduction.threshold;
        let parent_cost = self.config.reproduction.parent_cost;
        let child_energy = self.config.reproduction.child_energy;
        let hard_cap = self.config.creature.hard_cap;
        let body_radius = self.config.creature.body_radius;

        for org in &mut self.organisms {
            if !org.alive {
                continue;
            }
            if org.energy >= repro_threshold && (current_alive + offspring.len()) < hard_cap {
                org.energy -= parent_cost;
                let child_genome = org.genome.reproduce(&mut self.rng);
                let child_brain = child_genome.build_brain();

                let offset_angle = self.rng.gen_range(0.0..std::f64::consts::TAU);
                let offset_dist = body_radius * 2.0;
                let child = Organism {
                    id: self.next_id,
                    pos_x: wrap_coord(org.pos_x + offset_angle.cos() * offset_dist, world_w),
                    pos_y: wrap_coord(org.pos_y + offset_angle.sin() * offset_dist, world_h),
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
                    output_bins: [[0.0; crate::organism::NUM_ENTROPY_BINS];
                        crate::organism::NUM_OUTPUTS],
                };
                self.next_id += 1;
                births += 1;
                offspring.push(child);
            }
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
                let hazard = lambda + alpha * (beta * org.age as f64).exp();
                // Clamp to [0, 1] — at very old ages exp() can overflow
                let death_prob = hazard.min(1.0);
                if self.rng.r#gen::<f64>() < death_prob {
                    org.alive = false;
                    deaths += 1;
                }
            }
        }

        // Remove dead organisms to keep memory bounded
        self.organisms.retain(|o| o.alive);

        // 7. Regrow food
        self.food_grid.regrow();

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
        // Computed every tick for accurate tracking. With ~1000 organisms and
        // typically <20 species, this is ~20k distance computations — negligible
        // compared to the ~1000 brain forward passes.
        let species_threshold = 0.5;
        let species_count = if population > 0 {
            // Representatives: one genome per species (first assigned member)
            let mut representatives: Vec<&crate::genome::Genome> = Vec::new();
            for org in &self.organisms {
                let mut found = false;
                for rep in &representatives {
                    if org.genome.genome_distance(rep) < species_threshold {
                        found = true;
                        break;
                    }
                }
                if !found {
                    representatives.push(&org.genome);
                }
            }
            representatives.len()
        } else {
            0
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
                for o in 0..NUM_OUTPUTS {
                    org_entropy += shannon_entropy(&org.output_bins[o]);
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

        self.stats = TickStats {
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
        };
    }

    /// Check if the population has gone extinct.
    pub fn is_extinct(&self) -> bool {
        self.organisms.is_empty()
    }
}
