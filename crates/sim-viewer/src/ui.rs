//! egui control panel: stats, speed slider, pause/play, organism inspector.

use egui::{Align, Color32, Layout, Rect, Vec2};
use sim_core::Simulation;

/// Number of ticks of history to keep for sparklines.
const HISTORY_LEN: usize = 200;

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
    /// History ring buffers for sparklines.
    pub history: StatsHistory,
}

/// Ring buffers for recent stat values (sparkline display).
pub struct StatsHistory {
    pub population: Vec<f32>,
    pub mean_energy: Vec<f32>,
    pub cooperation_rate: Vec<f32>,
    pub mean_trophic: Vec<f32>,
    pub attacks: Vec<f32>,
    pub last_tick: u64,
}

impl Default for StatsHistory {
    fn default() -> Self {
        StatsHistory {
            population: Vec::with_capacity(HISTORY_LEN),
            mean_energy: Vec::with_capacity(HISTORY_LEN),
            cooperation_rate: Vec::with_capacity(HISTORY_LEN),
            mean_trophic: Vec::with_capacity(HISTORY_LEN),
            attacks: Vec::with_capacity(HISTORY_LEN),
            last_tick: 0,
        }
    }
}

impl StatsHistory {
    fn push(&mut self, sim: &Simulation) {
        if sim.tick == self.last_tick {
            return;
        }
        self.last_tick = sim.tick;

        fn ring_push(buf: &mut Vec<f32>, val: f32) {
            if buf.len() >= HISTORY_LEN {
                buf.remove(0);
            }
            buf.push(val);
        }

        let s = &sim.stats;
        ring_push(&mut self.population, s.population as f32);
        ring_push(&mut self.mean_energy, s.mean_energy as f32);
        ring_push(&mut self.cooperation_rate, s.cooperation_rate as f32);
        ring_push(&mut self.mean_trophic, s.mean_trophic_level as f32);
        ring_push(&mut self.attacks, s.attacks as f32);
    }
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            paused: false,
            ticks_per_frame: 1,
            selected_id: None,
            panel_width: 260.0,
            species_expanded: true,
            history: StatsHistory::default(),
        }
    }
}

/// Draw a mini sparkline chart inline.
fn sparkline(ui: &mut egui::Ui, data: &[f32], color: Color32, height: f32) {
    if data.len() < 2 {
        return;
    }
    let width = ui.available_width().min(200.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    // Background
    painter.rect_filled(rect, 2.0, Color32::from_rgba_unmultiplied(20, 20, 30, 180));

    let min_val = data.iter().copied().fold(f32::MAX, f32::min);
    let max_val = data.iter().copied().fold(f32::MIN, f32::max);
    let range = (max_val - min_val).max(0.001);

    let points: Vec<egui::Pos2> = data
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = rect.left() + (i as f32 / (data.len() - 1) as f32) * rect.width();
            let y = rect.bottom() - ((v - min_val) / range) * rect.height();
            egui::pos2(x, y)
        })
        .collect();

    // Draw line
    for w in points.windows(2) {
        painter.line_segment([w[0], w[1]], egui::Stroke::new(1.2_f32, color));
    }

    // Current value label
    if let Some(&last) = data.last() {
        let text = if last >= 1000.0 {
            format!("{:.0}", last)
        } else if last >= 1.0 {
            format!("{:.1}", last)
        } else {
            format!("{:.3}", last)
        };
        painter.text(
            egui::pos2(rect.right() - 2.0, rect.top() + 2.0),
            egui::Align2::RIGHT_TOP,
            text,
            egui::FontId::monospace(9.0),
            Color32::from_rgb(200, 200, 210),
        );
    }
}

