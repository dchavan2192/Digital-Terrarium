// Terrarium web demo — entry point and frame loop.
// Spec: docs/research/DEMO_UX_DESIGN.md · ABI: docs/research/SIM_WASM_ABI.md
//
// URL params: ?seed=1234 (shareable run) · ?mock=1 (force the JS mock sim) ·
//             ?debug=1 (fps / draw-call overlay) · ?scale=1.5 (creature size) ·
//             ?tour=1 (force the guided tour)

import { loadSim, F, L, DTH, DL, DEATH_CAUSES } from './sim.js';
import { createSpeciesColors, slotHex, cssGradient } from './palette.js';
import { createEventEngine, createRateLimiter } from './events.js';
import { fmtInt, dayOf, seasonName, speciesName, TIER_NAMES, trophicTier, fmtFixed, KPI_DEFS } from './format.js';
import { activityWord } from './describe.js';
import { buildLayers } from './layers.js';
import { createWorldMap } from './scene/world.js';
import { createStage } from './scene/stage.js';
import { createTerrain } from './scene/terrain.js';
import { createCreatures, VIS } from './scene/creatures.js';
import { createCameras } from './scene/cameras.js';
import { createLabels } from './scene/labels.js';
import { createKpis, createTooltip, SAMPLE_EVERY_TICKS } from './ui/kpis.js';
import { createTrendPanel } from './ui/trend.js';
import { createRunHistory } from './history.js';
import { createFeed } from './ui/feed.js';
import { createSpeciesTab } from './ui/species.js';
import { createInspector } from './ui/inspector.js';
import { createTour, tourSeen } from './ui/tour.js';
import { renderEnergy } from './ui/energy.js';
import { icon, podSilhouette } from './ui/icons.js';
import { memoryReadout, connectionBreakdown } from './memory.js';
import { demoModesOverlay, renderEcology } from './ecology-metrics.js';

const LOG = '[terrarium]';
const TICKS_PER_SEC = { 1: 15, 4: 60, 16: 240 }; // §3.7
const STEP_BUDGET_MS = 8; // §3.7: ≤ 8 ms of sim per frame, remainder carried over
const BIOME_LABEL_MS = 30000; // §3.2: contours on for 30 s, then 30% opacity

const $ = (s) => document.querySelector(s);
const params = new URLSearchParams(location.search);
const mqPhone = matchMedia('(max-width: 699px)');
const mqTablet = matchMedia('(min-width: 700px) and (max-width: 1099px)');
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches;
const debug = params.has('debug');
if (params.has('scale')) VIS.scale = Math.max(0.5, Math.min(4, Number(params.get('scale')) || 1));

function parseSeed(v) {
  const n = Number.parseInt(v, 10);
  return Number.isFinite(n) && n >= 0 ? n >>> 0 : null;
}
function randomSeed() {
  return Math.floor(Math.random() * 100000);
}

// --------------------------------------------------------------------------
// state
// --------------------------------------------------------------------------
const S = {
  sim: null,
  cfg: null,
  map: null,
  stage: null,
  terrain: null,
  creatures: null,
  cameras: null,
  labels: null,
  colors: createSpeciesColors(),
  engine: null,
  limiter: createRateLimiter(),
  layers: [],
  layer: null,
  paused: false,
  speed: 1,
  acc: 0,
  msPerTick: 0.5,
  lastStepAt: 0,
  stepInterval: 66,
  lastSampleTick: -Infinity,
  lastSpeciesTabAt: 0,
  lastClockAt: 0,
  lastInspectAt: 0,
  lastLabelsAt: 0,
  frame: 0,
  bootAt: 0,
  lastStats: null,
  baseConfigText: '',
  memoryEnabled: false,
  shadowEnabled: false,
  follow: { hold: 0, pose: { x: 0, y: 0, z: 0, heading: 0, simX: 0, simY: 0 } },
  deadId: -1,
  w: 1,
  h: 1,
  biomeLabels: [],
  droughtLabels: [],
  edgeLabel: null,
};
const pose = S.follow.pose;

function toast(text, ms = 3200) {
  const el = $('#toast');
  el.textContent = text;
  el.hidden = false;
  clearTimeout(toast._t);
  toast._t = setTimeout(() => {
    el.hidden = true;
  }, ms);
}

function fillIcons(root = document) {
  for (const el of root.querySelectorAll('[data-icon]')) el.innerHTML = icon(el.dataset.icon);
}

// --------------------------------------------------------------------------
// boot
// --------------------------------------------------------------------------
// Show which commit this demo was built from. The deploy workflow writes
// version.json; it is absent in local development, where the tag stays hidden.
async function showBuildTag() {
  try {
    const res = await fetch('version.json', { cache: 'no-store' });
    if (!res.ok) return;
    const v = await res.json();
    const tag = $('#buildTag');
    if (!tag || !v.sha) return;
    const when = v.date ? new Date(v.date).toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' }) : '';
    tag.textContent = `build ${String(v.sha).slice(0, 7)}${when ? ` · ${when}` : ''}`;
    tag.href = `https://github.com/dchavan2192/Digital-Terrarium/commit/${v.sha}`;
    tag.hidden = false;
  } catch {
    // No version file: nothing to show.
  }
}

