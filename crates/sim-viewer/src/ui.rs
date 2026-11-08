//! egui control panel: stats, speed slider, pause/play, organism inspector.

use egui::{Align, Layout};
use sim_core::Simulation;

/// Persistent UI state across frames.
pub struct UiState {
    pub paused: bool,
    /// How many simulation ticks to run per rendered frame.
    pub ticks_per_frame: u32,
    /// ID of the selected organism (click to inspect).
    pub selected_id: Option<u64>,
    /// Width of the side panel in pixels (used by camera to avoid pan conflicts).
    pub panel_width: f32,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            paused: false,
            ticks_per_frame: 1,
            selected_id: None,
            panel_width: 260.0,
        }
    }
}

/// Draw the egui side panel. Returns the panel width so the camera knows where the viewport starts.
pub fn draw_ui(egui_ctx: &egui::Context, sim: &Simulation, state: &mut UiState) {
    egui::SidePanel::left("control_panel")
        .default_width(260.0)
        .resizable(true)
        .show(egui_ctx, |ui| {
            state.panel_width = ui.available_width() + 16.0; // account for panel padding

            ui.heading("Terrarium");
            ui.separator();

            // --- Playback controls ---
            ui.horizontal(|ui| {
                if ui
                    .button(if state.paused { "  Play  " } else { " Pause " })
                    .clicked()
                {
                    state.paused = !state.paused;
                }
                if ui.button(" Step ").clicked() {
                    state.paused = true;
                    // The main loop will step once when this is set
                    state.ticks_per_frame = 1;
                }
            });

            ui.add_space(4.0);
            ui.label("Speed (ticks/frame):");
            ui.add(egui::Slider::new(&mut state.ticks_per_frame, 1..=50).logarithmic(true));

            ui.separator();

            // --- Simulation stats ---
            ui.heading("Stats");
            let stats = &sim.stats;
            ui.label(format!("Tick: {}", sim.tick));
            ui.label(format!("Population: {}", stats.population));
            ui.label(format!("Births (last tick): {}", stats.births));
            ui.label(format!("Deaths (last tick): {}", stats.deaths));
            ui.label(format!("Mean energy: {:.1}", stats.mean_energy));
            ui.label(format!("Mean neurons: {:.1}", stats.mean_brain_neurons));
            ui.label(format!("Species: {}", stats.species_count));
            ui.label(format!("Mean generation: {:.0}", stats.mean_generation));
            ui.label(format!(
                "Trophic: {:.2} (H:{} P:{})",
                stats.mean_trophic_level, stats.herbivore_count, stats.predator_count
            ));
            ui.label(format!(
                "Cooperation: {:.3} (kin: {:.0}%)",
                stats.cooperation_rate,
                stats.kin_share_ratio * 100.0
            ));
            ui.label(format!("FPS: {:.0}", macroquad::time::get_fps()));

            ui.separator();

            // --- Selected organism inspector ---
            ui.heading("Inspector");
            if let Some(id) = state.selected_id {
                if let Some(org) = sim.organisms.iter().find(|o| o.id == id && o.alive) {
                    ui.label(format!("ID: {}", org.id));
                    ui.label(format!("Age: {} ticks", org.age));
                    ui.label(format!(
                        "Energy: {:.1} / {:.0}",
                        org.energy, sim.config.energy.maximum
                    ));
                    ui.label(format!("Position: ({:.0}, {:.0})", org.pos_x, org.pos_y));
                    ui.label(format!("Heading: {:.1} deg", org.heading.to_degrees()));
                    ui.label(format!("Hue: {:.0}", org.genome.color_hue));
                    ui.label(format!("Body size: {:.2}", org.genome.body_size));
                    ui.label(format!("Signal: {:.2}", org.signal));
                    ui.label(format!("Mutation rate: {:.3}", org.genome.mutation_rate));
                    ui.label(format!(
                        "Brain: {}→{}→{} ({} weights)",
                        org.brain.inputs,
                        org.brain.hidden,
                        org.brain.outputs,
                        org.brain.weights.len()
                    ));
                    ui.label(format!("Generation: {}", org.generation));
                    ui.label(format!(
                        "Parent: {}",
                        org.parent_id
                            .map(|id| id.to_string())
                            .unwrap_or_else(|| "none (gen 0)".to_string())
                    ));
                    let total_intake = org.energy_from_food + org.energy_from_attacks;
                    let trophic = if total_intake > 0.0 {
                        1.0 + org.energy_from_attacks / total_intake
                    } else {
                        1.0
                    };
                    ui.label(format!("Trophic level: {:.2}", trophic));
                    ui.label(format!(
                        "Energy from: food {:.0} / attacks {:.0}",
                        org.energy_from_food, org.energy_from_attacks
                    ));

                    if ui.button("Deselect").clicked() {
                        state.selected_id = None;
                    }
                } else {
                    ui.label("Organism died or not found.");
                    if ui.button("Clear selection").clicked() {
                        state.selected_id = None;
                    }
                }
            } else {
                ui.label("Click a creature to inspect it.");
            }

            // Push keyboard hints to the bottom
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                ui.add_space(4.0);
                ui.label("Scroll: zoom | Right-drag: pan");
                ui.label("WASD/arrows: pan | Space: pause");
                ui.separator();
                ui.label("Controls");
            });
        });
}
