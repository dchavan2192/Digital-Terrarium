//! [`WasmSim`]: owns the [`Simulation`] plus every buffer handed to JS, and derives
//! per-tick observations (flags, speed, turn, deaths, events) by diffing ticks.
//!
//! Safe Rust only; the `extern "C"` wrappers live in `lib.rs`. Usable natively
//! through the rlib (e.g. for tests on the Pi).

use crate::species::{SpeciesChange, SpeciesTracker, TrackedSpecies};
use crate::{
    CREATURE_STRIDE, DEATH_STRIDE, DROUGHT_STRIDE, EVENT_CAPACITY, FRAME_LEN, MAX_DEATHS_PER_STEP,
    cause, event_mask, field, flags, grid, status,
};
use serde::Serialize;
use sim_core::Simulation;
use sim_core::config::Config;
use sim_core::organism::{NUM_ENTROPY_BINS, NUM_OUTPUTS, Organism, OrganismId, shannon_entropy};
use sim_core::sim::TickStats;
use sim_core::world::wraparound_delta;
use std::collections::VecDeque;

/// configs/default.toml, embedded at compile time.
pub const DEFAULT_CONFIG_TOML: &str = include_str!("../../../configs/default.toml");
/// configs/demo.toml: an overlay merged on top of default.toml.
pub const DEMO_OVERLAY_TOML: &str = include_str!("../../../configs/demo.toml");

pub const OUTPUT_LABELS: [&str; 7] = [
    "turn",
    "speed",
    "eat",
    "attack",
    "signal",
    "share",
    "construct",
];

/// Non-ray input labels, in the order sim.rs writes them after the vision rays.
pub const INPUT_TAIL_LABELS: [&str; 10] = [
    "energy",
    "heading",
    "heard_signal",
    "body_size",
    "kin_similarity",
    "temperature",
    "pheromone",
    "light",
    "niche_level",
    "food_here",
];

// ---------------------------------------------------------------------------
// Config text helpers
// ---------------------------------------------------------------------------

fn merge_toml(base: &mut toml::Value, over: toml::Value) {
    match (base, over) {
        (toml::Value::Table(b), toml::Value::Table(o)) => {
            for (k, v) in o {
                match b.get_mut(&k) {
                    Some(bv) if bv.is_table() && v.is_table() => merge_toml(bv, v),
                    _ => {
                        b.insert(k, v);
                    }
                }
            }
        }
        (b, o) => *b = o,
    }
}

/// default.toml ⊕ demo.toml ⊕ `extra` (each a TOML document; tables merge,
/// everything else — including arrays like `[[food.biomes]]` — replaces).
pub fn overlay_config_text(extra: &str) -> Result<String, String> {
    let mut base: toml::Value =
        toml::from_str(DEFAULT_CONFIG_TOML).map_err(|e| format!("default.toml: {e}"))?;
    let demo: toml::Value =
        toml::from_str(DEMO_OVERLAY_TOML).map_err(|e| format!("demo.toml: {e}"))?;
    merge_toml(&mut base, demo);
    if !extra.trim().is_empty() {
        let over: toml::Value = toml::from_str(extra).map_err(|e| format!("overlay: {e}"))?;
        merge_toml(&mut base, over);
    }
    toml::to_string(&base).map_err(|e| format!("re-serialize: {e}"))
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// One entry of the event feed. Serialized as `{"kind": "...", "t": tick, ...}`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    SpeciesBorn {
        t: u64,
        species: u32,
        parent: Option<u32>,
        size: usize,
    },
    SpeciesExtinct {
        t: u64,
        species: u32,
        peak: usize,
        lifespan: u64,
    },
    DroughtStart {
        t: u64,
        x: f32,
        y: f32,
        radius: f32,
        severity: f32,
        duration: u64,
    },
    DroughtEnd {
        t: u64,
        x: f32,
        y: f32,
        radius: f32,
    },
    Extinction {
        t: u64,
    },
    Birth {
        t: u64,
        id: u64,
        parent: Option<u64>,
        species: Option<u32>,
        generation: u64,
        x: f32,
        y: f32,
    },
    Death {
        t: u64,
        id: u64,
        species: Option<u32>,
        x: f32,
        y: f32,
        age: u64,
        generation: u64,
        last_energy: f32,
        cause: &'static str,
        killer: Option<u64>,
    },
}

impl Event {
    fn mask_bit(&self) -> u32 {
        match self {
            Event::SpeciesBorn { .. } | Event::SpeciesExtinct { .. } => event_mask::SPECIES,
            Event::DroughtStart { .. } | Event::DroughtEnd { .. } => event_mask::DROUGHT,
            Event::Extinction { .. } => event_mask::EXTINCTION,
            Event::Birth { .. } => event_mask::BIRTH,
            Event::Death { .. } => event_mask::DEATH,
        }
    }
}

#[derive(Serialize)]
struct EventsOut<'a> {
    dropped: u64,
    events: &'a VecDeque<Event>,
}

// ---------------------------------------------------------------------------
// JSON payloads
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct EnvOut {
    light_level: f64,
    is_night: bool,
    season_multiplier: f64,
}

#[derive(Serialize)]
struct StatsOut<'a> {
    tick: u64,
    stats: &'a TickStats,
    env: EnvOut,
    species: &'a [TrackedSpecies],
    established_species: usize,
}

#[derive(Serialize)]
struct NodeOut {
    id: u32,
    kind: &'static str,
    label: String,
    depth: f64,
    layer: usize,
}

