//! # sim-wasm
//!
//! Thin, read-only WebAssembly bindings for the Terrarium simulation, exposed as a
//! plain `extern "C"` ABI (no wasm-bindgen, no generated JS glue).
//!
//! - **sim-core stays rendering-free.** This crate only *reads* [`Simulation`]
//!   state between ticks and never touches the RNG, so a browser run is
//!   bit-identical to a headless run with the same config + seed.
//! - **Allocation-light.** Every output (creature table, grids, JSON) is a buffer
//!   owned by [`WasmSim`] that is cleared and refilled; JS reads it zero-copy.
//! - **Derived, not invented.** Action flags, speed/turn, deaths, stable species
//!   and the event feed are reconstructed by diffing ticks. What sim-core does not
//!   track is listed in `docs/research/SIM_WASM_ABI.md` ("v2 / needs sim-core").
//!
//! The ABI reference lives in `docs/research/SIM_WASM_ABI.md`; keep it in sync.

pub mod species;
pub mod state;

pub use state::WasmSim;

use std::cell::RefCell;

#[allow(unused_imports)]
use sim_core::Simulation; // for rustdoc links

// ---------------------------------------------------------------------------
// getrandom: linked by rand 0.8 but never called (sim is seeded from config).
// ---------------------------------------------------------------------------

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn no_os_entropy(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // Deliberately unsupported: a call would mean something wanted OS entropy,
    // which would break the same-seed-same-run invariant.
    Err(getrandom::Error::UNSUPPORTED)
}
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
getrandom::register_custom_getrandom!(no_os_entropy);

// ---------------------------------------------------------------------------
// ABI constants (keep in sync with docs/research/SIM_WASM_ABI.md)
// ---------------------------------------------------------------------------

/// Bump on any breaking layout change.
pub const ABI_VERSION: u32 = 1;

/// Status codes returned by `tr_init*` / `tr_reset`.
pub mod status {
    pub const OK: i32 = 0;
    pub const ERR_UTF8: i32 = -1;
    pub const ERR_CONFIG: i32 = -2;
    pub const ERR_NULL_PTR: i32 = -3;
    pub const ERR_NOT_INITIALIZED: i32 = -4;
    pub const ERR_UNSUPPORTED: i32 = -5;
}

/// f32 slots per creature row.
pub const CREATURE_STRIDE: usize = 24;

/// Creature row offsets (f32 units). 0..=15 follow DEMO_UX_DESIGN.md §9.2.
pub mod field {
    pub const ID: usize = 0;
    pub const X: usize = 1;
    pub const Y: usize = 2;
    pub const HEADING: usize = 3;
    pub const ENERGY_FRAC: usize = 4;
    pub const BODY_SIZE: usize = 5;
    pub const SPEED_NORM: usize = 6;
    pub const TURN: usize = 7;
    pub const SPECIES: usize = 8;
    pub const PREDATION_FRACTION: usize = 9;
    pub const FLAGS: usize = 10;
    pub const TARGET_ROW: usize = 11;
    pub const GENERATION: usize = 12;
    pub const AGE_FRAC: usize = 13;
    pub const SIGNAL: usize = 14;
    pub const DIET_PREFERENCE: usize = 15;
    pub const SPEED: usize = 16;
    pub const ENERGY: usize = 17;
    pub const HUE: usize = 18;
    pub const AGE: usize = 19;
    pub const METHYLATION_MEAN: usize = 20;
    pub const CONNECTIONS: usize = 21;
    pub const HIDDEN_NODES: usize = 22;
    pub const ELEVATION: usize = 23;
}

/// Bits in the creature FLAGS slot (exact small integer stored as f32).
pub mod flags {
    pub const ATE: u32 = 1;
    pub const ATTACKED: u32 = 2;
    /// v2 — needs sim-core; always 0 for now.
    pub const WAS_ATTACKED: u32 = 4;
    /// v2 — needs sim-core; always 0 for now.
    pub const SHARED: u32 = 8;
    /// v2 — needs sim-core; always 0 for now.
    pub const RECEIVED_SHARE: u32 = 16;
    /// v2 — needs sim-core; always 0 for now.
    pub const CONSTRUCTED: u32 = 32;
    pub const REPRODUCED: u32 = 64;
    pub const NEWBORN: u32 = 128;
    /// State: energy below the epigenetic stress level (methylation accumulating).
    pub const STRESSED: u32 = 256;
    /// State: predation fraction > 0.5 (sim-core's predator classification).
    pub const PREDATOR: u32 = 512;
}

