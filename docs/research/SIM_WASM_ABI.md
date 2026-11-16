# sim-wasm ABI reference (v1)

`crates/sim-wasm` compiles `sim-core` to `wasm32-unknown-unknown` as a **cdylib with a plain `extern "C"` ABI**.
It needs no wasm-bindgen and no generated JS glue: the module has **zero imports**, so you instantiate it with `{}`.
This doc is the contract for the Three.js front end (Phase 8, see `docs/research/DEMO_UX_DESIGN.md` §9).

- Build: `cargo build -p sim-wasm --target wasm32-unknown-unknown --release` → `target/wasm32-unknown-unknown/release/sim_wasm.wasm` (~1.1 MB unoptimised, of which ~0.2 MB is the `name` section; run `wasm-opt -Oz --strip-debug` in CI).
- `tr_abi_version()` returns `1`. It is bumped on any breaking layout change.
- Naming: exports use the `tr_` prefix. The UX spec's `tw_*` names map 1:1 (`tw_init`→`tr_init`, `tw_config_json_ptr/len`→`tr_config_ptr/tr_config_json`, and so on).
- **Determinism**: sim-wasm only reads `Simulation` between ticks and never draws from the RNG. Same config + seed gives the same run as `sim-runner`.
- **sim-core is untouched** apart from a new read-only `crates/sim-core/src/view.rs` (`light_level`, `is_night`, `season_multiplier`, `sample_elevation_grid`, `sample_biome_grid`).

## 0. Conventions

| Topic | Rule |
|---|---|
| Wasm types | `u32`/`i32`/pointers → JS `number`. **`u64` → JS `BigInt`** (`seed` args, `tr_tick`, `tr_seed`). Use `tr_tick_f64()` for a plain number. |
| Coordinates | Sim units: x ∈ [0, world.width), y ∈ [0, world.height). y = 0 is **north/cold**. The world is a torus (wraps both ways). |
| Buffers | Every `*_ptr()` points into a buffer owned by the module. It stays valid until the next call that refills **that** buffer. All buffers are reused, so steady state allocates nothing. |
| Memory growth | Any export may grow linear memory, which **detaches existing typed-array views**. Re-create views whenever `memory.buffer !== lastBuffer`. Cheapest approach: build views right after each `*_update()` call. |
| Ids | `u32` id buffers are exact. The f32 `ID` slot in the creature row is only exact below 2^24. |
| Traps | A Rust panic becomes a wasm trap (JS `RuntimeError: unreachable`). After a trap the instance is in an undefined state, so re-instantiate it. |
| Before init | Every getter returns 0 / null, and `tr_step` does nothing. |

## 1. Lifecycle and memory

| Signature | Notes |
|---|---|
| `tr_abi_version() -> u32` | `1` |
| `tr_alloc(len: u32) -> *mut u8` | Align 8. Use it to pass the config text in. |
| `tr_free(ptr: *mut u8, len: u32)` | Must pass the same `len` as the alloc. |
| `tr_init(toml_ptr: *const u8, toml_len: u32, seed: u64) -> i32` | Takes a **full** TOML config. `toml_len == 0` uses the embedded **default.toml ⊕ demo.toml** (see §9). `world.seed` is overridden by `seed`. Replaces any previous simulation. |
| `tr_init_overlay(toml_ptr, toml_len, seed: u64) -> i32` | Takes a **partial** TOML overlay, merged onto default.toml ⊕ demo.toml. Tables merge and other values replace (arrays such as `[[food.biomes]]` are replaced whole). Good for UI sliders: `"[creature]\nhard_cap = 300\n"`. |
| `tr_reset(seed: u64) -> i32` | Restarts with the last successful config and a new seed. Keeps the event mask and the announce threshold, and clears buffered events. |
| `tr_last_error_ptr() -> *const u8`, `tr_last_error_len() -> u32` | UTF-8 message from the last failed init. Empty after a success. |
| `tr_step(n: u32)` | Advances `n` ticks. See §3 for the batch semantics. |
| `tr_tick() -> u64`, `tr_tick_f64() -> f64`, `tr_seed() -> u64` | |
| `tr_world_width() -> f32`, `tr_world_height() -> f32` | |

