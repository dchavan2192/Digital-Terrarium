// sim.js — the ONLY module that touches WebAssembly.
//
// Contract: docs/research/SIM_WASM_ABI.md (sim-wasm ABI v1, `tr_*` exports).
//
// Two layers:
//  1. A raw "backend" with the ABI v1 shape: either the real WASM instance
//     (instantiateWasmBackend) or the JS mock (mock-sim.js), which mimics the
//     same buffers and JSON payloads.
//  2. A normalising facade (createSim) that the rest of the front end uses:
//       init(configText, seed) · reset(seed) · step(n) · tick()
//       frame()        -> reused { tick, light, season, population, droughts, species, births, deaths }
//       creatures()    -> reused { count, stride, data: Float32Array, ids: Uint32Array }
//       deaths()       -> reused { count, stride, data, ids }      (deaths in the last step)
//       grid(name)     -> Float32Array | null   ('food','fruit','pheromone','construction',
//                                                'elevation','temperature','biome')
//       droughts()     -> reused { count, stride, data }
//       stats()        -> stats sample (+ stable species list)  [allocates: ~1-10 Hz]
//       events()       -> drained event array, normalised to { type, tick, ... }
//       inspect(id)    -> inspector object | null               [allocates: ≤ 4 Hz]
//       config         -> normalised config (the front end hard-codes nothing)
//       caps           -> which optional features exist (UI hides what is missing)
//
// Typed-array views over WASM memory are cached and rebuilt whenever
// memory.buffer changes (growth detaches views) or a pointer/length changes.

import { createMockBackend } from './mock-sim.js';
import { lightLevel, seasonMultiplier } from './format.js';

export const ABI_SUPPORTED = 1;
const LOG = '[terrarium]';

/** Creature row layout (ABI v1 §4.2, 24 f32 per creature). */
export const L = {
  ID: 0, X: 1, Y: 2, HEADING: 3, ENERGY_FRAC: 4, BODY: 5, SPEED_NORM: 6, TURN: 7,
  SPECIES: 8, PREDATION: 9, FLAGS: 10, TARGET_ROW: 11, GEN: 12, AGE_FRAC: 13, SIGNAL: 14,
  DIET: 15, SPEED: 16, ENERGY: 17, HUE: 18, AGE: 19, METH: 20, CONN: 21, HIDDEN: 22, ELEV: 23,
};

/** Flag bits (ABI v1 §4.2). Event bits are OR-accumulated over a tr_step(n) batch. */
export const F = {
  ATE: 1, ATTACKED: 2, WAS_ATTACKED: 4, SHARED: 8, RECEIVED_SHARE: 16, CONSTRUCTED: 32,
  REPRODUCED: 64, NEWBORN: 128, STRESSED: 256, PREDATOR: 512,
};

/** Death row layout (§4.3, 6 f32). Cause: 0 starvation, 1 old age, 2 senescence, 3 killed. */
export const DTH = { X: 0, Y: 1, CAUSE: 2, SPECIES: 3, AGE: 4, GEN: 5 };
export const DEATH_CAUSES = ['starved', 'died of old age', 'died of senescence', 'was killed'];

/** Drought row layout (§4.5, 6 f32). */
export const DL = { X: 0, Y: 1, R: 2, SEVERITY: 3, REMAINING: 4, PROGRESS: 5 };

/** Frame scalars (§4.1, 8 f32). */
export const FR = { TICK: 0, LIGHT: 1, SEASON: 2, POP: 3, DROUGHTS: 4, SPECIES: 5, BIRTHS: 6, DEATHS: 7 };

export const GRID = { food: 0, fruit: 1, pheromone: 2, construction: 3, elevation: 4, temperature: 5, biome: 6 };
const STATIC_GRIDS = ['elevation', 'temperature', 'biome'];

// ---------------------------------------------------------------------------
// Real WASM backend
// ---------------------------------------------------------------------------

/** Instantiate from raw bytes (also used by the node smoke test). */
export async function instantiateWasmBackend(bytes) {
  const module = await WebAssembly.compile(bytes);
  const imports = {};
  const declared = WebAssembly.Module.imports(module);
  for (const imp of declared) {
    imports[imp.module] ??= {};
    if (imp.kind === 'function') {
      imports[imp.module][imp.name] = (...args) => {
        console.warn(`${LOG} WASM called unexpected import ${imp.module}.${imp.name}`, args);
        return 0;
      };
    }
  }
  if (declared.length) console.warn(`${LOG} terrarium.wasm declares imports (the ABI says zero; stubbed):`, declared);
  const instance = await WebAssembly.instantiate(module, imports);
  return wasmBackendFromExports(instance.exports, bytes.byteLength);
}

