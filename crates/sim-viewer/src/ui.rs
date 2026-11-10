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
    /// Whether the species panel is expanded.
    pub species_expanded: bool,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            paused: false,
            ticks_per_frame: 1,
            selected_id: None,
            panel_width: 260.0,
            species_expanded: true,
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
            ui.label(format!(
                "Species: {} (t={:.2})",
                stats.species_count, stats.speciation_threshold
            ));
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
            ui.label(format!("Brain entropy: {:.3}", stats.mean_brain_entropy));
            ui.label(format!(
                "Behavioral diversity: {:.3}",
                stats.behavioral_diversity
            ));
            ui.label(format!("Mean connections: {:.0}", stats.mean_connections));
            ui.label(format!("Mean crowding: {:.1}", stats.mean_crowding));
            // Seasonal food multiplier — Botkin (1990)
            let seasonal = sim_core::world::seasonal_multiplier(sim.tick, &sim.config.food.seasons);
            if sim.config.food.seasons.amplitude > 0.0 {
                ui.label(format!("Season: {:.0}%", seasonal * 100.0));
            }
            if stats.active_droughts > 0 {
                ui.label(format!("Droughts: {}", stats.active_droughts));
            }
            ui.label(format!("FPS: {:.0}", macroquad::time::get_fps()));

            ui.separator();

            // --- Species breakdown ---
            let species_header =
                egui::CollapsingHeader::new("Species").default_open(state.species_expanded);
            let response = species_header.show(ui, |ui| {
                let species = &sim.stats.species;
                if species.is_empty() {
                    ui.label("No species data yet.");
                } else {
                    // Show top species (cap at 12 to avoid UI overload)
                    let show_count = species.len().min(12);
                    for info in &species[..show_count] {
                        // Color swatch + population + trend arrow
                        ui.horizontal(|ui| {
                            // Color swatch from mean hue
                            let hue = info.mean_hue as f32 / 360.0;
                            let (r, g, b) = hsv_to_rgb(hue, 0.7, 0.9);
                            let color = egui::Color32::from_rgb(
                                (r * 255.0) as u8,
                                (g * 255.0) as u8,
                                (b * 255.0) as u8,
                            );
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, color);

                            // Trend arrow
                            let trend_str = if info.trend > 0.1 {
                                "▲"
                            } else if info.trend < -0.1 {
                                "▼"
                            } else {
                                "—"
                            };

                            ui.label(format!(
                                "{} pop:{} e:{:.0} sz:{:.1} c:{:.0}",
                                trend_str,
                                info.population,
                                info.mean_energy,
                                info.mean_body_size,
                                info.mean_connections,
                            ));
                        });

                        // Second line: trophic + generation
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            let diet = if info.herbivore_fraction > 0.75 {
                                "herb"
                            } else if info.predator_fraction > 0.5 {
                                "pred"
                            } else {
                                "omni"
                            };
                            ui.label(format!(
                                "gen:{:.0} troph:{:.2} ({})",
                                info.mean_generation, info.mean_trophic_level, diet,
                            ));
                        });
                        ui.add_space(2.0);
                    }
                    if species.len() > show_count {
                        ui.label(format!("… +{} more", species.len() - show_count));
                    }
                }
            });
            state.species_expanded = response.fully_open();

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
                        "Brain: {} nodes ({} hidden), {} connections",
                        org.brain.inputs + org.brain.hidden + org.brain.outputs,
                        org.brain.hidden,
                        org.brain.connection_count()
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
                    // Brain output entropy (Shannon, 1948)
                    let org_entropy: f64 = (0..sim_core::organism::NUM_OUTPUTS)
                        .map(|o| sim_core::organism::shannon_entropy(&org.output_bins[o]))
                        .sum::<f64>()
                        / sim_core::organism::NUM_OUTPUTS as f64;
                    ui.label(format!("Brain entropy: {:.3}", org_entropy));

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

/// Convert HSV to RGB. h in [0,1], s in [0,1], v in [0,1].
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match (i as i32) % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
