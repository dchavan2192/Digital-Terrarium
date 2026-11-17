# Results register

Updated: 2026-11-17 UTC. This register separates implementation checks, calibration observations, and scientific results. A working mechanic is not evidence that selection favors it.

## Established scientific results for the corrected current model

None registered yet. This is an evidence status, not a claim that the model cannot produce interesting outcomes.

## Evidence required for a result

Each accepted entry must contain:

1. A stable result ID, hypothesis, primary endpoint, and interpretation limits.
2. Exact source commit and source-tree status; uncommitted experimental code must be archived and identified.
3. Full effective configuration, all overrides, seed list, execution mode, platform, compiler/build information, and metric/schema versions.
4. Requested and completed ticks, completion/extinction status, and immutable run paths or a durable data archive with checksums.
5. Analysis script revision and exact command, including exclusion rules, uncertainty estimates, and treatment of extinction or censored recovery.
6. Independent replicates and appropriate controls. Use pilot seeds to choose settings, then separate validation seeds. Declare the run or evolved lineage as the experimental unit; organisms and time samples from one run are not independent replicates.
7. A completed review distinguishing observation, causal interpretation, novelty, and untested extrapolation. A well-designed null result is eligible.

A claim may enter README or the portfolio only after its entry is accepted here. The portfolio may describe implemented capabilities without presenting them as scientific findings.

## Registered paired engineering pilot: EXP-2026-11-17-PROGRESS-TIEBREAK

[Completed method comparison](results/TERMINAL_PROGRESS_COMPARISON_2026-11-17.md): with matched founders at three fresh seed labels, the opt-in terminal-progress tiebreak acquired both fixed visible training responses in 2/3 searches versus 0/3 terminal-only controls. All six searches/9,984 candidate evaluations are retained; one treatment remains partial. Clean source 72f7c5b, identical binaries and per-seed founders verified, independent geometry/ranking audit passed. This is small-n training-only feasibility, not confirmatory efficacy, held-out transfer, memory, lifetime learning or ecological adaptation. Several contacts occur at horizon tick20; robustness remains unmeasured. Full provenance, curves, hashes and all-arm raw archive are linked in the report.

## Registered engineering feasibility pilot: EXP-2026-11-17-VISIBLE-ACQUISITION

[Fixed-budget acquisition report](results/VISIBLE_ACQUISITION_PILOT_2026-11-17.md): all three declared searches completed 64 founders/25 replacements at source `5504a20`, clean Pi release build. None reached both correct responses; final best reward 0.5 in every search. Seed 2000 improved from zero; seeds 2001/2002 retained partially competent founders. All 4,992 candidate evaluations and full final populations are retained, with independent audit, controls, hashes and raw archive. This limits this search procedure under this budget; it is not a memory deficit, architecture impossibility, held-out success or ecological result. Competence remains a gate before delayed-memory selection.

## Registered exploratory diagnostic: EXP-2026-11-17-MEMORY-GROWTH

[Completed overnight report](results/MEMORY_GROWTH_PILOT_2026-11-17.md): all eight 200k-tick demo-setting worlds completed (four paired seeds, recurrence mutation 0/0.3). Late mean enabled connections: 224.828 off versus 495.864 on; recurrent additions account for about 97.75% of that difference. Late mean population: 336.426 versus 277.406. Strong mutation pressure, structural thresholds, cap blocking and divergent organism histories prevent adaptive-memory or selection-versus-drift claims. Independent audit and final-export checks pass; exact dirty-build provenance, full raw archive, hashes and all trajectories are linked in the report. This is an exploratory diagnostic, not an established scientific result or memory-benefit test.

## Historical claims withdrawn from current-model evidence

