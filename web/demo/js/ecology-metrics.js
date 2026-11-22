import { memoryOverlay } from './memory.js';

// Change only these keys in the demo overlay. Rebuild from the original text on
// every mode change so disabling shadow restores any original tracking setting.
export function shadowOverlay(base, enabled) {
  const lines = String(base || '').split('\n');
  let header = lines.findIndex(line => /^\s*\[activity_tracking\]\s*(?:#.*)?$/.test(line));
  if (header < 0) {
    lines.push('[activity_tracking]');
    header = lines.length - 1;
  }
  for (const [key, value] of [['shadow', String(enabled)], ...(enabled ? [['enabled', 'true']] : [])]) {
    let end = header + 1;
    while (end < lines.length && !/^\s*\[/.test(lines[end])) end++;
    const index = lines.findIndex((line, i) => i > header && i < end && new RegExp(`^\\s*${key}\\s*=`).test(line));
    const setting = `${key} = ${value}`;
    if (index >= 0) lines[index] = setting;
    else lines.splice(header + 1, 0, setting);
  }
  return lines.join('\n');
}

export function demoModesOverlay(base, { memoryEnabled, shadowEnabled }) {
  return memoryOverlay(shadowOverlay(base, shadowEnabled), memoryEnabled);
}

const nonnegative = value => Number.isFinite(value) && value >= 0 ? value : null;
const unit = value => Number.isFinite(value) && value >= 0 && value <= 1 ? value : null;
export function ecologyReadout(stats, config, kind = 'wasm') {
  const native = kind === 'wasm';
  const measured = native && Number.isFinite(stats?.tick) && stats.tick > 0
    && Number.isFinite(stats.population) && stats.population > 0;
  const tracking = native && config?.activityTrackingEnabled;
  const window = config?.activityWindowSize;
  const windowKnown = Number.isFinite(window) && window > 0;
  const sampled = tracking && windowKnown && Number.isFinite(stats?.tick) && stats.tick >= window;
  const trackingState = !native || config?.activityTrackingEnabled == null || (tracking && !windowKnown) ? 'unavailable'
    : !tracking ? 'off' : sampled ? 'ready' : 'waiting';
  const shadowState = !native || config?.shadowEnabled == null ? 'unavailable'
    : !config.shadowEnabled ? 'off' : trackingState;
  const activity = prefix => ({
    active: nonnegative(stats?.[`${prefix}active_traits`]),
    novel: nonnegative(stats?.[`${prefix}novel_traits`]),
    mean: nonnegative(stats?.[`${prefix}mean_cumulative_activity`]),
  });
  return {
    native, measured, trackingState, shadowState,
    dietMean: measured ? unit(stats.mean_diet_preference) : null,
    // The current engine computes E[d²]-E[d]²: variance, not SD or bimodality.
    dietVariance: measured ? nonnegative(stats.diet_variance) : null,
    hidden: measured ? nonnegative(stats.mean_hidden_nodes) : null,
    sampleTick: sampled ? Math.floor(stats.tick / window) * window : null,
    real: trackingState === 'ready' ? activity('') : null,
    shadow: shadowState === 'ready' ? activity('shadow_') : null,
    shadowPopulation: shadowState === 'ready' ? nonnegative(stats.shadow_population) : null,
  };
}

export function metricText(value, decimals = 2) {
  return !Number.isFinite(value) ? '–' : new Intl.NumberFormat('en-US', { maximumFractionDigits: decimals }).format(value);
}

export function renderEcology(root, stats, config, kind) {
  const readout = ecologyReadout(stats, config, kind);
  const set = (name, text) => { root.querySelector(`[data-ecology="${name}"]`).textContent = text; };
  set('diet', metricText(readout.dietMean, 3));
  set('variance', metricText(readout.dietVariance, 3));
  set('hidden', metricText(readout.hidden, 1));
  set('measurement', !readout.native ? 'Mock world: these native engine measurements are unavailable.'
    : !stats || stats.tick <= 0 ? 'Waiting for a measured tick…'
      : stats.population <= 0 ? 'No living creatures; population averages are unavailable.'
        : `Living population · tick ${metricText(stats.tick, 0)}`);
  const states = {
    off: 'Off · not measured', waiting: 'Waiting for first activity window…',
    unavailable: 'Unavailable in this build', ready: `Last activity window · tick ${metricText(readout.sampleTick, 0)}`,
  };
  set('real-state', states[readout.trackingState]);
  set('shadow-state', states[readout.shadowState]);
  for (const side of ['real', 'shadow']) for (const key of ['active', 'novel', 'mean']) {
    set(`${side}-${key}`, metricText(readout[side]?.[key] ?? null, key === 'mean' ? 2 : 0));
  }
  set('shadow-population', readout.shadowPopulation == null ? '' : `Neutral population: ${metricText(readout.shadowPopulation, 0)}.`);
  const sigma = config?.dietCompetitionSigma;
  const slope = config?.dietSlope;
  set('diet-settings', `Diet crowding σ: ${metricText(sigma, 2)}${sigma === 0 ? ' (off)' : ''} · feeding slope: ${metricText(slope, 2)}.`);
  return readout;
}