async function boot() {
  fillIcons();
  showBuildTag();
  S.bootAt = performance.now();
  const seed = parseSeed(params.get('seed')) ?? randomSeed();
  const { sim, error, configText } = await loadSim({ wasmUrl: 'terrarium.wasm', configUrl: 'demo.toml', seed, forceMock: params.has('mock') });
  S.sim = sim;
  S.baseConfigText = configText;
  S.memoryEnabled = params.get('memory') === '1' && sim.caps.memory;
  S.shadowEnabled = sim.config.shadowEnabled === true || (params.get('shadow') === '1' && sim.caps.shadow);
  if (S.memoryEnabled || (S.shadowEnabled && sim.config.shadowEnabled !== true)) {
    sim.init(demoModesOverlay(S.baseConfigText, S), seed);
  }
  S.cfg = sim.config;
  if (sim.kind === 'mock') {
    $('#mockBadge').hidden = false;
    if (!params.has('mock')) console.warn(`${LOG} running on MOCK data because the WASM failed:`, error);
  }
  try {
    setupWorld();
  } catch (e) {
    console.error(`${LOG} WebGL / scene setup failed`, e);
    $('#loadingText').innerHTML = 'This browser could not start WebGL, so the 3D view is unavailable. Try the <a href="../classic.html">classic viewer</a>.';
    return;
  }
  setupUi();
  startRun(seed, true);
  $('#loading').classList.add('is-done');
  setTimeout(() => ($('#loading').hidden = true), 600);
  requestAnimationFrame(loop);
  if (params.has('tour') || !tourSeen()) setTimeout(() => S.tour.start(), 1200);
}

function setupWorld() {
  const { sim, cfg } = S;
  const phone = mqPhone.matches;
  S.map = createWorldMap(cfg, sim.grid('elevation'));
  S.stage = createStage($('#viewport'), { phone, reducedMotion });
  S.terrain = createTerrain(S.stage.scene, S.map, cfg);
  S.creatures = createCreatures(S.stage.scene, S.map, cfg, { L, F }, S.colors);
  S.cameras = createCameras(S.stage.renderer.domElement, { reducedMotion });
  S.labels = createLabels($('#labels'));
  S.creatures.onSelectedDied = onSelectedDied;
  S.stage.onQuality = (q) => console.info(`${LOG} adaptive quality: dpr ${q.dpr}, shadows ${q.shadows ? 'on' : 'off'} (fps ${q.fps.toFixed(0)})`);
  // Biome labels (north edge of each zone).
  for (const b of cfg.biomes) {
    const name = b.multiplier >= 2.5 ? 'Lush oasis' : b.multiplier > 1.05 ? 'Fertile zone' : b.multiplier < 0.95 ? 'Barren' : 'Zone';
    const lab = S.labels.add(`${name} ×${fmtFixed(b.multiplier, b.multiplier % 1 ? 1 : 0)}${b.multiplier > 1 ? ' regrowth' : ''}`, 'lbl-biome');
    const y = Math.max(0, b.y - b.radius);
    lab.x = S.map.sceneX(b.x);
    lab.z = S.map.sceneZ(y);
    lab.y = S.map.heightAt(b.x, y) + 0.05;
    lab.visible = true;
    S.biomeLabels.push(lab);
  }
  for (let k = 0; k < S.terrain.rings.length; k++) S.droughtLabels.push(S.labels.add('', 'lbl-drought'));
  S.edgeLabel = S.labels.add('world edge · wraps around', 'lbl-edge');
  S.edgeLabel.x = 0;
  S.edgeLabel.z = -5;
  S.edgeLabel.y = 0.05;
  resize();
  new ResizeObserver(resize).observe($('#viewport'));
  if (phone) setView('map', true);
}

function resize() {
  if (!S.stage) return;
  const { w, h } = S.stage.size();
  S.w = w;
  S.h = h;
  S.cameras.resize(w, h);
  S.creatures.setResolution(w, h, S.stage.dpr);
}

// --------------------------------------------------------------------------
// run lifecycle
// --------------------------------------------------------------------------
function startRun(seed, first = false) {
  const { sim } = S;
  if (!first) {
    try {
      sim.reset(seed);
    } catch (e) {
      console.error(`${LOG} reset failed`, e);
      toast('Could not restart the simulation (see console).');
      return;
    }
  }
  S.cfg = sim.config;
  S.creatures.reset();
  S.colors.reset();
  S.kpis.reset();
  S.history.reset();
  S.trend.update();
  S.feed.clear();
  S.feed.setDayPeriod(S.cfg.dayPeriod);
  S.limiter.clear();
  S.engine = createEventEngine(S.cfg);
  S.speciesTab.clearHighlight();
  S.inspector.hide();
  S.acc = 0;
  S.lastSampleTick = -Infinity;
  S.lastStats = null;
  S.deadId = -1;
  S.follow.hold = 0;
  if (S.cameras.mode === 'follow') setView(mqPhone.matches ? 'map' : 'orbit');
  $('#heroCap').textContent = fmtInt(S.cfg.hardCap);
  $('#seedInput').value = String(seed);
  const url = new URL(location.href);
  url.searchParams.set('seed', String(seed));
  if (S.memoryEnabled) url.searchParams.set('memory', '1');
  else url.searchParams.delete('memory');
  if (S.shadowEnabled) url.searchParams.set('shadow', '1');
  else url.searchParams.delete('shadow');
  history.replaceState(null, '', url);
  $('#memoryToggle').textContent = S.memoryEnabled ? 'Memory on' : 'Memory off';
  $('#memoryToggle').setAttribute('aria-pressed', String(S.memoryEnabled));
  $('#memoryToggle').disabled = !sim.caps.memory;
  $('#memoryToggle').title = sim.caps.memory
    ? 'Experimental recurrent memory. Switching starts a new world with the same seed.'
    : 'Memory mode needs the current real WASM simulation; it is unavailable with mock data.';
  $('#memoryPanel').hidden = !S.memoryEnabled;
  $('#shadowToggle').textContent = S.sim.caps.shadow ? `Neutral comparison ${S.shadowEnabled ? 'on' : 'off'}` : 'Neutral comparison unavailable';
  $('#shadowToggle').setAttribute('aria-pressed', String(S.shadowEnabled));
  $('#shadowToggle').disabled = !S.sim.caps.shadow;
  $('#memoryInspectBtn').disabled = true;
  const recurrenceRate = S.cfg.raw?.config?.mutation?.add_recurrent_probability;
  $('#memoryMutation').textContent = Number.isFinite(recurrenceRate) && recurrenceRate >= 0 && recurrenceRate <= 1
    ? `${new Intl.NumberFormat('en-US', { style: 'percent', maximumFractionDigits: 1 }).format(recurrenceRate)} chance per birth to try adding a recurrent link.`
    : 'Demo mutation setting unavailable.';
  // Opening line, shown immediately (§1.1 item 5).
  S.feed.add(S.engine.opening(S.cfg.initialCount));
  S.limiter.force(performance.now());
  sample(performance.now());
  S.terrain.syncDroughts(S.sim.droughts(), DL, performance.now(), false);
  applyLayer(S.layer || S.layers[0], true);
  console.info(`${LOG} run started: seed ${seed}`);
}

