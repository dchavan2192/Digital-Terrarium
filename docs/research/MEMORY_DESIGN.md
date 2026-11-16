# Memory: recurrent connections (design, not yet implemented)

Status: **design only, written 2026-11-16. No code has been changed.** Roadmap item 2.1 / Phase 9b. The owner asked to start it; the session ended before implementation began.

## Why

Brains are feed-forward: each tick a creature reacts only to what it senses now. A recurrent connection carries a node's activation from the previous tick into the current one, which is the smallest change that gives a brain state. NEAT supports recurrent connections directly (Stanley & Miikkulainen 2002). Node identity is now population-wide (`genome::Innovations`), which this needs.

## Decisions

1. **Off by default.** One knob: `mutation.add_recurrent_probability`, default `0.0`. At 0 no recurrent gene can ever appear. Turning it on by default should wait for the validated food baseline and for behavioural tests (handoff item 6, roadmap Stage 2 gate).
2. **With it off, trajectories must be bit-identical to today's.** Guard the mutation draw with `p > 0.0 &&` so no random number is consumed when it is off. Fingerprints cannot be compared directly (the state gains fields), so compare physics instead. Reference states from the current engine (commit `389368f`) are saved locally, not committed:
   - `output/conformance/ref-pre-memory/state_tick3000.json` (default config, seed 42; fingerprint `470b1720b883d5c1`)
   - `output/conformance/ref-pre-memory-b/state_tick1500.json` (seed 9, `food.regrowth_rate=0.006`, `creature.hard_cap=2000`; fingerprint `eee0ea3f9ea62df1`)
   Regenerate with `sim-runner fingerprint --seed 42 --ticks 3000 --dump-dir DIR` on that commit if they are gone. After the change, organisms' ids, positions, energies and genomes, the food grid, both RNG states and the innovation registry must match.
3. **Semantics.** A recurrent gene `from ⇢ to` adds `weight × (activation of from on the previous tick)` to `to`'s input. Any source except the bias node (inputs give a one-tick delay line; hidden and output nodes give state; self-loops allowed). Target must be a hidden or output node. A newborn's previous activations are all zero.
4. **Genome.** `ConnectionGene` gains `recurrent: bool` (`#[serde(default)]`). Gene identity, sort order, crossover matching and `genome_distance` use the key `(from, to, recurrent)`, so one pair can hold both kinds. Feed-forward genes keep the depth rule; recurrent genes are exempt. New mutation `mutate_add_recurrent` (uses `math::random_index`, up to 20 tries, no duplicates). Add-node splits feed-forward genes only. Toggle and prune apply to any gene. `add_connection`'s duplicate check must ignore recurrent genes.
5. **Brain.** Keep the existing feed-forward list untouched and add a second list, `recurrent_connections` with its own `recurrent_gene_index`, sorted by the target's evaluation order. `forward(inputs, prev, act)` writes activations into a caller-supplied slice and reads `prev` for recurrent edges (an empty `prev` means zeros). `refresh_weights` must patch both lists. `connection_count()` counts both, so brain upkeep and the energy ledger stay balanced. Activity cost includes recurrent signals.
6. **Where the state lives.** `Organism::memory: Vec<f64>`, allocated once at birth with one entry per brain slot, empty for brains with no recurrent genes. The parallel sense/think phase stays read-only on organisms: new activations go into a per-tick scratch buffer on `Simulation` (`organisms × MAX_NODES`, reused, `#[serde(skip)]`), iterated with `par_chunks_mut(MAX_NODES)` (plain `chunks_mut` without the `parallel` feature). The sequential phase copies a creature's chunk into its `memory` only if its brain has recurrent edges. No allocation per creature per tick.
7. **Checkpoints.** `memory` is serialized, so continuation is exact. Validate: `memory` is empty or has exactly one entry per slot, all finite; recurrent genes have a non-bias source and a hidden/output target; the recurrent list maps one-to-one to enabled recurrent genes. Bump `CHECKPOINT_VERSION` to 4.
8. **Stats.** Append `mean_recurrent_connections` and `recurrent_brain_fraction` to `TickStats` (end of the CSV). Bump the runner's `stats_schema_version` to 3.
9. **Runner and demo.** Add `mutation.add_recurrent_probability` to `sweep.rs` `apply_param`. Inspector JSON edges gain `recurrent`; the brain graph draws them dashed (self-loops as a small arc).

## Tests to write

- Forced fixture: a self-loop on an output with constant inputs gives different outputs on tick 1 and tick 2 (state carries over); without the loop they are equal.
- Delay line: a recurrent edge from an input makes the output at tick t reflect the input at t − 1.
- OFF non-interference: at probability 0 no gene is recurrent, every `memory` is empty, and the next RNG draw is unchanged (the mechanism-effects pattern).
- Exact continuation: 300 ticks with a high recurrent probability, split at 150 with a checkpoint, equals the uninterrupted run.
- Cross-platform: Mac and WASM fingerprints match with recurrence on (generate a config with the key set and pass it to `conformance.mjs`).
- Crossover: a feed-forward and a recurrent gene on the same pair are both inherited and not confused.
- Energy ledger stays balanced with recurrent brains.

## After it works

- Behavioural test first (roadmap 2.4): a delayed-response task that a feed-forward brain cannot solve, to show memory is used and not merely present.
- Then a Pi comparison of recurrence on against off at the validated food setting, with the drought schedule held fixed (`food.droughts.schedule_seed`).
- Raising `MAX_NODES` (roadmap 2.2) is separate.