/// Grid layer ids.
pub mod grid {
    pub const FOOD: u32 = 0;
    pub const FRUIT: u32 = 1;
    pub const PHEROMONE: u32 = 2;
    pub const CONSTRUCTION: u32 = 3;
    pub const ELEVATION: u32 = 4;
    pub const TEMPERATURE: u32 = 5;
    pub const BIOME: u32 = 6;
    pub const COUNT: usize = 7;
}

/// f32 slots per drought row: x, y, radius, severity, ticks_remaining, progress.
pub const DROUGHT_STRIDE: usize = 6;

/// f32 slots per death row: x, y, cause, species, age, generation.
pub const DEATH_STRIDE: usize = 6;
/// Death rows kept per `tr_step` call (extra deaths are still counted in the frame).
pub const MAX_DEATHS_PER_STEP: usize = 4096;

/// Death cause codes (heuristic — see the ABI doc).
pub mod cause {
    pub const STARVATION: u8 = 0;
    pub const OLD_AGE: u8 = 1;
    pub const SENESCENCE: u8 = 2;
    pub const KILLED: u8 = 3;
    pub fn name(code: u8) -> &'static str {
        match code {
            STARVATION => "starvation",
            OLD_AGE => "old_age",
            SENESCENCE => "senescence",
            _ => "killed",
        }
    }
}

/// Frame scalars: tick, light, season_mult, population, droughts, species,
/// births_in_step, deaths_in_step.
pub const FRAME_LEN: usize = 8;

/// Event kinds recorded into the feed (`tr_events_set_mask`).
pub mod event_mask {
    pub const SPECIES: u32 = 1;
    pub const DROUGHT: u32 = 2;
    pub const EXTINCTION: u32 = 4;
    pub const BIRTH: u32 = 8;
    pub const DEATH: u32 = 16;
    pub const DEFAULT: u32 = SPECIES | DROUGHT | EXTINCTION;
    pub const ALL: u32 = 31;
}

/// Max buffered events between drains; oldest are dropped (and counted).
pub const EVENT_CAPACITY: usize = 1024;

// ---------------------------------------------------------------------------
// Global state (wasm32-unknown-unknown is single-threaded)
// ---------------------------------------------------------------------------

thread_local! {
    static STATE: RefCell<Option<WasmSim>> = const { RefCell::new(None) };
    static LAST_ERROR: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static STATE_JSON: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn with_sim<R>(default: R, f: impl FnOnce(&mut WasmSim) -> R) -> R {
    STATE.with(|s| match s.borrow_mut().as_mut() {
        Some(sim) => f(sim),
        None => default,
    })
}

fn set_error(msg: &str) {
    LAST_ERROR.with(|e| {
        let mut e = e.borrow_mut();
        e.clear();
        e.extend_from_slice(msg.as_bytes());
    });
}

/// # Safety
/// `ptr` must point to `len` readable bytes when `len > 0`.
unsafe fn read_str<'a>(ptr: *const u8, len: u32) -> Result<&'a str, i32> {
    if len == 0 {
        return Ok("");
    }
    if ptr.is_null() {
        set_error("null pointer with non-zero length");
        return Err(status::ERR_NULL_PTR);
    }
    // SAFETY: caller contract.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    std::str::from_utf8(bytes).map_err(|e| {
        set_error(&format!("config is not valid UTF-8: {e}"));
        status::ERR_UTF8
    })
}

fn install(result: Result<WasmSim, (i32, String)>) -> i32 {
    match result {
        Ok(sim) => {
            STATE.with(|s| *s.borrow_mut() = Some(sim));
            set_error("");
            status::OK
        }
        Err((code, msg)) => {
            set_error(&msg);
            code
        }
    }
}

// ---------------------------------------------------------------------------
// Memory + lifecycle
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn tr_abi_version() -> u32 {
    ABI_VERSION
}