export async function loadWasmBackend(url) {
  const t0 = performance.now();
  const resp = await fetch(url);
  if (!resp.ok) throw new Error(`fetch ${url} -> HTTP ${resp.status}`);
  const bytes = await resp.arrayBuffer();
  const b = await instantiateWasmBackend(bytes);
  console.info(`${LOG} ${url} loaded (${(bytes.byteLength / 1024).toFixed(0)} KiB, ABI v${b.abiVersion}) in ${(performance.now() - t0).toFixed(0)} ms`);
  return b;
}

export function wasmBackendFromExports(ex) {
  const required = ['memory', 'tr_alloc', 'tr_free', 'tr_init', 'tr_step', 'tr_creatures_update', 'tr_creatures_ptr', 'tr_creature_stride', 'tr_stats_json', 'tr_stats_ptr', 'tr_config_json', 'tr_config_ptr'];
  const missing = required.filter((k) => !(k in ex));
  if (missing.length) throw new Error(`terrarium.wasm is missing exports: ${missing.join(', ')}`);
  const has = (name) => typeof ex[name] === 'function';
  const abi = has('tr_abi_version') ? ex.tr_abi_version() : 0;
  if (abi !== ABI_SUPPORTED) console.warn(`${LOG} sim-wasm ABI v${abi}, front end built for v${ABI_SUPPORTED}. Continuing; expect layout issues.`);
  const dec = new TextDecoder();
  const enc = new TextEncoder();
  const cache = new Map();

  function view(key, Ctor, ptr, len) {
    const buf = ex.memory.buffer;
    let v = cache.get(key);
    if (!v || v.buffer !== buf || v.byteOffset !== ptr || v.length !== len) {
      v = new Ctor(buf, ptr, len);
      cache.set(key, v);
    }
    return v;
  }
  function str(ptr, len) {
    return len ? dec.decode(new Uint8Array(ex.memory.buffer, ptr, len)) : '';
  }
  function json(lenFn, ptrFn) {
    if (!has(lenFn) || !has(ptrFn)) return null;
    const len = ex[lenFn]();
    if (!len) return null;
    const text = str(ex[ptrFn](), len);
    try {
      return JSON.parse(text);
    } catch (e) {
      console.error(`${LOG} bad JSON from ${lenFn}:`, e, text.slice(0, 200));
      return null;
    }
  }
  function callInit(fn, text, seed) {
    const bytes = enc.encode(text || '');
    const ptr = bytes.length ? ex.tr_alloc(bytes.length) : 0;
    if (bytes.length) new Uint8Array(ex.memory.buffer, ptr, bytes.length).set(bytes); // view AFTER alloc
    const rc = fn(ptr, bytes.length, BigInt.asUintN(64, BigInt(seed)));
    if (bytes.length) ex.tr_free(ptr, bytes.length);
    cache.clear();
    return rc;
  }

  return {
    kind: 'wasm',
    abiVersion: abi,
    has,
    hasOverlay: has('tr_init_overlay'),
    init: (text, seed) => callInit(ex.tr_init, text, seed),
    initOverlay: (text, seed) => callInit(has('tr_init_overlay') ? ex.tr_init_overlay : ex.tr_init, text, seed),
    reset(seed) {
      cache.clear();
      return has('tr_reset') ? ex.tr_reset(BigInt.asUintN(64, BigInt(seed))) : -4;
    },
    lastError: () => (has('tr_last_error_len') ? str(ex.tr_last_error_ptr(), ex.tr_last_error_len()) : ''),
    step: (n) => ex.tr_step(n),
    tick: () => (has('tr_tick_f64') ? ex.tr_tick_f64() : Number(ex.tr_tick())),
    memoryCreatureId: () => (has('tr_memory_creature_id') ? ex.tr_memory_creature_id() : -1),
    frame: () => (has('tr_frame_ptr') ? view('frame', Float32Array, ex.tr_frame_ptr(), ex.tr_frame_len()) : null),
    creatureStride: () => ex.tr_creature_stride(),
    creaturesUpdate: () => ex.tr_creatures_update(),
    creatures: (n) => view('creatures', Float32Array, ex.tr_creatures_ptr(), n * ex.tr_creature_stride()),
    creatureIds: (n) => (has('tr_creature_ids_ptr') ? view('cids', Uint32Array, ex.tr_creature_ids_ptr(), n) : null),
    deathsCount: () => (has('tr_deaths_count') ? ex.tr_deaths_count() : 0),
    deathStride: () => (has('tr_death_stride') ? ex.tr_death_stride() : 6),
    deaths: (n) => view('deaths', Float32Array, ex.tr_deaths_ptr(), n * (has('tr_death_stride') ? ex.tr_death_stride() : 6)),
    deathIds: (n) => (has('tr_death_ids_ptr') ? view('dids', Uint32Array, ex.tr_death_ids_ptr(), n) : null),
    gridUpdate: (kind) => (has('tr_grid_update') ? ex.tr_grid_update(kind) : 0),
    grid: (kind, len) => (has('tr_grid_ptr') && len ? view(`g${kind}`, Float32Array, ex.tr_grid_ptr(kind), len) : null),
    elevationAt: (x, y) => (has('tr_elevation_at') ? ex.tr_elevation_at(x, y) : 0),
    droughtsUpdate: () => (has('tr_droughts_update') ? ex.tr_droughts_update() : 0),
    droughtStride: () => (has('tr_drought_stride') ? ex.tr_drought_stride() : 6),
    droughts: (n) => view('droughts', Float32Array, ex.tr_droughts_ptr(), n * (has('tr_drought_stride') ? ex.tr_drought_stride() : 6)),
    statsJson: () => json('tr_stats_json', 'tr_stats_ptr'),
    speciesJson: () => json('tr_species_json', 'tr_species_ptr'),
    configJson: () => json('tr_config_json', 'tr_config_ptr'),
    eventsJson: () => json('tr_events_json', 'tr_events_ptr'),
    setEventMask: (m) => has('tr_events_set_mask') && ex.tr_events_set_mask(m),
    setMinAnnounce: (n) => has('tr_species_set_min_announce') && ex.tr_species_set_min_announce(n),
    inspectJson(id) {
      if (!has('tr_inspect_json')) return null;
      const len = ex.tr_inspect_json(id >>> 0);
      if (!len) return null;
      try {
        return JSON.parse(str(ex.tr_inspect_ptr(), len));
      } catch (e) {
        console.error(`${LOG} bad inspect JSON`, e);
        return null;
      }
    },
  };
}