// --------------------------------------------------------------------------
// UI wiring
// --------------------------------------------------------------------------
function setupUi() {
  const tip = createTooltip($('#kpiTip'));
  S.tip = tip;
  // Clicking a headline tile expands it into a chart over time; clicking it again,
  // the close button or Esc collapses it. The 3D view resizes to the space left.
  S.history = createRunHistory(KPI_DEFS.length);
  S.historySample = new Float64Array(KPI_DEFS.length);
  const toggleTrend = (key) => {
    if (!key || S.trend.openKey === key) S.trend.close();
    else S.trend.open(key);
    S.kpis.setExpanded(S.trend.openKey);
  };
  S.toggleTrend = toggleTrend;
  S.kpis = createKpis($('#kpis'), tip, { onSelect: toggleTrend });
  S.trend = createTrendPanel($('#trend'), {
    defs: KPI_DEFS,
    history: S.history,
    sampleTicks: SAMPLE_EVERY_TICKS,
    getStats: () => S.lastStats,
    getCfg: () => S.cfg,
    onClose: () => toggleTrend(null),
  });
  S.feed = createFeed($('#feed'), $('#phoneLatest'), {
    colors: S.colors,
    onLocate: (x, y) => {
      if (S.cameras.mode === 'follow') return;
      S.cameras.flyTo(S.map.sceneX(x), S.map.sceneZ(y));
    },
    onSpecies: (id) => toggleSpeciesHighlight(id),
  });
  S.speciesTab = createSpeciesTab($('#speciesBody'), {
    colors: S.colors,
    onToggle: (id) => S.creatures.highlight(id),
  });
  S.inspector = createInspector($('#inspector'), {
    compact: mqPhone.matches,
    onClose: deselect,
    onBack: () => {
      deselect();
      setView(mqPhone.matches ? 'map' : 'orbit');
    },
    onFollow: () => setView('follow'),
    onJump: (id) => select(id),
  });
  S.tour = createTour($('#tour'), { onOpenSide: () => mqPhone.matches || openSheet(false) });

  // layers
  S.layers = buildLayers(S.sim.caps, S.cfg).filter((l) => l.available);
  const list = $('#layerList');
  list.innerHTML = S.layers
    .map((l, i) => `<label class="layer-opt"><input type="radio" name="layer" value="${l.id}" ${i === 0 ? 'checked' : ''}><span>${l.label}</span></label>`)
    .join('');
  list.addEventListener('change', (e) => applyLayer(S.layers.find((l) => l.id === e.target.value)));
  $('#layersFab').addEventListener('click', () => {
    const open = $('#layers').classList.toggle('is-open');
    $('#layersFab').setAttribute('aria-expanded', String(open));
  });

  // views
  for (const b of document.querySelectorAll('.view-btn')) b.addEventListener('click', () => setView(b.dataset.view));

  // time bar
  $('#playBtn').addEventListener('click', togglePause);
  for (const b of document.querySelectorAll('.speed')) b.addEventListener('click', () => setSpeed(Number(b.dataset.speed)));
  $('#newSeedBtn').addEventListener('click', () => startRun(randomSeed()));
  $('#memoryToggle').addEventListener('click', () => {
    if (!S.sim.caps.memory) return;
    const enabled = !S.memoryEnabled;
    const seed = S.sim.seed;
    try {
      S.sim.init(demoModesOverlay(S.baseConfigText, { memoryEnabled: enabled, shadowEnabled: S.shadowEnabled }), seed);
      S.memoryEnabled = enabled;
      startRun(seed, true);
      toast(enabled ? 'Experimental memory enabled. New world, same seed; recurrent links evolve in offspring.' : 'Memory disabled. New world, same seed.', 6000);
      if (enabled && (mqPhone.matches || mqTablet.matches)) openSheet(true);
    } catch (error) {
      console.error(`${LOG} memory mode restart failed`, error);
      toast('Could not change memory mode. Reload the page to restart safely.');
    }
  });
  $('#shadowToggle').addEventListener('click', () => {
    if (!S.sim.caps.shadow) return;
    const enabled = !S.shadowEnabled;
    const seed = S.sim.seed;
    try {
      S.sim.init(demoModesOverlay(S.baseConfigText, { memoryEnabled: S.memoryEnabled, shadowEnabled: enabled }), seed);
      S.shadowEnabled = enabled;
      startRun(seed, true);
      toast(`Neutral comparison ${enabled ? 'enabled' : 'disabled'}. New world, same seed; memory mode preserved.`, 5000);
    } catch (error) {
      console.error(`${LOG} neutral comparison restart failed`, error);
      toast('Could not change the neutral comparison. Reload to restart safely.');
    }
  });
  $('#memoryInspectBtn').addEventListener('click', () => {
    const id = S.sim.memoryCreatureId();
    if (id >= 0) select(id);
    else toast('No living creature with recurrent links yet. Let a few generations pass.');
  });
  $('#seedInput').addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      const s = parseSeed(e.target.value);
      if (s === null) toast('A seed is a whole number, e.g. 1234.');
      else startRun(s);
      e.target.blur();
    }
  });
  $('#copyBtn').addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(location.href);
      toast('Link copied. Same seed, same story.');
    } catch {
      toast(location.href, 6000);
    }
  });
  const ring = $('#dialRing');
  let segs = '';
  for (let i = 0; i < 24; i++) {
    const a = (i / 24) * Math.PI * 2 - Math.PI / 2;
    segs += `<line x1="${(12 + Math.cos(a) * 8.5).toFixed(2)}" y1="${(12 + Math.sin(a) * 8.5).toFixed(2)}" x2="${(12 + Math.cos(a) * 11).toFixed(2)}" y2="${(12 + Math.sin(a) * 11).toFixed(2)}" />`;
  }
  ring.innerHTML = segs;
  S.dialSegs = ring.children;

  // tabs
  const tabs = [$('#tabFeed'), $('#tabSpecies'), $('#tabEnergy'), $('#tabEcology')];
  const panels = [$('#feedPanel'), $('#speciesPanel'), $('#energyPanel'), $('#ecologyPanel')];
  tabs.forEach((t, i) =>
    t.addEventListener('click', () => {
      tabs.forEach((x, j) => {
        x.setAttribute('aria-selected', String(i === j));
        x.tabIndex = i === j ? 0 : -1;
        panels[j].hidden = i !== j;
      });
      if (i === 1 && S.lastStats) S.speciesTab.update(S.lastStats.species, S.lastStats.tick);
      if (i === 2) renderEnergy($('#energyBudget'), S.lastStats);
      if (i === 3) renderEcology($('#ecologyPanel'), S.lastStats, S.cfg, S.sim.kind);
      if (mqTablet.matches || mqPhone.matches) openSheet(true);
    }),
  );
  tabs.forEach((t, i) => t.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
      const n = tabs[(i + (e.key === 'ArrowRight' ? 1 : tabs.length - 1)) % tabs.length];
      n.focus();
      n.click();
    }
  }));

  // header
  $('#tourBtn').addEventListener('click', () => S.tour.start());
  $('#menuBtn').addEventListener('click', () => openSheet(!$('#app').classList.contains('sheet-open')));
  $('#phoneLatestBtn').addEventListener('click', () => openSheet(true));
  $('#sheetClose').addEventListener('click', () => openSheet(false));
  setupSheetDrag();
  $('#sheetHandle').addEventListener('click', () => openSheet(!$('#app').classList.contains('sheet-open')));

  // legend
  renderLegend();

  // picking + hover
  setupPointer();

  // keyboard (§8.5)
  window.addEventListener('keydown', onKey);
  document.addEventListener('visibilitychange', () => {
    S.acc = 0; // do not try to catch up after a hidden tab
  });
  if (debug) $('#debug').hidden = false;
}