| Claim or artifact | Status | Reason |
|---|---|---|
| Approximately 87% cooperation under amplified sharing | Withdrawn as evidence of adaptive altruism | The rule paid recipients more energy than donors spent, allowing resource-free net gain. Hue-based sharing did not independently establish genealogical kin selection. |
| Approximately 1,500-creature equilibrium and historical throughput table | Historical implementation measurements only | Rules/configurations changed; the population ceiling was binding. These do not characterize the current model or establish ecological self-regulation. |
| Logarithmic evolutionary activity across five seeds | Withdrawn as evidence of open-ended evolution | Pre-fix energetics/metric semantics and no neutral baseline. Finite-run activity growth cannot establish unbounded adaptive innovation. |
| Phase 7 ON/OFF plots in `docs/images/phase7/` | Historical exploratory artifacts | Their existence does not establish code/config/seed provenance or correctness under current mechanics. |

The source plots and narrative remain available in [EXPERIMENT_FINDINGS.md](EXPERIMENT_FINDINGS.md), [PROGRESS.md](PROGRESS.md), and Git history. Read the historical warning before reusing them.

## Registered calibration: CAL-2026-11-15-FOOD

[Phase 9a pilot report](results/PHASE9A_PILOT_2026-11-15.md): all six seed/rate runs survived 20k ticks with zero cap-blocked births. Mean late populations across two seeds were 567.58, 737.60, and 914.93 for regrowth 0.004, 0.006, and 0.008. Source `2f49287`, dirty-build flag explicitly retained; full config, machine-readable per-run measurements, checksums and reproducible analysis command are linked in the report. Status: calibration, not an established scientific result; held-out long-duration validation remains pending. No defaults changed.

## Registered implementation check: CHK-2026-11-15-PORTABILITY

Not a scientific result. Question: do different platforms follow the same trajectory from the same seed?

- **Before the fix** (source `ae181cf`): macOS native and WASM had identical state at tick 0 and different state at tick 1.
- **After the fix** (portable maths and fixed-width random indices; see `crates/sim-core/src/math.rs`): complete-state fingerprints were identical at ticks 0, 1, 100, 2,000 and 20,000 on three targets.

| Target | Build | Tick 20,000 fingerprint |
|---|---|---|
| macOS, aarch64, rayon parallel | release, rustc 1.99.1 | `11ea33d408844888` |
| Raspberry Pi 5 Linux, aarch64 | release, rustc 1.99.1 | `11ea33d408844888` |
| WASM, wasm32, single thread, node 22 | release | `11ea33d408844888` |

Configuration: `configs/default.toml`, seed 42. Commands are in [the runner README](../crates/sim-runner/README.md). The Pi was built from a copy of the working tree before the commit that contains this entry. Limits: one configuration and seed; x86-64 Linux is checked by CI, not recorded here; equality is measured, not proven for all inputs. Checkpoint files remain same-build only.

## Registered calibration: CAL-2026-11-16-FOOD-VALIDATION

[Held-out validation report](results/PHASE9A_VALIDATION_2026-11-16.md): 30/30 runs survived 200,000 ticks, with zero cap-blocked births. Rates 0.008, 0.010 and 0.012 all passed the predefined criteria; mean late populations across ten seeds were 913.92, 1067.22 and 1207.46 respectively (see JSON for full precision). The smallest acceptable rate, **0.008**, was approved by the owner together with cap 2,000 and applied to the defaults; current-engine revalidation subsequently passed (CAL-2026-11-17-BASELINE). Source `2b1c837`, pinned build `fab6214c28cf0b6d`, seeds 100–109, cap 2,000. This predates independent drought RNG, node identity fixes and memory; the subsequent pinned current-engine recheck is registered below. Full provenance, limitations, artifact checksums and reproduction command are in the report and linked JSON.

## Registered calibration: CAL-2026-11-17-BASELINE

[Current-engine recheck](results/CURRENT_BASELINE_RECHECK_2026-11-17.md): 10/10 seeds survived 200,000 ticks with zero cap blocking. Late population 893.629 (SD 12.5436; exploratory 95% t CI 884.656–902.602), lowest logged post-burn-in population 620. All four predefined engineering criteria passed for approved 0.008/cap 2,000. Source 0da954f, pinned build abad322241615ca2; reused seeds 100–109. All 30 artifact hashes independently verified; local summary equals the Pi finalizer. Sampled max temperature 75.7°C, no active throttling flags; 149 guarded cooling cycles. Calibration only: no equilibrium, memory-benefit or whole-batch second-machine replication claim. Full provenance and limits in report.