// ---------------------------------------------------------------------------
// Normalisation
// ---------------------------------------------------------------------------

function num(v, d) {
  return typeof v === 'number' && Number.isFinite(v) ? v : d;
}

/** Normalise the §2 config JSON. */
export function normaliseConfig(c = {}) {
  const full = c.config || {};
  const en = c.enabled || {};
  const W = num(c.world?.width, 1000);
  const H = num(c.world?.height, 1000);
  const cellSize = num(c.grid?.cell_size, 10);
  return {
    width: W,
    height: H,
    cellSize,
    cols: num(c.grid?.cols, Math.round(W / cellSize)),
    rows: num(c.grid?.rows, Math.round(H / cellSize)),
    maxPerCell: num(c.grid?.max_per_cell, 1),
    hardCap: num(c.hard_cap, 500),
    initialCount: num(c.initial_count, 300),
    maxEnergy: num(c.max_energy, 200),
    maxAge: num(c.max_age, 2000),
    maxSpeed: num(c.max_speed, 2),
    attackRange: num(full.combat?.attack_range, 8),
    dayEnabled: c.daynight ? c.daynight.enabled !== false : true,
    dayPeriod: num(c.daynight?.period, 600),
    seasonPeriod: num(c.seasons?.period, 3600),
    seasonAmplitude: num(c.seasons?.amplitude, 0.3),
    droughtsEnabled: c.droughts ? c.droughts.enabled !== false : true,
    droughtDuration: num(c.droughts?.duration, 1500),
    biomes: (c.biomes || full.food?.biomes || []).map((b) => ({ x: b.x, y: b.y, radius: b.radius, multiplier: num(b.regrowth_multiplier, 1) })),
    fruitEnabled: en.fruit !== false && (en.fruit === true || !!full.food?.fruit?.enabled),
    elevationEnabled: c.elevation ? c.elevation.enabled !== false : false,
    elevationMin: num(c.elevation?.min, -1),
    elevationMax: num(c.elevation?.max, 1),
    pheromonesEnabled: c.pheromones ? c.pheromones.enabled !== false : false,
    constructionEnabled: c.niche_construction ? c.niche_construction.enabled !== false : false,
    temperatureEnabled: en.temperature !== false,
    inputs: num(c.brain?.inputs, 20),
    outputs: num(c.brain?.outputs, 7),
    inputLabels: c.brain?.input_labels || null,
    outputLabels: c.brain?.output_labels || null,
    defaultAgingRate: num(full.aging?.aging_rate, 0.002),
    brainCost: num(full.energy?.brain_cost, 0.001),
    connectionCost: num(full.energy?.connection_cost, 0.001),
    activityCost: num(full.energy?.activity_weight_cost, 0),
    activityTrackingEnabled: typeof full.activity_tracking?.enabled === 'boolean' ? full.activity_tracking.enabled : null,
    activityWindowSize: num(full.activity_tracking?.window_size, NaN),
    shadowEnabled: typeof full.activity_tracking?.shadow === 'boolean' ? full.activity_tracking.shadow : null,
    dietCompetitionSigma: num(full.density_pressure?.diet_competition_sigma, NaN),
    dietSlope: num(full.food?.fruit?.diet_slope, NaN),
    seed: num(c.seed, 0),
    raw: c,
  };
}

