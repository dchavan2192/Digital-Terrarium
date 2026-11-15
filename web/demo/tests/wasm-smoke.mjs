// Exercise the actual deployment binary, rather than the JS mock backend.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { instantiateWasmBackend, createSim } from '../js/sim.js';

const bytes = await readFile(process.argv[2] || new URL('../terrarium.wasm', import.meta.url));
const config = await readFile(new URL('../demo.toml', import.meta.url), 'utf8');
const backend = await instantiateWasmBackend(bytes);
assert.equal(backend.kind, 'wasm');
const sim = createSim(backend);
sim.init(config, 42);
sim.step(25);
const s = sim.stats();
assert.equal(s.tick, 25);
assert.ok(s.population > 0);
assert.ok(Number.isFinite(s.energy_balance_residual), 'actual Rust ledger must reach frontend');
assert.ok(Math.abs(s.energy_balance_residual) < 1e-6);
assert.ok(s.energy_after > 0);
assert.ok(sim.creatures().count > 0);
console.log(`Real WASM smoke passed: tick ${s.tick}, population ${s.population}, energy residual ${s.energy_balance_residual}`);
