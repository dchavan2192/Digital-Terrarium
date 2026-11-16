# Results register

Updated: 2026-11-16. This register separates implementation checks, calibration observations, and scientific results. A working mechanic is not evidence that selection favors it.

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

[Held-out validation report](results/PHASE9A_VALIDATION_2026-11-16.md): 30/30 runs survived 200,000 ticks, with zero cap-blocked births. Rates 0.008, 0.010 and 0.012 all passed the predefined criteria; mean late populations across ten seeds were 913.92, 1067.22 and 1207.46 respectively (see JSON for full precision). The smallest acceptable rate, **0.008**, was approved by the owner together with cap 2,000 and applied to the defaults; current-engine revalidation is pending. Source `2b1c837`, pinned build `fab6214c28cf0b6d`, seeds 100–109, cap 2,000. This predates independent drought RNG, node identity fixes and memory; current-engine validation remains necessary. Full provenance, limitations, artifact checksums and reproduction command are in the report and linked JSON.

## Registered implementation check: CHK-2026-11-16-DELAYED-CUE

[Delayed-cue instrument report](results/DELAYED_CUE_INSTRUMENT_2026-11-16.md): a manufactured recurrent sign-persistence reference scored 2/2 balanced cues after 0, 1, 5, 20 and 100 blank ticks on the Pi. Feed-forward and same-network state-cleared controls scored 1/2 at each delay. Source `d35dd2c`; actual probe responses, build provenance and pre-execution protocol are linked. This validates a synthetic instrument, not evolved memory, learning, intelligence or ecological fitness. Exported-genome evaluation remains unimplemented.

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