function openSheet(open) {
  const app = $('#app');
  app.classList.toggle('sheet-open', open);
  $('#menuBtn').setAttribute('aria-expanded', String(open));
  $('#phoneLatestBtn').setAttribute('aria-expanded', String(open));
}

function setupSheetDrag() {
  const handle = $('#sheetHandle');
  let startY = 0;
  let startH = 0;
  handle.addEventListener('pointerdown', (e) => {
    if (!mqTablet.matches) return;
    startY = e.clientY;
    startH = $('#side').getBoundingClientRect().height;
    handle.setPointerCapture(e.pointerId);
  });
  handle.addEventListener('pointermove', (e) => {
    if (!handle.hasPointerCapture(e.pointerId)) return;
    const h = Math.max(120, Math.min(innerHeight * 0.85, startH + (startY - e.clientY)));
    document.documentElement.style.setProperty('--sheet-h', `${Math.round(h)}px`);
  });
}

function renderLegend() {
  const l = S.layer || S.layers[0];
  const el = $('#legend');
  const trophic = [0, 1, 2]
    .map((t) => `<span class="troph" title="${TIER_NAMES[t]}">${podSilhouette(t, 26)}<span>${TIER_NAMES[t]}</span></span>`)
    .join('');
  el.innerHTML = `
    <div class="legend-head">
      <span class="legend-title">${l.label}</span>
      <button type="button" class="legend-q" aria-expanded="false" aria-controls="legendMore" aria-label="What does this layer show?">${icon('help', 16)}</button>
    </div>
    <div class="legend-bar" style="background:${cssGradient(l.stops)}"></div>
    <div class="legend-ends"><span>${l.lo}</span><span>${l.hi}</span></div>
    ${l.fruit ? '<p class="legend-line"><i class="dot berry"></i> fruit (grows only in oases)</p>' : ''}
    <p class="legend-line"><i class="ring-ochre"></i> drought: food regrows at ${fmtInt(100 * (S.sim.config.raw?.droughts?.severity ?? 0.05))}%</p>
    <p class="legend-line"><i class="ring-dots"></i> biome zone (regrowth ×)</p>
    <div class="legend-more" id="legendMore" hidden>
      <p>${l.text}</p>
      <p class="tip-cite">${l.cite}</p>
      <p class="units mono">${l.units}</p>
    </div>
    <div class="legend-troph" title="Based on where its lifetime energy came from: plants vs. other creatures (Lindeman 1942 trophic levels).">
      <span class="legend-sub">Who eats whom</span><span class="troph-row">${trophic}</span>
    </div>
    <p class="legend-read mono tnum" id="cellRead" aria-hidden="true"></p>`;
  const q = el.querySelector('.legend-q');
  q.addEventListener('click', () => {
    const more = el.querySelector('.legend-more');
    more.hidden = !more.hidden;
    q.setAttribute('aria-expanded', String(!more.hidden));
  });
}