export function normaliseSpecies(list) {
  if (!Array.isArray(list)) return [];
  return list.map((s) => ({
    id: num(s.stable_id, num(s.id, -1)),
    parent: s.parent_stable_id ?? null,
    established: s.established !== false,
    population: num(s.population, 0),
    peak: num(s.peak, NaN),
    firstSeen: num(s.first_seen_tick, NaN),
    meanBodySize: num(s.mean_body_size, NaN),
    meanConnections: num(s.mean_connections, NaN),
    meanTrophic: num(s.mean_trophic_level, NaN),
    predatorFraction: num(s.predator_fraction, NaN),
    trend: num(s.trend, 0),
    meanHue: num(s.mean_hue, NaN),
  }));
}

export function normaliseInspect(r, cfg) {
  if (!r) return null;
  const e = r.energy || {};
  const b = r.brain || {};
  const hists = b.output_histograms || r.output_histograms || null;
  return {
    id: r.id,
    parentId: r.parent_id ?? null,
    rootId: r.root_ancestor_id ?? null,
    generation: num(r.generation, 0),
    age: num(r.age, 0),
    maxAge: num(r.max_age, cfg.maxAge),
    birthTick: num(r.birth_tick, 0),
    species: r.species ?? -1,
    childrenAlive: num(r.children_alive, NaN),
    x: r.x,
    y: r.y,
    speed: num(r.speed, 0),
    speedNorm: num(r.speed_norm, NaN),
    turn: num(r.turn, 0),
    flags: num(r.flags, 0),
    trophic: num(r.trophic_level, 1),
    traits: {
      bodySize: num(r.body_size, NaN),
      agingRate: num(r.aging_rate, NaN),
      preferredTemp: num(r.preferred_temp, NaN),
      diet: num(r.diet_preference, NaN),
      mutationRate: num(r.mutation_rate, NaN),
      mutationStep: num(r.mutation_step, NaN),
    },
    energy: {
      current: num(e.current, NaN),
      maximum: num(e.maximum, cfg.maxEnergy),
      fromFood: num(e.lifetime_from_food, 0),
      fromAttacks: num(e.lifetime_from_attacks, 0),
      predationFraction: num(e.predation_fraction, 0),
      costMetabolism: num(e.cost_size_metabolism, NaN),
      costBrain: num(e.cost_brain_static, NaN),
      costTemperature: num(e.cost_temperature, NaN),
      costMovement: num(e.cost_movement_last_tick, NaN),
    },
    brain: {
      inputs: num(b.num_inputs, cfg.inputs),
      outputs: num(b.num_outputs, cfg.outputs),
      hidden: num(b.num_hidden, 0),
      enabledConnections: num(b.enabled_connections, NaN),
      meanMethylation: num(b.mean_methylation, NaN),
      entropy: num(b.entropy, NaN),
      nodes: (b.nodes || []).map((n) => ({
        id: n.id,
        kind: n.kind === 'input' ? 'in' : n.kind === 'output' ? 'out' : n.kind,
        label: n.label,
        depth: num(n.depth, 0),
        layer: num(n.layer, 0),
      })),
      edges: b.edges || b.connections || [],
      outputHistograms: hists,
    },
    // v2 fields (absent in ABI v1): exact last outputs / inputs / activations.
    lastOutputs: Array.isArray(r.last_outputs) ? r.last_outputs : null,
    lastInputs: Array.isArray(r.last_inputs) ? r.last_inputs : null,
    activations: Array.isArray(r.activations) ? r.activations : null,
  };
}

