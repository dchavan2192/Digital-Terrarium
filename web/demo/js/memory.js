// Demo-only configuration and presentation helpers. Experiment defaults stay off.
export const DEMO_MEMORY_PROBABILITY = 0.3;

export function memoryOverlay(base, enabled) {
  const lines = String(base || '').split('\n');
  const header = lines.findIndex((line) => /^\s*\[mutation\]\s*(?:#.*)?$/.test(line));
  const setting = `add_recurrent_probability = ${enabled ? DEMO_MEMORY_PROBABILITY : '0.0'}`;
  if (header < 0) return `${lines.join('\n')}\n[mutation]\n${setting}\n`;
  let end = header + 1;
  while (end < lines.length && !/^\s*\[/.test(lines[end])) end++;
  const index = lines.findIndex((line, i) => i > header && i < end && /^\s*add_recurrent_probability\s*=/.test(line));
  if (index >= 0) lines[index] = setting;
  else lines.splice(header + 1, 0, setting);
  return lines.join('\n');
}

export function memoryReadout(stats) {
  if (!Number.isFinite(stats?.recurrent_brain_fraction) || !Number.isFinite(stats?.population)) return null;
  const fraction = Math.max(0, Math.min(1, stats.recurrent_brain_fraction));
  return { count: Math.round(fraction * stats.population), population: stats.population,
    percent: Math.round(100 * fraction) };
}

export function recurrentEdgeCount(brain) {
  return (brain?.edges || []).filter((edge) => edge.enabled !== false && edge.recurrent === true).length;
}

export function visibleBrainEdges(edges, strongestOnly) {
  if (!strongestOnly) return edges;
  const forward = edges.filter((edge) => !edge.recurrent)
    .sort((a, b) => Math.abs(b.effective_weight ?? b.weight) - Math.abs(a.effective_weight ?? a.weight)).slice(0, 30);
  return [...forward, ...edges.filter((edge) => edge.recurrent)];
}
