//! The 2D wraparound world: food grid and spatial index.
//!
//! The food grid is a coarse grid of cells, each holding a food value that
//! regrows over time. The spatial grid is a separate, finer grid used for
//! fast neighbor lookups (vision rays, eating range).

use crate::config::{
    DroughtConfig, ElevationConfig, FoodConfig, FruitConfig, PheromoneConfig, SeasonsConfig,
    WorldConfig,
};

/// The food resource layer. A grid of cells that regrow over time.
///
/// Supports spatial resource gradients via biome zones — each cell can have
/// a different regrowth rate, creating ecological niches that drive speciation.
///
/// Citation: Schluter (2000) "The Ecology of Adaptive Radiation"
#[derive(Debug, Clone)]
pub struct FoodGrid {
    pub cols: usize,
    pub rows: usize,
    pub cell_size: f64,
    pub max_per_cell: f64,
    pub regrowth_rate: f64,
    /// Flat row-major storage of food values.
    pub cells: Vec<f64>,
    /// Per-cell regrowth rates (base_rate × biome_multiplier). Same size as `cells`.
    /// If no biomes are configured, all cells share the base rate.
    regrowth_rates: Vec<f64>,
    /// World dimensions (needed for drought distance calculations).
    world_width: f64,
    world_height: f64,
    /// Active drought zones.
    /// Citation: Holling (1973), Connell (1978) intermediate disturbance hypothesis.
    pub droughts: Vec<ActiveDrought>,

    // --- Two-food-type system (algae + fruit) ---
    // Fruit only grows inside biome zones, creating resource patches that drive
    // specialist vs generalist tradeoffs (Schluter 2000, Grant & Grant 2006).
    /// Fruit food layer. Same grid dimensions as `cells` (algae).
    /// Only populated when `food.fruit.enabled = true`.
    pub fruit_cells: Vec<f64>,
    /// Per-cell fruit regrowth rates (0 outside biomes, biome_rate × multiplier inside).
    fruit_regrowth_rates: Vec<f64>,
}

/// A currently active drought zone.
#[derive(Debug, Clone)]
pub struct ActiveDrought {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub severity: f64,
    pub ticks_remaining: u64,
}

impl FoodGrid {
    pub fn new(world: &WorldConfig, food: &FoodConfig, fruit_config: &FruitConfig) -> Self {
        let cols = (world.width / food.cell_size).ceil() as usize;
        let rows = (world.height / food.cell_size).ceil() as usize;
        let num_cells = cols * rows;
        // Initial food is sparse — will be set proportional to regrowth rate after biome calc
        let mut cells = vec![0.0f64; num_cells];

        // Compute per-cell regrowth rates from biome zones
        // Also compute fruit regrowth rates: non-zero only inside biome zones
        let mut regrowth_rates = vec![food.regrowth_rate; num_cells];
        let mut fruit_regrowth_rates = vec![0.0f64; num_cells];

        let half_cell = food.cell_size * 0.5;
        if !food.biomes.is_empty() {
            for row in 0..rows {
                for col in 0..cols {
                    let cx = col as f64 * food.cell_size + half_cell;
                    let cy = row as f64 * food.cell_size + half_cell;
                    let mut best_multiplier = 1.0f64;
                    let mut best_fruit_rate = 0.0f64;
                    for biome in &food.biomes {
                        // Use wraparound distance so biomes work on toroidal world
                        let dx = wraparound_delta(cx, biome.x, world.width);
                        let dy = wraparound_delta(cy, biome.y, world.height);
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist <= biome.radius {
                            // Smooth falloff: full effect at center, fades at edge
                            let t = 1.0 - (dist / biome.radius);
                            let effective = 1.0 + (biome.regrowth_multiplier - 1.0) * t;
                            // Highest multiplier wins (biomes don't stack)
                            best_multiplier = best_multiplier.max(effective);
                            // Fruit rate: biome base rate × fruit multiplier
                            let fruit_rate = food.regrowth_rate
                                * biome.regrowth_multiplier
                                * fruit_config.biome_rate_multiplier
                                * t;
                            best_fruit_rate = best_fruit_rate.max(fruit_rate);
                        }
                    }
                    let idx = row * cols + col;
                    regrowth_rates[idx] = (food.regrowth_rate * best_multiplier).max(0.0);
                    fruit_regrowth_rates[idx] = best_fruit_rate;
                }
            }
        }

        // Initial food proportional to regrowth rate — biome zones start with food,
        // barren zones start nearly empty. Creates patchy resources from tick 0.
        let base = food.regrowth_rate;
        for i in 0..num_cells {
            let ratio = if base > 0.0 {
                (regrowth_rates[i] / base).min(3.0) / 3.0
            } else {
                0.0
            };
            cells[i] = food.max_per_cell * ratio;
        }

        FoodGrid {
            cols,
            rows,
            cell_size: food.cell_size,
            max_per_cell: food.max_per_cell,
            regrowth_rate: food.regrowth_rate,
            cells,
            regrowth_rates,
            world_width: world.width,
            world_height: world.height,
            droughts: Vec::new(),
            fruit_cells: vec![0.0f64; num_cells],
            fruit_regrowth_rates,
        }
    }

