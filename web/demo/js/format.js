// Formatting helpers + KPI definitions. Pure module (node-testable).
// Spec: docs/research/DEMO_UX_DESIGN.md §6.1.

const NF = new Intl.NumberFormat('en-US');

export function fmtInt(n) {
  if (!Number.isFinite(n)) return '–';
  return NF.format(Math.round(n));
}

export function fmtPct(r) {
  if (!Number.isFinite(r)) return '–';
  return `${Math.round(r * 100)}%`;
}

export function fmtFixed(n, d = 2) {
  if (!Number.isFinite(n)) return '–';
  return n.toFixed(d);
}

/** Day N = tick / daynight.period (§6.4). */
export function dayOf(tick, period) {
  return period > 0 ? Math.floor(tick / period) : 0;
}

export function trophicQualifier(t) {
  if (!Number.isFinite(t)) return '';
  if (t < 1.1) return 'Mostly grazers';
  if (t <= 1.4) return 'Some hunting';
  return 'Predators common';
}

/** Trophic role from predation fraction (§4.2): 0 grazer, 1 omnivore, 2 hunter. */
export function trophicTier(predationFraction) {
  if (predationFraction > 0.5) return 2;
  if (predationFraction >= 0.25) return 1;
  return 0;
}
export const TIER_NAMES = ['Grazer', 'Omnivore', 'Hunter'];

/** Arrow vs. a past value, with a relative dead-band. */
export function deltaArrow(now, past, rel = 0.02) {
  if (!Number.isFinite(now) || !Number.isFinite(past)) return { glyph: '', dir: 0 };
  const band = Math.max(Math.abs(past) * rel, 1e-9);
  if (now > past + band) return { glyph: '↑', dir: 1 };
  if (now < past - band) return { glyph: '↓', dir: -1 };
  return { glyph: '→', dir: 0 };
}

/** 8-way compass for a sim position. y = 0 is NORTH (cold edge, §5 temperature). */
export function compass(x, y, w, h) {
  const dx = x / w - 0.5;
  const dy = y / h - 0.5;
  if (Math.hypot(dx, dy) < 0.15) return 'centre';
  const ang = Math.atan2(-dy, dx); // north = +pi/2
  const names = ['east', 'north-east', 'north', 'north-west', 'west', 'south-west', 'south', 'south-east'];
  const k = Math.round(ang / (Math.PI / 4));
  return names[(k + 8) % 8];
}

/** Season name from the seasonal sine phase (multiplier = 1 + A sin(2πt/P)). */
export function seasonName(tick, period) {
  if (!(period > 0)) return '';
  const p = ((tick / period) % 1 + 1) % 1;
  if (p < 0.25) return 'spring';
  if (p < 0.5) return 'summer';
  if (p < 0.75) return 'autumn';
  return 'winter';
}

export function lightLevel(tick, period, enabled = true) {
  if (!enabled || !(period > 0)) return 1;
  return (1 + Math.sin((2 * Math.PI * tick) / period)) * 0.5;
}

export function seasonMultiplier(tick, period, amplitude) {
  if (!amplitude || !(period > 0)) return 1;
  return Math.max(0, 1 + amplitude * Math.sin((2 * Math.PI * tick) / period));
}

/** "Sp. 3" label for a stable species id. */
export function speciesName(id) {
  return id >= 0 ? `Sp. ${id}` : 'Unclassified';
}

/** Generated binomial-ish epithet (§6.3), e.g. "Lowland grazer". */
export function speciesEpithet({ meanTrophic, predatorFraction, meanBodySize }) {
  const pf = Number.isFinite(predatorFraction) ? predatorFraction : (meanTrophic ?? 1) - 1;
  const role = pf > 0.5 ? 'hunter' : pf >= 0.25 ? 'omnivore' : 'grazer';
  const size = meanBodySize >= 2.0 ? 'Large' : meanBodySize <= 1.2 ? 'Small' : '';
  return size ? `${size} ${role}` : role[0].toUpperCase() + role.slice(1);
}

