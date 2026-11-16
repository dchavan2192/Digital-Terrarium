// Print state fingerprints from the real WASM build in the same format as
// `sim-runner fingerprint`, so the browser engine can be compared tick for tick
// with native builds. This measures cross-platform agreement; it asserts nothing.
//
//   node web/demo/tests/conformance.mjs [wasm] [config.toml] [seed] [ticks]
//   cargo run -p sim-runner --release -- fingerprint --seed 42 --ticks 1,10,100,1000,5000
import { readFile } from 'node:fs/promises';

const wasmPath = process.argv[2] || new URL('../terrarium.wasm', import.meta.url);
const configPath = process.argv[3] || new URL('../../../configs/default.toml', import.meta.url);
const seed = BigInt(process.argv[4] || '42');
const ticks = (process.argv[5] || '1,10,100,1000,5000').split(',').map(Number);

const { instance } = await WebAssembly.instantiate(await readFile(wasmPath), {});
const x = instance.exports;
const toml = new TextEncoder().encode(await readFile(configPath, 'utf8'));
const ptr = x.tr_alloc(toml.length);
new Uint8Array(x.memory.buffer, ptr, toml.length).set(toml);
const status = x.tr_init(ptr, toml.length, seed);
x.tr_free(ptr, toml.length);
if (status !== 0) {
  const message = new TextDecoder().decode(new Uint8Array(x.memory.buffer, x.tr_last_error_ptr(), x.tr_last_error_len()));
  throw new Error(`tr_init failed (${status}): ${message}`);
}

for (const target of ticks) {
  const remaining = target - Number(x.tr_tick());
  if (remaining > 0) x.tr_step(remaining);
  const print = BigInt.asUintN(64, x.tr_state_fingerprint()).toString(16).padStart(16, "0");
  console.log(`tick=${x.tr_tick()} fingerprint=${print}`);
  if (process.env.DUMP_DIR) {
    const len = x.tr_state_json();
    const { writeFile, mkdir } = await import('node:fs/promises');
    await mkdir(process.env.DUMP_DIR, { recursive: true });
    await writeFile(`${process.env.DUMP_DIR}/state_tick${x.tr_tick()}.json`, new Uint8Array(x.memory.buffer, x.tr_state_json_ptr(), len));
  }
}
