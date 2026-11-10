//! Rendering: food grid as dim green cells, creatures as colored triangles.
//!
//! Design goals from the doc:
//! - Dark background
//! - Dim green food field
//! - Creatures as small triangles pointing where they head, colored by lineage hue

use macroquad::prelude::*;
use sim_core::Simulation;

use crate::camera::Camera;

/// Background color — near-black with a hint of blue.
const BG_COLOR: Color = Color::new(0.04, 0.04, 0.06, 1.0);

/// Draw the entire simulation frame.
pub fn draw_world(sim: &Simulation, camera: &Camera, selected_id: Option<u64>) {
    clear_background(BG_COLOR);
    draw_food_grid(sim, camera);
    draw_droughts(sim, camera);
    draw_creatures(sim, camera, selected_id);
    draw_world_border(sim, camera);
}

/// Render the food grid as colored rectangles. Only draw cells that are on screen.
fn draw_food_grid(sim: &Simulation, camera: &Camera) {
    let grid = &sim.food_grid;
    let cell_px = (grid.cell_size * camera.zoom) as f32;

    // Skip drawing individual cells if they'd be smaller than 2px — just show an average tint
    if cell_px < 2.0 {
        return;
    }

    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let wx = col as f64 * grid.cell_size;
            let wy = row as f64 * grid.cell_size;

            // Frustum cull: skip cells off-screen
            let (sx, sy) = camera.world_to_screen(wx, wy);
            if sx + cell_px < 0.0
                || sy + cell_px < 0.0
                || sx > screen_width()
                || sy > screen_height()
            {
                continue;
            }

            let food = grid.cells[row * grid.cols + col];
            let regrowth = grid.regrowth_at(col, row) as f32;
            let base_regrowth = grid.regrowth_rate as f32;

            if food < 0.01 && regrowth <= 0.0 {
                // Draw barren zones with a faint brown tint
                draw_rectangle(sx, sy, cell_px, cell_px, Color::new(0.08, 0.05, 0.03, 0.3));
                continue;
            }

            if food < 0.01 {
                continue;
            }

            // Color by biome: lush zones get brighter green, barren gets brownish
            let intensity = (food / grid.max_per_cell) as f32;
            let richness = if base_regrowth > 0.0 {
                (regrowth / base_regrowth).clamp(0.0, 4.0) / 4.0
            } else {
                0.0
            };
            // Blend from brown (barren) to bright green (lush)
            let r = 0.05 + 0.08 * (1.0 - richness);
            let g = 0.15 + 0.45 * intensity * (0.5 + 0.5 * richness);
            let b = 0.03 + 0.07 * richness;
            let color = Color::new(r, g, b, 0.5 + 0.4 * intensity);
            draw_rectangle(sx, sy, cell_px, cell_px, color);
        }
    }
}

/// Render active drought zones as semi-transparent red circles.
fn draw_droughts(sim: &Simulation, camera: &Camera) {
    for drought in &sim.food_grid.droughts {
        let (sx, sy) = camera.world_to_screen(drought.x, drought.y);
        let radius_px = (drought.radius * camera.zoom) as f32;

        // Fade opacity based on remaining time (pulses as it nears expiry)
        let alpha = 0.12 + 0.05 * ((drought.ticks_remaining as f32 * 0.01).sin());
        let color = Color::new(0.8, 0.15, 0.05, alpha);
        draw_circle(sx, sy, radius_px, color);

        // Thin border ring
        let border = Color::new(0.9, 0.2, 0.1, 0.25);
        draw_circle_lines(sx, sy, radius_px, 1.5, border);
    }
}

/// Render each creature as a small triangle pointing in its heading direction.
fn draw_creatures(sim: &Simulation, camera: &Camera, selected_id: Option<u64>) {
    let base_size = (sim.config.creature.body_radius * camera.zoom) as f32;
    // Ensure triangles are always at least a few pixels so they're visible when zoomed out
    let size = base_size.max(2.5);

    for org in &sim.organisms {
        if !org.alive {
            continue;
        }

        let (sx, sy) = camera.world_to_screen(org.pos_x, org.pos_y);

        // Frustum cull
        if sx < -size || sy < -size || sx > screen_width() + size || sy > screen_height() + size {
            continue;
        }

        let color = hue_to_color(
            org.genome.color_hue as f32,
            org.energy as f32 / sim.config.energy.maximum as f32,
        );

        // Triangle pointing in heading direction
        let h = org.heading as f32;
        let tip_x = sx + h.cos() * size * 1.5;
        let tip_y = sy + h.sin() * size * 1.5;
        let left_angle = h + 2.4; // ~140 degrees back
        let right_angle = h - 2.4;
        let back_x1 = sx + left_angle.cos() * size;
        let back_y1 = sy + left_angle.sin() * size;
        let back_x2 = sx + right_angle.cos() * size;
        let back_y2 = sy + right_angle.sin() * size;

        draw_triangle(
            Vec2::new(tip_x, tip_y),
            Vec2::new(back_x1, back_y1),
            Vec2::new(back_x2, back_y2),
            color,
        );

        // Selection highlight ring
        if selected_id == Some(org.id) {
            draw_circle_lines(sx, sy, size * 2.5, 1.5, WHITE);
        }
    }
}

/// Draw a faint border around the world so you can see the wraparound edges.
fn draw_world_border(sim: &Simulation, camera: &Camera) {
    let (x0, y0) = camera.world_to_screen(0.0, 0.0);
    let (x1, y1) = camera.world_to_screen(sim.config.world.width, sim.config.world.height);
    let border_color = Color::new(0.2, 0.2, 0.3, 0.5);
    draw_rectangle_lines(x0, y0, x1 - x0, y1 - y0, 2.0, border_color);
}

/// Convert HSV (hue + brightness) to an RGB color.
/// Hue in [0, 360), saturation fixed at 0.8, value scaled by energy ratio.
fn hue_to_color(hue: f32, energy_ratio: f32) -> Color {
    let s = 0.8;
    let v = 0.5 + 0.5 * energy_ratio.clamp(0.0, 1.0); // brighter when well-fed

    let h = hue / 60.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::new(r + m, g + m, b + m, 1.0)
}
