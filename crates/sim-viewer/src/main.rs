//! sim-viewer: Visual viewer for the Terrarium simulation.
//!
//! Renders the world with macroquad and provides an egui control panel for
//! inspecting and controlling the simulation in real time.
//!
//! Usage:
//!   cargo run -p sim-viewer
//!   cargo run -p sim-viewer -- path/to/config.toml

mod camera;
mod renderer;
mod ui;

use camera::Camera;
use macroquad::prelude::*;
use sim_core::{Config, Simulation};
use ui::UiState;

fn window_conf() -> Conf {
    Conf {
        window_title: "Terrarium".to_string(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        ..Default::default()
    }
}

/// Default config embedded at compile time for WASM builds (no filesystem access).
#[cfg(target_arch = "wasm32")]
const EMBEDDED_CONFIG: &str = include_str!("../../../configs/default.toml");

#[macroquad::main(window_conf)]
async fn main() {
    // Load config: embedded for WASM, from file for native
    #[cfg(target_arch = "wasm32")]
    let config = Config::parse(EMBEDDED_CONFIG).unwrap_or_else(|e| {
        eprintln!("Failed to parse embedded config: {}", e);
        panic!("Bad embedded config");
    });

    #[cfg(not(target_arch = "wasm32"))]
    let config = {
        let config_path = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "configs/default.toml".to_string());
        Config::from_file(&config_path).unwrap_or_else(|e| {
            eprintln!("Failed to load config '{}': {}", config_path, e);
            eprintln!("Usage: cargo run -p sim-viewer [-- path/to/config.toml]");
            std::process::exit(1);
        })
    };

    let mut sim = Simulation::new(config);
    let mut camera = Camera::new(sim.config.world.width, sim.config.world.height);
    let mut ui_state = UiState::default();

    loop {
        // --- Input ---
        camera.update(ui_state.panel_width);

        // Space bar toggles pause
        if is_key_pressed(KeyCode::Space) {
            ui_state.paused = !ui_state.paused;
        }

        // Left-click to select nearest creature
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx > ui_state.panel_width {
                let (wx, wy) = camera.screen_to_world(mx, my);
                ui_state.selected_id = find_nearest_organism(&sim, wx, wy, 20.0 / camera.zoom);
            }
        }

        // --- Simulate ---
        if !ui_state.paused && !sim.is_extinct() {
            for _ in 0..ui_state.ticks_per_frame {
                sim.step();
                if sim.is_extinct() {
                    break;
                }
            }
        }

        // --- Render ---
        renderer::draw_world(&sim, &camera, ui_state.selected_id);

        // Extinction notice
        if sim.is_extinct() {
            let msg = format!("EXTINCT at tick {}", sim.tick);
            let font_size = 40.0;
            let dims = measure_text(&msg, None, font_size as u16, 1.0);
            draw_text(
                &msg,
                screen_width() / 2.0 - dims.width / 2.0,
                screen_height() / 2.0,
                font_size,
                RED,
            );
        }

        // --- egui UI ---
        egui_macroquad::ui(|egui_ctx| {
            ui::draw_ui(egui_ctx, &sim, &mut ui_state);
        });
        egui_macroquad::draw();

        next_frame().await;
    }
}

/// Find the organism nearest to (wx, wy) within `max_dist` world units.
fn find_nearest_organism(sim: &Simulation, wx: f64, wy: f64, max_dist: f64) -> Option<u64> {
    let mut best_id = None;
    let mut best_dist = max_dist;

    for org in &sim.organisms {
        if !org.alive {
            continue;
        }
        let dx = org.pos_x - wx;
        let dy = org.pos_y - wy;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < best_dist {
            best_dist = dist;
            best_id = Some(org.id);
        }
    }

    best_id
}