function applyLayer(layer, quiet = false) {
  if (!layer) return;
  S.layer = layer;
  const r = document.querySelector(`input[name=layer][value="${layer.id}"]`);
  if (r) r.checked = true;
  S.terrain.setFruitVisible(layer.id === 'food');
  renderLegend();
  recolor(performance.now());
  if (!quiet) toast(`Layer: ${layer.label}`, 1400);
}

const layerParams = { lut: null, values: null, scale: 1, offset: 0, droughts: false }; // reused every 4th frame
function recolor(now) {
  const l = S.layer;
  if (!l) return;
  layerParams.lut = l.lut;
  layerParams.values = S.sim.grid(l.grid);
  layerParams.scale = l.scale;
  layerParams.offset = l.offset;
  layerParams.droughts = l.id === 'food';
  S.terrain.updateColors(layerParams, now);
}

function setView(mode, instant = false) {
  if (mode === 'follow' && S.creatures.selectedId < 0) return;
  S.cameras.setMode(mode);
  if (instant) S.cameras.update(0, null);
  for (const b of document.querySelectorAll('.view-btn')) b.setAttribute('aria-pressed', String(b.dataset.view === mode));
  S.terrain.edge.visible = mode === 'map';
  S.edgeLabel.visible = mode === 'map';
}

function setSpeed(n) {
  S.speed = n;
  for (const b of document.querySelectorAll('.speed')) b.setAttribute('aria-pressed', String(Number(b.dataset.speed) === n));
  S.acc = Math.min(S.acc, 1);
}

function togglePause() {
  S.paused = !S.paused;
  const b = $('#playBtn');
  b.innerHTML = icon(S.paused ? 'play' : 'pause');
  b.setAttribute('aria-label', S.paused ? 'Play (Space)' : 'Pause (Space)');
}

function toggleSpeciesHighlight(id) {
  const next = S.creatures.highlighted === id ? -2 : id;
  S.creatures.highlight(next);
  S.speciesTab.set(next);
  if (next >= 0) toast(`Highlighting ${speciesName(id)}. Click the event again, or press Esc, to clear.`, 2500);
}

function onKey(e) {
  const tag = (e.target && e.target.tagName) || '';
  // Focused controls keep their native keys (notably Space on mode buttons).
  if (['INPUT', 'TEXTAREA', 'SELECT', 'BUTTON', 'A'].includes(tag) || S.tour.active) return;
  if (e.metaKey || e.ctrlKey || e.altKey) return;
  switch (e.key) {
    case ' ':
      e.preventDefault();
      togglePause();
      break;
    case '1':
      setSpeed(1);
      break;
    case '2':
      setSpeed(4);
      break;
    case '3':
      setSpeed(16);
      break;
    case 'm':
    case 'M':
      setView(S.cameras.mode === 'map' ? 'orbit' : 'map');
      break;
    case 'f':
    case 'F':
      if (S.creatures.selectedId >= 0) setView(S.cameras.mode === 'follow' ? 'orbit' : 'follow');
      break;
    case 'l':
    case 'L': {
      const i = S.layers.indexOf(S.layer);
      applyLayer(S.layers[(i + 1) % S.layers.length]);
      break;
    }
    case 'Escape':
      if (S.trend.openKey) S.toggleTrend(null);
      else if (S.inspector.open) $('#inspBack').click();
      else if ($('#app').classList.contains('sheet-open')) openSheet(false);
      else if (S.creatures.highlighted > -2) toggleSpeciesHighlight(S.creatures.highlighted);
      S.tip.hide();
      break;
    case '?':
      S.tour.start();
      break;
    default:
  }
}

// --------------------------------------------------------------------------
// selection
// --------------------------------------------------------------------------
function select(id) {
  S.creatures.select(id);
  S.deadId = -1;
  S.inspector.show(id);
  S.lastInspectAt = 0;
  $('.view-btn[data-view=follow]').disabled = false;
  if (!mqPhone.matches && !mqTablet.matches) openSheet(false);
  else openSheet(true);
  S.cameras.stopAutoRotate();
}

function deselect() {
  S.creatures.select(-1);
  S.inspector.hide();
  $('.view-btn[data-view=follow]').disabled = true;
  if (S.cameras.mode === 'follow') setView(mqPhone.matches ? 'map' : 'orbit');
}

function onSelectedDied(id, cause, x, y) {
  S.deadId = id;
  S.inspector.died(cause, NaN);
  toast(`Creature #${fmtInt(id)} ${DEATH_CAUSES[cause] ?? 'died'}.`);
  S.creatures.select(-1);
  $('.view-btn[data-view=follow]').disabled = true;
  if (S.cameras.mode === 'follow') {
    S.follow.hold = performance.now() + 1500; // hold on the death spot, then back to overview
    pose.simX = x;
    pose.simY = y;
    pose.x = S.map.sceneX(x);
    pose.z = S.map.sceneZ(y);
    pose.y = S.map.heightAt(x, y);
  }
}

