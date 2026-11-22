import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { demoModesOverlay, shadowOverlay, ecologyReadout, metricText, renderEcology } from '../js/ecology-metrics.js';
import { createSim, normaliseConfig } from '../js/sim.js';

const parse = text => JSON.parse(execFileSync('python3', ['-c', 'import json,sys,tomllib; print(json.dumps(tomllib.loads(sys.stdin.read())))'], { input: text, encoding: 'utf8' }));
const base = `# An existing complete demo overlay, including an explicitly disabled tracker.
[creature]
hard_cap = 321
[food.fruit]
diet_slope = 0.8
[mutation]
weight_probability = 0.17
add_recurrent_probability = 0.0
[activity_tracking] # keep its other fields
enabled = false
shadow = false
window_size = 80
shadow_stream = 7
[sharing]
share_amount = 9
`;

test('all memory/neutral combinations preserve every unrelated TOML setting', () => {
  const original = parse(base);
  for (const memoryEnabled of [false, true]) for (const shadowEnabled of [false, true]) {
    const actual = parse(demoModesOverlay(base, { memoryEnabled, shadowEnabled }));
    const expected = structuredClone(original);
    expected.mutation.add_recurrent_probability = memoryEnabled ? 0.3 : 0;
    expected.activity_tracking.shadow = shadowEnabled;
    if (shadowEnabled) expected.activity_tracking.enabled = true;
    assert.deepEqual(actual, expected);
  }
  assert.deepEqual(parse(shadowOverlay('[food]\nregrowth_rate = 0.002', true)), {
    food: { regrowth_rate: .002 }, activity_tracking: { enabled: true, shadow: true },
  });
  assert.equal(parse(shadowOverlay(shadowOverlay(base, true), true)).activity_tracking.shadow_stream, 7);
});

test('mode changes and reset/new-seed fallback keep the full composed overlay identity', () => {
  let current, seed, lastOverlay;
  const backend = {
    kind: 'wasm', abiVersion: 1, has: () => false,
    initOverlay(text, value) { current = parse(text); seed = value; lastOverlay = text; return 0; },
    reset(value) { seed = value; return 1; }, // exercise facade re-init fallback
    configJson: () => ({ config: current, seed }),
    gridUpdate: () => 0, eventsJson: () => [], lastError: () => '',
  };
  const sim = createSim(backend);
  let modes = { memoryEnabled: false, shadowEnabled: true };
  sim.init(demoModesOverlay(base, modes), 42);
  assert.equal(sim.config.shadowEnabled, true);
  modes = { ...modes, memoryEnabled: true };
  sim.init(demoModesOverlay(base, modes), sim.seed);
  assert.equal(sim.seed, 42);
  const both = lastOverlay;
  assert.equal(current.mutation.add_recurrent_probability, .3);
  assert.equal(current.activity_tracking.shadow, true);
  sim.reset(43);
  assert.equal(sim.seed, 43);
  assert.equal(lastOverlay, both, 'new seed preserves the exact complete overlay text');
  modes = { ...modes, shadowEnabled: false };
  sim.init(demoModesOverlay(base, modes), sim.seed);
  assert.equal(current.mutation.add_recurrent_probability, .3);
  assert.equal(current.activity_tracking.enabled, false, 'shadow off restores original tracking mode');
  assert.equal(current.activity_tracking.shadow, false);
  modes = { ...modes, memoryEnabled: false, shadowEnabled: true };
  sim.init(demoModesOverlay(base, modes), sim.seed);
  assert.equal(current.mutation.add_recurrent_probability, 0);
  assert.equal(current.activity_tracking.shadow, true);
  assert.equal(current.creature.hard_cap, 321);
  assert.equal(current.food.fruit.diet_slope, .8);
  assert.equal(current.activity_tracking.shadow_stream, 7);
});

const config = { activityTrackingEnabled: true, activityWindowSize: 100, shadowEnabled: false };
const stats = { tick: 120, population: 100, mean_diet_preference: .4, diet_variance: .09,
  mean_hidden_nodes: 8.5, mean_brain_neurons: 36.5, active_traits: 4, novel_traits: 1,
  mean_cumulative_activity: 2.5, shadow_active_traits: 0, shadow_novel_traits: 0,
  shadow_mean_cumulative_activity: 0, shadow_population: 0 };

test('native variance and hidden-only counts use their exact engine fields', () => {
  const r = ecologyReadout(stats, config);
  assert.equal(r.dietVariance, .09, 'variance is not relabelled as standard deviation');
  assert.equal(r.dietMean, .4);
  assert.equal(r.hidden, 8.5, 'total brain neuron count is not a hidden neuron fallback');
  assert.equal(ecologyReadout({ ...stats, mean_hidden_nodes: undefined }, config).hidden, null);
  assert.equal(r.sampleTick, 100);
  assert.deepEqual(r.real, { active: 4, novel: 1, mean: 2.5 });
  assert.equal(r.shadowState, 'off');
  assert.equal(r.shadow, null, 'disabled shadow zeros must never appear as measurements');
});

test('off, unknown, tick zero, first-window wait, mock and extinction remain distinct', () => {
  assert.equal(ecologyReadout(stats, { ...config, activityTrackingEnabled: false }).real, null);
  assert.equal(ecologyReadout(stats, { ...config, shadowEnabled: null }).shadowState, 'unavailable');
  const zero = ecologyReadout({ ...stats, tick: 0 }, { ...config, shadowEnabled: true });
  assert.equal(zero.dietMean, null);
  assert.equal(zero.shadowState, 'waiting');
  assert.equal(zero.shadow, null);
  assert.equal(ecologyReadout({ ...stats, tick: 99 }, config).real, null);
  assert.equal(ecologyReadout(stats, { ...config, activityWindowSize: NaN }).trackingState, 'unavailable');
  const mock = ecologyReadout(stats, { ...config, shadowEnabled: true }, 'mock');
  assert.equal(mock.hidden, null);
  assert.equal(mock.shadowState, 'unavailable');
  const extinct = ecologyReadout({ ...stats, population: 0 }, config);
  assert.equal(extinct.dietMean, null);
  assert.equal(extinct.dietVariance, null);
  for (const value of [null, undefined, NaN, Infinity]) assert.equal(metricText(value), '–');
  assert.equal(metricText(0), '0', 'a measured zero is still shown');
});

test('real/neutral values render only after configured sampling, including genuine measured zeros', () => {
  const elements = new Map();
  const root = { querySelector(key) { if (!elements.has(key)) elements.set(key, {}); return elements.get(key); } };
  const text = key => elements.get(`[data-ecology="${key}"]`).textContent;
  renderEcology(root, stats, config, 'wasm');
  assert.equal(text('shadow-mean'), '–');
  assert.equal(text('shadow-state'), 'Off · not measured');
  assert.equal(text('variance'), '0.09');
  renderEcology(root, stats, { ...config, shadowEnabled: true }, 'wasm');
  assert.equal(text('shadow-mean'), '0');
  assert.match(text('shadow-state'), /tick 100/);
  renderEcology(root, null, config, 'wasm');
  assert.match(text('measurement'), /Waiting/);
  assert.equal(text('real-active'), '–');
});

test('configuration availability is explicit and does not turn missing neutral support into false', () => {
  assert.equal(normaliseConfig({}).shadowEnabled, null);
  assert.equal(normaliseConfig({}).activityTrackingEnabled, null);
  assert.equal(normaliseConfig({ config: { activity_tracking: { enabled: true, shadow: false, window_size: 100 } } }).shadowEnabled, false);
});
