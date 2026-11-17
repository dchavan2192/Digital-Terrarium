import test from 'node:test';
import assert from 'node:assert/strict';
import { memoryOverlay, memoryReadout, connectionBreakdown, recurrentEdgeCount, visibleBrainEdges } from '../js/memory.js';

test('demo memory overlay preserves other settings and replaces an existing mutation key', () => {
  const base = '[food]\nregrowth_rate = 0.002\n[mutation]\nweight_probability = 0.1\nadd_recurrent_probability = 0.0 # off\n[sharing]\nshare_amount = 5\n';
  const on = memoryOverlay(base, true);
  assert.match(on, /add_recurrent_probability = 0.3/);
  assert.match(on, /weight_probability = 0.1/);
  assert.match(on, /\[sharing\]\nshare_amount = 5/);
  assert.equal(on.match(/\[mutation\]/g).length, 1);
  assert.equal(on.match(/add_recurrent_probability/g).length, 1);
  assert.match(memoryOverlay(on, false), /add_recurrent_probability = 0.0/);
  assert.match(memoryOverlay('[food]\nregrowth_rate = 0.002', true), /\[mutation\]\nadd_recurrent_probability = 0.3/);
  assert.match(memoryOverlay('[mutation]\nweight_probability = 0.1\n[food]\nregrowth_rate = 0.002', false), /\[mutation\]\nadd_recurrent_probability = 0.0\nweight_probability/);
});

test('memory count is measured presence, missing data remains unavailable', () => {
  assert.equal(memoryReadout({ population: 100 }), null);
  assert.equal(memoryReadout({ population: 100, recurrent_brain_fraction: NaN }), null);
  assert.deepEqual(memoryReadout({ population: 4, recurrent_brain_fraction: 0.25 }), { count: 1, population: 4, percent: 25 });
  assert.equal(recurrentEdgeCount({ edges: [{ recurrent: true }, { recurrent: true, enabled: false }, { recurrent: false }] }), 1);
});

test('simplified brain diagram retains weak recurrent edges', () => {
  const forward = Array.from({ length: 50 }, (_, i) => ({ weight: i + 1 }));
  const memory = { recurrent: true, weight: 0.001 };
  const edges = [...forward, memory];
  const visible = visibleBrainEdges(edges, true);
  assert.equal(visible.length, 31);
  assert.ok(visible.includes(memory));
  assert.equal(visibleBrainEdges(edges, false), edges);
});


test('connection breakdown uses both measured edge counts and rejects unavailable or inconsistent metrics', () => {
  assert.deepEqual(connectionBreakdown(300, 50), { total: 300, recurrent: 50, feedForward: 250 });
  assert.deepEqual(connectionBreakdown(231, 0), { total: 231, recurrent: 0, feedForward: 231 });
  for (const [total, recurrent] of [[300, undefined], [300, NaN], [Infinity, 0], [10, 11], [-1, 0], [10, -1]]) {
    assert.equal(connectionBreakdown(total, recurrent), null);
  }
});
