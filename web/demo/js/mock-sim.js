// Deterministic JS stand-in for crates/sim-wasm. NOT the real simulation.
//
// It implements the same raw "backend" shape that sim.js builds over the WASM
// exports, following docs/research/SIM_WASM_ABI.md v1 (24-f32 creature rows,
// u32 ids, OR-accumulated flags, deaths buffer, 8-f32 frame, grid kinds 0..6,
// {kind, t} events, config/stats/species/inspect JSON shapes), so every
// renderer/UI code path is exercised identically.
// Used when ?mock=1 is in the URL, or automatically when terrarium.wasm fails
// to load (sim.js logs a loud warning in that case).
//
// The fake ecology: agents wander a sine heightfield, steer toward food, eat
// algae/fruit, pay metabolism, reproduce with mutation, sometimes found new
// species, sometimes hunt; droughts spawn and expire; food regrows per biome
// with seasons. Plausible-looking, deterministic per seed (mulberry32).
//
// Pure module: no DOM, so node can import it.

export const MOCK_STRIDE = 24;
export const MOCK_DROUGHT_STRIDE = 6;
export const MOCK_DEATH_STRIDE = 6;
const TAU = Math.PI * 2;

function mulberry32(seed) {
  let a = seed >>> 0;
  return function rand() {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Minimal TOML reader: [table], [[array.of.tables]], key = number|bool|"string". */
export function parseTomlLite(text) {
  const root = {};
  let cur = root;
  for (const raw of String(text || '').split(/\r?\n/)) {
    const line = raw.replace(/\s+#.*$/, '').replace(/^#.*$/, '').trim();
    if (!line) continue;
    let m = line.match(/^\[\[([\w.]+)\]\]$/);
    if (m) {
      const path = m[1].split('.');
      let o = root;
      for (let i = 0; i < path.length - 1; i++) o = o[path[i]] ??= {};
      const arr = (o[path[path.length - 1]] ??= []);
      cur = {};
      arr.push(cur);
      continue;
    }
    m = line.match(/^\[([\w.]+)\]$/);
    if (m) {
      cur = root;
      for (const k of m[1].split('.')) cur = cur[k] ??= {};
      continue;
    }
    m = line.match(/^([\w]+)\s*=\s*(.+)$/);
    if (m) {
      const v = m[2].trim();
      cur[m[1]] = v === 'true' ? true : v === 'false' ? false : /^".*"$/.test(v) ? v.slice(1, -1) : Number(v);
    }
  }
  return root;
}

const DEFAULTS = {
  world: { width: 1000, height: 1000, seed: 42 },
  food: {
    cell_size: 10, max_per_cell: 1, regrowth_rate: 0.0005, energy_per_food: 15,
    seasons: { amplitude: 0.3, period: 3600 },
    droughts: { spawn_rate: 0.0006, duration: 1500, radius: 150, severity: 0.05, max_active: 3 },
    biomes: [
      { x: 250, y: 250, radius: 200, regrowth_multiplier: 3 },
      { x: 750, y: 750, radius: 200, regrowth_multiplier: 3 },
      { x: 500, y: 500, radius: 150, regrowth_multiplier: 2 },
      { x: 750, y: 250, radius: 180, regrowth_multiplier: 0.3 },
    ],
    fruit: { enabled: true, energy_per_fruit: 20, biome_rate_multiplier: 2 },
  },
  creature: { initial_count: 300, hard_cap: 500, body_radius: 3, max_speed: 2, max_turn: 0.2, max_age: 2000, vision: { ray_count: 5, fov: 2.094, range: 60 } },
  energy: { starting: 60, maximum: 200, base_metabolism: 0.3, movement_cost: 0.15, brain_cost: 0.001, connection_cost: 0.001 },
  reproduction: { threshold: 60, parent_cost: 30, child_energy: 30 },
  combat: { attack_range: 8, attack_cost: 5, attack_damage: 20, energy_transfer_ratio: 0.75 },
  brain: { inputs: 19, hidden: 8, outputs: 7 },
  daynight: { enabled: true, period: 600 },
  elevation: { enabled: true, slope_factor: 5, octaves: 4, seed: 7777 },
  pheromones: { enabled: true, max_level: 10 },
  niche_construction: { enabled: true, max_level: 1 },
  temperature: { enabled: true, cost_factor: 0.5 },
  aging: { aging_rate: 0.002 },
};

function deepMerge(base, over) {
  const out = Array.isArray(base) ? base.slice() : { ...base };
  for (const k of Object.keys(over || {})) {
    const v = over[k];
    out[k] = v && typeof v === 'object' && !Array.isArray(v) && base[k] && typeof base[k] === 'object' ? deepMerge(base[k], v) : v;
  }
  return out;
}

const INPUT_TAIL = ['energy', 'heading', 'heard_signal', 'body_size', 'kin_similarity', 'temperature', 'pheromone', 'light', 'niche_level'];
const OUTPUTS = ['turn', 'speed', 'eat', 'attack', 'signal', 'share', 'construct'];

// Flag bits — identical to SIM_WASM_ABI.md §4.2.
const ATE = 1;
const ATTACKED = 2;
const REPRODUCED = 64;
const NEWBORN = 128;
const STRESSED = 256;
const PREDATOR = 512;
const EVENT_BITS = 0xff;
// Event mask bits (§7): 1 SPECIES, 2 DROUGHT, 4 EXTINCTION, 8 BIRTH, 16 DEATH.
const MASK = { species: 1, drought: 2, extinction: 4, birth: 8, death: 16 };
const CAUSE = { starvation: 0, old_age: 1, senescence: 2, killed: 3 };

export function createMockBackend() {
  let cfg = null;
  let rand = Math.random;
  let tick = 0;
  let W = 1000;
  let H = 1000;
  let cols = 100;
  let rows = 100;
  let cs = 10;
  let food;
  let fruit;
  let pher;
  let built;
  let rate; // per-cell base regrowth multiplier (biome)
  let elev;
  let temp;
  let elevRange = [-1, 1];
  let orgs = [];
  let nextId = 1;
  let nextSpecies = 1;
  let speciesMeta = new Map(); // id -> {parent, firstSeen, peak, hue, hunter, coop, size, established, ema}
  let droughts = [];
  let events = [];
  let eventMask = MASK.species | MASK.drought | MASK.extinction;
  let minAnnounce = 5;
  let creatureBuf = new Float32Array(0);
  let idBuf = new Uint32Array(0);
  let deathBuf = new Float32Array(MOCK_DEATH_STRIDE * 64);
  let deathIdBuf = new Uint32Array(64);
  let deathCount = 0;
  let droughtBuf = new Float32Array(MOCK_DROUGHT_STRIDE * 8);
  const frameBuf = new Float32Array(8);
  let elevParams = [];
  let counters = null;
  let currentSeed = 42;
  let lastText = '';
  let lastError = '';
  let bins = null;
  const BIN = 50;

  function push(kind, ev) {
    if (eventMask & MASK[kind === 'species_born' || kind === 'species_extinct' ? 'species' : kind.startsWith('drought') ? 'drought' : kind]) {
      events.push({ kind, t: tick, ...ev });
      if (events.length > 1024) events.shift();
    }
  }

  function heightAt(x, y) {
    let h = 0;
    let amp = 1;
    for (const [fx, fy, px, py] of elevParams) {
      h += amp * Math.sin(fx * x + px) * Math.sin(fy * y + py);
      amp *= 0.5;
    }
    return h;
  }
  const cellIndex = (x, y) => Math.min(rows - 1, Math.max(0, Math.floor(y / cs))) * cols + Math.min(cols - 1, Math.max(0, Math.floor(x / cs)));
  const wrap = (v, m) => ((v % m) + m) % m;
  function wdelta(a, b, m) {
    let d = b - a;
    if (d > m / 2) d -= m;
    else if (d < -m / 2) d += m;
    return d;
  }
  const predFrac = (o) => (o.fromFood + o.fromAttacks > 0 ? o.fromAttacks / (o.fromFood + o.fromAttacks) : 0);

  function newSpecies(parent, hue) {
    const id = nextSpecies++;
    speciesMeta.set(id, {
      parent,
      firstSeen: tick,
      peak: 0,
      hue,
      hunter: rand() < 0.3 ? 0.3 + rand() * 0.6 : rand() * 0.15,
      coop: rand() * 0.9,
      size: 0.7 + rand() * 1.8,
      established: false,
      ema: 0,
      alive: true,
    });
    return id;
  }

  function makeOrg(x, y, species, parent) {
    const sm = speciesMeta.get(species);
    const id = nextId++;
    return {
      id, x, y,
      heading: rand() * TAU,
      speed: 0,
      speedNorm: 0,
      turn: 0,
      energy: cfg.energy.starting,
      bodySize: Math.min(3, Math.max(0.5, sm.size + (rand() - 0.5) * 0.3)),
      hue: wrap(sm.hue + (rand() - 0.5) * 8, 360),
      species,
      generation: parent ? parent.generation + 1 : 0,
      age: 0,
      birthTick: tick,
      parentId: parent ? parent.id : null,
      root: parent ? parent.root : id,
      diet: Math.min(1, Math.max(0, (parent ? parent.diet : rand()) + (rand() - 0.5) * 0.1)),
      prefTemp: Math.min(1, Math.max(0, (parent ? parent.prefTemp : y / H) + (rand() - 0.5) * 0.1)),
      hunter: Math.min(1, Math.max(0, sm.hunter + (rand() - 0.5) * 0.1)),
      nHidden: 4 + Math.floor(rand() * 8),
      nConn: Math.max(10, (parent ? parent.nConn : 150) + Math.floor((rand() - 0.4) * 6)),
      fromFood: 0,
      fromAttacks: 0,
      signal: rand() * 2 - 1,
      flags: NEWBORN,
      phase: rand() * TAU,
      hitBy: 0,
    };
  }

  function build(text, seed) {
    const parsed = parseTomlLite(text);
    cfg = deepMerge(DEFAULTS, parsed);
    if (Array.isArray(parsed?.food?.biomes)) cfg.food.biomes = parsed.food.biomes;
    currentSeed = Number(seed) >>> 0;
    rand = mulberry32(currentSeed ^ 0x9e3779b9);
    tick = 0;
    W = cfg.world.width;
    H = cfg.world.height;
    cs = cfg.food.cell_size;
    cols = Math.round(W / cs);
    rows = Math.round(H / cs);
    const n = cols * rows;
    food = new Float32Array(n);
    fruit = new Float32Array(n);
    pher = new Float32Array(n);
    built = new Float32Array(n);
    rate = new Float32Array(n);
    elev = new Float32Array(n);
    temp = new Float32Array(n);
    const er = mulberry32(cfg.elevation.seed || 7777);
    elevParams = [];
    for (let k = 0; k < (cfg.elevation.octaves || 4); k++) {
      const f = (TAU / W) * (1 << k) * (1 + Math.floor(er() * 2));
      elevParams.push([f, f * (0.8 + er() * 0.4), er() * TAU, er() * TAU]);
    }
    let lo = Infinity;
    let hi = -Infinity;
    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        const i = r * cols + c;
        const x = (c + 0.5) * cs;
        const y = (r + 0.5) * cs;
        let m = 1;
        for (const b of cfg.food.biomes) {
          const d = Math.hypot(wdelta(b.x, x, W), wdelta(b.y, y, H));
          if (d < b.radius) m *= 1 + (b.regrowth_multiplier - 1) * Math.min(1, (1 - d / b.radius) * 1.6);
        }
        rate[i] = m;
        elev[i] = cfg.elevation.enabled ? heightAt(x, y) : 0;
        lo = Math.min(lo, elev[i]);
        hi = Math.max(hi, elev[i]);
        temp[i] = Math.min(1, y / H);
        food[i] = Math.min(1, 0.3 + 0.5 * rand() * Math.min(1, m / 2));
        fruit[i] = m > 1.2 && cfg.food.fruit.enabled ? rand() * 0.6 : 0;
      }
    }
    elevRange = [lo, hi];
    orgs = [];
    nextId = 1;
    nextSpecies = 1;
    speciesMeta = new Map();
    droughts = [];
    events = [];
    deathCount = 0;
    counters = { births: 0, deaths: 0, attacks: 0, shares: 0, totalBirths: 0, totalDeaths: 0, stepBirths: 0, stepDeaths: 0 };
    const founders = [];
    for (let s = 0; s < 6; s++) founders.push(newSpecies(null, rand() * 360));
    for (let i = 0; i < cfg.creature.initial_count; i++) {
      orgs.push(makeOrg(rand() * W, rand() * H, founders[Math.floor(rand() * founders.length)], null));
    }
    for (const id of founders) speciesMeta.get(id).established = true;
    const cap = cfg.creature.hard_cap + 16;
    creatureBuf = new Float32Array(cap * MOCK_STRIDE);
    idBuf = new Uint32Array(cap);
    bins = { cols: Math.ceil(W / BIN), rows: Math.ceil(H / BIN), cells: [] };
    for (let i = 0; i < bins.cols * bins.rows; i++) bins.cells.push([]);
    fillFrame();
  }

  function rebin() {
    for (const c of bins.cells) c.length = 0;
    for (const o of orgs) {
      const bc = Math.min(bins.cols - 1, Math.floor(o.x / BIN));
      const br = Math.min(bins.rows - 1, Math.floor(o.y / BIN));
      bins.cells[br * bins.cols + bc].push(o);
    }
  }

  function nearest(o, range) {
    const bc = Math.floor(o.x / BIN);
    const br = Math.floor(o.y / BIN);
    let best = null;
    let bd = range * range;
    for (let dr = -1; dr <= 1; dr++) {
      for (let dc = -1; dc <= 1; dc++) {
        const c = (bc + dc + bins.cols) % bins.cols;
        const r = (br + dr + bins.rows) % bins.rows;
        for (const p of bins.cells[r * bins.cols + c]) {
          if (p === o || p.dead) continue;
          const dx = wdelta(o.x, p.x, W);
          const dy = wdelta(o.y, p.y, H);
          const d2 = dx * dx + dy * dy;
          if (d2 < bd) {
            bd = d2;
            best = p;
          }
        }
      }
    }
    return best;
  }

  function recordDeath(o, cause) {
    if (deathCount * MOCK_DEATH_STRIDE >= deathBuf.length) {
      const nb = new Float32Array(deathBuf.length * 2);
      nb.set(deathBuf);
      deathBuf = nb;
      const ni = new Uint32Array(deathIdBuf.length * 2);
      ni.set(deathIdBuf);
      deathIdBuf = ni;
    }
    const j = deathCount * MOCK_DEATH_STRIDE;
    deathBuf[j] = o.x;
    deathBuf[j + 1] = o.y;
    deathBuf[j + 2] = CAUSE[cause];
    deathBuf[j + 3] = o.species;
    deathBuf[j + 4] = o.age;
    deathBuf[j + 5] = o.generation;
    deathIdBuf[deathCount] = o.id;
    deathCount++;
  }

  function stepOnce() {
    tick++;
    counters.births = 0;
    counters.deaths = 0;
    counters.attacks = 0;
    counters.shares = 0;
    const E = cfg.energy;
    const maxSpeed = cfg.creature.max_speed;
    const season = 1 + cfg.food.seasons.amplitude * Math.sin((TAU * tick) / cfg.food.seasons.period);
    const light = cfg.daynight.enabled ? (1 + Math.sin((TAU * tick) / cfg.daynight.period)) / 2 : 1;
    rebin();
    const born = [];
    for (const o of orgs) {
      o.age++;
      const look = 25;
      const la = o.heading - 0.5;
      const ra = o.heading + 0.5;
      const fl = food[cellIndex(wrap(o.x + Math.cos(la) * look, W), wrap(o.y + Math.sin(la) * look, H))];
      const fr = food[cellIndex(wrap(o.x + Math.cos(ra) * look, W), wrap(o.y + Math.sin(ra) * look, H))];
      const turn = Math.max(-1, Math.min(1, (fr - fl) * 1.5 + (rand() - 0.5) * 0.6));
      o.heading = wrap(o.heading + turn * cfg.creature.max_turn, TAU);
      o.turn = turn;
      const ci = cellIndex(o.x, o.y);
      const hungry = 1 - o.energy / E.maximum;
      let sp = 0.25 + 0.6 * hungry * (1 - food[ci]) + 0.15 * Math.sin(o.phase + tick * 0.01);
      if (light < 0.3) sp *= 0.6;
      sp = Math.max(0, Math.min(1, sp));
      const step = (sp * maxSpeed) / Math.max(0.5, o.bodySize) * 1.5;
      const nx = wrap(o.x + Math.cos(o.heading) * step, W);
      const ny = wrap(o.y + Math.sin(o.heading) * step, H);
      const slow = cfg.elevation.enabled ? 1 / (1 + cfg.elevation.slope_factor * Math.abs(heightAt(nx, ny) - heightAt(o.x, o.y))) : 1;
      const px = o.x;
      const py = o.y;
      o.x = wrap(o.x + Math.cos(o.heading) * step * slow, W);
      o.y = wrap(o.y + Math.sin(o.heading) * step * slow, H);
      o.speed = Math.hypot(wdelta(px, o.x, W), wdelta(py, o.y, H));
      o.speedNorm = Math.min(1, o.speed / (maxSpeed / Math.max(0.5, o.bodySize)));
      const c2 = cellIndex(o.x, o.y);
      const algae = Math.min(food[c2], 0.15) * (1 - o.diet * 0.5);
      const fr2 = Math.min(fruit[c2], 0.1) * o.diet;
      if (algae > 0.01 || fr2 > 0.01) {
        food[c2] -= algae;
        fruit[c2] -= fr2;
        const gain = algae * cfg.food.energy_per_food + fr2 * cfg.food.fruit.energy_per_fruit;
        o.energy = Math.min(E.maximum, o.energy + gain);
        o.fromFood += gain;
        o.flags |= ATE;
      }
      if (o.hunter > 0.3 && rand() < o.hunter * 0.08) {
        const prey = nearest(o, cfg.combat.attack_range);
        if (prey && prey.species !== o.species) {
          const dmg = cfg.combat.attack_damage * (o.bodySize / (o.bodySize + prey.bodySize));
          prey.energy -= dmg;
          prey.hitBy = o.id;
          const gain = dmg * cfg.combat.energy_transfer_ratio;
          o.energy = Math.min(E.maximum, o.energy + gain - cfg.combat.attack_cost);
          o.fromAttacks += gain;
          o.flags |= ATTACKED;
          counters.attacks++;
        }
      }
      const sm = speciesMeta.get(o.species);
      if (sm && rand() < sm.coop * 0.01) counters.shares++;
      o.energy -= E.base_metabolism * Math.pow(o.bodySize, 0.75) + E.movement_cost * sp * sp + E.connection_cost * o.nConn * 0.3 + E.brain_cost * (o.nHidden + 26);
      pher[c2] = Math.min(1, pher[c2] + 0.01);
      built[c2] = Math.max(-1, Math.min(1, built[c2] + (sm && sm.coop > 0.6 ? 0.002 : -0.0005)));
      if (o.energy > cfg.reproduction.threshold && orgs.length + born.length < cfg.creature.hard_cap && rand() < 0.02) {
        o.energy -= cfg.reproduction.parent_cost;
        let species = o.species;
        if (rand() < 0.006) species = newSpecies(o.species, wrap(o.hue + 40 + rand() * 80, 360));
        const child = makeOrg(wrap(o.x + (rand() - 0.5) * 12, W), wrap(o.y + (rand() - 0.5) * 12, H), species, o);
        child.energy = cfg.reproduction.child_energy;
        o.flags |= REPRODUCED;
        born.push(child);
        counters.births++;
        counters.totalBirths++;
        counters.stepBirths++;
        push('birth', { id: child.id, parent: o.id, species: child.species, generation: child.generation, x: child.x, y: child.y });
      }
    }
    const survivors = [];
    for (const o of orgs) {
      let cause = null;
      if (o.age >= cfg.creature.max_age) cause = 'old_age';
      else if (o.energy <= 0) cause = o.hitBy ? 'killed' : 'starvation';
      else if (rand() < 0.0001 + 0.00001 * Math.exp(0.002 * o.age)) cause = 'senescence';
      if (cause) {
        o.dead = true;
        counters.deaths++;
        counters.totalDeaths++;
        counters.stepDeaths++;
        recordDeath(o, cause);
        push('death', { id: o.id, species: o.species, x: o.x, y: o.y, age: o.age, generation: o.generation, last_energy: o.energy, cause, killer: cause === 'killed' ? o.hitBy : null });
      } else {
        o.hitBy = 0;
        survivors.push(o);
      }
    }
    orgs = survivors.concat(born);
    const D = cfg.food.droughts;
    for (const d of droughts) {
      d.remaining--;
      if (d.remaining <= 0) push('drought_end', { x: d.x, y: d.y, radius: d.radius });
    }
    droughts = droughts.filter((d) => d.remaining > 0);
    if (droughts.length < D.max_active && rand() < D.spawn_rate) {
      const d = { x: rand() * W, y: rand() * H, radius: D.radius, severity: D.severity, remaining: D.duration };
      droughts.push(d);
      push('drought_start', { x: d.x, y: d.y, radius: d.radius, severity: d.severity, duration: D.duration });
    }
    const base = cfg.food.regrowth_rate * season * 4; // the mock eats faster, so it regrows faster
    const n = cols * rows;
    for (let i = 0; i < n; i++) {
      let m = rate[i] * (1 + built[i] * 0.5);
      if (droughts.length) {
        const x = ((i % cols) + 0.5) * cs;
        const y = (Math.floor(i / cols) + 0.5) * cs;
        for (const d of droughts) {
          const dx = wdelta(d.x, x, W);
          const dy = wdelta(d.y, y, H);
          if (dx * dx + dy * dy < d.radius * d.radius) m *= d.severity;
        }
      }
      food[i] = Math.min(cfg.food.max_per_cell, food[i] + base * m);
      if (rate[i] > 1.2 && cfg.food.fruit.enabled) fruit[i] = Math.min(cfg.food.max_per_cell, fruit[i] + base * m * 0.5);
      pher[i] *= 0.995;
      built[i] *= 0.999;
    }
    if (orgs.length === 0 && survivors.length === 0 && counters.deaths > 0) push('extinction', {});
  }

  function aggregateSpecies() {
    const agg = new Map();
    for (const o of orgs) {
      let a = agg.get(o.species);
      if (!a) agg.set(o.species, (a = { n: 0, size: 0, conn: 0, gen: 0, troph: 0, pred: 0, herb: 0, energy: 0, hue: 0 }));
      a.n++;
      a.size += o.bodySize;
      a.conn += o.nConn;
      a.gen += o.generation;
      a.energy += o.energy;
      a.hue = o.hue;
      const t = predFrac(o);
      a.troph += 1 + t;
      if (t > 0.5) a.pred++;
      if (t < 0.25) a.herb++;
    }
    return agg;
  }

  /** Once per step(n): update species lifecycle (born/extinct events) like the tracker. */
  function updateSpecies() {
    const agg = aggregateSpecies();
    for (const [id, m] of speciesMeta) {
      if (!m.alive) continue;
      const a = agg.get(id);
      const pop = a ? a.n : 0;
      m.pop = pop;
      m.peak = Math.max(m.peak, pop);
      m.ema = m.ema ? m.ema + (pop - m.ema) * 0.05 : pop;
      if (!m.established && pop >= minAnnounce) {
        m.established = true;
        push('species_born', { species: id, parent: m.parent, size: pop });
      }
      if (pop === 0) {
        m.alive = false;
        if (m.established) push('species_extinct', { species: id, peak: m.peak, lifespan: tick - m.firstSeen });
      }
    }
    return agg;
  }

  function speciesList() {
    const agg = aggregateSpecies();
    const out = [];
    for (const [id, a] of agg) {
      const m = speciesMeta.get(id);
      out.push({
        stable_id: id,
        parent_stable_id: m.parent,
        first_seen_tick: m.firstSeen,
        established: m.established,
        population: a.n,
        peak: Math.max(m.peak, a.n),
        mean_hue: a.hue,
        mean_energy: a.energy / a.n,
        mean_body_size: a.size / a.n,
        mean_connections: a.conn / a.n,
        mean_generation: a.gen / a.n,
        mean_trophic_level: a.troph / a.n,
        herbivore_fraction: a.herb / a.n,
        predator_fraction: a.pred / a.n,
        trend: m.ema > 0 ? (a.n - m.ema) / m.ema : 0,
      });
    }
    out.sort((a, b) => a.stable_id - b.stable_id); // the ABI orders by stable_id
    return out;
  }

  function fillFrame() {
    let est = 0;
    for (const m of speciesMeta.values()) if (m.alive && m.established && (m.pop ?? 1) > 0) est++;
    frameBuf[0] = tick;
    frameBuf[1] = cfg.daynight.enabled ? (1 + Math.sin((TAU * (tick + 1)) / cfg.daynight.period)) / 2 : 1;
    frameBuf[2] = Math.max(0, 1 + cfg.food.seasons.amplitude * Math.sin((TAU * tick) / cfg.food.seasons.period));
    frameBuf[3] = orgs.length;
    frameBuf[4] = droughts.length;
    frameBuf[5] = est;
    frameBuf[6] = counters.stepBirths;
    frameBuf[7] = counters.stepDeaths;
  }

  function configJson() {
    const rays = cfg.creature.vision.ray_count;
    const inputLabels = [];
    for (let i = 0; i < cfg.brain.inputs; i++) inputLabels.push(i < rays * 2 ? `ray${i >> 1}_${i % 2 ? 'type' : 'dist'}` : INPUT_TAIL[i - rays * 2] || `in${i}`);
    return {
      abi_version: 1,
      seed: currentSeed,
      world: { width: W, height: H },
      grid: { cols, rows, cell_size: cs, max_per_cell: cfg.food.max_per_cell },
      hard_cap: cfg.creature.hard_cap,
      initial_count: cfg.creature.initial_count,
      body_radius: cfg.creature.body_radius,
      max_speed: cfg.creature.max_speed,
      max_turn: cfg.creature.max_turn,
      vision_range: cfg.creature.vision.range,
      max_energy: cfg.energy.maximum,
      max_age: cfg.creature.max_age,
      reproduction_threshold: cfg.reproduction.threshold,
      daynight: { enabled: cfg.daynight.enabled, period: cfg.daynight.period },
      seasons: { amplitude: cfg.food.seasons.amplitude, period: cfg.food.seasons.period },
      droughts: { enabled: cfg.food.droughts.spawn_rate > 0, duration: cfg.food.droughts.duration, radius: cfg.food.droughts.radius, severity: cfg.food.droughts.severity, max_active: cfg.food.droughts.max_active },
      pheromones: { enabled: cfg.pheromones.enabled, max_level: cfg.pheromones.max_level },
      niche_construction: { enabled: cfg.niche_construction.enabled, max_level: cfg.niche_construction.max_level },
      elevation: { enabled: cfg.elevation.enabled, min: elevRange[0], max: elevRange[1] },
      brain: { inputs: cfg.brain.inputs, outputs: cfg.brain.outputs, hidden_initial: cfg.brain.hidden, input_labels: inputLabels, output_labels: OUTPUTS.slice(0, cfg.brain.outputs) },
      biomes: cfg.food.biomes,
      enabled: { fruit: cfg.food.fruit.enabled, temperature: cfg.temperature.enabled, pheromones: cfg.pheromones.enabled, daynight: cfg.daynight.enabled, elevation: cfg.elevation.enabled, niche_construction: cfg.niche_construction.enabled, droughts: cfg.food.droughts.spawn_rate > 0 },
      layout: { creature_stride: MOCK_STRIDE, death_stride: MOCK_DEATH_STRIDE, drought_stride: MOCK_DROUGHT_STRIDE, frame_len: 8 },
      config: cfg,
    };
  }

  const api = {
    kind: 'mock',
    abiVersion: 1,
    has: () => true,
    hasOverlay: true,
    init(text, seed) {
      return api.initOverlay(text, seed);
    },
    initOverlay(text, seed) {
      try {
        lastText = text || '';
        build(lastText, seed);
        lastError = '';
        return 0;
      } catch (e) {
        lastError = String(e && e.message);
        return -2;
      }
    },
    reset(seed) {
      build(lastText, seed);
      return 0;
    },
    lastError: () => lastError,
    step(n) {
      deathCount = 0;
      counters.stepBirths = 0;
      counters.stepDeaths = 0;
      for (const o of orgs) o.flags &= ~EVENT_BITS; // event flags OR-accumulate over the batch
      for (let i = 0; i < n; i++) stepOnce();
      updateSpecies();
      fillFrame();
    },
    tick: () => tick,
    frame: () => frameBuf,
    creatureStride: () => MOCK_STRIDE,
    creaturesUpdate() {
      const S = MOCK_STRIDE;
      if (creatureBuf.length < orgs.length * S) {
        creatureBuf = new Float32Array(orgs.length * S * 2);
        idBuf = new Uint32Array(orgs.length * 2);
      }
      const b = creatureBuf;
      const stress = 0.3 * cfg.reproduction.threshold;
      for (let k = 0; k < orgs.length; k++) {
        const o = orgs[k];
        const t = predFrac(o);
        const j = k * S;
        idBuf[k] = o.id;
        b[j] = o.id;
        b[j + 1] = o.x;
        b[j + 2] = o.y;
        b[j + 3] = o.heading;
        b[j + 4] = Math.max(0, o.energy) / cfg.energy.maximum;
        b[j + 5] = o.bodySize;
        b[j + 6] = o.speedNorm;
        b[j + 7] = o.turn;
        b[j + 8] = speciesMeta.get(o.species)?.established ? o.species : -1;
        b[j + 9] = t;
        b[j + 10] = (o.flags & EVENT_BITS) | (o.energy < stress ? STRESSED : 0) | (t > 0.5 ? PREDATOR : 0);
        b[j + 11] = -1;
        b[j + 12] = o.generation;
        b[j + 13] = o.age / cfg.creature.max_age;
        b[j + 14] = o.signal;
        b[j + 15] = o.diet;
        b[j + 16] = o.speed;
        b[j + 17] = o.energy;
        b[j + 18] = o.hue;
        b[j + 19] = o.age;
        b[j + 20] = 0.05;
        b[j + 21] = o.nConn;
        b[j + 22] = o.nHidden;
        b[j + 23] = cfg.elevation.enabled ? heightAt(o.x, o.y) : 0;
      }
      return orgs.length;
    },
    creatures: () => creatureBuf,
    creatureIds: () => idBuf,
    deathsCount: () => deathCount,
    deathStride: () => MOCK_DEATH_STRIDE,
    deaths: () => deathBuf,
    deathIds: () => deathIdBuf,
    gridUpdate: (kind) => (kind >= 0 && kind <= 6 ? cols * rows : 0),
    grid(kind) {
      return [food, fruit, pher, built, elev, temp, rate][kind] || null;
    },
    elevationAt: (x, y) => (cfg.elevation.enabled ? heightAt(x, y) : 0),
    droughtsUpdate() {
      const S = MOCK_DROUGHT_STRIDE;
      if (droughtBuf.length < droughts.length * S) droughtBuf = new Float32Array(droughts.length * S * 2);
      for (let k = 0; k < droughts.length; k++) {
        const d = droughts[k];
        droughtBuf.set([d.x, d.y, d.radius, d.severity, d.remaining, 1 - d.remaining / cfg.food.droughts.duration], k * S);
      }
      return droughts.length;
    },
    droughtStride: () => MOCK_DROUGHT_STRIDE,
    droughts: () => droughtBuf,
    statsJson() {
      const sl = speciesList();
      let conn = 0;
      let hid = 0;
      let troph = 0;
      let pred = 0;
      let herb = 0;
      let energy = 0;
      let maxGen = 0;
      let gen = 0;
      const roots = new Set();
      for (const o of orgs) {
        conn += o.nConn;
        hid += o.nHidden;
        energy += o.energy;
        gen += o.generation;
        maxGen = Math.max(maxGen, o.generation);
        roots.add(o.root);
        const t = predFrac(o);
        troph += 1 + t;
        if (t > 0.5) pred++;
        if (t < 0.25) herb++;
      }
      const n = Math.max(1, orgs.length);
      let coop = 0;
      for (const s of sl) coop += (speciesMeta.get(s.stable_id).coop * s.population) / n;
      return {
        tick,
        stats: {
          tick,
          population: orgs.length,
          births: counters.births,
          deaths: counters.deaths,
          total_births: counters.totalBirths,
          total_deaths: counters.totalDeaths,
          attacks: counters.attacks,
          mean_energy: energy / n,
          mean_brain_neurons: hid / n + cfg.brain.inputs + cfg.brain.outputs,
          species_count: sl.length,
          mean_generation: gen / n,
          shares: counters.shares,
          cooperation_rate: coop,
          kin_share_ratio: 0.93,
          mean_trophic_level: troph / n,
          herbivore_count: herb,
          predator_count: pred,
          mean_brain_entropy: 0.5,
          mean_connections: conn / n,
          active_droughts: droughts.length,
          max_generation: maxGen,
          lineage_diversity: roots.size,
        },
        env: {
          light_level: frameBuf[1],
          is_night: cfg.daynight.enabled && frameBuf[1] < 0.5,
          season_multiplier: frameBuf[2],
        },
        species: sl,
        established_species: sl.filter((s) => s.established).length,
      };
    },
    speciesJson: () => speciesList(),
    configJson,
    eventsJson() {
      const out = { dropped: 0, events };
      events = [];
      return out;
    },
    setEventMask(m) {
      eventMask = m;
    },
    setMinAnnounce(n) {
      minAnnounce = n;
    },
    inspectJson(id) {
      const o = orgs.find((q) => q.id === id);
      if (!o) return null;
      const r = mulberry32(o.id * 2654435761);
      const ni = cfg.brain.inputs;
      const no = cfg.brain.outputs;
      const labels = configJson().brain.input_labels;
      const nodes = [{ id: 0, kind: 'bias', label: 'bias', depth: 0, layer: 0 }];
      for (let i = 0; i < ni; i++) nodes.push({ id: 1 + i, kind: 'in', label: labels[i], depth: 0, layer: 0 });
      for (let k = 0; k < no; k++) nodes.push({ id: 1 + ni + k, kind: 'out', label: OUTPUTS[k] || `out${k}`, depth: 1, layer: 2 });
      const hidden = [];
      for (let h = 0; h < o.nHidden; h++) {
        const hid = 1 + ni + no + h;
        hidden.push(hid);
        nodes.push({ id: hid, kind: 'hidden', label: `h${hid}`, depth: 0.3 + r() * 0.4, layer: 1 });
      }
      const edges = [];
      for (let c = 0; c < o.nConn; c++) {
        const toHidden = r() < 0.5 && hidden.length;
        const from = toHidden || r() < 0.5 ? Math.floor(r() * (ni + 1)) : hidden[Math.floor(r() * hidden.length)] ?? 1;
        const to = toHidden ? hidden[Math.floor(r() * hidden.length)] : 1 + ni + Math.floor(r() * no);
        const w = (r() * 2 - 1) * 2;
        const meth = r() < 0.15 ? r() * 0.6 : 0;
        edges.push({ from, to, weight: w, enabled: r() > 0.05, methylation: meth, effective_weight: w * (1 - meth) });
      }
      const t = predFrac(o);
      const hist = OUTPUTS.slice(0, no).map((name, k) => {
        const b = [];
        const centre = Math.sin(o.id * 0.7 + k + tick * 0.01);
        for (let i = 0; i < 8; i++) b.push(Math.exp(-(((-1 + (2 * (i + 0.5)) / 8 - centre) * 2) ** 2)));
        return { name, bins: b, entropy: 0.4 + 0.4 * r() };
      });
      const children = orgs.filter((q) => q.parentId === o.id).length;
      return {
        id: o.id,
        parent_id: o.parentId,
        root_ancestor_id: o.root,
        generation: o.generation,
        age: o.age,
        max_age: cfg.creature.max_age,
        birth_tick: o.birthTick,
        species: speciesMeta.get(o.species)?.established ? o.species : null,
        children_alive: children,
        x: o.x,
        y: o.y,
        heading: o.heading,
        signal: o.signal,
        speed: o.speed,
        speed_norm: o.speedNorm,
        turn: o.turn,
        flags: o.flags,
        trophic_level: 1 + t,
        body_size: o.bodySize,
        color_hue: o.hue,
        aging_rate: 0.002 * (0.8 + r() * 0.4),
        preferred_temp: o.prefTemp,
        diet_preference: o.diet,
        mutation_rate: 0.1,
        mutation_step: 0.2,
        energy: {
          current: o.energy,
          maximum: cfg.energy.maximum,
          lifetime_from_food: o.fromFood,
          lifetime_from_attacks: o.fromAttacks,
          predation_fraction: t,
          cost_size_metabolism: cfg.energy.base_metabolism * Math.pow(o.bodySize, 0.75),
          cost_brain_static: cfg.energy.brain_cost * (o.nHidden + ni + no) + cfg.energy.connection_cost * o.nConn,
          cost_temperature: cfg.energy.base_metabolism * cfg.temperature.cost_factor * Math.abs(o.y / H - o.prefTemp),
          cost_movement_last_tick: cfg.energy.movement_cost * o.speedNorm ** 2,
          untracked: ['brain_activity', 'crowding', 'attack_cost', 'shares', 'construction', 'reproduction'],
        },
        brain: {
          num_inputs: ni,
          num_outputs: no,
          num_hidden: o.nHidden,
          num_layers: 3,
          enabled_connections: edges.filter((c) => c.enabled).length,
          total_connections: edges.length,
          mean_methylation: 0.05,
          entropy: 0.5,
          nodes,
          edges,
          output_histograms: hist,
        },
      };
    },
  };
  return api;
}