/// Draw a horizontal progress/stat bar.
fn stat_bar(ui: &mut egui::Ui, label: &str, value: f32, max: f32, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(label);
        let frac = (value / max).clamp(0.0, 1.0);
        let width = ui.available_width().min(120.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 14.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 2.0, Color32::from_rgb(30, 30, 40));

        // Fill
        let fill_rect =
            Rect::from_min_size(rect.min, Vec2::new(rect.width() * frac, rect.height()));
        painter.rect_filled(fill_rect, 2.0, color);

        // Value text
        let text = if value >= 100.0 {
            format!("{:.0}", value)
        } else if value >= 1.0 {
            format!("{:.1}", value)
        } else {
            format!("{:.2}", value)
        };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(9.0),
            Color32::from_rgb(220, 220, 230),
        );
    });
}

/// Draw the egui side panel.
pub fn draw_ui(egui_ctx: &egui::Context, sim: &Simulation, state: &mut UiState) {
    // Update history
    state.history.push(sim);

    egui::SidePanel::left("control_panel")
        .default_width(280.0)
        .resizable(true)
        .show(egui_ctx, |ui| {
            state.panel_width = ui.available_width() + 16.0;

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
                    state.ticks_per_frame = 1;
                }
                ui.label(format!("Tick {}", sim.tick));
            });

            ui.add_space(2.0);
            ui.label("Speed (ticks/frame):");
            ui.add(egui::Slider::new(&mut state.ticks_per_frame, 1..=50).logarithmic(true));

            ui.separator();

            // --- Population ---
            let stats = &sim.stats;
            let hard_cap = sim.config.creature.hard_cap as f32;

            ui.label("Population");
            stat_bar(
                ui,
                "",
                stats.population as f32,
                hard_cap,
                Color32::from_rgb(63, 185, 80),
            );
            sparkline(
                ui,
                &state.history.population,
                Color32::from_rgb(63, 185, 80),
                30.0,
            );
            ui.horizontal(|ui| {
                ui.label(format!(
                    "B:{} D:{} Gen:{:.0}",
                    stats.births, stats.deaths, stats.mean_generation
                ));
            });

            ui.add_space(4.0);

            // --- Energy ---
            ui.label("Energy");
            stat_bar(
                ui,
                "",
                stats.mean_energy as f32,
                sim.config.energy.maximum as f32,
                Color32::from_rgb(210, 153, 34),
            );
            sparkline(
                ui,
                &state.history.mean_energy,
                Color32::from_rgb(210, 153, 34),
                30.0,
            );

            ui.add_space(4.0);

            // --- Cooperation ---
            ui.label("Cooperation");
            stat_bar(
                ui,
                "",
                stats.cooperation_rate as f32,
                1.0,
                Color32::from_rgb(88, 166, 255),
            );
            sparkline(
                ui,
                &state.history.cooperation_rate,
                Color32::from_rgb(88, 166, 255),
                30.0,
            );
            if stats.cooperation_rate > 0.001 {
                ui.label(format!(
                    "Kin-directed: {:.0}%  Shares: {}",
                    stats.kin_share_ratio * 100.0,
                    stats.shares
                ));
            }

            ui.add_space(4.0);

            // --- Ecology ---
            ui.label("Ecology");
            stat_bar(
                ui,
                "Trophic",
                stats.mean_trophic_level as f32,
                2.0,
                Color32::from_rgb(248, 81, 73),
            );
            ui.horizontal(|ui| {
                ui.label(format!(
                    "H:{} P:{} Atk:{}",
                    stats.herbivore_count, stats.predator_count, stats.attacks
                ));
            });
            sparkline(
                ui,
                &state.history.attacks,
                Color32::from_rgb(248, 81, 73),
                24.0,
            );

            ui.add_space(4.0);

            // --- Brain ---
            ui.label("Brains");
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Conn:{:.0} Entropy:{:.2} Div:{:.2}",
                    stats.mean_connections, stats.mean_brain_entropy, stats.behavioral_diversity
                ));
            });

            ui.add_space(4.0);

            // --- Environment ---
            ui.label("Environment");
            ui.horizontal(|ui| {
                ui.label(format!("Species: {}", stats.species_count));
                ui.label(format!("Crowding: {:.1}", stats.mean_crowding));
            });
            let seasonal = sim_core::world::seasonal_multiplier(sim.tick, &sim.config.food.seasons);
            if sim.config.food.seasons.amplitude > 0.0 {
                stat_bar(
                    ui,
                    "Season",
                    seasonal as f32,
                    1.5,
                    Color32::from_rgb(57, 210, 192),
                );
            }
            if stats.active_droughts > 0 {
                ui.horizontal(|ui| {
                    ui.label(format!("Droughts: {}", stats.active_droughts));
                    let drought_color = Color32::from_rgb(248, 81, 73);
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 4.0, drought_color);
                });
            }

            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.0}", macroquad::time::get_fps()));
            });

            ui.separator();

            // --- Species breakdown ---
            let species_header =
                egui::CollapsingHeader::new("Species").default_open(state.species_expanded);
            let response = species_header.show(ui, |ui| {
                let species = &sim.stats.species;
                if species.is_empty() {
                    ui.label("No species data yet.");
                } else {
                    let show_count = species.len().min(12);
                    for info in &species[..show_count] {
                        ui.horizontal(|ui| {
                            // Color swatch
                            let hue = info.mean_hue as f32 / 360.0;
                            let (r, g, b) = hsv_to_rgb(hue, 0.7, 0.9);
                            let color = Color32::from_rgb(
                                (r * 255.0) as u8,
                                (g * 255.0) as u8,
                                (b * 255.0) as u8,
                            );
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, color);

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
            let inspector_header = egui::CollapsingHeader::new("Inspector").default_open(true);
            inspector_header.show(ui, |ui| {
                if let Some(id) = state.selected_id {
                    if let Some(org) = sim.organisms.iter().find(|o| o.id == id && o.alive) {
                        ui.horizontal(|ui| {
                            ui.label(format!("ID: {}", org.id));
                            ui.label(format!("Gen: {}", org.generation));
                            ui.label(format!("Age: {}", org.age));
                        });

                        // Energy bar
                        stat_bar(
                            ui,
                            "Energy",
                            org.energy as f32,
                            sim.config.energy.maximum as f32,
                            Color32::from_rgb(210, 153, 34),
                        );

                        ui.horizontal(|ui| {
                            ui.label(format!("Pos: ({:.0}, {:.0})", org.pos_x, org.pos_y));
                            ui.label(format!("Hd: {:.0}°", org.heading.to_degrees()));
                        });

                        ui.horizontal(|ui| {
                            ui.label(format!("Size: {:.2}", org.genome.body_size));
                            ui.label(format!("Hue: {:.0}", org.genome.color_hue));
                            ui.label(format!("Sig: {:.2}", org.signal));
                        });

                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "Brain: {}h {}c",
                                org.brain.hidden,
                                org.brain.connection_count()
                            ));
                            ui.label(format!("Mut: {:.3}", org.genome.mutation_rate));
                        });

                        let total_intake = org.energy_from_food + org.energy_from_attacks;
                        let trophic = if total_intake > 0.0 {
                            1.0 + org.energy_from_attacks / total_intake
                        } else {
                            1.0
                        };
                        ui.horizontal(|ui| {
                            ui.label(format!("Troph: {:.2}", trophic));
                            ui.label(format!(
                                "Food:{:.0} Atk:{:.0}",
                                org.energy_from_food, org.energy_from_attacks
                            ));
                        });

                        let org_entropy: f64 = (0..sim_core::organism::NUM_OUTPUTS)
                            .map(|o| sim_core::organism::shannon_entropy(&org.output_bins[o]))
                            .sum::<f64>()
                            / sim_core::organism::NUM_OUTPUTS as f64;
                        ui.label(format!("Brain entropy: {:.3}", org_entropy));

                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "Parent: {}",
                                org.parent_id
                                    .map(|id| id.to_string())
                                    .unwrap_or_else(|| "—".to_string())
                            ));
                        });

                        if ui.button("Deselect").clicked() {
                            state.selected_id = None;
                        }
                    } else {
                        ui.label("Organism died or not found.");
                        if ui.button("Clear").clicked() {
                            state.selected_id = None;
                        }
                    }
                } else {
                    ui.label("Click a creature to inspect.");
                }
            });

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