/** Event normalisation: ABI uses {kind, t}; keep both spellings readable. */
function normaliseEvent(e) {
  return { ...e, type: e.kind ?? e.type, tick: num(e.t, num(e.tick, 0)) };
}

// ---------------------------------------------------------------------------
// Facade
// ---------------------------------------------------------------------------

export function createSim(backend) {
  const creatureOut = { count: 0, stride: 24, data: new Float32Array(0), ids: null };
  const deathOut = { count: 0, stride: 6, data: new Float32Array(0), ids: null };
  const droughtOut = { count: 0, stride: 6, data: new Float32Array(0) };
  const frameOut = { tick: 0, light: 1, season: 1, population: 0, droughts: 0, species: 0, births: 0, deaths: 0 };
  const staticGrids = {};
  let overlayText = '';
  let seed = 0;

  const sim = {
    kind: backend.kind,
    backend,
    config: null,
    caps: {},

    /** configText is a TOML overlay (web/demo/demo.toml); '' = embedded demo config. */
    init(configText, newSeed) {
      overlayText = configText || '';
      seed = newSeed >>> 0;
      const rc = backend.initOverlay(overlayText, seed);
      if (rc !== 0) {
        const msg = backend.lastError();
        throw new Error(`tr_init_overlay failed (status ${rc})${msg ? `: ${msg}` : ''}`);
      }
      afterInit();
    },

    reset(newSeed) {
      seed = newSeed >>> 0;
      const rc = backend.reset(seed);
      if (rc !== 0) sim.init(overlayText, seed);
      else afterInit();
    },

    get seed() {
      return seed;
    },

    step(n) {
      if (n > 0) backend.step(n);
    },

    tick() {
      return backend.tick();
    },

    frame() {
      const f = backend.frame();
      const tick = backend.tick();
      frameOut.tick = tick;
      if (f && f.length >= 8) {
        frameOut.light = f[FR.LIGHT];
        frameOut.season = f[FR.SEASON];
        frameOut.population = f[FR.POP];
        frameOut.droughts = f[FR.DROUGHTS];
        frameOut.species = f[FR.SPECIES];
        frameOut.births = f[FR.BIRTHS];
        frameOut.deaths = f[FR.DEATHS];
      } else {
        frameOut.light = lightLevel(tick, sim.config.dayPeriod, sim.config.dayEnabled);
        frameOut.season = seasonMultiplier(tick, sim.config.seasonPeriod, sim.config.seasonAmplitude);
      }
      return frameOut;
    },

    creatures() {
      const n = backend.creaturesUpdate();
      creatureOut.count = n;
      creatureOut.stride = backend.creatureStride();
      creatureOut.data = backend.creatures(n);
      creatureOut.ids = backend.creatureIds(n);
      return creatureOut;
    },

    deaths() {
      const n = backend.deathsCount();
      deathOut.count = n;
      deathOut.stride = backend.deathStride();
      if (n) {
        deathOut.data = backend.deaths(n);
        deathOut.ids = backend.deathIds(n);
      }
      return deathOut;
    },

    grid(name) {
      if (STATIC_GRIDS.includes(name)) return staticGrids[name] || null;
      const kind = GRID[name];
      if (kind === undefined) return null;
      const len = backend.gridUpdate(kind);
      return len ? backend.grid(kind, len) : null;
    },

    elevationAt(x, y) {
      return backend.elevationAt(x, y);
    },

    droughts() {
      const n = backend.droughtsUpdate();
      droughtOut.count = n;
      droughtOut.stride = backend.droughtStride();
      if (n) droughtOut.data = backend.droughts(n);
      return droughtOut;
    },

    /** Stats sample. Allocates (JSON); call at sample cadence, not every frame. */
    stats() {
      const raw = backend.statsJson();
      if (!raw) return null;
      const s = raw.stats || {};
      const env = raw.env || {};
      const species = normaliseSpecies(raw.species || (backend.speciesJson ? backend.speciesJson() : null));
      const tick = num(raw.tick, num(s.tick, 0));
      const c = sim.config;
      const est = species.filter((x) => x.established && x.population > 0);
      return {
        ...s,
        tick,
        population: num(s.population, 0),
        speciesCount: num(raw.established_species, est.length || num(s.species_count, 0)),
        light: num(env.light_level, lightLevel(tick, c.dayPeriod, c.dayEnabled)),
        isNight: env.is_night ?? false,
        season: num(env.season_multiplier, seasonMultiplier(tick, c.seasonPeriod, c.seasonAmplitude)),
        seasonPhase: c.seasonPeriod > 0 ? ((tick / c.seasonPeriod) % 1 + 1) % 1 : NaN,
        seasonAmplitude: c.seasonAmplitude,
        hardCap: c.hardCap,
        species,
      };
    },

    events() {
      const raw = backend.eventsJson();
      if (!raw) return [];
      if (raw.dropped) console.debug(`${LOG} ${raw.dropped} sim events dropped (queue full)`);
      const list = Array.isArray(raw.events) ? raw.events : [];
      return list.map(normaliseEvent);
    },

    inspect(id) {
      return normaliseInspect(backend.inspectJson(id), sim.config);
    },

    memoryCreatureId() {
      return typeof backend.memoryCreatureId === 'function' ? backend.memoryCreatureId() : -1;
    },
  };

  function afterInit() {
    sim.config = normaliseConfig(backend.configJson() || {});
    const c = sim.config;
    const n = c.cols * c.rows;
    for (const name of STATIC_GRIDS) {
      const len = backend.gridUpdate(GRID[name]);
      const v = len ? backend.grid(GRID[name], len) : null;
      staticGrids[name] = v && v.length === n ? Float32Array.from(v) : null; // static: copy once
    }
    sim.caps = {
      elevation: !!staticGrids.elevation && c.elevationEnabled,
      fruit: c.fruitEnabled,
      pheromone: c.pheromonesEnabled,
      construction: c.constructionEnabled,
      temperature: c.temperatureEnabled,
      biome: !!staticGrids.biome,
      droughts: c.droughtsEnabled,
      inspect: backend.has('tr_inspect_json'),
      memory: backend.kind === 'wasm' && backend.has('tr_memory_creature_id'),
      shadow: backend.kind === 'wasm' && typeof c.shadowEnabled === 'boolean',
      deaths: backend.has('tr_deaths_count'),
      // ABI v1 gaps (SIM_WASM_ABI.md §10). UI that needs these stays hidden.
      shareFlags: false,
      constructFlags: false,
      attackTargets: false,
      liveOutputs: false,
      energyBudget: false,
    };
    backend.eventsJson(); // drain init-time events so the feed starts clean
    console.info(`${LOG} sim ready (${sim.kind}, ABI v${backend.abiVersion}): ${c.cols}x${c.rows} grid, world ${c.width}x${c.height}, cap ${c.hardCap}, seed ${seed}`, sim.caps);
  }

  return sim;
}

