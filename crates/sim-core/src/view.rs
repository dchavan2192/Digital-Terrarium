//! Read-only observation helpers for front ends (sim-wasm, viewers).
//!
//! Nothing here mutates the simulation or touches the RNG, so calling these
//! between ticks cannot affect determinism. There is no rendering code here:
//! these functions only sample simulation state into plain numbers/buffers.

use crate::sim::Simulation;
use crate::world::seasonal_multiplier;

impl Simulation {
    /// Day/night light level in `[0, 1]` for the tick that will be processed next
    /// (1.0 when the day/night cycle is disabled).
    ///
    /// Mirrors the formula at the top of [`Simulation::step`]:
    /// `L = (1 + sin(2π · tick / period)) / 2`. Night is `L < 0.5`.
    /// Citation: Kronfeld-Schor & Dayan (2003)
    pub fn light_level(&self) -> f64 {
        let dn = &self.config.daynight;
        if dn.enabled && dn.period > 0.0 {
            let phase = std::f64::consts::TAU * self.tick as f64 / dn.period;
            (1.0 + crate::math::sin(phase)) * 0.5
        } else {
            1.0
        }
    }

    /// Whether the next tick is a night tick (vision range halved).
    pub fn is_night(&self) -> bool {
        self.config.daynight.enabled && self.light_level() < 0.5
    }

    /// Seasonal food-regrowth multiplier for the current tick.
    /// Citation: Botkin (1990)
    pub fn season_multiplier(&self) -> f64 {
        seasonal_multiplier(self.tick, &self.config.food.seasons)
    }

    /// Sample the analytic elevation map at food-grid cell centres into `out`
    /// (row-major, `food_grid.cols × food_grid.rows`). Zeros when elevation is
    /// disabled. `out` is cleared and reused.
    pub fn sample_elevation_grid(&self, out: &mut Vec<f32>) {
        let fg = &self.food_grid;
        let (w, h) = (self.config.world.width, self.config.world.height);
        let enabled = self.config.elevation.enabled;
        out.clear();
        out.reserve(fg.cols * fg.rows);
        for r in 0..fg.rows {
            let y = ((r as f64 + 0.5) * fg.cell_size).min(h);
            for c in 0..fg.cols {
                let x = ((c as f64 + 0.5) * fg.cell_size).min(w);
                let v = if enabled {
                    self.elevation_map.height_at(x, y)
                } else {
                    0.0
                };
                out.push(v as f32);
            }
        }
    }

    /// Per-cell biome regrowth multiplier (`cell base rate / food.regrowth_rate`)
    /// into `out` (row-major, food-grid dims). Static: excludes seasons, droughts
    /// and niche construction. `out` is cleared and reused.
    pub fn sample_biome_grid(&self, out: &mut Vec<f32>) {
        let fg = &self.food_grid;
        let base = self.config.food.regrowth_rate;
        out.clear();
        out.reserve(fg.cols * fg.rows);
        for r in 0..fg.rows {
            for c in 0..fg.cols {
                let m = if base > 0.0 {
                    fg.regrowth_at(c, r) / base
                } else {
                    0.0
                };
                out.push(m as f32);
            }
        }
    }
}