## Registered implementation check: CHK-2026-11-16-DELAYED-CUE

[Delayed-cue instrument report](results/DELAYED_CUE_INSTRUMENT_2026-11-16.md): a manufactured recurrent sign-persistence reference scored 2/2 balanced cues after 0, 1, 5, 20 and 100 blank ticks on the Pi. Feed-forward and same-network state-cleared controls scored 1/2 at each delay. Source `d35dd2c`; actual probe responses, build provenance and pre-execution protocol are linked. This validates a synthetic instrument, not evolved memory, learning, intelligence or ecological fitness. Exported-genome evaluation remains unimplemented.

## Registered implementation check: CHK-2026-11-16-EXPORTED-DELAYED-CUE

[Exported-genome negative-control report](results/EXPORTED_GENOME_ASSAY_2026-11-16.md): all 898 genomes from the prespecified seed-100 donor snapshot had no recurrent connections, scored 1/2 balanced cues, and produced identical intact/state-cleared responses. Evaluator source `aa6e844`, clean native release; donor source `0da954f`, tick 200,001 via a separate one-tick export bridge from the completed run. Full provenance, checksum summary and protocol are linked. This validates evaluation of evolved feed-forward exports; it is not evidence about memory benefits or cognition.

## Registered implementation check: CHK-2026-11-16-DELAYED-CHOICE-ARENA

[Controlled movement arena report](results/DELAYED_CHOICE_ARENA_2026-11-16.md): manufactured recurrent controller reward 1.0; feed-forward and same-network state-cleared controls 0.5, all choosing after five movement ticks. All 898 evolved feed-forward donors had identical opposite-cue and intact/reset paths: 7 chose left, 891 timed out. The timeouts do not identify a memory deficit; visible-cue/motor controls remain necessary. Clean source `0bc1be2`; full traces, protocol, plot, hashes and public raw-data archive are linked. Instrument validation only; no new evolutionary result.

## Registered implementation check: CHK-2026-11-17-VISIBLE-CUE-COMPETENCE

[Visible-cue diagnostic](results/VISIBLE_CUE_CONTROL_2026-11-17.md): manufactured recurrent, feed-forward and state-cleared controls all score 1.0 when the cue stays visible; delayed controls retain 1/.5/.5. All 898 natural feed-forward donors were evaluated unchanged in both conditions. Delayed: 7 correct, 7 wrong, 1,782 timeouts. Visible: 0 correct, 31 wrong, 1,765 timeouts. Intact/reset records match; the delayed behavior matches archived 0bc1be2 exactly. Clean source bda6f35; hashes/pairs/public raw outputs linked. Task competence is not established, so delayed failure cannot be interpreted as a memory deficit. This is an instrument diagnostic, not a memory-selection result.

## Calibration observations awaiting registration

The progress log describes post-sharing-fix Pi pilots and a separate demo tuning pass. They are useful leads for choosing calibration ranges, but are not promoted here until exact run metadata, configurations, raw outputs, and analysis are reconciled. Presentation tuning and experimental calibration must remain separate.

## Entry template

```text
ID:
Status: proposed / calibration / accepted / superseded / withdrawn
Question and preregistration:
Primary endpoint and smallest effect of interest:
Code commit and tree status:
Configuration and overrides:
Seeds and independent experimental unit:
Platform / compiler / build / execution mode:
Metric and schema versions:
Run paths, checksums, completion/extinction:
Analysis commit and command:
Controls, exclusions, censoring, multiple comparisons:
Estimate and uncertainty:
Replication:
Interpretation and limitations:
Primary-paper comparison and novelty boundary:
Reviewer and decision:
```