// ---------------------------------------------------------------------------
// KPI definitions (§6.1). `get(stats)` returns a number; `fmt` returns display parts.
// ---------------------------------------------------------------------------

export const KPI_DEFS = [
  {
    key: 'population',
    label: 'Creatures alive',
    get: (s) => s.population,
    fmt: (v, s, cfg) => ({ value: fmtInt(v), unit: cfg && cfg.hardCap ? `/ ${fmtInt(cfg.hardCap)} cap` : '' }),
    what: 'How many creatures are alive.',
    why: 'Populations boom when food is plentiful and crash in droughts. The cap is the world’s carrying capacity.',
    cite: 'Lotka 1925; Volterra 1926',
  },
  {
    key: 'species',
    label: 'Species',
    get: (s) => s.speciesCount,
    fmt: (v) => ({ value: fmtInt(v), unit: '' }),
    what: 'Groups of creatures whose brains and bodies have evolved far enough apart to count as distinct.',
    why: 'New species arise as lineages adapt to different parts of the world.',
    cite: 'Stanley & Miikkulainen 2002 (NEAT); Schluter 2000',
  },
  {
    key: 'brain',
    label: 'Brain size',
    get: (s) => s.mean_connections,
    fmt: (v, s) => ({
      value: fmtInt(v),
      unit: Number.isFinite(s.mean_brain_neurons) ? `links · ${fmtInt(s.mean_brain_neurons)} neurons` : 'links',
    }),
    what: 'Average number of connections in a creature’s brain.',
    why: 'Brains grow by mutation but cost energy to run, so the size shows whether intelligence is paying for itself.',
    cite: 'Aiello & Wheeler 1995; Laughlin et al. 1998',
  },
  {
    key: 'coop',
    label: 'Cooperation',
    get: (s) => s.cooperation_rate,
    fmt: (v) => ({ value: fmtPct(v), unit: 'share energy' }),
    what: 'Share of creatures that donated energy to a neighbour recently.',
    why: 'Nearly all sharing is between relatives, so helping kin helps your own genes.',
    cite: 'Hamilton 1964 (rB > C); Trivers 1971',
    extra: (s) => (Number.isFinite(s.kin_share_ratio) ? `Kin share ratio now: ${fmtPct(s.kin_share_ratio)} of gifts go to relatives.` : ''),
  },
  {
    key: 'trophic',
    label: 'Food chain',
    get: (s) => s.mean_trophic_level,
    fmt: (v) => ({ value: fmtFixed(v, 2), unit: trophicQualifier(v) }),
    what: '1.0 means everyone eats plants and 2.0 means everyone eats other creatures.',
    why: 'Rising values mean predation is evolving.',
    cite: 'Lindeman 1942',
    extra: (s) => (Number.isFinite(s.predator_count) ? `${fmtInt(s.predator_count)} creatures get most of their energy by hunting.` : ''),
  },
];

/** Min/max of a numeric ring (for tooltips). */
export function minMax(arr, n) {
  let lo = Infinity;
  let hi = -Infinity;
  for (let i = 0; i < n; i++) {
    const v = arr[i];
    if (v < lo) lo = v;
    if (v > hi) hi = v;
  }
  return n ? [lo, hi] : [NaN, NaN];
}

/** SVG polyline points for a sparkline (w×h px), oldest -> newest. */
export function sparkPoints(values, n, w = 120, h = 28, pad = 3) {
  if (n < 2) return { points: '', lastX: w - pad, lastY: h / 2 };
  const [lo, hi] = minMax(values, n);
  const span = hi - lo || 1;
  let pts = '';
  let x = 0;
  let y = 0;
  for (let i = 0; i < n; i++) {
    x = pad + ((w - 2 * pad) * i) / (n - 1);
    y = h - pad - ((h - 2 * pad) * (values[i] - lo)) / span;
    pts += `${x.toFixed(1)},${y.toFixed(1)} `;
  }
  return { points: pts.trim(), lastX: x, lastY: y };
}
