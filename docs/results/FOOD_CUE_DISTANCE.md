# Remembering the cue increased births when both food patches were within reach

At the closest starting distance, all three previously evolved controllers ate at all 16 food opportunities and produced **eight direct offspring each** with reliable cues. Clearing memory before the choice reduced each founder to eight meals and four offspring. Shuffling the cues with memory intact also gave each founder eight meals and four offspring. Food timing, location and quantity were identical across these conditions. [Study plan](FOOD_CUE_DISTANCE_PLAN.md)

## Births with a reliable cue

Each cell gives direct offspring **with memory intact / with memory cleared**. Distance is the starting gap to the edge of the food row, in simulation world units. Each founder had one native movement step to get there.

| Starting gap | Seed 6226, selected | Seed 6320, fixed-delay training | Seed 6320, variable-delay training |
|---|---:|---:|---:|
| 0.0625 | 8 / 4 | 8 / 4 | 8 / 4 |
| 0.125 | 4 / 4 | 8 / 4 | 8 / 4 |
| 0.25 (original) | 0 / 0 | 4 / 4 | 4 / 4 |
| 0.5 | 0 / 0 | 0 / 0 | 0 / 0 |

The earlier transfer found that controllers could turn correctly and still stop short of food. This study moved the start equally close to the two patches and kept each controller’s actions intact. The two seed-6320 controllers first showed a birth advantage at 0.125 units. Seed 6226 required the closer 0.0625-unit condition. At the farthest distance every intact founder missed every meal. When the cue was shown again at choice after clearing memory, the fixed-delay seed-6320 controller reached eight meals and produced four offspring even at that distance. The raw archive includes both cue conditions and all three interventions at every distance. [Original transfer study](FOOD_CUE_TRANSFER_2026-11-24.md)

## What this establishes

Within these three controllers, remembering could increase reproduction when the remembered turn carried the organism far enough to eat. The same founder still paid native movement and neural costs and reproduced through the normal energy threshold. This is a direct reproductive benefit in a controlled arena. The cases were selected for sustained recall and represent two founding populations. The distances were chosen after observing earlier shortfalls. The study does not estimate the frequency of the benefit or establish that selection in a competitive ecosystem favors memory.

## Reproduce the comparison

The study ran locally on schedule seed 6400 from clean commit `a4d554f79eb36988bd434f2e8456aaaeff884dc6`. The raw archive has SHA-256 `96ab947ea828a837a62c93e6d2c7e42c05ddabca871549a22bd89df4bd20ede6`. The independent checker reconstructs landings, meals and direct births, checks identical cue and food schedules and compares realized turns and displacements across distances. The original-distance control reproduces the earlier archived event records. There are 72 founder lifetimes and 1,152 scheduled food opportunities. [Runner and cross-distance audit](../../analysis/run_food_cue_distance.py) · [Full event records, manifests and summaries](food_cue_distance_raw.tar.gz)

```sh
cargo build --release -p sim-runner
RAYON_NUM_THREADS=1 python3 analysis/run_food_cue_distance.py --binary target/release/sim-runner --output output/local-food-cue-distance
```

The next comparison should lock the closest and original gaps and evaluate independently founded controllers, including ones that failed the earlier recall task. The unit of replication is the founding population. Report the reliable-cue intact-minus-reset birth difference for each, alongside the shuffled-cue comparison.