#[derive(Serialize)]
struct EdgeOut {
    from: u32,
    to: u32,
    weight: f64,
    enabled: bool,
    methylation: f64,
    effective_weight: f64,
    /// True for a memory connection: the target reads the source's previous-tick value.
    recurrent: bool,
}

#[derive(Serialize)]
struct OutputHistOut {
    name: &'static str,
    bins: [f64; NUM_ENTROPY_BINS],
    entropy: f64,
}

#[derive(Serialize)]
struct EnergyOut {
    current: f64,
    maximum: f64,
    lifetime_from_food: f64,
    lifetime_from_attacks: f64,
    predation_fraction: f64,
    cost_size_metabolism: f64,
    cost_brain_static: f64,
    cost_temperature: f64,
    cost_movement_last_tick: f64,
    untracked: [&'static str; 6],
}

#[derive(Serialize)]
struct BrainOut {
    num_inputs: usize,
    num_outputs: usize,
    num_hidden: usize,
    num_layers: usize,
    enabled_connections: usize,
    total_connections: usize,
    mean_methylation: f64,
    entropy: f64,
    nodes: Vec<NodeOut>,
    edges: Vec<EdgeOut>,
    output_histograms: Vec<OutputHistOut>,
}

#[derive(Serialize)]
struct InspectOut {
    id: u64,
    parent_id: Option<u64>,
    root_ancestor_id: u64,
    generation: u64,
    age: u64,
    max_age: u64,
    birth_tick: u64,
    species: Option<u32>,
    children_alive: usize,
    x: f64,
    y: f64,
    heading: f64,
    signal: f64,
    speed: f32,
    speed_norm: f32,
    turn: f32,
    flags: u32,
    trophic_level: f64,
    body_size: f64,
    color_hue: f64,
    aging_rate: f64,
    preferred_temp: f64,
    diet_preference: f64,
    mutation_rate: f64,
    mutation_step: f64,
    energy: EnergyOut,
    brain: BrainOut,
}

// ---------------------------------------------------------------------------
// Diff bookkeeping
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Snap {
    id: OrganismId,
    x: f64,
    y: f64,
    heading: f64,
    energy: f64,
    age: u64,
    generation: u64,
    food: f64,
    attacks: f64,
    /// Event flags accumulated earlier in the current `step(n)` batch.
    carry: u32,
}

#[derive(Clone, Copy, Default)]
struct Derived {
    speed: f32,
    turn: f32,
    flags: u32,
}

/// Flags that describe state (recomputed every tick) rather than events (OR-accumulated).
const STATE_FLAGS: u32 = flags::STRESSED | flags::PREDATOR;

// ---------------------------------------------------------------------------
// WasmSim
// ---------------------------------------------------------------------------

pub struct WasmSim {
    pub sim: Simulation,
    base_config: Config,
    seed: u64,

    creatures: Vec<f32>,
    creature_ids: Vec<u32>,
    deaths: Vec<f32>,
    death_ids: Vec<u32>,
    frame: [f32; FRAME_LEN],
    grids: [Vec<f32>; grid::COUNT],
    elevation_range: (f32, f32),
    droughts: Vec<f32>,
    stats_json: Vec<u8>,
    species_json: Vec<u8>,
    inspect_json: Vec<u8>,
    events_json: Vec<u8>,
    config_json: Vec<u8>,

    prev: Vec<Snap>,
    order: Vec<u32>,
    derived: Vec<Derived>,
    dead: Vec<Snap>,
    born: Vec<usize>,
    hitters: Vec<(OrganismId, f64, f64)>,
    prev_droughts: Vec<(f64, f64, f64)>,
    batch_births: u32,
    batch_deaths: u32,

    pub species: SpeciesTracker,
    events: VecDeque<Event>,
    events_dropped: u64,
    event_mask: u32,
    was_extinct: bool,
}

impl WasmSim {
    /// Build from a parsed config, overriding `world.seed`.
    pub fn new(config: Config, seed: u64) -> Result<Self, (i32, String)> {
        if config.brain.inputs > sim_core::brain::MAX_INPUTS
            || config.brain.outputs > sim_core::brain::MAX_OUTPUTS
        {
            return Err((
                status::ERR_UNSUPPORTED,
                format!(
                    "brain.inputs/outputs exceed MAX_INPUTS={} / MAX_OUTPUTS={}",
                    sim_core::brain::MAX_INPUTS,
                    sim_core::brain::MAX_OUTPUTS
                ),
            ));
        }
        let mut cfg = config.clone();
        cfg.world.seed = seed;
        let sim = Simulation::new(cfg);
        let cap = sim.config.creature.hard_cap.max(sim.organisms.len()) + 16;
        let cells = sim.food_grid.cols * sim.food_grid.rows;
        let mut s = WasmSim {
            sim,
            base_config: config,
            seed,
            creatures: Vec::with_capacity(cap * CREATURE_STRIDE),
            creature_ids: Vec::with_capacity(cap),
            deaths: Vec::with_capacity(256 * DEATH_STRIDE),
            death_ids: Vec::with_capacity(256),
            frame: [0.0; FRAME_LEN],
            grids: std::array::from_fn(|_| Vec::with_capacity(cells)),
            elevation_range: (0.0, 0.0),
            droughts: Vec::with_capacity(16 * DROUGHT_STRIDE),
            stats_json: Vec::with_capacity(8 * 1024),
            species_json: Vec::with_capacity(4 * 1024),
            inspect_json: Vec::with_capacity(32 * 1024),
            events_json: Vec::with_capacity(16 * 1024),
            config_json: Vec::with_capacity(16 * 1024),
            prev: Vec::with_capacity(cap),
            order: Vec::with_capacity(cap),
            derived: Vec::with_capacity(cap),
            dead: Vec::with_capacity(64),
            born: Vec::with_capacity(64),
            hitters: Vec::with_capacity(64),
            prev_droughts: Vec::with_capacity(16),
            batch_births: 0,
            batch_deaths: 0,
            species: SpeciesTracker::new(),
            events: VecDeque::with_capacity(EVENT_CAPACITY),
            events_dropped: 0,
            event_mask: event_mask::DEFAULT,
            was_extinct: false,
        };
        s.after_rebuild();
        Ok(s)
    }