    /// Eat fruit at a position. Returns the amount consumed.
    #[inline]
    pub fn eat_fruit_at(&mut self, x: f64, y: f64, max_bite: f64) -> f64 {
        let idx = self.cell_index(x, y);
        let available = self.fruit_cells[idx];
        let consumed = available.min(max_bite);
        self.fruit_cells[idx] -= consumed;
        consumed
    }

    /// Read fruit level at a position (for vision rays).
    #[inline]
    pub fn fruit_at(&self, x: f64, y: f64) -> f64 {
        let idx = self.cell_index(x, y);
        self.fruit_cells[idx]
    }

    /// Regrow food in every cell using per-cell regrowth rates,
    /// modulated by the seasonal multiplier and active droughts.
    ///
    /// Also regrows fruit in biome cells at the per-cell fruit rate.
    ///
    /// Seasonal formula: `multiplier = 1.0 + amplitude * sin(2π * tick / period)`
    /// Citation: Botkin (1990) "Forest Dynamics: An Ecological Model"
    /// Citation: Holling (1973), Connell (1978) — drought disturbance
    pub fn regrow(&mut self, tick: u64, seasons: &SeasonsConfig, fruit_enabled: bool) {
        let max = self.max_per_cell;
        let seasonal = seasonal_multiplier(tick, seasons);
        let half_cell = self.cell_size * 0.5;

        for row in 0..self.rows {
            for col in 0..self.cols {
                let idx = row * self.cols + col;
                if self.cells[idx] >= max {
                    continue;
                }
                let rate = self.regrowth_rates[idx];

                // Check if this cell is inside any active drought
                let drought_mult = if self.droughts.is_empty() {
                    1.0
                } else {
                    let cx = col as f64 * self.cell_size + half_cell;
                    let cy = row as f64 * self.cell_size + half_cell;
                    let mut mult = 1.0f64;
                    for d in &self.droughts {
                        let dx = wraparound_delta(cx, d.x, self.world_width);
                        let dy = wraparound_delta(cy, d.y, self.world_height);
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < d.radius {
                            // Smooth falloff: full severity at center, fading at edge
                            let t = 1.0 - (dist / d.radius);
                            let local_sev = 1.0 - (1.0 - d.severity) * t;
                            mult = mult.min(local_sev);
                        }
                    }
                    mult
                };

                self.cells[idx] = (self.cells[idx] + rate * seasonal * drought_mult).min(max);

                // Regrow fruit in biome cells
                if fruit_enabled {
                    let fruit_rate = self.fruit_regrowth_rates[idx];
                    if fruit_rate > 0.0 {
                        self.fruit_cells[idx] =
                            (self.fruit_cells[idx] + fruit_rate * seasonal * drought_mult).min(max);
                    }
                }
            }
        }

        // Tick down drought timers
        for d in &mut self.droughts {
            d.ticks_remaining = d.ticks_remaining.saturating_sub(1);
        }
        self.droughts.retain(|d| d.ticks_remaining > 0);
    }

