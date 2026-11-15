// KPI strip: 5 tiles with plain-English labels, big tabular numbers, sparklines
// of the last 2,000 ticks (100 points, one per 20 ticks), delta vs 500 ticks ago,
// and a what / why / citation tooltip. Spec §6.1.

import { KPI_DEFS, deltaArrow, sparkPoints, minMax, fmtFixed, fmtInt } from '../format.js';
import { slotHex } from '../palette.js';

export const SPARK_POINTS = 100;
export const SAMPLE_EVERY_TICKS = 20;
const DELTA_BACK = 500 / SAMPLE_EVERY_TICKS; // 25 samples

export function createKpis(root, tooltip) {
  const hist = {};
  let n = 0;
  let head = 0;
  const tmp = new Float64Array(SPARK_POINTS);
  const tiles = KPI_DEFS.map((def) => {
    hist[def.key] = new Float64Array(SPARK_POINTS);
    const el = document.createElement('button');
    el.type = 'button';
    el.className = 'kpi';
    el.dataset.key = def.key;
    el.setAttribute('aria-describedby', 'kpiTip');
    el.innerHTML = `
      <span class="kpi-label">${def.label}</span>
      <span class="kpi-row">
        <span class="kpi-value tnum">–</span>
        <span class="kpi-unit"></span>
        <span class="kpi-delta" aria-hidden="true"></span>
      </span>
      <svg class="spark" viewBox="0 0 120 28" width="120" height="28" aria-hidden="true">
        <polyline fill="none" points="" />
        <circle r="2.5" cx="-10" cy="-10" />
      </svg>`;
    root.insertBefore(el, root.querySelector('.legend-chip'));
    const tile = {
      def,
      el,
      value: el.querySelector('.kpi-value'),
      unit: el.querySelector('.kpi-unit'),
      delta: el.querySelector('.kpi-delta'),
      line: el.querySelector('polyline'),
      dot: el.querySelector('circle'),
      last: null,
    };
    const show = () => tooltip.show(el, tipHtml(tile));
    el.addEventListener('mouseenter', show);
    el.addEventListener('focus', show);
    el.addEventListener('click', show);
    el.addEventListener('mouseleave', () => tooltip.hide());
    el.addEventListener('blur', () => tooltip.hide());
    return tile;
  });

  function series(key, out) {
    // oldest -> newest into `out`
    const h = hist[key];
    for (let i = 0; i < n; i++) out[i] = h[(head - n + i + SPARK_POINTS) % SPARK_POINTS];
    return n;
  }

  function tipHtml(tile) {
    const { def } = tile;
    const k = series(def.key, tmp);
    const [lo, hi] = minMax(tmp, k);
    const range = k > 1 ? `<p class="tip-range tnum">Last ${fmtInt(k * SAMPLE_EVERY_TICKS)} ticks: ${fmtNum(lo)} – ${fmtNum(hi)}</p>` : '';
    const extra = tile.last && def.extra ? def.extra(tile.last) : '';
    return `<p class="tip-title">${def.label}</p>
      <p><strong>What it means.</strong> ${def.what}</p>
      <p><strong>Why it matters.</strong> ${def.why}</p>
      ${extra ? `<p>${extra}</p>` : ''}${range}
      <p class="tip-cite">${def.cite}</p>`;
  }

  function fmtNum(v) {
    return Math.abs(v) < 10 && v % 1 ? fmtFixed(v, 2) : fmtInt(v);
  }

  return {
    /** Push one sample (call every SAMPLE_EVERY_TICKS ticks). */
    push(stats, cfg) {
      for (const t of tiles) {
        const v = t.def.get(stats);
        hist[t.def.key][head] = Number.isFinite(v) ? v : 0;
      }
      head = (head + 1) % SPARK_POINTS;
      n = Math.min(SPARK_POINTS, n + 1);
      this.render(stats, cfg);
    },
    render(stats, cfg) {
      for (const t of tiles) {
        t.last = stats;
        const v = t.def.get(stats);
        const f = t.def.fmt(v, stats, cfg);
        if (t.value.textContent !== f.value) t.value.textContent = f.value;
        if (t.unit.textContent !== f.unit) t.unit.textContent = f.unit;
        const k = series(t.def.key, tmp);
        const back = k > DELTA_BACK ? tmp[k - 1 - DELTA_BACK] : k > 1 ? tmp[0] : NaN;
        const d = deltaArrow(v, back);
        t.delta.textContent = d.glyph;
        t.delta.dataset.dir = String(d.dir);
        const sp = sparkPoints(tmp, k);
        t.line.setAttribute('points', sp.points);
        t.dot.setAttribute('cx', sp.lastX.toFixed(1));
        t.dot.setAttribute('cy', sp.lastY.toFixed(1));
        t.el.setAttribute('aria-label', `${t.def.label}: ${f.value} ${f.unit}`);
      }
    },
    /** Swatches next to "Colour = species" (top species by population). */
    setSwatches(chip, species, colors) {
      const top = species.filter((s) => s.established && s.population > 0).sort((a, b) => b.population - a.population).slice(0, 8);
      const html = top.map((s) => `<i class="sw" style="--c:${slotHex(colors.slot(s.id))}" title="Sp. ${s.id}"></i>`).join('');
      if (chip.dataset.html !== html) {
        chip.dataset.html = html;
        chip.innerHTML = html;
      }
    },
    reset() {
      n = 0;
      head = 0;
      for (const k in hist) hist[k].fill(0);
    },
  };
}

/** One shared floating tooltip (KPIs, legends, buttons). */
export function createTooltip(el) {
  let anchor = null;
  return {
    show(target, html) {
      anchor = target;
      el.innerHTML = html;
      el.hidden = false;
      const r = target.getBoundingClientRect();
      const tw = el.offsetWidth;
      const th = el.offsetHeight;
      let x = r.left + r.width / 2 - tw / 2;
      x = Math.max(8, Math.min(window.innerWidth - tw - 8, x));
      let y = r.bottom + 8;
      if (y + th > window.innerHeight - 8) y = Math.max(8, r.top - th - 8);
      el.style.transform = `translate(${Math.round(x)}px, ${Math.round(y)}px)`;
    },
    hide() {
      el.hidden = true;
      anchor = null;
    },
    get anchor() {
      return anchor;
    },
  };
}
