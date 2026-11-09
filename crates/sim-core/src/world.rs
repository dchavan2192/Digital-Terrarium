//! The 2D wraparound world: food grid and spatial index.
//!
//! The food grid is a coarse grid of cells, each holding a food value that
//! regrows over time. The spatial grid is a separate, finer grid used for
//! fast neighbor lookups (vision rays, eating range).

use crate::config::{FoodConfig, SeasonsConfig, WorldConfig};

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
}

impl FoodGrid {
    pub fn new(world: &WorldConfig, food: &FoodConfig) -> Self {
        let cols = (world.width / food.cell_size).ceil() as usize;
        let rows = (world.height / food.cell_size).ceil() as usize;
        let num_cells = cols * rows;
        // Start every cell at max food so the world begins lush
        let cells = vec![food.max_per_cell; num_cells];

        // Compute per-cell regrowth rates from biome zones
        let mut regrowth_rates = vec![food.regrowth_rate; num_cells];
        if !food.biomes.is_empty() {
            let half_cell = food.cell_size * 0.5;
            for row in 0..rows {
                for col in 0..cols {
                    let cx = col as f64 * food.cell_size + half_cell;
                    let cy = row as f64 * food.cell_size + half_cell;
                    let mut best_multiplier = 1.0f64;
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
                        }
                    }
                    let idx = row * cols + col;
                    regrowth_rates[idx] = (food.regrowth_rate * best_multiplier).max(0.0);
                }
            }
        }

        FoodGrid {
            cols,
            rows,
            cell_size: food.cell_size,
            max_per_cell: food.max_per_cell,
            regrowth_rate: food.regrowth_rate,
            cells,
            regrowth_rates,
        }
    }

    /// Regrow food in every cell using per-cell regrowth rates,
    /// modulated by the seasonal multiplier.
    ///
    /// Seasonal formula: `multiplier = 1.0 + amplitude * sin(2π * tick / period)`
    /// Citation: Botkin (1990) "Forest Dynamics: An Ecological Model"
    pub fn regrow(&mut self, tick: u64, seasons: &SeasonsConfig) {
        let max = self.max_per_cell;
        let seasonal = seasonal_multiplier(tick, seasons);
        for (cell, &rate) in self.cells.iter_mut().zip(self.regrowth_rates.iter()) {
            if *cell < max {
                *cell = (*cell + rate * seasonal).min(max);
            }
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
