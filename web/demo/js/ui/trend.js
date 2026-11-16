// Expanded trend chart: click a headline tile to see that measure over time.
//
// The time window is dynamic. "Whole run" grows with the run; the two sliding
// windows follow the latest ticks at full resolution. Droughts are shaded, so a
// viewer can see what a population crash or a cooperation spike coincided with.

import { chartRange, levelBands, niceScale, nearestIndex } from '../history.js';
import { fmtInt, dayOf } from '../format.js';

export const WINDOWS = [
  { id: 'recent', label: 'Last 2,000 ticks', ticks: 2000 },
  { id: 'longer', label: 'Last 10,000 ticks', ticks: 10000 },
  { id: 'all', label: 'Whole run', ticks: null },
];

const PAD = { left: 52, right: 14, top: 12, bottom: 26 };

/**
 * @param {HTMLElement} root the #trend section
 * @param {{defs: object[], history: object, sampleTicks: number, getStats: () => object, getCfg: () => object, onClose: () => void}} opts
 */
export function createTrendPanel(root, { defs, history, sampleTicks, getStats, getCfg, onClose }) {
  const title = root.querySelector('#trendTitle');
  const now = root.querySelector('#trendNow');
  const read = root.querySelector('#trendRead');
  const note = root.querySelector('#trendNote');
  const canvas = root.querySelector('#trendCanvas');
  const ranges = root.querySelector('#trendRanges');
  const ctx = canvas.getContext('2d');
  const max = history.maxPoints();
  const buf = { ticks: new Float64Array(max), values: new Float64Array(max), flags: new Uint8Array(max) };

  let key = null; // which measure is open
  let windowId = 'all';
  let n = 0; // points currently drawn
  let hover = -1; // hovered point index, or -1
  let geometry = null; // last draw's scales, for hit-testing

  ranges.innerHTML = WINDOWS.map(
    (w) => `<button type="button" class="seg" data-window="${w.id}" aria-pressed="${w.id === windowId}">${w.label}</button>`,
  ).join('');
  ranges.addEventListener('click', (e) => {
    const b = e.target.closest('button[data-window]');
    if (!b) return;
    windowId = b.dataset.window;
    for (const el of ranges.querySelectorAll('button')) el.setAttribute('aria-pressed', String(el === b));
    hover = -1;
    draw();
  });
  root.querySelector('#trendClose').addEventListener('click', () => onClose());

  canvas.addEventListener('pointermove', (e) => {
    if (!geometry || !n) return;
    const r = canvas.getBoundingClientRect();
    const tick = geometry.t0 + ((e.clientX - r.left - PAD.left) / geometry.plotW) * (geometry.t1 - geometry.t0);
    const i = nearestIndex(buf.ticks, n, tick);
    if (i !== hover) {
      hover = i;
      draw();
    }
  });
  canvas.addEventListener('pointerleave', () => {
    if (hover !== -1) {
      hover = -1;
      draw();
    }
  });
  if (typeof ResizeObserver !== 'undefined') new ResizeObserver(() => draw()).observe(canvas);

  function def() {
    return defs.find((d) => d.key === key);
  }

  function text(d, v) {
    const f = d.fmt(v, getStats() || {}, getCfg());
    // Units such as "/ 500 cap" describe the live value, not a historical one.
    return f.value;
  }

  function css(name) {
    return getComputedStyle(root).getPropertyValue(name).trim() || '#888';
  }

  function draw() {
    if (root.hidden || !key) return;
    const d = def();
    const series = defs.indexOf(d);
    const win = WINDOWS.find((w) => w.id === windowId);
    n = history.read(series, win.ticks, buf);
    if (hover >= n) hover = -1;

    const stats = getStats();
    const cfg = getCfg();
    title.textContent = d.label;
    if (stats) {
      const f = d.fmt(d.get(stats), stats, cfg);
      now.textContent = `${f.value}${f.unit ? ` ${f.unit}` : ''} now`;
    }

    // Size the backing store to the element, at device resolution.
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    if (!w || !h) return;
    if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);

    const plotW = w - PAD.left - PAD.right;
    const plotH = h - PAD.top - PAD.bottom;
    const ink3 = css('--ink-3');
    const line = css('--line');
    ctx.font = '11px Inter, system-ui, sans-serif';
    ctx.textBaseline = 'middle';

    if (n < 2) {
      ctx.fillStyle = ink3;
      ctx.textAlign = 'center';
      ctx.fillText('Collecting data…', PAD.left + plotW / 2, PAD.top + plotH / 2);
      read.textContent = '';
      geometry = null;
      return;
    }

    const t0 = buf.ticks[0];
    const t1 = buf.ticks[n - 1];
    const data = chartRange(buf.values, n);
    const { lo, hi, step } = niceScale(data.lo, data.hi);
    const x = (t) => PAD.left + ((t - t0) / Math.max(1, t1 - t0)) * plotW;
    const y = (v) => PAD.top + (1 - (v - lo) / (hi - lo)) * plotH;
    geometry = { t0, t1, plotW };

    // Drought bands behind everything else: stronger where more droughts overlap.
    ctx.fillStyle = css('--warn');
    for (const [a, b, level] of levelBands(buf.ticks, buf.flags, n)) {
      const xa = x(a);
      ctx.globalAlpha = Math.min(0.42, 0.1 * level);
      ctx.fillRect(xa, PAD.top, Math.max(2, x(b) - xa), plotH);
    }
    ctx.globalAlpha = 1;

    // Horizontal grid lines with value labels.
    ctx.strokeStyle = line;
    ctx.fillStyle = ink3;
    ctx.lineWidth = 1;
    ctx.textAlign = 'right';
    for (let v = lo; v <= hi + step * 1e-6; v += step) {
      const gy = Math.round(y(v)) + 0.5;
      ctx.beginPath();
      ctx.moveTo(PAD.left, gy);
      ctx.lineTo(PAD.left + plotW, gy);
      ctx.stroke();
      ctx.fillText(text(d, v), PAD.left - 8, gy);
    }

    // Time axis: start and end of the window.
    ctx.textBaseline = 'alphabetic';
    const period = cfg && cfg.dayPeriod;
    const stamp = (t) => (period ? `Day ${dayOf(t, period)} · tick ${fmtInt(t)}` : `Tick ${fmtInt(t)}`);
    ctx.textAlign = 'left';
    ctx.fillText(stamp(t0), PAD.left, h - 8);
    ctx.textAlign = 'right';
    ctx.fillText(stamp(t1), PAD.left + plotW, h - 8);

    // The measure itself.
    ctx.strokeStyle = css('--ink-1');
    ctx.lineWidth = 1.75;
    ctx.lineJoin = 'round';
    ctx.beginPath();
    for (let i = 0; i < n; i++) {
      if (i) ctx.lineTo(x(buf.ticks[i]), y(buf.values[i]));
      else ctx.moveTo(x(buf.ticks[i]), y(buf.values[i]));
    }
    ctx.stroke();

    // Marker: the hovered point, or the latest one.
    const m = hover >= 0 ? hover : n - 1;
    const mx = x(buf.ticks[m]);
    const my = y(buf.values[m]);
    if (hover >= 0) {
      ctx.strokeStyle = ink3;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(Math.round(mx) + 0.5, PAD.top);
      ctx.lineTo(Math.round(mx) + 0.5, PAD.top + plotH);
      ctx.stroke();
    }
    ctx.fillStyle = css('--accent');
    ctx.beginPath();
    ctx.arc(mx, my, 3.5, 0, Math.PI * 2);
    ctx.fill();

    // Text readout (also what screen readers get).
    let min = Infinity;
    let maxV = -Infinity;
    for (let i = 0; i < n; i++) {
      if (buf.values[i] < min) min = buf.values[i];
      if (buf.values[i] > maxV) maxV = buf.values[i];
    }
    if (hover >= 0) {
      const k = buf.flags[hover];
      const dry = k ? ` · ${k} drought${k > 1 ? 's' : ''} active` : '';
      read.textContent = `${stamp(buf.ticks[hover])}: ${text(d, buf.values[hover])}${dry}`;
    } else {
      const merged = win.ticks == null && history.stride > 1 ? ` Each point averages ${fmtInt(history.stride * sampleTicks)} ticks.` : '';
      read.textContent = `Lowest ${text(d, min)}, highest ${text(d, maxV)} in this window.${merged}`;
    }
    canvas.setAttribute('aria-label', `${d.label} over time. ${read.textContent}`);
  }

  return {
    /** Show the chart for one measure. */
    open(k) {
      key = k;
      hover = -1;
      const d = def();
      note.textContent = `${d.what} ${d.why} Shading marks droughts; darker means more at once.`;
      root.hidden = false;
      draw();
    },
    close() {
      key = null;
      root.hidden = true;
    },
    /** Call after each new sample; redraws only while open. */
    update() {
      draw();
    },
    get openKey() {
      return key;
    },
  };
}
