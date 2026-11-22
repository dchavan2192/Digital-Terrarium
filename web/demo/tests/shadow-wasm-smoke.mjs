// Bounded manufactured engineering regression, not a study cohort or hypothesis test.
// Four runs, each ending at tick 1000, on the unchanged browser demo configuration.
// Usage: node web/demo/tests/shadow-wasm-smoke.mjs [path/to/sim_wasm.wasm]
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { instantiateWasmBackend, createSim, L } from '../js/sim.js';
import { demoModesOverlay, ecologyReadout } from '../js/ecology-metrics.js';

const bytes = await readFile(process.argv[2] || new URL('../terrarium.wasm', import.meta.url));
const base = await readFile(new URL('../../../configs/demo.toml', import.meta.url), 'utf8');
const sim = createSim(await instantiateWasmBackend(bytes));
const seed = 42;
const checkpoints = [100, 1000];

function snapshot() {
  const stats = sim.stats();
  const creatures = sim.creatures();
  assert.equal(stats.tick, sim.tick());
  assert.ok(stats.population > 0 && creatures.count === stats.population);
  let sum = 0, squares = 0, hidden = 0;
  for (let i = 0; i < creatures.count; i++) {
    const offset = i * creatures.stride;
    const diet = creatures.data[offset + L.DIET];
    sum += diet;
    squares += diet * diet;
    hidden += creatures.data[offset + L.HIDDEN];
  }
  const dietMean = sum / creatures.count;
  const variance = Math.max(0, squares / creatures.count - dietMean * dietMean);
  assert.ok(Number.isFinite(stats.mean_diet_preference));
  assert.ok(Number.isFinite(stats.diet_variance));
  assert.ok(Number.isFinite(stats.mean_hidden_nodes));
  // Creature rows are f32 mirrors of f64 engine state. This checks the current
  // variance convention against the actual population, rather than its name.
  assert.ok(Math.abs(stats.mean_diet_preference - dietMean) < 1e-6);
  assert.ok(Math.abs(stats.diet_variance - variance) < 1e-6);
  assert.ok(Math.abs(stats.mean_hidden_nodes - hidden / creatures.count) < 1e-10);
  const readout = ecologyReadout(stats, sim.config, sim.kind);
  assert.equal(readout.trackingState, 'ready');
  assert.equal(readout.hidden, stats.mean_hidden_nodes);
  assert.equal(readout.dietVariance, stats.diet_variance);
  return { stats: structuredClone(stats),
    count: creatures.count, stride: creatures.stride,
    creatures: Array.from(creatures.data.subarray(0, creatures.count * creatures.stride)),
    ids: creatures.ids ? Array.from(creatures.ids.subarray(0, creatures.count)) : null };
}

function sampleRun() {
  return checkpoints.map(tick => {
    sim.step(tick - sim.tick());
    return snapshot();
  });
}

function withoutShadow(snapshot) {
  return { ...snapshot, stats: Object.fromEntries(Object.entries(snapshot.stats)
    .filter(([key]) => !key.startsWith('shadow_'))) };
}

sim.init(demoModesOverlay(base, { memoryEnabled: false, shadowEnabled: false }), seed);
assert.equal(sim.caps.shadow, true);
assert.equal(sim.config.shadowEnabled, false);
assert.equal(sim.config.activityTrackingEnabled, true);
const offConfig = structuredClone(sim.config.raw.config);
const off = sampleRun();
for (const sample of off) {
  assert.equal(sample.stats.shadow_population, 0, 'off uses an unmeasured zero sentinel in Rust');
  const readout = ecologyReadout(sample.stats, sim.config, sim.kind);
  assert.equal(readout.shadowState, 'off');
  assert.equal(readout.shadow, null, 'off sentinel is never presented as a measured neutral zero');
}

sim.init(demoModesOverlay(base, { memoryEnabled: false, shadowEnabled: true }), seed);
assert.equal(sim.config.shadowEnabled, true);
assert.equal(sim.config.activityTrackingEnabled, true);
const expectedOn = structuredClone(offConfig);
expectedOn.activity_tracking.enabled = true;
expectedOn.activity_tracking.shadow = true;
assert.deepEqual(sim.config.raw.config, expectedOn, 'neutral toggle changes only its intended configuration fields');
const on = sampleRun();
assert.deepEqual(on.map(withoutShadow), off.map(withoutShadow),
  'same-seed main-world creature buffers, IDs and all non-shadow stats match at ticks 100 and 1000');
for (const sample of on) {
  assert.equal(sample.stats.shadow_population, sample.stats.population);
  const readout = ecologyReadout(sample.stats, sim.config, sim.kind);
  assert.equal(readout.shadowState, 'ready');
  assert.ok(readout.shadow !== null);
  for (const value of Object.values(readout.shadow)) assert.ok(Number.isFinite(value) && value >= 0);
}

sim.init(demoModesOverlay(base, { memoryEnabled: true, shadowEnabled: true }), seed);
assert.equal(sim.config.shadowEnabled, true);
assert.equal(sim.config.raw.config.mutation.add_recurrent_probability, 0.3);
const bothConfig = structuredClone(sim.config.raw.config);
const expectedBoth = structuredClone(expectedOn);
expectedBoth.mutation.add_recurrent_probability = 0.3;
assert.deepEqual(bothConfig, expectedBoth, 'memory toggle retains the complete neutral configuration');
const both = sampleRun();
sim.reset(seed);
assert.equal(sim.tick(), 0);
assert.deepEqual(sim.config.raw.config, bothConfig, 'same-seed reset preserves the complete composed configuration');
assert.equal(sim.config.shadowEnabled, true);
assert.equal(sim.config.raw.config.mutation.add_recurrent_probability, 0.3);
assert.deepEqual(sampleRun(), both, 'combined memory+neutral reset reproduces actual measured states');

const last = on.at(-1).stats;
console.log(`Real neutral demo smoke passed: seed ${seed}, tick ${last.tick}, population ${last.population}, diet mean ${last.mean_diet_preference}, diet variance ${last.diet_variance}, hidden neurons ${last.mean_hidden_nodes}; sampled main-world equality and combined-mode reset verified.`);
