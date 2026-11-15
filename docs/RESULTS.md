# Results register

Updated: 2026-11-15. This register separates implementation checks, calibration observations, and scientific results. A working mechanic is not evidence that selection favors it.

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