Status codes: `0` OK, `-1` invalid UTF-8, `-2` config parse/validation error (the message says why), `-3` null pointer with len > 0, `-4` not initialised, `-5` unsupported config (brain inputs > 64 or outputs > 16).

## 2. Init-time config JSON

`tr_config_json() -> u32 len`, `tr_config_ptr() -> *const u8`. Built once at init/reset. Read it once and hard-code nothing.

```jsonc
{
  "abi_version": 1, "seed": 42,
  "world": {"width": 1000, "height": 1000},
  "grid": {"cols": 100, "rows": 100, "cell_size": 10, "max_per_cell": 1},
  "hard_cap": 500, "initial_count": 300, "body_radius": 3, "max_speed": 2, "max_turn": 0.2,
  "vision_range": 60, "max_energy": 200, "max_age": 2000, "reproduction_threshold": 60,
  "daynight": {"enabled": true, "period": 600},
  "seasons": {"amplitude": 0.3, "period": 3600},
  "droughts": {"enabled": true, "duration": 1500, "radius": 150, "severity": 0.05, "max_active": 3},
  "pheromones": {"enabled": true, "max_level": 10},
  "niche_construction": {"enabled": true, "max_level": 1},
  "elevation": {"enabled": true, "min": -1.50, "max": 1.73},     // measured over the elevation grid
  "brain": {"inputs": 19, "outputs": 7, "hidden_initial": 8,
            "input_labels": ["ray0_dist","ray0_type", "...", "energy","heading","heard_signal","body_size",
                             "kin_similarity","temperature","pheromone","light","niche_level","food_here"],
            "output_labels": ["turn","speed","eat","attack","signal","share","construct"]},
  "biomes": [{"x": 250, "y": 250, "radius": 200, "regrowth_multiplier": 3.0}, "..."],
  "enabled": {"fruit": true, "temperature": true, "pheromones": true, "daynight": true, "elevation": true,
              "niche_construction": true, "epigenetics": true, "sexual_reproduction": true,
              "droughts": true, "activity_tracking": true},
  "layout": {"creature_stride": 24, "death_stride": 6, "drought_stride": 6, "frame_len": 8},
  "config": { /* the complete sim_core::Config, every TOML section */ }
}
```

## 3. Stepping semantics

`tr_step(n)` runs `n` ticks. Inside it, sim-wasm snapshots each tick and diffs it against the next:

- **Event flags** (creature `FLAGS` bits 1–128) are **OR-accumulated across the n ticks and cleared at the start of each `tr_step`**, so a 16× batch doesn't lose cues. State bits (256, 512) describe the final tick.
- `SPEED`, `SPEED_NORM` and `TURN` describe the **last** tick of the batch.
- The **deaths buffer** and the frame's births/deaths counters cover the whole batch.
- **Stable species** are re-clustered once per `tr_step` call, after its last tick (§6).
- Events accumulate in a queue until you drain it (§7).

## 4. Per-frame binary buffers

### 4.1 Frame scalars

`tr_frame_ptr() -> *const f32`, `tr_frame_len() -> u32` (= 8). Refreshed by init, reset and every `tr_step`.

| off | value |
|---|---|
| 0 | tick (f32, approximate above 2^24; use `tr_tick_f64` for an exact value) |
| 1 | light level [0,1] for the next tick (night is < 0.5; always 1 when day/night is off) |
| 2 | seasonal regrowth multiplier |
| 3 | population |
| 4 | active droughts |
| 5 | established stable species (§6) |
| 6 | births during the last `tr_step` |
| 7 | deaths during the last `tr_step` |

### 4.2 Creatures

`tr_creatures_update() -> u32 count` refills the buffers. Then use `tr_creatures_ptr() -> *const f32`, `tr_creature_ids_ptr() -> *const u32`, `tr_creature_stride() -> u32` (= 24) and `tr_creature_count() -> u32` (the count from the last update).