    /// Parse a full TOML config (empty → default.toml ⊕ demo.toml) and build.
    pub fn from_toml(text: &str, seed: u64) -> Result<Self, (i32, String)> {
        let owned;
        let text = if text.trim().is_empty() {
            owned = overlay_config_text("").map_err(|e| (status::ERR_CONFIG, e))?;
            owned.as_str()
        } else {
            text
        };
        let config = Config::parse(text).map_err(|e| (status::ERR_CONFIG, e.to_string()))?;
        WasmSim::new(config, seed)
    }

    /// default.toml ⊕ demo.toml ⊕ `overlay`, then build.
    pub fn from_overlay(overlay: &str, seed: u64) -> Result<Self, (i32, String)> {
        let text = overlay_config_text(overlay).map_err(|e| (status::ERR_CONFIG, e))?;
        let config = Config::parse(&text).map_err(|e| (status::ERR_CONFIG, e.to_string()))?;
        WasmSim::new(config, seed)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Restart from the stored config with a new seed. Event mask and species
    /// announce threshold are kept; buffered events are discarded.
    pub fn reset(&mut self, seed: u64) {
        let mut cfg = self.base_config.clone();
        cfg.world.seed = seed;
        self.seed = seed;
        self.sim = Simulation::new(cfg);
        self.events.clear();
        self.events_dropped = 0;
        self.after_rebuild();
    }

    fn after_rebuild(&mut self) {
        self.derived.clear();
        self.derived
            .resize(self.sim.organisms.len(), Derived::default());
        for d in &mut self.derived {
            d.flags = flags::NEWBORN;
        }
        self.prev_droughts.clear();
        self.deaths.clear();
        self.death_ids.clear();
        self.batch_births = 0;
        self.batch_deaths = 0;
        self.was_extinct = false;
        self.species.clear();
        self.build_order();
        self.update_species();
        self.sim
            .sample_elevation_grid(&mut self.grids[grid::ELEVATION as usize]);
        self.sim
            .sample_biome_grid(&mut self.grids[grid::BIOME as usize]);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for &v in &self.grids[grid::ELEVATION as usize] {
            lo = lo.min(v);
            hi = hi.max(v);
        }
        self.elevation_range = if lo <= hi { (lo, hi) } else { (0.0, 0.0) };
        let fg = &self.sim.food_grid;
        let h = self.sim.config.world.height;
        let temp = &mut self.grids[grid::TEMPERATURE as usize];
        temp.clear();
        for r in 0..fg.rows {
            let t = (((r as f64 + 0.5) * fg.cell_size) / h).clamp(0.0, 1.0) as f32;
            temp.extend(std::iter::repeat_n(t, fg.cols));
        }
        self.build_config_json();
        self.frame_update();
    }

    fn build_config_json(&mut self) {
        let cfg = &self.sim.config;
        let fg = &self.sim.food_grid;
        let ray_count = cfg.creature.vision.ray_count;
        let input_labels: Vec<String> = (0..cfg.brain.inputs)
            .map(|i| input_label(i, ray_count))
            .collect();
        let output_labels: Vec<String> = (0..cfg.brain.outputs).map(output_label).collect();
        let meta = serde_json::json!({
            "abi_version": crate::ABI_VERSION,
            "seed": self.seed,
            "world": { "width": cfg.world.width, "height": cfg.world.height },
            "grid": {
                "cols": fg.cols, "rows": fg.rows,
                "cell_size": fg.cell_size, "max_per_cell": fg.max_per_cell,
            },
            "hard_cap": cfg.creature.hard_cap,
            "initial_count": cfg.creature.initial_count,
            "body_radius": cfg.creature.body_radius,
            "max_speed": cfg.creature.max_speed,
            "max_turn": cfg.creature.max_turn,
            "vision_range": cfg.creature.vision.range,
            "max_energy": cfg.energy.maximum,
            "max_age": cfg.creature.max_age,
            "reproduction_threshold": cfg.reproduction.threshold,
            "daynight": { "enabled": cfg.daynight.enabled, "period": cfg.daynight.period },
            "seasons": {
                "amplitude": cfg.food.seasons.amplitude,
                "period": cfg.food.seasons.period,
            },
            "droughts": {
                "enabled": cfg.food.droughts.spawn_rate > 0.0,
                "duration": cfg.food.droughts.duration,
                "radius": cfg.food.droughts.radius,
                "severity": cfg.food.droughts.severity,
                "max_active": cfg.food.droughts.max_active,
            },
            "pheromones": {
                "enabled": cfg.pheromones.enabled,
                "max_level": cfg.pheromones.max_level,
            },
            "niche_construction": {
                "enabled": cfg.niche_construction.enabled,
                "max_level": cfg.niche_construction.max_level,
            },
            "elevation": {
                "enabled": cfg.elevation.enabled,
                "min": self.elevation_range.0,
                "max": self.elevation_range.1,
            },
            "brain": {
                "inputs": cfg.brain.inputs,
                "outputs": cfg.brain.outputs,
                "hidden_initial": cfg.brain.hidden,
                "input_labels": input_labels,
                "output_labels": output_labels,
            },
            "biomes": &cfg.food.biomes,
            "enabled": {
                "fruit": cfg.food.fruit.enabled,
                "temperature": cfg.temperature.enabled,
                "pheromones": cfg.pheromones.enabled,
                "daynight": cfg.daynight.enabled,
                "elevation": cfg.elevation.enabled,
                "niche_construction": cfg.niche_construction.enabled,
                "epigenetics": cfg.epigenetics.enabled,
                "sexual_reproduction": cfg.reproduction.sexual,
                "droughts": cfg.food.droughts.spawn_rate > 0.0,
                "activity_tracking": cfg.activity_tracking.enabled,
            },
            "layout": {
                "creature_stride": CREATURE_STRIDE,
                "death_stride": DEATH_STRIDE,
                "drought_stride": DROUGHT_STRIDE,
                "frame_len": FRAME_LEN,
            },
            "config": cfg,
        });
        self.config_json.clear();
        let _ = serde_json::to_writer(&mut self.config_json, &meta);
    }

    // ------------------------------------------------------------------ step

    /// Advance `n` ticks. Event flags OR-accumulate over the batch; speed/turn
    /// describe the batch's last tick; deaths/frame counters cover the batch.
    pub fn step(&mut self, n: u32) {
        if n == 0 {
            return;
        }
        self.deaths.clear();
        self.death_ids.clear();
        self.batch_births = 0;
        self.batch_deaths = 0;
        for t in 0..n {
            self.snapshot(t > 0);
            self.sim.step();
            self.diff();
        }
        self.update_species();
        self.frame_update();
    }

    fn update_species(&mut self) {
        let mut threshold = self.sim.stats.speciation_threshold;
        if threshold.is_nan() || threshold <= 0.0 {
            threshold = self.sim.config.speciation.fixed_threshold;
        }
        let tick = self.sim.tick;
        let mut changes: Vec<Event> = Vec::new();
        self.species.update(
            &self.sim.organisms,
            &self.order,
            threshold,
            tick,
            |c| match c {
                SpeciesChange::Born {
                    species,
                    parent,
                    size,
                } => changes.push(Event::SpeciesBorn {
                    t: tick,
                    species,
                    parent,
                    size,
                }),
                SpeciesChange::Extinct {
                    species,
                    peak,
                    lifespan,
                    established,
                } => {
                    if established {
                        changes.push(Event::SpeciesExtinct {
                            t: tick,
                            species,
                            peak,
                            lifespan,
                        })
                    }
                }
            },
        );
        for e in changes {
            self.push_event(e);
        }
    }

    /// `self.order` = organism indices in ascending id order. sim-core keeps the
    /// vec sorted by id already (monotonic ids, births appended, order-preserving
    /// retain), so this is normally just an O(n) check.
    fn build_order(&mut self) {
        let orgs = &self.sim.organisms;
        self.order.clear();
        self.order.extend(0..orgs.len() as u32);
        if !orgs.windows(2).all(|w| w[0].id < w[1].id) {
            self.order.sort_unstable_by_key(|&i| orgs[i as usize].id);
        }
    }

    fn snapshot(&mut self, carry: bool) {
        self.build_order();
        self.prev.clear();
        for &i in &self.order {
            let i = i as usize;
            let o = &self.sim.organisms[i];
            let carry_flags = if carry {
                self.derived.get(i).map_or(0, |d| d.flags & !STATE_FLAGS)
            } else {
                0
            };
            self.prev.push(Snap {
                id: o.id,
                x: o.pos_x,
                y: o.pos_y,
                heading: o.heading,
                energy: o.energy,
                age: o.age,
                generation: o.generation,
                food: o.energy_from_food,
                attacks: o.energy_from_attacks,
                carry: carry_flags,
            });
        }
    }

    fn diff(&mut self) {
        self.build_order();
        let n = self.sim.organisms.len();
        self.derived.clear();
        self.derived.resize(n, Derived::default());
        self.dead.clear();
        self.born.clear();
        self.hitters.clear();

        let cfg = &self.sim.config;
        let (w, h) = (cfg.world.width, cfg.world.height);
        let max_turn = cfg.creature.max_turn;

        // Merge-walk previous (id order) against current (id order).
        let (mut i, mut j) = (0usize, 0usize);
        loop {
            let p = self.prev.get(i).copied();
            let c = self.order.get(j).map(|&k| k as usize);
            match (p, c) {
                (None, None) => break,
                (Some(s), Some(k)) if s.id == self.sim.organisms[k].id => {
                    let o = &self.sim.organisms[k];
                    let dx = wraparound_delta(s.x, o.pos_x, w);
                    let dy = wraparound_delta(s.y, o.pos_y, h);
                    let speed = (dx * dx + dy * dy).sqrt();
                    let mut dh = o.heading - s.heading;
                    if dh > std::f64::consts::PI {
                        dh -= std::f64::consts::TAU;
                    } else if dh < -std::f64::consts::PI {
                        dh += std::f64::consts::TAU;
                    }
                    let d = &mut self.derived[k];
                    d.speed = speed as f32;
                    d.turn = if max_turn > 0.0 {
                        (dh / max_turn).clamp(-1.0, 1.0) as f32
                    } else {
                        0.0
                    };
                    d.flags = s.carry;
                    if o.energy_from_food > s.food {
                        d.flags |= flags::ATE;
                    }
                    if o.energy_from_attacks > s.attacks {
                        d.flags |= flags::ATTACKED;
                        self.hitters.push((o.id, o.pos_x, o.pos_y));
                    }
                    i += 1;
                    j += 1;
                }
                (Some(s), Some(k)) if s.id < self.sim.organisms[k].id => {
                    self.dead.push(s);
                    i += 1;
                }
                (Some(s), None) => {
                    self.dead.push(s);
                    i += 1;
                }
                (_, Some(k)) => {
                    self.derived[k].flags |= flags::NEWBORN;
                    self.born.push(k);
                    j += 1;
                }
            }
        }

        // State flags.
        let stress_energy = if cfg.epigenetics.enabled {
            cfg.epigenetics.stress_repro_fraction * cfg.reproduction.threshold
        } else {
            f64::NEG_INFINITY
        };
        for (k, o) in self.sim.organisms.iter().enumerate() {
            if o.energy < stress_energy {
                self.derived[k].flags |= flags::STRESSED;
            }
            if predation_fraction(o) > 0.5 {
                self.derived[k].flags |= flags::PREDATOR;
            }
        }

        let tick = self.sim.tick;
        self.batch_births += self.born.len() as u32;
        self.batch_deaths += self.dead.len() as u32;

        // Births: mark parents (REPRODUCED), optional events.
        for bi in 0..self.born.len() {
            let k = self.born[bi];
            let o = &self.sim.organisms[k];
            let (id, parent, generation, x, y) =
                (o.id, o.parent_id, o.generation, o.pos_x, o.pos_y);
            if let Some(pid) = parent
                && let Some(pk) = self.index_of(pid)
            {
                self.derived[pk].flags |= flags::REPRODUCED;
            }
            if self.event_mask & event_mask::BIRTH != 0 {
                let species = parent.and_then(|p| self.species.species_of_id(p));
                self.push_event(Event::Birth {
                    t: tick,
                    id,
                    parent,
                    species,
                    generation,
                    x: x as f32,
                    y: y as f32,
                });
            }
        }

        // Deaths: heuristic cause, death buffer, optional events.
        if !self.dead.is_empty() {
            let cfg = &self.sim.config;
            let max_age = cfg.creature.max_age;
            // A victim moves at most max_speed / min_body_size (0.5) before attacks resolve.
            let kill_r = cfg.combat.attack_range + cfg.creature.max_speed * 2.0;
            let kill_r2 = kill_r * kill_r;
            let starve = starvation_bound(cfg);
            for di in 0..self.dead.len() {
                let s = self.dead[di];
                let mut killer = None;
                let mut best = kill_r2;
                for &(hid, hx, hy) in &self.hitters {
                    let dx = wraparound_delta(s.x, hx, w);
                    let dy = wraparound_delta(s.y, hy, h);
                    let d2 = dx * dx + dy * dy;
                    if d2 <= best {
                        best = d2;
                        killer = Some(hid);
                    }
                }
                let code = if s.age + 1 >= max_age {
                    cause::OLD_AGE
                } else if killer.is_some() {
                    cause::KILLED
                } else if s.energy <= starve {
                    cause::STARVATION
                } else {
                    cause::SENESCENCE
                };
                if code != cause::KILLED {
                    killer = None;
                }
                let species = self.species.species_of_id(s.id);
                if self.death_ids.len() < MAX_DEATHS_PER_STEP {
                    self.death_ids.push(s.id as u32);
                    self.deaths.extend_from_slice(&[
                        s.x as f32,
                        s.y as f32,
                        code as f32,
                        species.map_or(-1.0, |v| v as f32),
                        (s.age + 1) as f32,
                        s.generation as f32,
                    ]);
                }
                if self.event_mask & event_mask::DEATH != 0 {
                    self.push_event(Event::Death {
                        t: tick,
                        id: s.id,
                        species,
                        x: s.x as f32,
                        y: s.y as f32,
                        age: s.age + 1,
                        generation: s.generation,
                        last_energy: s.energy as f32,
                        cause: cause::name(code),
                        killer,
                    });
                }
            }
        }

        // Droughts never move, so (x, y) identifies one for its lifetime.
        let duration = self.sim.config.food.droughts.duration;
        let mut started: [Option<Event>; 8] = Default::default();
        let mut ended: [Option<Event>; 8] = Default::default();
        let (mut ns, mut ne) = (0, 0);
        {
            let cur = &self.sim.food_grid.droughts;
            for d in cur {
                let is_new = !self.prev_droughts.iter().any(|p| p.0 == d.x && p.1 == d.y);
                if is_new && ns < started.len() {
                    started[ns] = Some(Event::DroughtStart {
                        t: tick,
                        x: d.x as f32,
                        y: d.y as f32,
                        radius: d.radius as f32,
                        severity: d.severity as f32,
                        duration,
                    });
                    ns += 1;
                }
            }
            for p in &self.prev_droughts {
                let gone = !cur.iter().any(|d| p.0 == d.x && p.1 == d.y);
                if gone && ne < ended.len() {
                    ended[ne] = Some(Event::DroughtEnd {
                        t: tick,
                        x: p.0 as f32,
                        y: p.1 as f32,
                        radius: p.2 as f32,
                    });
                    ne += 1;
                }
            }
            self.prev_droughts.clear();
            self.prev_droughts
                .extend(cur.iter().map(|d| (d.x, d.y, d.radius)));
        }
        for e in ended.into_iter().chain(started).flatten() {
            self.push_event(e);
        }

        let extinct = n == 0;
        if extinct && !self.was_extinct {
            self.push_event(Event::Extinction { t: tick });
        }
        self.was_extinct = extinct;
    }

    fn index_of(&self, id: OrganismId) -> Option<usize> {
        let orgs = &self.sim.organisms;
        self.order
            .binary_search_by_key(&id, |&k| orgs[k as usize].id)
            .ok()
            .map(|p| self.order[p] as usize)
    }

    fn push_event(&mut self, e: Event) {
        if self.event_mask & e.mask_bit() == 0 {
            return;
        }
        if self.events.len() >= EVENT_CAPACITY {
            self.events.pop_front();
            self.events_dropped += 1;
        }
        self.events.push_back(e);
    }

    // ------------------------------------------------------- per-frame views

    fn frame_update(&mut self) {
        self.frame = [
            self.sim.tick as f32,
            self.sim.light_level() as f32,
            self.sim.season_multiplier() as f32,
            self.sim.organisms.len() as f32,
            self.sim.food_grid.droughts.len() as f32,
            self.species.established_count() as f32,
            self.batch_births as f32,
            self.batch_deaths as f32,
        ];
    }

    pub fn frame(&self) -> &[f32; FRAME_LEN] {
        &self.frame
    }

    /// Refill the creature table + id buffer. Returns the creature count.
    pub fn creatures_update(&mut self) -> u32 {
        let cfg = &self.sim.config;
        let max_energy = cfg.energy.maximum;
        let max_age = cfg.creature.max_age.max(1) as f64;
        let max_speed = cfg.creature.max_speed;
        let elevation_on = cfg.elevation.enabled;
        self.creatures.clear();
        self.creature_ids.clear();
        for (k, o) in self.sim.organisms.iter().enumerate() {
            let d = self.derived.get(k).copied().unwrap_or_default();
            let g = &o.genome;
            let top = max_speed / g.body_size.max(1e-9);
            let speed_norm = if top > 0.0 {
                (d.speed as f64 / top).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let species = self
                .species
                .assignment
                .get(k)
                .copied()
                .filter(|&s| s != u32::MAX)
                .map_or(-1.0, |s| s as f32);
            let meth = if g.connections.is_empty() {
                0.0
            } else {
                g.connections.iter().map(|c| c.methylation).sum::<f64>()
                    / g.connections.len() as f64
            };
            let elev = if elevation_on {
                self.sim.elevation_map.height_at(o.pos_x, o.pos_y)
            } else {
                0.0
            };
            let mut row = [0.0f32; CREATURE_STRIDE];
            row[field::ID] = o.id as f32;
            row[field::X] = o.pos_x as f32;
            row[field::Y] = o.pos_y as f32;
            row[field::HEADING] = o.heading as f32;
            row[field::ENERGY_FRAC] = (o.energy / max_energy).clamp(0.0, 1.0) as f32;
            row[field::BODY_SIZE] = g.body_size as f32;
            row[field::SPEED_NORM] = speed_norm as f32;
            row[field::TURN] = d.turn;
            row[field::SPECIES] = species;
            row[field::PREDATION_FRACTION] = predation_fraction(o) as f32;
            row[field::FLAGS] = d.flags as f32;
            row[field::TARGET_ROW] = -1.0;
            row[field::GENERATION] = o.generation as f32;
            row[field::AGE_FRAC] = (o.age as f64 / max_age) as f32;
            row[field::SIGNAL] = o.signal as f32;
            row[field::DIET_PREFERENCE] = g.diet_preference as f32;
            row[field::SPEED] = d.speed;
            row[field::ENERGY] = o.energy as f32;
            row[field::HUE] = g.color_hue as f32;
            row[field::AGE] = o.age as f32;
            row[field::METHYLATION_MEAN] = meth as f32;
            row[field::CONNECTIONS] = o.brain.connection_count() as f32;
            row[field::HIDDEN_NODES] = g.hidden_nodes.len() as f32;
            row[field::ELEVATION] = elev as f32;
            self.creatures.extend_from_slice(&row);
            self.creature_ids.push(o.id as u32);
        }
        self.sim.organisms.len() as u32
    }

    pub fn creatures(&self) -> &[f32] {
        &self.creatures
    }
    pub fn creature_ids(&self) -> &[u32] {
        &self.creature_ids
    }
    pub fn deaths(&self) -> &[f32] {
        &self.deaths
    }
    pub fn death_ids(&self) -> &[u32] {
        &self.death_ids
    }

    /// Refill a dynamic grid layer (static ones are built at init). Returns the
    /// element count (cols × rows), or 0 for an unknown kind.
    pub fn grid_update(&mut self, kind: u32) -> u32 {
        let cfg = &self.sim.config;
        let fg = &self.sim.food_grid;
        let cells = fg.cols * fg.rows;
        let Some(buf) = self.grids.get_mut(kind as usize) else {
            return 0;
        };
        match kind {
            grid::FOOD => {
                buf.clear();
                buf.extend(fg.cells.iter().map(|&v| v as f32));
            }
            grid::FRUIT => {
                buf.clear();
                if fg.fruit_cells.len() == cells {
                    buf.extend(fg.fruit_cells.iter().map(|&v| v as f32));
                } else {
                    buf.resize(cells, 0.0);
                }
            }
            grid::PHEROMONE => {
                let max = cfg.pheromones.max_level;
                let inv = if max > 0.0 { 1.0 / max } else { 0.0 };
                buf.clear();
                buf.extend(
                    self.sim
                        .pheromone_grid
                        .cells
                        .iter()
                        .map(|&v| (v * inv).clamp(0.0, 1.0) as f32),
                );
            }
            grid::CONSTRUCTION => {
                let max = cfg.niche_construction.max_level;
                let inv = if max > 0.0 { 1.0 / max } else { 0.0 };
                buf.clear();
                buf.extend(
                    self.sim
                        .construction_grid
                        .cells
                        .iter()
                        .map(|&v| (v * inv).clamp(-1.0, 1.0) as f32),
                );
            }
            _ => {} // ELEVATION / TEMPERATURE / BIOME are static
        }
        buf.len() as u32
    }

    pub fn grid(&self, kind: u32) -> &[f32] {
        self.grids.get(kind as usize).map_or(&[], |g| g.as_slice())
    }

    pub fn droughts_update(&mut self) -> u32 {
        let duration = self.sim.config.food.droughts.duration.max(1) as f64;
        self.droughts.clear();
        for d in &self.sim.food_grid.droughts {
            let progress = (1.0 - d.ticks_remaining as f64 / duration).clamp(0.0, 1.0);
            self.droughts.extend_from_slice(&[
                d.x as f32,
                d.y as f32,
                d.radius as f32,
                d.severity as f32,
                d.ticks_remaining as f32,
                progress as f32,
            ]);
        }
        (self.droughts.len() / DROUGHT_STRIDE) as u32
    }

    pub fn droughts(&self) -> &[f32] {
        &self.droughts
    }

    // ------------------------------------------------------------------ JSON

    pub fn stats_json(&mut self) -> &[u8] {
        let out = StatsOut {
            tick: self.sim.tick,
            stats: &self.sim.stats,
            env: EnvOut {
                light_level: self.sim.light_level(),
                is_night: self.sim.is_night(),
                season_multiplier: self.sim.season_multiplier(),
            },
            species: &self.species.species,
            established_species: self.species.established_count(),
        };
        self.stats_json.clear();
        let _ = serde_json::to_writer(&mut self.stats_json, &out);
        &self.stats_json
    }

    pub fn species_json(&mut self) -> &[u8] {
        self.species_json.clear();
        let _ = serde_json::to_writer(&mut self.species_json, &self.species.species);
        &self.species_json
    }

    pub fn config_json(&self) -> &[u8] {
        &self.config_json
    }

    pub fn stats_json_buf(&self) -> &[u8] {
        &self.stats_json
    }
    pub fn species_json_buf(&self) -> &[u8] {
        &self.species_json
    }
    pub fn inspect_json_buf(&self) -> &[u8] {
        &self.inspect_json
    }
    pub fn events_json_buf(&self) -> &[u8] {
        &self.events_json
    }

    /// Drain buffered events as JSON.
    pub fn events_json(&mut self) -> &[u8] {
        self.events_json.clear();
        let _ = serde_json::to_writer(
            &mut self.events_json,
            &EventsOut {
                dropped: self.events_dropped,
                events: &self.events,
            },
        );
        self.events.clear();
        self.events_dropped = 0;
        &self.events_json
    }

    pub fn set_event_mask(&mut self, mask: u32) {
        self.event_mask = mask;
    }

    pub fn set_species_min_announce(&mut self, n: u32) {
        self.species.min_announce = (n as usize).max(1);
    }

    /// Inspector JSON for one creature; empty if `id` is not alive.
    pub fn inspect_json(&mut self, id: OrganismId) -> &[u8] {
        self.inspect_json.clear();
        self.build_order();
        let Some(k) = self.index_of(id) else {
            return &self.inspect_json;
        };
        let out = self.inspect(k);
        let _ = serde_json::to_writer(&mut self.inspect_json, &out);
        &self.inspect_json
    }

    fn inspect(&self, k: usize) -> InspectOut {
        let o = &self.sim.organisms[k];
        let g = &o.genome;
        let cfg = &self.sim.config;
        let d = self.derived.get(k).copied().unwrap_or_default();
        let (ni, no) = (g.num_inputs, g.num_outputs);
        let ray_count = cfg.creature.vision.ray_count;

        let mut depths: Vec<f64> = g.hidden_nodes.iter().map(|n| n.depth).collect();
        depths.push(0.0);
        depths.push(1.0);
        depths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        depths.dedup();
        let layer_of = |depth: f64| depths.iter().position(|&x| x == depth).unwrap_or(0);

        let mut nodes = Vec::with_capacity(1 + ni + no + g.hidden_nodes.len());
        nodes.push(NodeOut {
            id: 0,
            kind: "bias",
            label: "bias".into(),
            depth: 0.0,
            layer: 0,
        });
        for i in 0..ni {
            nodes.push(NodeOut {
                id: (1 + i) as u32,
                kind: "in",
                label: input_label(i, ray_count),
                depth: 0.0,
                layer: 0,
            });
        }
        for oi in 0..no {
            nodes.push(NodeOut {
                id: (1 + ni + oi) as u32,
                kind: "out",
                label: output_label(oi),
                depth: 1.0,
                layer: layer_of(1.0),
            });
        }
        for hn in &g.hidden_nodes {
            nodes.push(NodeOut {
                id: hn.id,
                kind: "hidden",
                label: format!("h{}", hn.id),
                depth: hn.depth,
                layer: layer_of(hn.depth),
            });
        }
        let edges: Vec<EdgeOut> = g
            .connections
            .iter()
            .map(|c| EdgeOut {
                from: c.from,
                to: c.to,
                weight: c.weight,
                enabled: c.enabled,
                methylation: c.methylation,
                effective_weight: c.weight * (1.0 - c.methylation),
                recurrent: c.recurrent,
            })
            .collect();
        let output_histograms: Vec<OutputHistOut> = (0..NUM_OUTPUTS)
            .map(|oi| OutputHistOut {
                name: OUTPUT_LABELS.get(oi).copied().unwrap_or("?"),
                bins: o.output_bins[oi],
                entropy: shannon_entropy(&o.output_bins[oi]),
            })
            .collect();
        let entropy = output_histograms.iter().map(|h| h.entropy).sum::<f64>() / NUM_OUTPUTS as f64;
        let mean_meth = if g.connections.is_empty() {
            0.0
        } else {
            g.connections.iter().map(|c| c.methylation).sum::<f64>() / g.connections.len() as f64
        };

        let pred = predation_fraction(o);
        let temp_cost = if cfg.temperature.enabled {
            let local = o.pos_y / cfg.world.height;
            cfg.energy.base_metabolism
                * cfg.temperature.cost_factor
                * (local - g.preferred_temp).abs()
        } else {
            0.0
        };
        let speed = d.speed as f64;
        let top = cfg.creature.max_speed / g.body_size.max(1e-9);
        let children_alive = self
            .sim
            .organisms
            .iter()
            .filter(|c| c.parent_id == Some(o.id))
            .count();
        let species = self
            .species
            .assignment
            .get(k)
            .copied()
            .filter(|&s| s != u32::MAX);

        InspectOut {
            id: o.id,
            parent_id: o.parent_id,
            root_ancestor_id: o.root_ancestor_id,
            generation: o.generation,
            age: o.age,
            max_age: cfg.creature.max_age,
            birth_tick: o.birth_tick,
            species,
            children_alive,
            x: o.pos_x,
            y: o.pos_y,
            heading: o.heading,
            signal: o.signal,
            speed: d.speed,
            speed_norm: if top > 0.0 {
                (speed / top).clamp(0.0, 1.0) as f32
            } else {
                0.0
            },
            turn: d.turn,
            flags: d.flags,
            trophic_level: 1.0 + pred,
            body_size: g.body_size,
            color_hue: g.color_hue,
            aging_rate: g.aging_rate,
            preferred_temp: g.preferred_temp,
            diet_preference: g.diet_preference,
            mutation_rate: g.mutation_rate,
            mutation_step: g.mutation_step,
            energy: EnergyOut {
                current: o.energy,
                maximum: cfg.energy.maximum,
                lifetime_from_food: o.energy_from_food,
                lifetime_from_attacks: o.energy_from_attacks,
                predation_fraction: pred,
                cost_size_metabolism: cfg.energy.base_metabolism * g.body_size.powf(0.75),
                cost_brain_static: cfg.energy.brain_cost * o.brain.neuron_count() as f64
                    + cfg.energy.connection_cost * o.brain.connection_count() as f64,
                cost_temperature: temp_cost,
                cost_movement_last_tick: cfg.energy.movement_cost * speed * speed,
                untracked: [
                    "brain_activity",
                    "crowding",
                    "attack_cost",
                    "shares",
                    "construction",
                    "reproduction",
                ],
            },
            brain: BrainOut {
                num_inputs: ni,
                num_outputs: no,
                num_hidden: g.hidden_nodes.len(),
                num_layers: depths.len(),
                enabled_connections: o.brain.connection_count(),
                total_connections: g.connections.len(),
                mean_methylation: mean_meth,
                entropy,
                nodes,
                edges,
                output_histograms,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Fraction of lifetime intake from predation; trophic level = 1 + this (Lindeman 1942).
pub fn predation_fraction(o: &Organism) -> f64 {
    let total = o.energy_from_food + o.energy_from_attacks;
    if total > 0.0 {
        o.energy_from_attacks / total
    } else {
        0.0
    }
}

pub fn input_label(i: usize, ray_count: usize) -> String {
    if i < ray_count * 2 {
        format!(
            "ray{}_{}",
            i / 2,
            if i.is_multiple_of(2) { "dist" } else { "type" }
        )
    } else {
        INPUT_TAIL_LABELS
            .get(i - ray_count * 2)
            .map_or_else(|| format!("in{i}"), |s| (*s).to_string())
    }
}

pub fn output_label(i: usize) -> String {
    OUTPUT_LABELS
        .get(i)
        .map_or_else(|| format!("out{i}"), |s| (*s).to_string())
}

/// Upper bound on what one tick can drain without being attacked. A death whose
/// pre-tick energy was at or below this is classed as starvation.
fn starvation_bound(cfg: &Config) -> f64 {
    let e = &cfg.energy;
    // body_size lives in [0.5, 3.0] (genome.rs): worst-case metabolism and speed.
    let size_metab = e.base_metabolism * 3.0_f64.powf(0.75);
    let max_move = cfg.creature.max_speed / 0.5;
    let temp = if cfg.temperature.enabled {
        e.base_metabolism * cfg.temperature.cost_factor
    } else {
        0.0
    };
    size_metab
        + temp
        + e.movement_cost * max_move * max_move
        + e.brain_cost * sim_core::brain::MAX_NODES as f64
        + e.connection_cost * sim_core::genome::MAX_CONNECTIONS as f64
        + cfg.combat.attack_cost
        + cfg.sharing.share_amount
        + cfg.niche_construction.energy_cost
}
