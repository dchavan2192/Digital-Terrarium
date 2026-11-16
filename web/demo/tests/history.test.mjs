import test from 'node:test';
import assert from 'node:assert/strict';
import { createRunHistory, chartRange, levelBands, niceScale, nearestIndex } from '../js/history.js';

function out(h) {
  const n = h.maxPoints();
  return { ticks: new Float64Array(n), values: new Float64Array(n), flags: new Uint8Array(n) };
}

test('a short run is stored exactly', () => {
  const h = createRunHistory(2, { recent: 8, whole: 8 });
  for (let i = 1; i <= 5; i++) h.push(i * 20, [i, i * 10], i === 3);
  const o = out(h);
  assert.equal(h.read(1, null, o), 5);
  assert.deepEqual([...o.ticks.slice(0, 5)], [20, 40, 60, 80, 100]);
  assert.deepEqual([...o.values.slice(0, 5)], [10, 20, 30, 40, 50]);
  assert.deepEqual([...o.flags.slice(0, 5)], [0, 0, 1, 0, 0]);
  assert.equal(h.stride, 1);
});

test('the whole run always fits, spans the run, and keeps its mean', () => {
  const h = createRunHistory(1, { recent: 16, whole: 64 });
  const total = 10_000;
  let sum = 0;
  for (let i = 1; i <= total; i++) {
    const v = Math.sin(i / 50) * 100 + i * 0.01;
    sum += v;
    h.push(i * 20, [v], false);
  }
  const o = out(h);
  const n = h.read(0, null, o);
  assert.ok(n > 0 && n <= 64, `point count ${n} must fit the capacity`);
  assert.ok(h.stride >= total / 64, 'stride grew as the run got longer');
  for (let i = 1; i < n; i++) assert.ok(o.ticks[i] > o.ticks[i - 1], 'ticks ascend');
  assert.ok(o.ticks[0] <= h.stride * 20, 'first point is at the start of the run');
  assert.ok(o.ticks[n - 1] > (total - h.stride) * 20, 'last point is near the end of the run');
  // Every stored point is the mean of `stride` samples, so the mean of the stored
  // points equals the mean of the samples they cover.
  const covered = n * h.stride;
  let coveredSum = 0;
  for (let i = 1; i <= covered; i++) coveredSum += Math.sin(i / 50) * 100 + i * 0.01;
  let stored = 0;
  for (let i = 0; i < n; i++) stored += o.values[i];
  assert.ok(Math.abs(stored / n - coveredSum / covered) < 1e-6);
  assert.ok(Number.isFinite(sum));
});

test('merging keeps the highest level seen anywhere in the merged span', () => {
  const h = createRunHistory(1, { recent: 4, whole: 4 });
  for (let i = 1; i <= 16; i++) h.push(i, [0], i === 6 ? 3 : i === 7 ? 1 : 0);
  const o = out(h);
  const n = h.read(0, null, o);
  const levels = [...o.flags.slice(0, n)];
  assert.equal(Math.max(...levels), 3, 'the strongest drought level survives compaction');
  const idx = levels.indexOf(3);
  assert.ok(o.ticks[idx] >= 6 && o.ticks[idx] - h.stride < 6, 'and stays at the right time');
  assert.equal(levels.filter(Boolean).length, 1);
});

test('a sliding window returns the latest ticks at full resolution', () => {
  const h = createRunHistory(1, { recent: 50, whole: 8 });
  for (let i = 1; i <= 400; i++) h.push(i * 20, [i], false);
  const o = out(h);
  const n = h.read(0, 200, o); // last 200 ticks = 11 samples (inclusive ends)
  assert.equal(n, 11);
  assert.equal(o.ticks[n - 1], 8000);
  assert.equal(o.ticks[0], 7800);
  assert.equal(o.values[n - 1], 400);
  // Asking for more than the ring holds returns what it has, newest last.
  const all = h.read(0, 1e9, o);
  assert.equal(all, 50);
  assert.equal(o.ticks[all - 1], 8000);
  assert.equal(h.recentSpan, 49 * 20);
});

test('reset empties both stores', () => {
  const h = createRunHistory(1, { recent: 4, whole: 4 });
  for (let i = 1; i <= 30; i++) h.push(i, [i], true);
  h.reset();
  const o = out(h);
  assert.equal(h.read(0, null, o), 0);
  assert.equal(h.read(0, 100, o), 0);
  assert.equal(h.stride, 1);
  h.push(5, [7], false);
  assert.equal(h.read(0, null, o), 1);
  assert.equal(o.values[0], 7);
});

test('non-finite samples are stored as zero, not NaN', () => {
  const h = createRunHistory(1, { recent: 4, whole: 4 });
  h.push(1, [NaN], false);
  h.push(2, [undefined], false);
  const o = out(h);
  h.read(0, null, o);
  assert.deepEqual([...o.values.slice(0, 2)], [0, 0]);
});

test('chart helpers', () => {
  assert.deepEqual(chartRange(Float64Array.from([10, 20, 15]), 3), { lo: 10, hi: 20 });
  const flat = chartRange(Float64Array.from([5, 5, 5]), 3);
  assert.ok(flat.lo < 5 && flat.hi > 5, 'a flat line still gets a visible range');
  assert.deepEqual(chartRange(new Float64Array(0), 0), { lo: 0, hi: 1 });

  const ticks = Float64Array.from([10, 20, 30, 40, 50, 60]);
  // Each band runs to the tick where the level changes, so bands meet without gaps.
  assert.deepEqual(levelBands(ticks, Uint8Array.from([0, 1, 1, 2, 0, 1]), 6), [[20, 40, 1], [40, 50, 2], [60, 60, 1]]);
  assert.deepEqual(levelBands(ticks, new Uint8Array(6), 6), []);
  assert.deepEqual(levelBands(ticks, Uint8Array.from([2, 2, 2, 2, 2, 2]), 6), [[10, 60, 2]]);

  assert.deepEqual(niceScale(57, 463), { lo: 0, hi: 500, step: 100 });
  assert.deepEqual(niceScale(212, 236), { lo: 210, hi: 240, step: 5 });
  const pct = niceScale(0, 0.13);
  assert.ok(Math.abs(pct.step - 0.05) < 1e-12 && pct.lo === 0 && Math.abs(pct.hi - 0.15) < 1e-12);
  const tight = niceScale(1.01, 1.16);
  assert.ok(tight.lo <= 1.01 && tight.hi >= 1.16 && tight.lo >= 0.95);
  assert.deepEqual(niceScale(5, 5), { lo: 4, hi: 6, step: 1 });

  assert.equal(nearestIndex(ticks, 6, 34), 2);
  assert.equal(nearestIndex(ticks, 6, 36), 3);
  assert.equal(nearestIndex(ticks, 6, -5), 0);
  assert.equal(nearestIndex(ticks, 6, 999), 5);
  assert.equal(nearestIndex(ticks, 0, 5), -1);
});