Row `i` starts at f32 index `i*24`. The row order equals sim order, which is ascending id. It is **not** stable across steps, so key by id. Offsets 0–15 follow UX spec §9.2.

| off | field | units / range | source |
|---|---|---|---|
| 0 | id (f32) | exact < 2^24; prefer the u32 ids buffer | `org.id` |
| 1 | x | world units | |
| 2 | y | world units | |
| 3 | heading | rad [0, 2π) | |
| 4 | energy_frac | [0,1] = energy / max_energy | |
| 5 | body_size | [0.5, 3.0] | genome |
| 6 | speed_norm | [0,1] = displacement / (max_speed / body_size) | **derived** (last tick). Under-reads on slopes, because elevation slows movement |
| 7 | turn | [-1,1] = Δheading / max_turn | **derived** (last tick) |
| 8 | species | stable species id, -1 if none | **sim-wasm tracker** (§6) |
| 9 | predation_fraction | [0,1] = from_attacks / (from_food + from_attacks); trophic level = 1 + this | derived |
| 10 | flags | bitfield as an exact integer: `flags = row[10] \| 0` | see below |
| 11 | target_row | always -1 in v1 | **v2 / needs sim-core** |
| 12 | generation | integer | |
| 13 | age_frac | age / max_age | |
| 14 | signal | [-1,1] broadcast | |
| 15 | diet_preference | [0,1] (0 = algae, 1 = fruit) | genome |
| 16 | speed | world units per tick (last tick) | derived |
| 17 | energy | raw | |
| 18 | hue | [0,360) heritable colour gene (kin marker) | genome |
| 19 | age | ticks | |
| 20 | methylation_mean | [0,1] over all connection genes | genome |
| 21 | connections | enabled brain connections | brain |
| 22 | hidden_nodes | count | genome |
| 23 | elevation | terrain height at (x, y); 0 when elevation is off | `elevation_map.height_at` |

Flag bits:

| bit | name | v1 status |
|---|---|---|
| 1 | ATE | yes: lifetime food energy rose |
| 2 | ATTACKED | yes: landed a hit (lifetime attack energy rose). A missed attack, where the cost is paid but there is no target, is not visible |
| 4 | WAS_ATTACKED | always 0 (v2) |
| 8 | SHARED | always 0 (v2) |
| 16 | RECEIVED_SHARE | always 0 (v2) |
| 32 | CONSTRUCTED | always 0 (v2) |
| 64 | REPRODUCED | yes: a new organism with `parent_id` = this one appeared |
| 128 | NEWBORN | yes: also set on the whole initial population right after init/reset |
| 256 | STRESSED (state) | energy < `epigenetics.stress_repro_fraction × reproduction.threshold` (methylation accumulating) |
| 512 | PREDATOR (state) | predation_fraction > 0.5, matching sim-core's predator classification |

### 4.3 Deaths (during the last `tr_step`)

`tr_deaths_count() -> u32`, `tr_deaths_ptr() -> *const f32`, `tr_death_ids_ptr() -> *const u32`, `tr_death_stride() -> u32` (= 6). At most 4096 rows per step; the frame counter still counts every death.

| off | field |
|---|---|
| 0 | x (position at the start of the death tick) |
| 1 | y |
| 2 | cause: 0 starvation, 1 old age, 2 senescence (Gompertz-Makeham roll), 3 killed |
| 3 | stable species id (-1 if it was born and died inside the same batch) |
| 4 | age at death |
| 5 | generation |

**The cause is a heuristic.** sim-core does not record it.
- Old age means `age + 1 ≥ max_age`.
- Killed means that, on the death tick, a creature that landed a hit is within `attack_range + 2·max_speed` of the victim.
- Starvation means that pre-tick energy was ≤ the worst-case non-combat per-tick drain.
- Everything else is senescence.

Exact causes are a v2 item (§10).

### 4.4 Grids

All grids share the food grid's dims: `tr_grid_cols()`, `tr_grid_rows()`, `tr_grid_cell_size()` (also in the config JSON). The layout is row-major, `index = row*cols + col`, and row 0 is y ∈ [0, cell_size), the north edge. Cell (c, r) covers x ∈ [c·cs, (c+1)·cs).