    /// Maybe spawn a new drought. Called once per tick from the sim loop.
    /// Uses the sim RNG to maintain determinism.
    pub fn maybe_spawn_drought(&mut self, config: &DroughtConfig, rng: &mut impl rand::Rng) {
        if config.spawn_rate <= 0.0 || self.droughts.len() >= config.max_active {
            return;
        }
        if rng.r#gen::<f64>() < config.spawn_rate {
            let x = rng.gen_range(0.0..self.world_width);
            let y = rng.gen_range(0.0..self.world_height);
            self.droughts.push(ActiveDrought {
                x,
                y,
                radius: config.radius,
                severity: config.severity,
                ticks_remaining: config.duration,
            });
        }
    }

    /// Get the regrowth rate at a given cell index (for visualization).
    #[inline]
    pub fn regrowth_at(&self, col: usize, row: usize) -> f64 {
        self.regrowth_rates[row * self.cols + col]
    }

    /// Get the grid index for a world position.
    #[inline(always)]
    fn cell_index(&self, x: f64, y: f64) -> usize {
        let col = ((x / self.cell_size) as usize).min(self.cols - 1);
        let row = ((y / self.cell_size) as usize).min(self.rows - 1);
        row * self.cols + col
    }

    /// Try to eat food at a position. Returns the amount of food consumed.
    #[inline]
    pub fn eat_at(&mut self, x: f64, y: f64, max_bite: f64) -> f64 {
        let idx = self.cell_index(x, y);
        let available = self.cells[idx];
        let consumed = available.min(max_bite);
        self.cells[idx] -= consumed;
        consumed
    }

    /// Read food level at a position (for vision rays).
    #[inline]
    pub fn food_at(&self, x: f64, y: f64) -> f64 {
        let idx = self.cell_index(x, y);
        self.cells[idx]
    }
}

/// Per-cell pheromone layer enabling stigmergic (indirect) communication.
///
/// Each creature deposits pheromone at its current cell each tick. Pheromone
/// decays multiplicatively, creating concentration gradients over time.
/// The grid uses the same cell size as `FoodGrid` for cache coherence.
///
/// Brain input (normalized level in [0,1]) is wired in the FOURTH step.
///
/// Citation: Wilson, E.O. (1975) "Sociobiology: The New Synthesis"
/// Citation: Deneubourg et al. (1990) "The Self-Organizing Exploratory Pattern
///   of the Argentine Ant" *J. Insect Behavior* 3(2):159–168
#[derive(Debug, Clone)]
pub struct PheromoneGrid {
    pub cols: usize,
    pub rows: usize,
    pub cell_size: f64,
    /// Flat row-major pheromone values.
    pub cells: Vec<f64>,
    deposit_rate: f64,
    decay_rate: f64,
    max_level: f64,
}

impl PheromoneGrid {
    /// Create a zeroed pheromone grid matching the food grid dimensions.
    pub fn new(world: &WorldConfig, cell_size: f64, cfg: &PheromoneConfig) -> Self {
        let cols = (world.width / cell_size).ceil() as usize;
        let rows = (world.height / cell_size).ceil() as usize;
        PheromoneGrid {
            cols,
            rows,
            cell_size,
            cells: vec![0.0f64; cols * rows],
            deposit_rate: cfg.deposit_rate,
            decay_rate: cfg.decay_rate,
            max_level: cfg.max_level,
        }
    }

    /// Deposit pheromone at world position (x, y) — called once per creature per tick.
    #[inline]
    pub fn deposit(&mut self, x: f64, y: f64) {
        let idx = self.cell_index(x, y);
        self.cells[idx] = (self.cells[idx] + self.deposit_rate).min(self.max_level);
    }

    /// Multiplicative decay — applied once per tick after all deposits.
    #[inline]
    pub fn decay(&mut self) {
        for c in self.cells.iter_mut() {
            *c *= self.decay_rate;
        }
    }

