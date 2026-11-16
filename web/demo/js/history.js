// Run history for the expandable trend charts. Pure data: no DOM, no simulation.
//
// Two stores are filled from the same samples:
//   - `recent`: a ring of the latest samples at full resolution, for the sliding
//     windows ("last 2,000 ticks", "last 10,000 ticks").
//   - `whole`: the entire run in a fixed amount of memory. When it fills up, pairs
//     of neighbouring points are merged into their mean and the sampling stride
//     doubles, so a run of any length still fits and still spans tick 0 to now.
//
// A small level travels with every point (here: how many droughts were active),
// merged by taking the highest level in the span.

export const RECENT_CAPACITY = 600; // 12,000 ticks at one sample per 20 ticks
export const WHOLE_CAPACITY = 1024; // must be even

/**
 * @param {number} seriesCount number of values per sample
 * @param {{recent?: number, whole?: number}} [capacity]
 */
export function createRunHistory(seriesCount, capacity = {}) {
  const recentCap = capacity.recent ?? RECENT_CAPACITY;
  const wholeCap = capacity.whole ?? WHOLE_CAPACITY;
  if (wholeCap < 2 || wholeCap % 2) throw new Error('whole-run capacity must be even');

  const recent = {
    ticks: new Float64Array(recentCap),
    values: Array.from({ length: seriesCount }, () => new Float64Array(recentCap)),
    flags: new Uint8Array(recentCap),
    n: 0,
    head: 0,
  };
  const whole = {
    ticks: new Float64Array(wholeCap),
    values: Array.from({ length: seriesCount }, () => new Float64Array(wholeCap)),
    flags: new Uint8Array(wholeCap),
    n: 0,
    stride: 1,
  };
  // Samples waiting to be averaged into the next whole-run point.
  const acc = { sum: new Float64Array(seriesCount), n: 0, tick: 0, flag: 0 };

  function compact() {
    const half = wholeCap / 2;
    for (let i = 0; i < half; i++) {
      whole.ticks[i] = whole.ticks[2 * i + 1];
      whole.flags[i] = Math.max(whole.flags[2 * i], whole.flags[2 * i + 1]);
      for (let k = 0; k < seriesCount; k++) {
        whole.values[k][i] = (whole.values[k][2 * i] + whole.values[k][2 * i + 1]) / 2;
      }
    }
    whole.n = half;
    whole.stride *= 2;
  }

  return {
    /**
     * Add one sample. `sample[k]` is the value of series k; non-finite counts as 0.
     * `level` is a small non-negative integer (0-255), e.g. the number of droughts.
     */
    push(tick, sample, level) {
      const f = Math.max(0, Math.min(255, Math.round(Number(level) || 0)));
      recent.ticks[recent.head] = tick;
      recent.flags[recent.head] = f;
      for (let k = 0; k < seriesCount; k++) {
        const v = Number.isFinite(sample[k]) ? sample[k] : 0;
        recent.values[k][recent.head] = v;
        acc.sum[k] += v;
      }
      recent.head = (recent.head + 1) % recentCap;
      recent.n = Math.min(recentCap, recent.n + 1);

      acc.n++;
      acc.tick = tick;
      acc.flag = Math.max(acc.flag, f);
      if (acc.n === whole.stride) {
        whole.ticks[whole.n] = acc.tick;
        whole.flags[whole.n] = acc.flag;
        for (let k = 0; k < seriesCount; k++) whole.values[k][whole.n] = acc.sum[k] / acc.n;
        whole.n++;
        acc.sum.fill(0);
        acc.n = 0;
        acc.flag = 0;
        if (whole.n === wholeCap) compact();
      }
    },

    reset() {
      recent.n = 0;
      recent.head = 0;
      whole.n = 0;
      whole.stride = 1;
      acc.sum.fill(0);
      acc.n = 0;
      acc.flag = 0;
    },

    /** Samples merged into each whole-run point (1, 2, 4, ...). */
    get stride() {
      return whole.stride;
    },
    get wholeLength() {
      return whole.n;
    },
    get recentLength() {
      return recent.n;
    },
    /** Ticks the full-resolution ring currently spans. */
    get recentSpan() {
      if (recent.n < 2) return 0;
      const first = recent.ticks[(recent.head - recent.n + recentCap) % recentCap];
      const last = recent.ticks[(recent.head - 1 + recentCap) % recentCap];
      return last - first;
    },

    /**
     * Copy one series, oldest first, into `out` ({ticks, values, flags} typed arrays
     * of at least `maxPoints()` entries). `windowTicks` selects the latest ticks at
     * full resolution; `null` selects the whole run. Returns the number of points.
     */
    read(series, windowTicks, out) {
      if (windowTicks == null) {
        for (let i = 0; i < whole.n; i++) {
          out.ticks[i] = whole.ticks[i];
          out.values[i] = whole.values[series][i];
          out.flags[i] = whole.flags[i];
        }
        return whole.n;
      }
      if (!recent.n) return 0;
      const lastTick = recent.ticks[(recent.head - 1 + recentCap) % recentCap];
      let count = 0;
      for (let i = 0; i < recent.n; i++) {
        const j = (recent.head - recent.n + i + recentCap) % recentCap;
        if (recent.ticks[j] < lastTick - windowTicks) continue;
        out.ticks[count] = recent.ticks[j];
        out.values[count] = recent.values[series][j];
        out.flags[count] = recent.flags[j];
        count++;
      }
      return count;
    },

    maxPoints() {
      return Math.max(recentCap, wholeCap);
    },
  };
}

