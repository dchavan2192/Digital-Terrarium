// Colour logic for the Terrarium demo. Pure module: no DOM, no three.js, so the
// node smoke test can import it.
//
// Spec: docs/research/DEMO_UX_DESIGN.md §4.2 (species palette + stability rule),
// §5 (data-layer colormaps), §8.1 (tokens).
//
// All colours handed to three.js are LINEAR floats (three r160 treats vertex and
// instance colours as linear working-space values). CSS keeps the sRGB hex.

/** Species palette, Okabe & Ito (2008) lifted for a dark UI. Index 0 = rank 1. */
export const SPECIES_HEX = [
  '#E69F00', // amber
  '#56B4E9', // sky
  '#7FD1AE', // mint
  '#F0E442', // pollen
  '#4C8FD6', // lake blue
  '#E07B39', // vermilion
  '#CC79A7', // orchid
  '#D9D2C3', // bone
];
export const OTHER_HEX = '#9A948A';
export const OTHER_SLOT = SPECIES_HEX.length; // 8

/** Fixed accent colours used by the scene (sRGB hex). */
export const SCENE_HEX = {
  bg: '#141613',
  dryOchre: '#C98B3C',
  leaf: '#B9D88A',
  alert: '#E5484D',
  honey: '#F2C94C',
  lavender: '#A58AD6',
  bone: '#F4F1E8',
  outline: '#1E211C',
  outlinePredator: '#0E0F0C',
  berry: '#C86B9A',
  lowland: '#1D2A2E', // cooler, darker tone mixed into low terrain (no water, §3.2)
  moon: '#9DB2D6',
  sun: '#FFF1DC',
  selection: '#F4F1E8',
};

export function hexToRgb01(hex) {
  const h = hex.replace('#', '');
  const n = parseInt(h.length === 3 ? h.split('').map((c) => c + c).join('') : h, 16);
  return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
}

export function srgbToLinear(c) {
  return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}

/** sRGB hex -> [r,g,b] linear floats. */
export function hexToLinear(hex) {
  const [r, g, b] = hexToRgb01(hex);
  return [srgbToLinear(r), srgbToLinear(g), srgbToLinear(b)];
}

/** Linear RGB for slots 0..7 plus OTHER at index 8, packed as Float32Array(27). */
export const SPECIES_LINEAR = (() => {
  const out = new Float32Array((SPECIES_HEX.length + 1) * 3);
  [...SPECIES_HEX, OTHER_HEX].forEach((hex, i) => {
    const c = hexToLinear(hex);
    out[i * 3] = c[0];
    out[i * 3 + 1] = c[1];
    out[i * 3 + 2] = c[2];
  });
  return out;
})();

export function slotHex(slot) {
  return slot >= 0 && slot < SPECIES_HEX.length ? SPECIES_HEX[slot] : OTHER_HEX;
}

// ---------------------------------------------------------------------------
// Colormap LUTs (§5). Stops are interpolated in sRGB, then stored linear.
// ---------------------------------------------------------------------------

export const COLORMAPS = {
  // Food: custom sequential "lichen", monotone lightness, one green hue.
  lichen: ['#2A2620', '#4E5A34', '#7E9A4A', '#B9D88A'],
  // Temperature: Crameri vik-like diverging, colour-blind safe.
  vik: ['#2B5C8A', '#E8E4D8', '#B5562E'],
  // Scent trails: neutral terrain -> magma-ish (low values look unpainted).
  scent: ['#3A3A34', '#5B3A7A', '#C46FA8', '#F6C7A0'],
  // Built ground: degraded <- neutral -> enriched.
  built: ['#8A5A3C', '#3A3A34', '#6FB3A0'],
  // Elevation: cividis (Nuñez et al. 2018).
  cividis: ['#00224E', '#123570', '#3B496C', '#575D6D', '#707173', '#8A8779', '#A69D75', '#C4B56C', '#E4CF5B', '#FEE838'],
};

/** Build an n-entry LUT (Float32Array n*3, linear) from sRGB hex stops. */
export function buildLUT(stops, n = 256) {
  const rgb = stops.map(hexToRgb01);
  const out = new Float32Array(n * 3);
  for (let i = 0; i < n; i++) {
    const t = (i / (n - 1)) * (rgb.length - 1);
    const k = Math.min(rgb.length - 2, Math.floor(t));
    const f = t - k;
    for (let c = 0; c < 3; c++) {
      const v = rgb[k][c] + (rgb[k + 1][c] - rgb[k][c]) * f;
      out[i * 3 + c] = srgbToLinear(v);
    }
  }
  return out;
}

/** CSS linear-gradient string for a colormap (used by the legend bar). */
export function cssGradient(stops) {
  return `linear-gradient(90deg, ${stops.join(', ')})`;
}

// ---------------------------------------------------------------------------
// Stable species -> palette slot assignment (§4.2 colour stability rule)
// ---------------------------------------------------------------------------

/**
 * Assigns palette slots to stable species ids.
 * - On the first update, species are ranked by population, so rank 1 gets slot 0.
 * - Later, a species that reaches `minClaim` members takes the LOWEST FREE slot.
 * - When a species disappears, its slot is freed after `cooldownMs`, so a
 *   recycled colour never seems to "resurrect" a dead species.
 * - If every slot is taken, newcomers are "other" (slot 8) until one frees.
 *
 * `minClaim` is a front-end noise filter (1-member clusters flicker in and out).
 */
export function createSpeciesColors({ slots = SPECIES_HEX.length, cooldownMs = 10000, minClaim = 3 } = {}) {
  const owner = new Array(slots).fill(-1); // species id per slot
  const freeAt = new Float64Array(slots); // time (ms) when a released slot may be reused
  const slotOf = new Map(); // species id -> slot
  let first = true;

  function claim(id, now) {
    for (let s = 0; s < slots; s++) {
      if (owner[s] === -1 && now >= freeAt[s]) {
        owner[s] = id;
        slotOf.set(id, s);
        return s;
      }
    }
    return OTHER_SLOT;
  }

  return {
    /** @param list [{id, population}] species alive now. @param now ms (real time). */
    update(list, now) {
      const alive = new Set();
      for (const sp of list) if (sp.population > 0) alive.add(sp.id);
      // Release slots of vanished species.
      for (let s = 0; s < slots; s++) {
        if (owner[s] !== -1 && !alive.has(owner[s])) {
          slotOf.delete(owner[s]);
          owner[s] = -1;
          freeAt[s] = now + cooldownMs;
        }
      }
      const sorted = list.slice().sort((a, b) => b.population - a.population || a.id - b.id);
      for (const sp of sorted) {
        if (slotOf.has(sp.id) || sp.population <= 0) continue;
        if (!first && sp.population < minClaim) continue;
        claim(sp.id, now);
      }
      first = false;
    },
    /** Slot 0..7, or OTHER_SLOT (8) for unassigned ids. Allocation-free. */
    slot(id) {
      const s = slotOf.get(id);
      return s === undefined ? OTHER_SLOT : s;
    },
    hex(id) {
      return slotHex(this.slot(id));
    },
    reset() {
      owner.fill(-1);
      freeAt.fill(0);
      slotOf.clear();
      first = true;
    },
  };
}
