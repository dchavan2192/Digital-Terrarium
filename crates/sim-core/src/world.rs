//! The 2D wraparound world: food grid and spatial index.
//!
//! The food grid is a coarse grid of cells, each holding a food value that
//! regrows over time. The spatial grid is a separate, finer grid used for
//! fast neighbor lookups (vision rays, eating range).

use crate::config::{FoodConfig, WorldConfig};

/// The food resource layer. A grid of cells that regrow over time.
#[derive(Debug, Clone)]
pub struct FoodGrid {
    pub cols: usize,
    pub rows: usize,
    pub cell_size: f64,
    pub max_per_cell: f64,
    pub regrowth_rate: f64,
    /// Flat row-major storage of food values.
    pub cells: Vec<f64>,
}

impl FoodGrid {
    pub fn new(world: &WorldConfig, food: &FoodConfig) -> Self {
        let cols = (world.width / food.cell_size).ceil() as usize;
        let rows = (world.height / food.cell_size).ceil() as usize;
        // Start every cell at max food so the world begins lush
        let cells = vec![food.max_per_cell; cols * rows];
        FoodGrid {
            cols,
            rows,
            cell_size: food.cell_size,
            max_per_cell: food.max_per_cell,
            regrowth_rate: food.regrowth_rate,
            cells,
        }
    }

    /// Regrow food in every cell, capped at max_per_cell.
    pub fn regrow(&mut self) {
        for cell in &mut self.cells {
            *cell = (*cell + self.regrowth_rate).min(self.max_per_cell);
        }
    }

    /// Get the grid index for a world position.
    fn cell_index(&self, x: f64, y: f64) -> usize {
        let col = ((x / self.cell_size) as usize).min(self.cols - 1);
        let row = ((y / self.cell_size) as usize).min(self.rows - 1);
        row * self.cols + col
    }

    /// Try to eat food at a position. Returns the amount of food consumed (0.0 to available).
    /// A creature eats up to `max_bite` from the cell it's standing on.
    pub fn eat_at(&mut self, x: f64, y: f64, max_bite: f64) -> f64 {
        let idx = self.cell_index(x, y);
        let available = self.cells[idx];
        let consumed = available.min(max_bite);
        self.cells[idx] -= consumed;
        consumed
    }

    /// Read food level at a position (for vision rays).
    pub fn food_at(&self, x: f64, y: f64) -> f64 {
        let idx = self.cell_index(x, y);
        self.cells[idx]
    }
}

/// Spatial hash grid for fast organism neighbor lookups.
///
/// Each cell holds a list of organism indices. Rebuilt every tick from scratch
/// (cheaper than incremental updates for our population size).
#[derive(Debug, Clone)]
pub struct SpatialGrid {
    pub cols: usize,
    pub rows: usize,
    pub cell_size: f64,
    pub world_width: f64,
    pub world_height: f64,
    /// Each cell holds indices into the organism vec.
    pub cells: Vec<Vec<usize>>,
}

impl SpatialGrid {
    /// Create a new spatial grid. Cell size should be >= the largest query radius
    /// (vision range) so a query only checks the surrounding 9 cells.
    pub fn new(world: &WorldConfig, cell_size: f64) -> Self {
        let cols = (world.width / cell_size).ceil() as usize;
        let rows = (world.height / cell_size).ceil() as usize;
        let cells = vec![Vec::new(); cols * rows];
        SpatialGrid {
            cols,
            rows,
            cell_size,
            world_width: world.width,
            world_height: world.height,
            cells,
        }
    }

    /// Clear all cells. Called at the start of each tick before reinserting.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    /// Insert an organism index at a world position.
    pub fn insert(&mut self, x: f64, y: f64, index: usize) {
        let col = ((x / self.cell_size) as usize).min(self.cols - 1);
        let row = ((y / self.cell_size) as usize).min(self.rows - 1);
        self.cells[row * self.cols + col].push(index);
    }

    /// Query all organism indices within a radius of (x, y).
    /// Returns indices; caller filters by actual distance.
    /// Handles wraparound by checking neighboring cells across edges.
    pub fn query_radius(&self, x: f64, y: f64, radius: f64) -> Vec<usize> {
        let mut results = Vec::new();
        let cell_radius = (radius / self.cell_size).ceil() as isize;
        let center_col = (x / self.cell_size) as isize;
        let center_row = (y / self.cell_size) as isize;

        for dr in -cell_radius..=cell_radius {
            for dc in -cell_radius..=cell_radius {
                let col = ((center_col + dc).rem_euclid(self.cols as isize)) as usize;
                let row = ((center_row + dr).rem_euclid(self.rows as isize)) as usize;
                results.extend_from_slice(&self.cells[row * self.cols + col]);
            }
        }
        results
    }
}

/// Wraparound distance between two points on a toroidal world.
pub fn wraparound_distance(x1: f64, y1: f64, x2: f64, y2: f64, width: f64, height: f64) -> f64 {
    let dx = wraparound_delta(x1, x2, width);
    let dy = wraparound_delta(y1, y2, height);
    (dx * dx + dy * dy).sqrt()
}

/// Signed shortest delta along one wrapped axis.
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
pub fn wrap_coord(v: f64, size: f64) -> f64 {
    let v = v % size;
    if v < 0.0 { v + size } else { v }
}