`tr_grid_update(kind) -> u32 len` (0 = bad kind), then `tr_grid_ptr(kind) -> *const f32`. Each kind has its own buffer, so views of different layers don't invalidate each other (memory growth still does).

| kind | layer | range | cadence |
|---|---|---|---|
| 0 | FOOD (algae) | [0, grid.max_per_cell] | dynamic, refresh every few frames |
| 1 | FRUIT | raw ≥ 0 (zeros when fruit is off) | dynamic |
| 2 | PHEROMONE | [0,1] = level / pheromones.max_level | dynamic |
| 3 | CONSTRUCTION | [-1,1] = level / niche_construction.max_level | dynamic |
| 4 | ELEVATION | raw height at cell centres; min/max in the config JSON | **static** (built at init; update is a no-op) |
| 5 | TEMPERATURE | [0,1] = y / height at cell centres (0 = cold north) | static |
| 6 | BIOME | regrowth multiplier = cell base rate / food.regrowth_rate (1 = baseline, 3 = oasis, 0.3 = barren) | static (no seasons, droughts or construction) |

`tr_elevation_at(x: f32, y: f32) -> f32` gives the exact analytic height, for placing a followed creature or the camera.

### 4.5 Droughts

`tr_droughts_update() -> u32 count`, `tr_droughts_ptr() -> *const f32`, `tr_drought_stride() -> u32` (= 6). Each row is `[x, y, radius, severity, ticks_remaining, progress]`, where `progress` = 1 − remaining/duration ∈ [0,1]. Regrowth inside the zone is multiplied by `severity`.

## 5. Stats JSON (about 1 Hz)

`tr_stats_json() -> u32 len`, `tr_stats_ptr() -> *const u8`.

```jsonc
{
  "tick": 1234,
  "stats": { /* sim_core::sim::TickStats, every field: tick, population, births, deaths, total_births,
     total_deaths, attacks, total_attacks, mean_energy, mean_brain_neurons, species_count, mean_generation,
     shares, total_shares, cooperation_rate, kin_share_ratio, mean_trophic_level, herbivore_count,
     predator_count, mean_brain_entropy, behavioral_diversity, mean_connections, mean_crowding,
     speciation_threshold, mean_brain_activity, sexual_births, asexual_births, active_droughts,
     novel_traits, active_traits, mean_cumulative_activity, max_generation, lineage_diversity,
     mean_construction_level, mean_methylation, total_sexual_births, total_asexual_births,
     founder_lineages, net_construction_level, total_cumulative_activity
     (per-tick fields describe the LAST tick only; sim-core's own `species` list is serde-skipped) */ },
  "env": {"light_level": 0.73, "is_night": false, "season_multiplier": 1.12},
  "species": [ /* stable species, same objects as §6 */ ],
  "established_species": 7
}
```

`TickStats` is serialised straight from sim-core, so new fields appear automatically. NaN serialises as `null`.

## 6. Stable species (`tr_species_json() -> len`, `tr_species_ptr()`)

sim-core's `SpeciesInfo.id` is a per-tick index that reshuffles, and it discards the per-organism assignment. sim-wasm keeps its own tracker (`crates/sim-wasm/src/species.rs`). It uses the same NEAT compatibility distance (`Genome::genome_distance`) and **this tick's `speciation_threshold`** (`speciation.fixed_threshold` at tick 0), with persistent representatives:

1. An organism first tries its previous species. A newborn tries its parent's species.
2. Failing that, it joins the oldest species whose representative is within the threshold.
3. Otherwise it founds a new species whose `parent_stable_id` is its parent's species.

A representative is kept until its source organism dies. It is then replaced by the oldest living member. Ids are never reused. Counts can differ slightly from `stats.species_count`. The tracker runs once per `tr_step` call and costs about one `genome_distance` per creature.

The JSON is an array ordered by `stable_id`; sort it client-side. Each entry:

```jsonc
{"stable_id": 9, "parent_stable_id": 3, "first_seen_tick": 5120, "established": true,
 "population": 42, "peak": 61, "mean_hue": 212.4 /* circular mean */, "mean_energy": 88.1,
 "mean_body_size": 1.4, "mean_connections": 190.2, "mean_generation": 37.5,
 "mean_trophic_level": 1.08, "herbivore_fraction": 0.9, "predator_fraction": 0.02,
 "trend": 0.12 /* (pop − EMA)/EMA, EMA τ = 500 ticks */}
```

`established` turns true once the species reaches both `min_announce` members (default 5, set with `tr_species_set_min_announce(n: u32)`) and 5% of the living population, which is also when `species_born` fires. Since 2026-11-16 a creature's species is decided once, at birth, against the species' fixed founding genome; creatures are no longer re-tested on every update. This suppresses one-off mutant flicker. Colour slot suggestion: `stable_id % palette.length`, or `mean_hue`.

## 7. Events (`tr_events_json() -> len` drains the queue, `tr_events_ptr()`)

Shape: `{"dropped": 0, "events": [ ... ]}`. The queue holds at most 1024 events between drains; when it is full the oldest are dropped and counted in `dropped`. Every event has `"kind"` and `"t"`, the tick number as reported by `stats.tick`.

Use `tr_events_set_mask(mask: u32)` to choose what is recorded. The default is `SPECIES|DROUGHT|EXTINCTION` = 7. Bits: 1 SPECIES, 2 DROUGHT, 4 EXTINCTION, 8 BIRTH, 16 DEATH (per individual; noisy).

```jsonc
{"kind": "species_born",   "t": 5120, "species": 9, "parent": 3, "size": 5}
{"kind": "species_extinct","t": 9800, "species": 4, "peak": 140, "lifespan": 8200}   // established species only
{"kind": "drought_start",  "t": 700,  "x": 412.0, "y": 88.5, "radius": 150, "severity": 0.05, "duration": 1500}
{"kind": "drought_end",    "t": 2200, "x": 412.0, "y": 88.5, "radius": 150}
{"kind": "extinction",     "t": 40210}
{"kind": "birth", "t": 812, "id": 1203, "parent": 455, "species": 2, "generation": 7, "x": 1.0, "y": 2.0}
{"kind": "death", "t": 813, "id": 990, "species": 2, "x": 1.0, "y": 2.0, "age": 1500, "generation": 6,
 "last_energy": 3.2, "cause": "starvation|old_age|senescence|killed", "killer": 1001 /* or null */}
```

Population crash/boom, predation and cooperation surges, records and season changes should be computed in JS from the stats history (UX spec §9.4).

## 8. Inspector (`tr_inspect_json(id: u32) -> len`, `tr_inspect_ptr()`)

Returns length 0 if `id` is not alive. Building it allocates a little and scans the population once (for `children_alive`), so call it at 4 Hz or less.

```jsonc
{
  "id": 1203, "parent_id": 455, "root_ancestor_id": 12, "generation": 7, "age": 340, "max_age": 2000,
  "birth_tick": 812, "species": 2, "children_alive": 3,
  "x": 1.0, "y": 2.0, "heading": 3.1, "signal": -0.2, "speed": 1.1, "speed_norm": 0.6, "turn": -0.4, "flags": 65,
  "trophic_level": 1.05, "body_size": 1.2, "color_hue": 211.0, "aging_rate": 0.002, "preferred_temp": 0.4,
  "diet_preference": 0.1, "mutation_rate": 0.1, "mutation_step": 0.2,
  "energy": {"current": 88, "maximum": 200, "lifetime_from_food": 410, "lifetime_from_attacks": 22,
             "predation_fraction": 0.05,
             "cost_size_metabolism": 0.34, "cost_brain_static": 0.24, "cost_temperature": 0.02,
             "cost_movement_last_tick": 0.18,   // per-tick estimates reconstructed from config + state
             "untracked": ["brain_activity","crowding","attack_cost","shares","construction","reproduction"]},
  "brain": {
    "num_inputs": 19, "num_outputs": 7, "num_hidden": 9, "num_layers": 3,
    "enabled_connections": 201, "total_connections": 223, "mean_methylation": 0.03, "entropy": 0.41,
    "nodes": [{"id": 0, "kind": "bias", "label": "bias", "depth": 0, "layer": 0},
              {"id": 1, "kind": "in", "label": "ray0_dist", "depth": 0, "layer": 0}, "...",
              {"id": 20, "kind": "out", "label": "turn", "depth": 1, "layer": 2},
              {"id": 27, "kind": "hidden", "label": "h27", "depth": 0.5, "layer": 1}],
    "edges": [{"from": 0, "to": 27, "weight": 0.31, "enabled": true, "methylation": 0.0, "effective_weight": 0.31, "recurrent": false}],
    "output_histograms": [{"name": "turn", "bins": [0.1, "... 8 values"], "entropy": 0.52}]
  }
}
```