/// Allocate `len` bytes (align 8) for passing data in. Free with `tr_free(ptr, len)`.
#[unsafe(no_mangle)]
pub extern "C" fn tr_alloc(len: u32) -> *mut u8 {
    if len == 0 {
        return std::ptr::NonNull::<u64>::dangling().as_ptr().cast();
    }
    match std::alloc::Layout::from_size_align(len as usize, 8) {
        // SAFETY: non-zero size.
        Ok(layout) => unsafe { std::alloc::alloc(layout) },
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `ptr` must come from `tr_alloc(len)` with the same `len`, freed at most once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tr_free(ptr: *mut u8, len: u32) {
    if ptr.is_null() || len == 0 {
        return;
    }
    if let Ok(layout) = std::alloc::Layout::from_size_align(len as usize, 8) {
        // SAFETY: caller contract — same layout as the allocation.
        unsafe { std::alloc::dealloc(ptr, layout) };
    }
}

/// Build from a FULL TOML config (`toml_len == 0` → default.toml ⊕ demo.toml).
/// `world.seed` is overridden by `seed`. Returns 0 or a negative status.
///
/// # Safety
/// `toml_ptr` must point to `toml_len` readable bytes (or `toml_len == 0`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tr_init(toml_ptr: *const u8, toml_len: u32, seed: u64) -> i32 {
    // SAFETY: forwarded caller contract.
    match unsafe { read_str(toml_ptr, toml_len) } {
        Ok(text) => install(WasmSim::from_toml(text, seed)),
        Err(code) => code,
    }
}

/// Build from default.toml ⊕ demo.toml ⊕ the given partial TOML overlay
/// (e.g. `"[creature]\nhard_cap = 300\n"`). Returns 0 or a negative status.
///
/// # Safety
/// `toml_ptr` must point to `toml_len` readable bytes (or `toml_len == 0`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tr_init_overlay(toml_ptr: *const u8, toml_len: u32, seed: u64) -> i32 {
    // SAFETY: forwarded caller contract.
    match unsafe { read_str(toml_ptr, toml_len) } {
        Ok(text) => install(WasmSim::from_overlay(text, seed)),
        Err(code) => code,
    }
}

