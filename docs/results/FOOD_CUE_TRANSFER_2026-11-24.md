# Evolved controllers remembered the cue. Could they reach the food?

Three controllers that previously remembered a cue across 0–32 blank steps retained their turns when transferred to the two-patch arena. **None produced more offspring with reliable cues and memory intact than with memory cleared.** For two controllers sharing founder seed 6320, intact memory produced four offspring with reliable cues and two with shuffled cues. Clearing memory produced four in both environments. The seed-6226 controller never reached either patch. [Prespecified cohort and protocol](FOOD_CUE_TRANSFER_PLAN.md) · [Earlier calibration with a hand-built controller](FOOD_CUE_PAYOFF_2026-11-23.md)

## All paired lifetimes

Each entry gives **meals / direct offspring** from 16 food opportunities. Each controller supplied its own turning, speed and eating actions. The two seed-6320 controllers came from different training schedules in the same founding population.

| Donor | Intervention | Reliable cue | Shuffled cue |
|---|---|---:|---:|
| 6226 selected | Intact | 0 / 0 | 0 / 0 |
| 6226 selected | Clear memory at choice | 0 / 0 | 0 / 0 |
| 6226 selected | Clear and show cue again | 0 / 0 | 0 / 0 |
| 6320 fixed delay | Intact | 8 / 4 | 4 / 2 |
| 6320 fixed delay | Clear memory at choice | 8 / 4 | 8 / 4 |
| 6320 fixed delay | Clear and show cue again | 8 / 4 | 4 / 2 |
| 6320 variable delay | Intact | 8 / 4 | 4 / 2 |
| 6320 variable delay | Clear memory at choice | 8 / 4 | 8 / 4 |
| 6320 variable delay | Clear and show cue again | 8 / 4 | 4 / 2 |

**The two comparisons answer different questions.** For each 6320 donor the intact reliable-minus-shuffled birth difference was +2, and the corresponding reset difference was zero. Within the reliable environment, the intact-minus-reset difference was zero. Within the shuffled environment it was −2. The positive interaction came entirely from the cost of following the shuffled cue. The interaction and both within-environment differences were zero for seed 6226.

## Where transfer stopped

The two target cells start at coordinate y = 40. The founder started at y = 39.75 and had one step to commit. Seed 6226's intact landings ranged from y = 39.864 to 39.900, short of both cells. The seed-6320 controllers reached the right cell on every negative cue with memory intact. On every positive cue they turned left as expected but stopped short of the target row. In the reliable environment all eight right-side opportunities delivered a meal. In the shuffled environment only four did. After a reset each controller consistently reached one side, obtaining eight meals in either environment. All three founders ate whenever they reached a cell containing food.

This identifies movement distance as the immediate limit in this arena. Re-showing the cue reproduced the intact pattern. A next local comparison can vary the starting distance symmetrically while preserving the original schedule, all donor actions, and the present distance as a control. It should report turns, landings, intake and births for both sides and retain every donor. These are three controllers selected for prior recall performance, from two founding populations. The case series gives no estimate of how frequently memory increases reproduction in independently evolved populations or in the open ecosystem.

## Source and replay

The three published genomes were selected by archive digest, founding seed and training arm. The runner preserved all connections and output actions, moved the single input to the native cue channel, and standardized body size to one. Preflight compared all seven action outputs and neural activity on every step through feeding, across cue signs, delays and interventions. The independent auditor compared every observed turn to the original stationary controller and rebuilt the food and energy ledgers and parent-linked births. All 18 lifetimes completed 152 ticks. All 288 events passed.

The pilot ran locally with one worker and a clean release build of `3d0c434f585c41a749efa274fda1b6233f462e03`. The archive contains the three original genomes, source and binary hashes, effective configurations, realized schedules, every event and a compact summary. Its SHA-256 is `b1303de9464c0d691960b9ee28b287f3d2c98c01d20e22e86e5b5a09786d955f`. [Complete pilot archive](food_cue_transfer_raw.tar.gz) · [Runner](../../analysis/run_food_cue_transfer.py) · [Independent audit](../../analysis/check_food_cue_payoff.py)

Run from the repository root using a new output directory.

```sh
cargo build --release -p sim-runner
RAYON_NUM_THREADS=1 python3 analysis/run_food_cue_transfer.py --seed 6400 --output output/local-donor-transfer
```