/** Lowest and highest of the first `n` values; a flat line still gets a range. */
export function chartRange(values, n) {
  if (!n) return { lo: 0, hi: 1 };
  let lo = Infinity;
  let hi = -Infinity;
  for (let i = 0; i < n; i++) {
    if (values[i] < lo) lo = values[i];
    if (values[i] > hi) hi = values[i];
  }
  if (hi - lo < 1e-9) {
    const pad = Math.max(Math.abs(hi) * 0.1, 0.5);
    return { lo: Math.max(0, lo - pad) === 0 && lo >= 0 ? Math.max(0, lo - pad) : lo - pad, hi: hi + pad };
  }
  return { lo, hi };
}

/**
 * Runs of equal non-zero level as [startTick, endTick, level], for shaded bands
 * whose strength shows how many droughts were active. A run ends at the tick of
 * the next point (where the level changes), so neighbouring bands meet without a
 * gap; the final run ends at the last point.
 */
export function levelBands(ticks, levels, n) {
  const bands = [];
  let start = -1;
  for (let i = 0; i <= n; i++) {
    const level = i < n ? levels[i] : 0;
    if (start >= 0 && level !== levels[start]) {
      bands.push([ticks[start], ticks[Math.min(i, n - 1)], levels[start]]);
      start = -1;
    }
    if (start < 0 && level > 0) start = i;
  }
  return bands;
}

/**
 * Round axis bounds and step so grid lines fall on tidy values (1, 2, 2.5 or 5
 * times a power of ten). Values that are never negative get an axis that does
 * not dip below zero.
 */
export function niceScale(lo, hi, lines = 4) {
  if (!(hi > lo)) return { lo: lo - 1, hi: hi + 1, step: 1 };
  const raw = (hi - lo) / lines;
  const power = 10 ** Math.floor(Math.log10(raw));
  const unit = raw / power;
  // A little slack keeps the axis from doubling its range for a few percent of overshoot.
  const step = (unit <= 1.2 ? 1 : unit <= 2.4 ? 2 : unit <= 3 ? 2.5 : unit <= 6 ? 5 : 10) * power;
  // The small epsilon keeps 0.30000000000000004-style values on their grid line.
  return {
    lo: Math.floor(lo / step + 1e-9) * step,
    hi: Math.ceil(hi / step - 1e-9) * step,
    step,
  };
}

/** Index of the point whose tick is closest to `tick` (ticks ascending). */
export function nearestIndex(ticks, n, tick) {
  if (!n) return -1;
  let lo = 0;
  let hi = n - 1;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (ticks[mid] < tick) lo = mid + 1;
    else hi = mid;
  }
  if (lo > 0 && Math.abs(ticks[lo - 1] - tick) <= Math.abs(ticks[lo] - tick)) return lo - 1;
  return lo;
}
