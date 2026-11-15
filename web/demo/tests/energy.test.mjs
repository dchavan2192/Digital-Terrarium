import test from 'node:test';
import assert from 'node:assert/strict';
import { energySummary, renderEnergy } from '../js/ui/energy.js';

function tick() {
  return {
    tick: 10, energy_before: 100, energy_after: 102,
    energy_food_input: 5, energy_fruit_input: 0,
    energy_basal_cost: 4, energy_movement_cost: 0,
    energy_brain_structure_cost: 0, energy_brain_activity_cost: 0,
    energy_crowding_cost: 0, energy_temperature_cost: 0,
    energy_construction_cost: 0, energy_attack_cost: 0,
    energy_combat_loss: 0, energy_sharing_loss: 0, energy_cap_overflow: 0,
    energy_offspring_input: 30, energy_reproduction_cost: 30,
    energy_death_removal: -1, energy_balance_residual: 0,
  };
}

test('missing or partial ledger data is not presented as a balanced budget', () => {
  assert.equal(energySummary(null), null);
  assert.equal(energySummary({ tick: 1 }), null);
  assert.equal(energySummary({ ...tick(), energy_food_input: NaN }), null);
  const root = { innerHTML: '' };
  renderEnergy(root, { tick: 100, population: 50 });
  assert.match(root.innerHTML, /unavailable/);
  assert.doesNotMatch(root.innerHTML, /✓ Energy accounted for/);
});

test('death overdrafts and newborn transfers retain their accounting signs', () => {
  const b = energySummary(tick());
  assert.equal(b.delta, 2);
  assert.equal(b.deaths, 1);
  assert.equal(b.reproduction, 0);
  assert.equal(b.balanced, true);
  assert.equal(energySummary({ ...tick(), energy_death_removal: 5 }).deaths, -5);
});

test('numeric noise is tolerated but meaningful residuals remain visible', () => {
  assert.equal(energySummary({ ...tick(), energy_balance_residual: 1e-12 }).balanced, true);
  assert.equal(energySummary({ ...tick(), energy_balance_residual: 1 }).balanced, false);
  const root = { innerHTML: '' };
  renderEnergy(root, { ...tick(), energy_balance_residual: 1 });
  assert.match(root.innerHTML, /accounting mismatch/);
  assert.match(root.innerHTML, /Tick 10/);
});