/**
 * Fetch the overlay config + WASM, then init. Never throws for WASM problems:
 * it logs loudly and falls back to the mock so the page still renders.
 */
export async function loadSim({ wasmUrl = 'terrarium.wasm', configUrl = 'demo.toml', seed = 42, forceMock = false } = {}) {
  let configText = '';
  try {
    const r = await fetch(configUrl);
    if (r.ok) configText = await r.text();
    else console.warn(`${LOG} ${configUrl} -> HTTP ${r.status}; using the config embedded in the WASM`);
  } catch (e) {
    console.warn(`${LOG} could not fetch ${configUrl}; using the embedded config`, e);
  }

  let backend = null;
  let error = null;
  if (!forceMock) {
    try {
      backend = await loadWasmBackend(wasmUrl);
    } catch (e) {
      error = e;
      console.error(`${LOG} FAILED to load ${wasmUrl}. Falling back to the JS MOCK simulation (fake data!). Build it with: cargo build -p sim-wasm --target wasm32-unknown-unknown --release && cp target/wasm32-unknown-unknown/release/sim_wasm.wasm web/demo/terrarium.wasm`, e);
    }
  }
  if (!backend) backend = createMockBackend();

  let sim = createSim(backend);
  try {
    sim.init(configText, seed);
  } catch (e) {
    console.error(`${LOG} init with ${configUrl} failed:`, e);
    error = e;
    try {
      console.warn(`${LOG} retrying with the embedded demo config`);
      sim.init('', seed);
    } catch (e2) {
      console.error(`${LOG} embedded config failed too; falling back to the mock`, e2);
      sim = createSim(createMockBackend());
      sim.init(configText, seed);
    }
  }
  return { sim, error, configText };
}