Node ids: 0 is bias, 1..=inputs are inputs, then outputs, then hidden nodes (any id above that). `layer` is the rank of a node's distinct `depth`, from 0 (inputs) to `num_layers−1` (outputs), which gives a ready layered layout. `effective_weight = weight × (1 − methylation)` is what the brain actually uses. `output_histograms` are sim-core's decayed activation histograms (τ ≈ 100 ticks, 8 bins over [-1,1]) and are the closest thing to "recent outputs" available in v1.

## 9. Configs

- `tr_init` with `toml_len == 0` uses **`configs/default.toml` ⊕ `configs/demo.toml`**, both embedded at compile time.
- `configs/demo.toml` is an **overlay**, not a full config, so it follows any retuning of default.toml. It changes only what you watch: season period 10000 → 3600, droughts 0.0002 → 0.0006/tick lasting 1500 ticks, and day/night period 40 → 600 (40 ticks would flicker at 60 fps).
- The cap stays at default.toml's 500, which suits the 500-creature target.
- Don't use the demo overlay for experiments.

## 10. What the front end cannot get yet: v2 / needs sim-core

sim-wasm derives everything it can without touching sim.rs, genome.rs, activity.rs or config.rs. The following need small, observation-only additions to sim-core: no RNG draws, fixed-size, determinism preserved.

| Missing | Why it is impossible today | Suggested sim-core change |
|---|---|---|
| Missed attacks, `WAS_ATTACKED`, `SHARED`, `RECEIVED_SHARE`, `CONSTRUCTED` flags; `target_row` | `Action` is private and transient; the share/attack/construct outcome and target are never stored | `pub last_action: ActionRecord { flags: u8, target: Option<OrganismId>, speed, turn }` on `Organism` |
| Exact speed_norm / turn (brain outputs) | Same; v1 derives them from displacement and heading deltas | same as above |
| Exact death cause and killer | Dead organisms are `retain`ed away; no cause recorded | push `DeathRecord { id, x, y, cause, killer }` into a reused `pub deaths_this_tick: Vec<_>` before `retain` |
| sim-core's own per-organism species and stable ids | `species_assignments` is a local and `species_representatives` is private | `pub species_id: u32` on `Organism` + stable ids on representatives (v1 uses sim-wasm's tracker instead) |
| Energy budget over the last N ticks (food / fruit / hunting / shares in; metabolism / movement / brain / activity / crowding / temperature / construction / attack / shares out / reproduction) | Only lifetime `energy_from_food` and `energy_from_attacks` exist; per-tick costs are locals in `step()` | `[f32; 12]` EMA accumulator on `Organism` |
| Algae vs fruit split of food energy | Both added to `energy_from_food` | part of the EMA above |
| Last sensor inputs (20) / outputs (7), node activations | Built on the stack and discarded; `Brain::forward` exposes no activations | `last_inputs`/`last_outputs` on `Organism`, plus `Brain::forward_with_activations` (pure; can be re-run from sim-wasm) |
| Per-organism crowding (neighbour count) and brain activity cost | In the private `Action` buffer | part of `last_action` |
| `children_total` | No counter; v1 gives `children_alive` by scanning | `children_total: u32` on `Organism` |
| Full ancestry beyond parent/root | Only `parent_id`, `root_ancestor_id`, `clade_id` | a lineage log in sim-wasm (possible later without sim-core changes) |

