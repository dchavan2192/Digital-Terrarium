# Which feedback connections sustained cue recall?

Three of 160 archived champions passed the retention criterion at every blank delay from zero through 32. All three had a positive recurrent connection from the turn output back to itself. Disabling just that edge reduced their passing-delay counts as follows. Correct responses to both visible cues after state reset persisted in each controller.

| Founder seed and arm | Intact delays passed | Without turn self-loop | Remaining passing delays |
|---|---:|---:|---|
| 6226, selected | 33 | 1 | 0 |
| 6320, fixed training | 33 | 2 | 0, 1 |
| 6320, variable training | 33 | 0 | none |

The self-loop feeds the previous turn activation back into the next turn calculation. These single-edge interventions show that each of these controllers depended on that connection for its observed long-delay performance. Other connections contributed too. For seed 6226, removing the recurrent turn-to-hidden-node-10 edge reduced success to five delays. Testing each removal against the original network leaves open whether any subset of edges suffices by itself. The two seed-6320 controllers share founders, so the three cases represent two independent founder seeds.

**Timing-sensitive controllers.** Among the six previously identified controllers that passed at 19 blanks, reversed at 20 and recovered at 21, five lost every passing delay when one particular negative recurrent edge was disabled. For two of these five it was a hidden-node self-loop (seeds 6212 and 6221). For three it connected the turn output to a hidden node (6207, 6225 and 6231). For the sixth (6201), removing a negative hidden self-loop reduced the number of passing delays from 16 to six. These are dependencies of the measured responses within the archived circuits.

**Scope and checks.** This exploratory analysis used all 96 wiring-study champions and 64 delay-training champions. Each eligible feedback edge was independently disabled in a fresh copy of its genome. We evaluated 476 single-edge removals plus intact and all-feedback-disabled controls across 33 delays (26,268 condition-delay records). Erasing all feedback made the two hidden responses identical in every case. The intact replay matched archived native hidden readouts and visible reset readouts in all 832 original champion-delay conditions. Each passing result requires a worst signed hidden margin of at least 0.1, identical hidden responses after boundary reset, and correct visible responses after reset. Delay zero means the probe immediately follows the cue. The finite test window bounds the inference about persistence.

The CSV includes both raw hidden and visible-reset turns, the affected edge's endpoints and weight, and the pass indicator. Run from the repository root:

```bash
python3 analysis/lesion_retention_edges.py output/retention_edge_lesions.csv.gz docs/results/retention_wiring_raw.tar.gz docs/results/delay_training_raw.tar.gz
```

[All lesion records](retention_edge_lesions.csv.gz) · [Original wiring study](RETENTION_WIRING_2026-11-23.md) · [Delay replay and training study](RETENTION_DELAY_TRAINING_2026-11-23.md)