function setupPointer() {
  const cvs = S.stage.renderer.domElement;
  const hoverTip = $('#hoverTip');
  const hit = { x: 0, y: 0 };
  let down = null;
  let hoverId = -1;
  let hoverSince = 0;
  let lastMove = 0;

  function pickRadius() {
    // 15 sim units (§3.6), widened when zoomed out so ~14 px always works.
    const cam = S.cameras.camera;
    let simPerPx;
    if (cam.isOrthographicCamera) simPerPx = ((cam.top - cam.bottom) / cam.zoom / S.h) * (S.map.W / 10);
    else simPerPx = ((cam.position.length() * 2 * Math.tan((cam.fov * Math.PI) / 360)) / S.h) * (S.map.W / 10);
    return Math.max(15, 14 * simPerPx);
  }

  cvs.addEventListener('pointerdown', (e) => {
    down = { x: e.clientX, y: e.clientY, t: performance.now() };
  });
  cvs.addEventListener('pointerup', (e) => {
    if (!down) return;
    const moved = Math.hypot(e.clientX - down.x, e.clientY - down.y);
    const quick = performance.now() - down.t < 600;
    down = null;
    if (moved > 6 || !quick) return;
    const p = S.stage.screenToSim(e.clientX, e.clientY, S.cameras.camera, S.map, hit);
    const id = p ? S.creatures.pick(p.x, p.y, pickRadius()) : -1;
    if (id >= 0) select(id);
    else if (S.inspector.open && S.cameras.mode !== 'follow') deselect();
  });
  cvs.addEventListener('pointermove', (e) => {
    if (e.pointerType !== 'mouse') return;
    const now = performance.now();
    if (now - lastMove < 60) return;
    lastMove = now;
    const p = S.stage.screenToSim(e.clientX, e.clientY, S.cameras.camera, S.map, hit);
    const read = $('#cellRead');
    if (!p) {
      hoverTip.hidden = true;
      if (read) read.textContent = '';
      return;
    }
    // terrain readout (§5): "Food 0.62 · Temp mild · Elev 0.31"
    if (read) {
      const c = Math.min(S.cfg.cols - 1, Math.floor(p.x / S.cfg.cellSize));
      const r = Math.min(S.cfg.rows - 1, Math.floor(p.y / S.cfg.cellSize));
      const i = r * S.cfg.cols + c;
      const food = S.sim.grid('food');
      const elev = S.sim.grid('elevation');
      const t = (r + 0.5) / S.cfg.rows;
      read.textContent = `Food ${food ? fmtFixed(food[i], 2) : '–'} · Temp ${t < 0.33 ? 'cold' : t > 0.66 ? 'warm' : 'mild'}${elev ? ` · Elev ${fmtFixed(elev[i], 2)}` : ''}`;
    }
    const id = S.creatures.pick(p.x, p.y, pickRadius());
    if (id !== hoverId) {
      hoverId = id;
      hoverSince = now;
      hoverTip.hidden = true;
    }
    if (id >= 0) {
      cvs.style.cursor = 'pointer';
      setTimeout(() => {
        if (hoverId !== id || performance.now() - hoverSince < 240) return;
        const cr = S.sim.creatures();
        const i = S.creatures.indexOf(id);
        if (i < 0) return;
        const o = i * cr.stride;
        const sp = cr.data[o + L.SPECIES] | 0;
        const tier = trophicTier(cr.data[o + L.PREDATION]);
        const act = activityWord(cr.data[o + L.FLAGS] | 0, { ATTACK: F.ATTACKED, REPRO: F.REPRODUCED, EAT: F.ATE }, cr.data[o + L.SPEED_NORM]);
        hoverTip.innerHTML = `${sp >= 0 ? `${speciesName(sp)} <i class="sw" style="--c:${slotHex(S.colors.slot(sp))}"></i>` : 'Unclassified'} · Gen ${fmtInt(cr.data[o + L.GEN])} · ${act} <span class="muted">(${TIER_NAMES[tier]})</span>`;
        const vr = $('#viewport').getBoundingClientRect();
        hoverTip.style.transform = `translate(${Math.round(e.clientX - vr.left + 14)}px, ${Math.round(e.clientY - vr.top + 14)}px)`;
        hoverTip.hidden = false;
      }, 250);
    } else {
      cvs.style.cursor = '';
    }
  });
  cvs.addEventListener('pointerleave', () => {
    hoverTip.hidden = true;
    hoverId = -1;
  });
}

