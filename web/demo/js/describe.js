// "What is it doing?" sentence + trait wording for the inspector.
// Pure module (node-testable). Spec: docs/research/DEMO_UX_DESIGN.md §7.1, §7.5.
//
// Rules are evaluated in spec priority order. A rule whose data the ABI does not
// provide (e.g. sharing, construction, vision-ray contents) is skipped, never guessed.

import { fmtFixed, fmtPct } from './format.js';

/**
 * @param c {
 *   attacked, shared, kin, ateNow, onFruit, reproducedRecently, constructing,
 *   speedNorm, raySeesFood, raySeesCreature, attackOut, energyFrac, isNight,
 *   targetRelSize ('smaller'|'larger'|null), targetSameSpecies (bool|null)
 * } — every field optional.
 */
export function whatIsItDoing(c) {
  if (c.attacked) {
    const size = c.targetRelSize ? `${c.targetRelSize} ` : '';
    const kin = c.targetSameSpecies === true ? 'same-species ' : c.targetSameSpecies === false ? 'other-species ' : '';
    return size || kin ? `Attacking a ${size}${kin}creature.` : 'Attacking another creature.';
  }
  if (c.shared) {
    const who = c.kin === undefined ? 'a neighbour' : c.kin > 0.5 ? 'a relative' : 'a stranger';
    const k = c.kin === undefined ? '' : ` (kin similarity ${fmtFixed(c.kin, 2)})`;
    return `Sharing energy with ${who}${k}.`;
  }
  if (c.reproducedRecently) return 'Just had offspring.';
  if (c.ateNow) return `Grazing on ${c.onFruit ? 'fruit' : 'algae'}.`;
  if (c.constructing !== undefined && c.constructing > 0.5) return 'Improving the ground here.';
  if (c.speedNorm > 0.6 && c.raySeesFood) return 'Heading toward food it can see.';
  if (c.speedNorm > 0.6 && c.raySeesCreature && c.attackOut > 0) return 'Chasing another creature.';
  if (c.energyFrac !== undefined && c.energyFrac < 0.25) return 'Running low on energy and searching for food.';
  if (c.isNight && c.speedNorm < 0.2) return 'Resting in the dark.';
  return 'Wandering.';
}

/** Short hover label activity word: "Grazing" / "Hunting" / … */
export function activityWord(flags, F, speedNorm) {
  if (flags & F.ATTACK) return 'Attacking';
  if (flags & F.REPRO) return 'Breeding';
  if (flags & F.EAT) return 'Grazing';
  if (speedNorm < 0.1) return 'Resting';
  return 'Wandering';
}

export function sizeWord(bs) {
  if (bs >= 2.2) return 'large';
  if (bs <= 1.1) return 'small';
  return 'medium';
}

export function tempWord(pref) {
  if (pref < 0.35) return 'cool north';
  if (pref > 0.65) return 'warm south';
  return 'mild middle';
}

export function dietWords(predationFraction) {
  const meat = Math.max(0, Math.min(1, predationFraction || 0));
  return `${fmtPct(1 - meat)} plants · ${fmtPct(meat)} meat`;
}

export function entropyWord(h) {
  if (!Number.isFinite(h)) return '–';
  const w = h > 0.66 ? 'varied behaviour' : h > 0.33 ? 'some variety' : 'repetitive behaviour';
  return `${w} (${fmtFixed(h, 2)})`;
}

export function agingWord(rate, defaultRate = 0.002) {
  if (!Number.isFinite(rate)) return '–';
  const r = rate / defaultRate;
  if (r < 0.85) return 'slowly';
  if (r > 1.15) return 'quickly';
  return 'at a typical pace';
}

/** Mean of a histogram over [-1, 1] (bin centres). Used as a "recent output" proxy. */
export function histogramMean(bins) {
  if (!bins || !bins.length) return 0;
  const n = bins.length;
  let s = 0;
  let w = 0;
  for (let i = 0; i < n; i++) {
    const centre = -1 + (2 * (i + 0.5)) / n;
    s += centre * bins[i];
    w += bins[i];
  }
  return w > 0 ? s / w : 0;
}
