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
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

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
}

/// Summary statistics for the most recent tick.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TickStats {
    pub tick: u64,
    pub population: usize,
    pub births: usize,
    pub deaths: usize,
    pub mean_energy: f64,
    pub mean_brain_neurons: f64,
}

impl Simulation {
    /// Create a new simulation from config. Spawns the initial population.
    pub fn new(config: Config) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(config.world.seed);
        let food_grid = FoodGrid::new(&config.world, &config.food);

        // Spatial grid cell size = vision range so neighbor queries check at most 9 cells
        let spatial_cell_size = config.creature.vision.range;
        let spatial_grid = SpatialGrid::new(&config.world, spatial_cell_size);

        // Spawn initial population at random positions
        let mut organisms = Vec::with_capacity(config.creature.initial_count);
        let mut next_id: OrganismId = 0;

        for _ in 0..config.creature.initial_count {
            let genome = Genome::random(&config.brain, &config.mutation, &mut rng);
            let brain = genome.build_brain(&config.brain);
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
        }
    }

    /// Advance the simulation by one tick.
    pub fn step(&mut self) {
        let mut births = 0usize;
        let mut deaths = 0usize;

        // 1. Rebuild spatial index
        self.spatial_grid.clear();
        for (i, org) in self.organisms.iter().enumerate() {
            if org.alive {
                self.spatial_grid.insert(org.pos_x, org.pos_y, i);
            }
        }

        // 2-3. Sense → Think → Act (collected as deferred actions)
        let organism_count = self.organisms.len();
        let mut actions: Vec<OrganismAction> = Vec::with_capacity(organism_count);

        for i in 0..organism_count {
            if !self.organisms[i].alive {
                actions.push(OrganismAction::Dead);
                continue;
            }

            // --- Sense: cast vision rays ---
            let ray_count = self.config.creature.vision.ray_count;
            let fov = self.config.creature.vision.fov;
            let range = self.config.creature.vision.range;
            let heading = self.organisms[i].heading;
            let ox = self.organisms[i].pos_x;
            let oy = self.organisms[i].pos_y;

            let mut ray_distances = vec![1.0; ray_count]; // 1.0 = nothing seen (max range)
            let mut ray_types = vec![-1.0; ray_count]; // -1.0 = nothing

            let start_angle = heading - fov / 2.0;
            let angle_step = if ray_count > 1 {
                fov / (ray_count - 1) as f64
            } else {
                0.0
            };

            // Check nearby organisms for vision
            let nearby = self.spatial_grid.query_radius(ox, oy, range);

            for r in 0..ray_count {
                let ray_angle = start_angle + r as f64 * angle_step;
                let ray_dx = ray_angle.cos();
                let ray_dy = ray_angle.sin();

                let mut closest_dist = range;
                let mut closest_type = -1.0;

                // Check food along the ray (sample at intervals)
                let step_size = self.config.food.cell_size;
                let mut d = step_size;
                while d < range {
                    let sx = wrap_coord(ox + ray_dx * d, self.config.world.width);
                    let sy = wrap_coord(oy + ray_dy * d, self.config.world.height);
                    if self.food_grid.food_at(sx, sy) > 0.1 && d < closest_dist {
                        closest_dist = d;
                        closest_type = 0.0; // food
                        break; // first hit along this ray
                    }
                    d += step_size;
                }

                // Check creatures along the ray
                for &ni in &nearby {
                    if ni == i {
                        continue;
                    }
                    if !self.organisms[ni].alive {
                        continue;
                    }
                    let nx = self.organisms[ni].pos_x;
                    let ny = self.organisms[ni].pos_y;
                    let dx = wraparound_delta(ox, nx, self.config.world.width);
                    let dy = wraparound_delta(oy, ny, self.config.world.height);
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > range || dist < 0.001 {
                        continue;
                    }
                    // Check if this creature is within the ray's cone
                    let to_angle = dy.atan2(dx);
                    let angle_diff = angle_diff_signed(ray_angle, to_angle);
                    // Accept if within half a ray-step angle (or a minimum angular width)
                    let acceptance = if ray_count > 1 {
                        (angle_step / 2.0).max(0.1)
                    } else {
                        0.2
                    };
                    if angle_diff.abs() < acceptance && dist < closest_dist {
                        closest_dist = dist;
                        closest_type = 1.0; // creature
                    }
                }

                ray_distances[r] = closest_dist / range; // normalize to [0, 1]
                ray_types[r] = closest_type;
            }

            // Build sensor inputs
            let sensor_inputs = self.organisms[i].build_sensor_inputs(
                &ray_distances,
                &ray_types,
                self.config.energy.maximum,
                0.0, // signal placeholder (v1: no signals yet)
            );

            // --- Think: run the brain ---
            let output = self.organisms[i].brain.forward(&sensor_inputs);

            actions.push(OrganismAction::Act {
                turn: output.turn,
                speed: output.speed,
                eat: output.eat,
            });
        }

        // 3. Apply actions: movement and eating
        for (i, action) in actions.iter().enumerate() {
            if let OrganismAction::Act { turn, speed, eat } = action {
                let org = &mut self.organisms[i];

                // Turn
                org.heading += turn * self.config.creature.max_turn;
                org.heading = wrap_angle(org.heading);

                // Move
                let actual_speed = speed * self.config.creature.max_speed;
                org.pos_x += org.heading.cos() * actual_speed;
                org.pos_y += org.heading.sin() * actual_speed;
                org.pos_x = wrap_coord(org.pos_x, self.config.world.width);
                org.pos_y = wrap_coord(org.pos_y, self.config.world.height);

                // Eat
                if *eat > 0.0 {
                    let consumed = self.food_grid.eat_at(org.pos_x, org.pos_y, 1.0);
                    org.energy += consumed * self.config.food.energy_per_food;
                    org.energy = org.energy.min(self.config.energy.maximum);
                }

                // 4. Metabolize
                let movement_cost = self.config.energy.movement_cost * actual_speed * actual_speed;
                let brain_cost = self.config.energy.brain_cost * org.brain.neuron_count() as f64;
                org.energy -= self.config.energy.base_metabolism + movement_cost + brain_cost;

                org.age += 1;
            }
        }

        // 5. Reproduce
        let mut offspring: Vec<Organism> = Vec::new();
        let current_alive = self.organisms.iter().filter(|o| o.alive).count();

        for org in &mut self.organisms {
            if !org.alive {
                continue;
            }
            if org.energy >= self.config.reproduction.threshold
                && (current_alive + offspring.len()) < self.config.creature.hard_cap
            {
                org.energy -= self.config.reproduction.parent_cost;
                let child_genome = org.genome.reproduce(&mut self.rng);
                let child_brain = child_genome.build_brain(&self.config.brain);

                // Spawn child near parent with a slight random offset
                let offset_angle = self.rng.gen_range(0.0..std::f64::consts::TAU);
                let offset_dist = self.config.creature.body_radius * 2.0;
                let child = Organism {
                    id: self.next_id,
                    pos_x: wrap_coord(
                        org.pos_x + offset_angle.cos() * offset_dist,
                        self.config.world.width,
                    ),
                    pos_y: wrap_coord(
                        org.pos_y + offset_angle.sin() * offset_dist,
                        self.config.world.height,
                    ),
                    heading: self.rng.gen_range(0.0..std::f64::consts::TAU),
                    energy: self.config.reproduction.child_energy,
                    age: 0,
                    genome: child_genome,
                    brain: child_brain,
                    alive: true,
                };
                self.next_id += 1;
                births += 1;
                offspring.push(child);
            }
        }
        self.organisms.extend(offspring);

        // 6. Die (energy depleted or old age)
        for org in &mut self.organisms {
            if !org.alive {
                continue;
            }
            if org.energy <= 0.0 || org.age >= self.config.creature.max_age {
                org.alive = false;
                deaths += 1;
            }
        }

        // Remove dead organisms to keep memory bounded
        self.organisms.retain(|o| o.alive);

        // 7. Regrow food
        self.food_grid.regrow();

        // Update stats
        let population = self.organisms.len();
        let mean_energy = if population > 0 {
            self.organisms.iter().map(|o| o.energy).sum::<f64>() / population as f64
        } else {
            0.0
        };
        let mean_brain_neurons = if population > 0 {
            self.organisms
                .iter()
                .map(|o| o.brain.neuron_count() as f64)
                .sum::<f64>()
                / population as f64
        } else {
            0.0
        };

        self.tick += 1;
        self.stats = TickStats {
            tick: self.tick,
            population,
            births,
            deaths,
            mean_energy,
            mean_brain_neurons,
        };
    }

    /// Check if the population has gone extinct.
    pub fn is_extinct(&self) -> bool {
        self.organisms.is_empty()
    }
}

/// Deferred action for one organism in a tick.
enum OrganismAction {
    Dead,
    Act { turn: f64, speed: f64, eat: f64 },
}

/// Normalize an angle to [0, 2π).
fn wrap_angle(a: f64) -> f64 {
    let a = a % std::f64::consts::TAU;
    if a < 0.0 {
        a + std::f64::consts::TAU
    } else {
        a
    }
}

/// Signed angular difference, result in [-π, π].
fn angle_diff_signed(a: f64, b: f64) -> f64 {
    let mut d = b - a;
    while d > std::f64::consts::PI {
        d -= std::f64::consts::TAU;
    }
    while d < -std::f64::consts::PI {
        d += std::f64::consts::TAU;
    }
    d
}

use rand::Rng;
