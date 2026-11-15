// Data-layer registry (§5). One layer at a time (radio). Each layer maps a grid
// to t ∈ [0,1] via t = (value - offset) * scale and looks t up in a 256-entry LUT.
// Pure module (no DOM / three.js).

import { COLORMAPS, buildLUT } from './palette.js';

/**
 * @param caps   sim.caps
 * @param cfg    sim.config (normalised)
 * @returns array of layer descriptors; `available` is false when the ABI/config
 *          lacks the data, and the UI hides those entries.
 */
export function buildLayers(caps, cfg) {
  const elevSpan = Math.max(1e-6, cfg.elevationMax - cfg.elevationMin);
  return [
    {
      id: 'food',
      label: 'Food',
      key: 'f',
      grid: 'food',
      dynamic: true,
      available: true,
      stops: COLORMAPS.lichen,
      lut: buildLUT(COLORMAPS.lichen),
      offset: 0,
      scale: 1 / (cfg.maxPerCell || 1),
      lo: 'no food',
      hi: 'full',
      units: `cell max = ${cfg.maxPerCell} food units`,
      text: 'Food regrows each tick, faster in oases, slower in droughts and winter.' + (caps.fruit ? ' Creatures specialise on algae or fruit.' : ''),
      cite: caps.fruit ? 'Grant & Grant 2006' : 'Schluter 2000; Rosenzweig 1995',
      fruit: caps.fruit,
    },
    {
      id: 'temperature',
      label: 'Temperature',
      grid: 'temperature',
      dynamic: false,
      available: !!caps.temperature,
      stops: COLORMAPS.vik,
      lut: buildLUT(COLORMAPS.vik),
      offset: 0,
      scale: 1,
      lo: 'N cold',
      hi: 'S warm',
      units: 'temperature = y / world height',
      text: 'Each creature has an evolved preferred temperature. Living far from it costs extra energy.',
      cite: 'Bergmann 1847',
    },
    {
      id: 'scent',
      label: 'Scent trails',
      grid: 'pheromone',
      dynamic: true,
      available: !!caps.pheromone,
      stops: COLORMAPS.scent,
      lut: buildLUT(COLORMAPS.scent),
      offset: 0,
      scale: 1,
      lo: 'faint',
      hi: 'strong',
      units: 'level / pheromones.max_level',
      text: 'Creatures leave scent wherever they walk; others can smell it. Indirect communication like ant trails.',
      cite: 'Wilson 1975; Deneubourg et al. 1990',
    },
    {
      id: 'built',
      label: 'Built ground',
      grid: 'construction',
      dynamic: true,
      available: !!caps.construction,
      stops: COLORMAPS.built,
      lut: buildLUT(COLORMAPS.built),
      offset: -1,
      scale: 0.5,
      lo: 'degraded',
      hi: 'enriched',
      units: '−1 degraded … +1 enriched',
      text: 'Creatures can spend energy to improve the ground (more regrowth) or degrade it. Their changes outlive them.',
      cite: 'Odling-Smee, Laland & Feldman 2003',
    },
    {
      id: 'elevation',
      label: 'Elevation',
      grid: 'elevation',
      dynamic: false,
      available: !!caps.elevation,
      stops: COLORMAPS.cividis,
      lut: buildLUT(COLORMAPS.cividis),
      offset: cfg.elevationMin,
      scale: 1 / elevSpan,
      lo: 'low',
      hi: 'high',
      units: `height ${cfg.elevationMin.toFixed(2)} … ${cfg.elevationMax.toFixed(2)}`,
      text: 'Hills slow creatures down. Climbing costs speed.',
      cite: 'MacArthur & Wilson 1967',
    },
  ];
}
