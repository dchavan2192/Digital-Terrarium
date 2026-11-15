// Latest-tick reserve accounting, passed through from sim-core without inference.
const COSTS = [
  ['Staying alive', 'energy_basal_cost'],
  ['Moving', 'energy_movement_cost'],
  ['Brain upkeep', 'energy_brain_structure_cost'],
  ['Brain activity', 'energy_brain_activity_cost'],
  ['Crowding', 'energy_crowding_cost'],
  ['Temperature', 'energy_temperature_cost'],
  ['Building', 'energy_construction_cost'],
  ['Attacking', 'energy_attack_cost'],
  ['Combat losses', 'energy_combat_loss'],
  ['Sharing losses', 'energy_sharing_loss'],
  ['Reserve overflow', 'energy_cap_overflow'],
];
const FLOW_KEYS = ['energy_food_input', 'energy_fruit_input', 'energy_offspring_input',
  'energy_reproduction_cost', 'energy_death_removal', ...COSTS.map(([, key]) => key)];

export function energySummary(s) {
  if (!s || !['energy_before', 'energy_after', 'energy_balance_residual', ...FLOW_KEYS].every((key) => Number.isFinite(s[key]))) return null;
  const costs = COSTS.map(([label, key]) => ({ label, value: s[key] }));
  const magnitude = Math.abs(s.energy_before) + Math.abs(s.energy_after)
    + FLOW_KEYS.reduce((sum, key) => sum + Math.abs(s[key]), 0);
  return {
    tick: s.tick, reserves: s.energy_after, delta: s.energy_after - s.energy_before,
    food: s.energy_food_input, fruit: s.energy_fruit_input, costs,
    reproduction: s.energy_offspring_input - s.energy_reproduction_cost,
    deaths: -s.energy_death_removal,
    balanced: Math.abs(s.energy_balance_residual) <= 1e-9 * (1 + magnitude),
    residual: s.energy_balance_residual,
  };
}

const fmt = (v) => v.toLocaleString(undefined, { maximumFractionDigits: 2 });
const signed = (v) => `${v > 0 ? '+' : ''}${fmt(v)}`;
const row = (label, value, cls = '') => `<div class="budget-row ${cls}"><dt>${label}</dt><dd class="tnum">${value}</dd></div>`;

export function renderEnergy(root, stats) {
  const b = energySummary(stats);
  if (!b || !b.tick) {
    root.innerHTML = '<p class="muted">Measured energy accounting is unavailable for this tick. Mock data is never used to fill this budget.</p>';
    return;
  }
  root.innerHTML = `<p class="budget-reserve tnum">${fmt(b.reserves)} <span>stored energy</span></p>
    <p class="caption tnum">Tick ${fmt(b.tick)} · reserve change ${signed(b.delta)}</p>
    <dl class="budget-flows">
      ${row('Food harvested', `+${fmt(b.food)}`, 'budget-in')}
      ${row('Fruit harvested', `+${fmt(b.fruit)}`, 'budget-in')}
      ${b.costs.map((c) => row(c.label, c.value ? `−${fmt(c.value)}` : '0')).join('')}
      ${row('Parents → newborns (net)', signed(b.reproduction))}
      ${row('Death reserve adjustment', signed(b.deaths))}
    </dl>
    <p class="budget-check" data-balanced="${b.balanced}">${b.balanced ? '✓ Energy accounted for' : 'Energy accounting mismatch'}</p>
    <details class="budget-detail"><summary>About this measurement</summary>
      <p>Harvest is counted before full reserves discard overflow. Parent spending and newborn reserves are shown together. Deaths remove remaining reserves; clearing a temporary negative reserve is a positive accounting adjustment, not energy from food.</p>
      <p class="tnum">Unexplained difference: ${b.residual.toExponential(2)} energy units.</p>
    </details>`;
}