// --------------------------------------------------------------------------
// sampling (stats -> KPIs, feed, species) — allocates, so only every ~20 ticks
// --------------------------------------------------------------------------
function sample(now) {
  const s = S.sim.stats();
  if (!s) return;
  S.lastStats = s;
  if (!$('#ecologyPanel').hidden) renderEcology($('#ecologyPanel'), s, S.cfg, S.sim.kind);
  if (S.memoryEnabled) {
    const memory = s.tick > 0 ? memoryReadout(s) : null;
    const links = s.tick > 0 && s.population > 0 ? connectionBreakdown(s.mean_connections, s.mean_recurrent_connections) : null;
    $('#memoryCount').textContent = memory
      ? `${fmtInt(memory.count)} / ${fmtInt(memory.population)} creatures have recurrent links (${memory.percent}%).`
      : 'Waiting for a measured tick…';
    $('#memoryInspectBtn').disabled = !memory || memory.count === 0;
    $('#memoryForward').textContent = links ? fmtFixed(links.feedForward, 1) : '–';
    $('#memoryRecurrent').textContent = links ? fmtFixed(links.recurrent, 1) : '–';
    $('#memoryTotal').textContent = links ? fmtFixed(links.total, 1) : '–';
  }
  if (!$('#energyPanel').hidden) renderEnergy($('#energyBudget'), s);
  S.lastSampleTick = s.tick;
  S.colors.update(s.species.filter((x) => x.established), now);
  S.kpis.push(s, S.cfg);
  // Tick 0 has no measured statistics yet (they are all zero), so history starts
  // with the first real tick.
  if (s.tick > 0) {
    for (let k = 0; k < KPI_DEFS.length; k++) S.historySample[k] = KPI_DEFS[k].get(s);
    S.history.push(s.tick, S.historySample, s.active_droughts);
    S.trend.update();
  }
  S.kpis.setSwatches($('#chipSwatches'), s.species, S.colors);
  const dt = S.sim.droughts();
  const droughts = [];
  for (let j = 0; j < dt.count; j++) {
    const o = j * dt.stride;
    droughts.push({ x: dt.data[o + DL.X], y: dt.data[o + DL.Y], radius: dt.data[o + DL.R], severity: dt.data[o + DL.SEVERITY], remaining: dt.data[o + DL.REMAINING] });
  }
  for (const ev of S.engine.ingest({ ...s, droughts })) S.limiter.push(ev);
  for (const ev of S.sim.events()) {
    if (ev.type === 'extinction') {
      S.limiter.push({ kind: 'extinction', priority: 'high', tick: ev.tick, icon: 'cut', text: 'Every creature has died. Life did not make it this time: press New for another world.' });
    }
  }
  if (now - S.lastSpeciesTabAt > 1000 && !$('#speciesPanel').hidden) {
    S.lastSpeciesTabAt = now;
    S.speciesTab.update(s.species, s.tick);
  }
}

// --------------------------------------------------------------------------
// frame loop
// --------------------------------------------------------------------------
let lastT = performance.now();

function loop(t) {
  requestAnimationFrame(loop);
  const now = performance.now();
  const dt = Math.min(0.1, Math.max(0, (t - lastT) / 1000));
  lastT = t;
  S.frame++;
  const { sim } = S;
  const tps = TICKS_PER_SEC[S.speed];

  // --- step the sim within the frame budget; remainder carries over (§3.7)
  let stepped = 0;
  if (!S.paused) {
    S.acc = Math.min(S.acc + dt * tps, tps * 0.5);
    const due = Math.floor(S.acc);
    if (due > 0) {
      const n = Math.max(1, Math.min(due, Math.floor(STEP_BUDGET_MS / Math.max(0.05, S.msPerTick))));
      const t0 = performance.now();
      try {
        sim.step(n);
      } catch (e) {
        console.error(`${LOG} sim.step trapped; pausing. A WASM trap leaves the instance undefined, so reload the page.`, e);
        S.paused = true;
        toast('The simulation crashed (see console). Reload the page to continue.', 8000);
        return;
      }
      const ms = performance.now() - t0;
      S.msPerTick += (ms / n - S.msPerTick) * 0.2;
      S.acc -= n;
      stepped = n;
      S.stepInterval = (1000 * n) / tps;
      S.lastStepAt = now;
    }
  }
  const tick = sim.tick();
  const cr = sim.creatures();
  if (stepped) S.creatures.ingest(cr, sim.deaths(), DTH, now, S.speed === 16, tick);
  const alpha = S.paused ? 1 : Math.min(1, (now - S.lastStepAt) / Math.max(1, S.stepInterval));

  // --- environment
  const fr = sim.frame();
  S.stage.setDaylight(fr.light, dt, S.speed < 4);
  if (stepped || S.frame % 4 === 0) S.terrain.syncDroughts(sim.droughts(), DL, now, true);
  if (S.frame % 4 === 0) recolor(now);
  if (S.frame % 8 === 0 && S.layer && S.layer.id === 'food' && S.cfg.fruitEnabled) S.terrain.updateFruit(sim.grid('fruit'));
  const sinceBoot = now - S.bootAt;
  if (sinceBoot > BIOME_LABEL_MS) {
    const k = Math.min(1, (sinceBoot - BIOME_LABEL_MS) / 2000);
    const o = 1 - 0.7 * k;
    S.terrain.setBiomeOpacity(0.55 * o);
    for (const b of S.biomeLabels) b.opacity = o;
  }

  // --- creatures + camera
  S.creatures.update(cr, now, dt, alpha);
  let followPose = null;
  if (S.cameras.mode === 'follow') {
    if (S.follow.hold) {
      followPose = pose;
      if (now > S.follow.hold) {
        S.follow.hold = 0;
        setView(mqPhone.matches ? 'map' : 'orbit');
      }
    } else if (S.creatures.pose(S.creatures.selectedId, pose)) {
      followPose = pose;
    }
  }
  S.cameras.update(dt, followPose);
  S.stage.render(S.cameras.camera);

  // --- overlay labels (every 2nd frame)
  if (S.frame % 2 === 0) {
    const rings = S.terrain.rings;
    for (let k = 0; k < rings.length; k++) {
      const d = rings[k];
      const lab = S.droughtLabels[k];
      lab.visible = d.active;
      if (d.active) {
        const y = Math.max(0, d.y - d.r);
        lab.x = S.map.sceneX(d.x);
        lab.z = S.map.sceneZ(y);
        lab.y = S.map.heightAt(d.x, y) + 0.06;
        if (S.frame % 30 === 0 || !lab.text) S.labels.setText(lab, `Drought · ${fmtInt(d.remaining)} ticks left`);
      }
    }
    S.labels.update(S.cameras.camera, S.w, S.h);
  }

  // --- stats sample every SAMPLE_EVERY_TICKS ticks (at most once per frame)
  if (tick - S.lastSampleTick >= SAMPLE_EVERY_TICKS) sample(now);

  // --- feed (rate-limited)
  const ev = S.limiter.length ? S.limiter.poll(now, S.speed === 16, tick) : null;
  if (ev) S.feed.add(ev);

  // --- clock (4 Hz)
  if (now - S.lastClockAt > 250) {
    S.lastClockAt = now;
    updateClock(tick, fr);
  }

  // --- inspector (4 Hz)
  if (S.inspector.open && S.deadId < 0 && now - S.lastInspectAt > 250) {
    S.lastInspectAt = now;
    refreshInspector(now, tick, fr, cr);
  }

  if (debug && S.frame % 15 === 0) {
    const info = S.stage.renderer.info.render;
    $('#debug').textContent = `${sim.kind} · ${S.stage.fps.toFixed(0)} fps · ${info.calls} calls · ${fmtInt(info.triangles)} tris · ${S.msPerTick.toFixed(2)} ms/tick · dpr ${S.stage.dpr} · pop ${cr.count}`;
  }
}