    /// Return pheromone level at world position, normalized to [0, 1].
    #[inline]
    pub fn level_at(&self, x: f64, y: f64) -> f64 {
        let idx = self.cell_index(x, y);
        if self.max_level > 0.0 {
            (self.cells[idx] / self.max_level).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    #[inline(always)]
    fn cell_index(&self, x: f64, y: f64) -> usize {
        let col = ((x / self.cell_size) as usize).min(self.cols - 1);
        let row = ((y / self.cell_size) as usize).min(self.rows - 1);
        row * self.cols + col
    }
}

/// Multi-octave sine height map — procedural elevation without external crates.
///
/// Height at (x, y):
///   h = Σ_{k=0}^{octaves-1} (0.5^k) * sin(2^k * fx * x + φkx) * sin(2^k * fy * y + φky)
///
/// Movement penalty applied in the tick loop:
///   effective_speed *= 1.0 / (1.0 + slope_factor * |h(to) - h(from)|)
///
/// This creates ridges, valleys, and island-like enclosed patches that
/// fragment the population and drive allopatric speciation.
///
/// Citation: MacArthur & Wilson (1967) "The Theory of Island Biogeography"
/// Citation: Wright (1943) "Isolation by Distance" Genetics 28(2):114-138
/// Citation: Musgrave (1994) fBm multi-octave noise for terrain synthesis
#[derive(Debug, Clone)]
pub struct ElevationMap {
    octaves: usize,
    slope_factor: f64,
    /// Per-octave x/y base frequencies and phase offsets (four values per octave).
    params: Vec<(f64, f64, f64, f64)>, // (fx, fy, phase_x, phase_y)
}

impl ElevationMap {
    /// Build the height map from config. No external crates — pure sine arithmetic.
    pub fn new(world: &WorldConfig, cfg: &ElevationConfig) -> Self {
        // Derive frequencies and phases from cfg.seed using a simple LCG-style hash.
        // We want them to vary so the map is non-trivial; exact values don't matter.
        let mut state = cfg.seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut params = Vec::with_capacity(cfg.octaves);
        for k in 0..cfg.octaves {
            state = state
                .wrapping_mul(0x6c62272e07bb0142)
                .wrapping_add(0x62b8_2175_229d_6a5b);
            let fx = (1.0 + (state & 0xf) as f64 * 0.5) / world.width;
            state = state
                .wrapping_mul(0x6c62272e07bb0142)
                .wrapping_add(0x62b8_2175_229d_6a5b);
            let fy = (1.0 + (state & 0xf) as f64 * 0.5) / world.height;
            state = state
                .wrapping_mul(0x6c62272e07bb0142)
                .wrapping_add(0x62b8_2175_229d_6a5b);
            let phase_x = (state & 0x3ff) as f64 / 0x3ff as f64 * std::f64::consts::TAU;
            state = state
                .wrapping_mul(0x6c62272e07bb0142)
                .wrapping_add(0x62b8_2175_229d_6a5b);
            let phase_y = (state & 0x3ff) as f64 / 0x3ff as f64 * std::f64::consts::TAU;
            // Scale by 2^k for octave frequency, amplitude 0.5^k for each octave
            let freq_scale = (1u64 << k) as f64;
            params.push((
                fx * freq_scale * std::f64::consts::TAU,
                fy * freq_scale * std::f64::consts::TAU,
                phase_x,
                phase_y,
            ));
        }
        ElevationMap {
            octaves: cfg.octaves,
            slope_factor: cfg.slope_factor,
            params,
        }
    }

    /// Return height at world position, normalized to roughly [-1, 1].
    #[inline]
    pub fn height_at(&self, x: f64, y: f64) -> f64 {
        let mut h = 0.0f64;
        let mut amplitude = 1.0f64;
        for &(fx, fy, px, py) in self.params.iter().take(self.octaves) {
            h += amplitude * (fx * x + px).sin() * (fy * y + py).sin();
            amplitude *= 0.5;
        }
        h
    }

    /// Speed multiplier for a move from (x0, y0) to (x1, y1).
    /// Returns a value in (0, 1]: 1.0 on flat terrain, lower on steep slopes.
    #[inline]
    pub fn speed_multiplier(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
        let dh = (self.height_at(x1, y1) - self.height_at(x0, y0)).abs();
        1.0 / (1.0 + self.slope_factor * dh)
    }
}

/// Spatial hash grid for fast organism neighbor lookups.
///
/// Uses a flat index buffer instead of Vec-per-cell to avoid allocation churn.
/// Each cell stores a (start, count) into a shared index buffer.
#[derive(Debug, Clone)]
pub struct SpatialGrid {
    pub cols: usize,
    pub rows: usize,
    pub cell_size: f64,
    pub world_width: f64,
    pub world_height: f64,
    /// Per-cell: (start_index_in_indices, count).
    cell_ranges: Vec<(u32, u32)>,
    /// Flat buffer of organism indices, grouped by cell.
    indices: Vec<usize>,
    /// Temporary counts used during build.
    counts: Vec<u32>,
}

impl SpatialGrid {
    /// Create a new spatial grid. Cell size should be >= the largest query radius
    /// (vision range) so a query only checks the surrounding 9 cells.
    pub fn new(world: &WorldConfig, cell_size: f64) -> Self {
        let cols = (world.width / cell_size).ceil() as usize;
        let rows = (world.height / cell_size).ceil() as usize;
        let num_cells = cols * rows;
        SpatialGrid {
            cols,
            rows,
            cell_size,
            world_width: world.width,
            world_height: world.height,
            cell_ranges: vec![(0, 0); num_cells],
            indices: Vec::new(),
            counts: vec![0; num_cells],
        }
    }

    /// Rebuild the spatial grid from scratch. Two-pass counting sort — zero allocation
    /// after the first tick (reuses buffers).
    pub fn rebuild(&mut self, positions: &[(f64, f64, bool)]) {
        let num_cells = self.cols * self.rows;

        // Pass 1: count organisms per cell
        for c in self.counts.iter_mut() {
            *c = 0;
        }
        for (i, &(x, y, alive)) in positions.iter().enumerate() {
            if !alive {
                continue;
            }
            let _ = i; // used in pass 2
            let cell = self.cell_index(x, y);
            self.counts[cell] += 1;
        }

        // Compute prefix sums → cell_ranges
        let mut offset: u32 = 0;
        for cell in 0..num_cells {
            self.cell_ranges[cell] = (offset, self.counts[cell]);
            offset += self.counts[cell];
        }

        // Resize indices buffer (no alloc if capacity already enough)
        let total = offset as usize;
        self.indices.resize(total, 0);

        // Pass 2: scatter organism indices into the buffer
        // Reset counts to use as write cursors
        for c in self.counts.iter_mut() {
            *c = 0;
        }
        for (i, &(x, y, alive)) in positions.iter().enumerate() {
            if !alive {
                continue;
            }
            let cell = self.cell_index(x, y);
            let (start, _) = self.cell_ranges[cell];
            let write_pos = start + self.counts[cell];
            self.indices[write_pos as usize] = i;
            self.counts[cell] += 1;
        }
    }

    #[inline(always)]
    fn cell_index(&self, x: f64, y: f64) -> usize {
        let col = ((x / self.cell_size) as usize).min(self.cols - 1);
        let row = ((y / self.cell_size) as usize).min(self.rows - 1);
        row * self.cols + col
    }

    /// Iterate over organism indices in a cell. Returns a slice.
    #[inline]
    pub fn cell_contents(&self, col: usize, row: usize) -> &[usize] {
        let idx = row * self.cols + col;
        let (start, count) = self.cell_ranges[idx];
        &self.indices[start as usize..(start + count) as usize]
    }

    /// Call `f` for each organism index near (x, y) within the grid's cell radius.
    /// Only checks the 3×3 neighborhood (cell_size >= query radius).
    #[inline]
    pub fn for_each_nearby(&self, x: f64, y: f64, mut f: impl FnMut(usize)) {
        let center_col = (x / self.cell_size) as isize;
        let center_row = (y / self.cell_size) as isize;
        let cols = self.cols as isize;
        let rows = self.rows as isize;

        for dr in -1..=1 {
            let row = ((center_row + dr).rem_euclid(rows)) as usize;
            for dc in -1..=1 {
                let col = ((center_col + dc).rem_euclid(cols)) as usize;
                let idx = row * self.cols + col;
                let (start, count) = self.cell_ranges[idx];
                for k in start..(start + count) {
                    f(self.indices[k as usize]);
                }
            }
        }
    }
}

/// Compute the seasonal regrowth multiplier for a given tick.
///
/// Returns `1.0 + amplitude * sin(2π * tick / period)`, clamped to [0, ∞).
/// When amplitude is 0, returns 1.0 (no seasonal effect).
///
/// Citation: Botkin (1990) "Forest Dynamics: An Ecological Model"
#[inline]
pub fn seasonal_multiplier(tick: u64, seasons: &SeasonsConfig) -> f64 {
    if seasons.amplitude == 0.0 || seasons.period <= 0.0 {
        return 1.0;
    }
    let phase = std::f64::consts::TAU * tick as f64 / seasons.period;
    (1.0 + seasons.amplitude * phase.sin()).max(0.0)
}

/// Wraparound distance between two points on a toroidal world.
#[inline]
pub fn wraparound_distance(x1: f64, y1: f64, x2: f64, y2: f64, width: f64, height: f64) -> f64 {
    let dx = wraparound_delta(x1, x2, width);
    let dy = wraparound_delta(y1, y2, height);
    (dx * dx + dy * dy).sqrt()
}

/// Signed shortest delta along one wrapped axis.
#[inline(always)]
pub fn wraparound_delta(a: f64, b: f64, size: f64) -> f64 {
    let mut d = b - a;
    if d > size * 0.5 {
        d -= size;
    } else if d < -size * 0.5 {
        d += size;
    }
    d
}

/// Wrap a coordinate into [0, size).
#[inline(always)]
pub fn wrap_coord(v: f64, size: f64) -> f64 {
    let v = v % size;
    if v < 0.0 { v + size } else { v }
}