/// Restart with the last successful config and a new seed.
#[unsafe(no_mangle)]
pub extern "C" fn tr_reset(seed: u64) -> i32 {
    with_sim(status::ERR_NOT_INITIALIZED, |s| {
        s.reset(seed);
        status::OK
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_last_error_len() -> u32 {
    LAST_ERROR.with(|e| e.borrow().len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_last_error_ptr() -> *const u8 {
    LAST_ERROR.with(|e| e.borrow().as_ptr())
}

/// Advance `n` ticks (no-op before init or when n == 0).
#[unsafe(no_mangle)]
pub extern "C" fn tr_step(n: u32) {
    with_sim((), |s| s.step(n));
}

/// Current tick (JS BigInt).
#[unsafe(no_mangle)]
pub extern "C" fn tr_tick() -> u64 {
    with_sim(0, |s| s.sim.tick)
}

/// FNV-1a fingerprint of the complete canonical simulation state (JS BigInt), or
/// 0 if no simulation is running or the state fails validation. Matches the
/// 16-hex-digit value printed by `sim-runner fingerprint`, so the browser build
/// can be compared tick for tick with native builds.
#[unsafe(no_mangle)]
pub extern "C" fn tr_state_fingerprint() -> u64 {
    with_sim(0, |s| {
        s.sim
            .state_fingerprint()
            .ok()
            .and_then(|hex| u64::from_str_radix(&hex, 16).ok())
            .unwrap_or(0)
    })
}

/// Serialize the complete canonical state (the bytes behind
/// `tr_state_fingerprint`) into a reused buffer; returns its length, or 0 on
/// failure. For diagnosing where two platforms first differ, not for the UI.
#[unsafe(no_mangle)]
pub extern "C" fn tr_state_json() -> u32 {
    let bytes = with_sim(Vec::new(), |s| s.sim.state_json().unwrap_or_default());
    STATE_JSON.with(|b| {
        let len = bytes.len() as u32;
        *b.borrow_mut() = bytes;
        len
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_state_json_ptr() -> *const u8 {
    STATE_JSON.with(|b| b.borrow().as_ptr())
}

/// Current tick as a plain JS number (exact below 2^53).
#[unsafe(no_mangle)]
pub extern "C" fn tr_tick_f64() -> f64 {
    with_sim(0.0, |s| s.sim.tick as f64)
}

/// Seed of the running simulation (JS BigInt).
#[unsafe(no_mangle)]
pub extern "C" fn tr_seed() -> u64 {
    with_sim(0, |s| s.seed())
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_world_width() -> f32 {
    with_sim(0.0, |s| s.sim.config.world.width as f32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_world_height() -> f32 {
    with_sim(0.0, |s| s.sim.config.world.height as f32)
}

// ---------------------------------------------------------------------------
// Per-frame buffers
// ---------------------------------------------------------------------------

/// Pointer to the FRAME_LEN f32 frame scalars (refreshed by init/reset/step).
#[unsafe(no_mangle)]
pub extern "C" fn tr_frame_ptr() -> *const f32 {
    with_sim(std::ptr::null(), |s| s.frame().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_frame_len() -> u32 {
    FRAME_LEN as u32
}

/// Refill the creature table and id buffer; returns the creature count.
#[unsafe(no_mangle)]
pub extern "C" fn tr_creatures_update() -> u32 {
    with_sim(0, |s| s.creatures_update())
}

/// Count as of the last `tr_creatures_update`.
#[unsafe(no_mangle)]
pub extern "C" fn tr_creature_count() -> u32 {
    with_sim(0, |s| s.creature_ids().len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_creatures_ptr() -> *const f32 {
    with_sim(std::ptr::null(), |s| s.creatures().as_ptr())
}

/// u32 ids, one per creature row (exact; use instead of the f32 ID slot).
#[unsafe(no_mangle)]
pub extern "C" fn tr_creature_ids_ptr() -> *const u32 {
    with_sim(std::ptr::null(), |s| s.creature_ids().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_creature_stride() -> u32 {
    CREATURE_STRIDE as u32
}

/// Deaths during the last `tr_step` call (rows of DEATH_STRIDE f32).
#[unsafe(no_mangle)]
pub extern "C" fn tr_deaths_count() -> u32 {
    with_sim(0, |s| s.death_ids().len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_deaths_ptr() -> *const f32 {
    with_sim(std::ptr::null(), |s| s.deaths().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_death_ids_ptr() -> *const u32 {
    with_sim(std::ptr::null(), |s| s.death_ids().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_death_stride() -> u32 {
    DEATH_STRIDE as u32
}

// ---------------------------------------------------------------------------
// Grids + droughts
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn tr_grid_cols() -> u32 {
    with_sim(0, |s| s.sim.food_grid.cols as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_grid_rows() -> u32 {
    with_sim(0, |s| s.sim.food_grid.rows as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_grid_cell_size() -> f32 {
    with_sim(0.0, |s| s.sim.food_grid.cell_size as f32)
}

/// Refill layer `kind` (static layers: no-op). Returns cols*rows, 0 = bad kind.
#[unsafe(no_mangle)]
pub extern "C" fn tr_grid_update(kind: u32) -> u32 {
    with_sim(0, |s| s.grid_update(kind))
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_grid_ptr(kind: u32) -> *const f32 {
    with_sim(std::ptr::null(), |s| s.grid(kind).as_ptr())
}

/// Terrain height at a world position (0 when elevation is disabled).
#[unsafe(no_mangle)]
pub extern "C" fn tr_elevation_at(x: f32, y: f32) -> f32 {
    with_sim(0.0, |s| {
        if s.sim.config.elevation.enabled {
            s.sim.elevation_map.height_at(x as f64, y as f64) as f32
        } else {
            0.0
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_droughts_update() -> u32 {
    with_sim(0, |s| s.droughts_update())
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_droughts_ptr() -> *const f32 {
    with_sim(std::ptr::null(), |s| s.droughts().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_drought_stride() -> u32 {
    DROUGHT_STRIDE as u32
}

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------

/// Init-time config + derived metadata. Returns byte length (filled at init/reset).
#[unsafe(no_mangle)]
pub extern "C" fn tr_config_json() -> u32 {
    with_sim(0, |s| s.config_json().len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_config_ptr() -> *const u8 {
    with_sim(std::ptr::null(), |s| s.config_json().as_ptr())
}

/// Serialize TickStats + env + stable species; returns byte length.
#[unsafe(no_mangle)]
pub extern "C" fn tr_stats_json() -> u32 {
    with_sim(0, |s| s.stats_json().len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_stats_ptr() -> *const u8 {
    with_sim(std::ptr::null(), |s| s.stats_json_buf().as_ptr())
}

/// Serialize only the stable species list; returns byte length.
#[unsafe(no_mangle)]
pub extern "C" fn tr_species_json() -> u32 {
    with_sim(0, |s| s.species_json().len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_species_ptr() -> *const u8 {
    with_sim(std::ptr::null(), |s| s.species_json_buf().as_ptr())
}

/// Population at which a tracked species is announced (`species_born`). Default 5.
#[unsafe(no_mangle)]
pub extern "C" fn tr_species_set_min_announce(n: u32) {
    with_sim((), |s| s.set_species_min_announce(n));
}

/// Inspector JSON for creature `id`; returns byte length (0 = not alive).
#[unsafe(no_mangle)]
pub extern "C" fn tr_inspect_json(id: u32) -> u32 {
    with_sim(0, |s| s.inspect_json(id as u64).len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_inspect_ptr() -> *const u8 {
    with_sim(std::ptr::null(), |s| s.inspect_json_buf().as_ptr())
}

/// Drain buffered events into JSON; returns byte length.
#[unsafe(no_mangle)]
pub extern "C" fn tr_events_json() -> u32 {
    with_sim(0, |s| s.events_json().len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tr_events_ptr() -> *const u8 {
    with_sim(std::ptr::null(), |s| s.events_json_buf().as_ptr())
}

/// Choose which event kinds are recorded (see `event_mask`; default SPECIES|DROUGHT|EXTINCTION).
#[unsafe(no_mangle)]
pub extern "C" fn tr_events_set_mask(mask: u32) {
    with_sim((), |s| s.set_event_mask(mask));
}