function updateClock(tick, fr) {
  const c = S.cfg;
  $('#tickText').textContent = `Tick ${fmtInt(tick)}`;
  $('#dayText').textContent = `Day ${fmtInt(dayOf(tick, c.dayPeriod))}`;
  const season = seasonName(tick, c.seasonPeriod);
  $('#seasonText').textContent = season ? `Season: ${season}` : '';
  if (S.lastStats) $('#genText').textContent = `Gen ${fmtInt(S.lastStats.max_generation ?? 0)}`;
  if (c.dayEnabled && c.dayPeriod > 0) {
    const phase = ((tick / c.dayPeriod) % 1 + 1) % 1;
    const cur = Math.floor(phase * 24);
    for (let i = 0; i < S.dialSegs.length; i++) S.dialSegs[i].setAttribute('class', i === cur ? 'on' : i < cur ? 'past' : '');
    const night = fr.light < 0.5;
    const di = $('#dialIcon');
    if (di.dataset.icon !== (night ? 'moon' : 'sun')) {
      di.dataset.icon = night ? 'moon' : 'sun';
      di.innerHTML = icon(di.dataset.icon, 14);
    }
    $('#dial').setAttribute('aria-label', night ? 'Night: creatures see half as far' : 'Day');
  }
}

function refreshInspector(now, tick, fr, cr) {
  const id = S.inspector.id;
  const d = S.sim.inspect(id);
  if (!d) {
    if (S.deadId !== id) {
      S.deadId = id;
      S.inspector.died(-1, NaN);
    }
    return;
  }
  const i = S.creatures.indexOf(id);
  let targetRelSize = null;
  let targetSameSpecies = null;
  if (i >= 0 && d.flags & F.ATTACKED) {
    // ABI v1 has no attack target (target_row = -1); sim-core attacks the nearest
    // neighbour, so describe the nearest one.
    const o = i * cr.stride;
    let best = -1;
    let bd = Infinity;
    for (let j = 0; j < cr.count; j++) {
      if (j === i) continue;
      const dx = S.map.wrapDelta(cr.data[o + L.X], cr.data[j * cr.stride + L.X], S.cfg.width);
      const dy = S.map.wrapDelta(cr.data[o + L.Y], cr.data[j * cr.stride + L.Y], S.cfg.height);
      const dd = dx * dx + dy * dy;
      if (dd < bd) {
        bd = dd;
        best = j;
      }
    }
    if (best >= 0 && bd < (S.cfg.attackRange * 3) ** 2) {
      targetRelSize = cr.data[best * cr.stride + L.BODY] < cr.data[o + L.BODY] ? 'smaller' : 'larger';
      const a = cr.data[o + L.SPECIES] | 0;
      const b = cr.data[best * cr.stride + L.SPECIES] | 0;
      targetSameSpecies = a >= 0 && b >= 0 ? a === b : null;
    }
  }
  let onFruit = false;
  if (S.cfg.fruitEnabled && d.traits.diet > 0.5) {
    const fg = S.sim.grid('fruit');
    const c = Math.min(S.cfg.cols - 1, Math.floor(d.x / S.cfg.cellSize));
    const r = Math.min(S.cfg.rows - 1, Math.floor(d.y / S.cfg.cellSize));
    onFruit = !!fg && fg[r * S.cfg.cols + c] > 0.05;
  }
  S.inspector.update(d, {
    cfg: S.cfg,
    colors: S.colors,
    F,
    now,
    isNight: S.cfg.dayEnabled && fr.light < 0.5,
    reproducedRecently: tick - S.creatures.lastReproTick(id) <= 20,
    onFruit,
    targetRelSize,
    targetSameSpecies,
    parentAlive: d.parentId !== null && S.creatures.indexOf(d.parentId) >= 0,
  });
}

boot().catch((e) => {
  console.error(`${LOG} boot failed`, e);
  const t = document.querySelector('#loadingText');
  if (t) t.textContent = `The demo failed to start: ${e && e.message ? e.message : e}. See the browser console.`;
});