## 11. JS loading example

```js
const res = await fetch('sim_wasm.wasm');
const { instance } = await WebAssembly.instantiateStreaming(res, {});   // zero imports
const w = instance.exports, dec = new TextDecoder(), enc = new TextEncoder();
const str = (ptr, len) => dec.decode(new Uint8Array(w.memory.buffer, ptr, len));

function init(tomlText = '', seed = 42n, overlay = false) {          // '' → embedded demo config
  const bytes = enc.encode(tomlText);
  const p = w.tr_alloc(bytes.length);
  new Uint8Array(w.memory.buffer, p, bytes.length).set(bytes);        // view made AFTER alloc
  const rc = (overlay ? w.tr_init_overlay : w.tr_init)(p, bytes.length, seed);
  w.tr_free(p, bytes.length);
  if (rc !== 0) throw new Error(`tr_init ${rc}: ${str(w.tr_last_error_ptr(), w.tr_last_error_len())}`);
  return JSON.parse(str(w.tr_config_ptr(), w.tr_config_json()));
}

const cfg = init();                                   // or init('[creature]\nhard_cap = 300\n', 7n, true)
const { cols, rows } = cfg.grid, STRIDE = cfg.layout.creature_stride;
const elevation = new Float32Array(w.memory.buffer, w.tr_grid_ptr(4), w.tr_grid_update(4)).slice(); // static: copy once

function frame(ticksPerFrame) {
  w.tr_step(ticksPerFrame);
  const n = w.tr_creatures_update();
  // Views are cheap: re-create them every frame so memory growth can never bite.
  const buf = w.memory.buffer;
  const rowsF = new Float32Array(buf, w.tr_creatures_ptr(), n * STRIDE);
  const ids = new Uint32Array(buf, w.tr_creature_ids_ptr(), n);
  for (let i = 0; i < n; i++) {
    const o = i * STRIDE, x = rowsF[o + 1], y = rowsF[o + 2], heading = rowsF[o + 3];
    const flags = rowsF[o + 10] | 0, species = rowsF[o + 8];
    // instancedMesh.setMatrixAt(i, ...); colour by species; pulse if (flags & 1) ate, (flags & 2) attacked
  }
  const deaths = new Float32Array(buf, w.tr_deaths_ptr(), w.tr_deaths_count() * w.tr_death_stride());
  const fr = new Float32Array(w.memory.buffer, w.tr_frame_ptr(), w.tr_frame_len());  // light = fr[1]
}
setInterval(() => {                                    // ~1 Hz dashboard + event feed
  const stats = JSON.parse(str(w.tr_stats_ptr(), w.tr_stats_json()));
  const { events } = JSON.parse(str(w.tr_events_ptr(), w.tr_events_json()));
}, 1000);
```

## 12. Export list (v1: 46 `tr_*` functions + `memory`)

`tr_abi_version, tr_alloc, tr_free, tr_init, tr_init_overlay, tr_reset, tr_last_error_ptr, tr_last_error_len, tr_step, tr_tick, tr_tick_f64, tr_seed, tr_world_width, tr_world_height, tr_frame_ptr, tr_frame_len, tr_creatures_update, tr_creature_count, tr_creatures_ptr, tr_creature_ids_ptr, tr_creature_stride, tr_deaths_count, tr_deaths_ptr, tr_death_ids_ptr, tr_death_stride, tr_grid_cols, tr_grid_rows, tr_grid_cell_size, tr_grid_update, tr_grid_ptr, tr_elevation_at, tr_droughts_update, tr_droughts_ptr, tr_drought_stride, tr_config_json, tr_config_ptr, tr_stats_json, tr_stats_ptr, tr_species_json, tr_species_ptr, tr_species_set_min_announce, tr_inspect_json, tr_inspect_ptr, tr_events_json, tr_events_ptr, tr_events_set_mask`, plus `__getrandom_custom`, an internal hook that always fails and must never be called.
