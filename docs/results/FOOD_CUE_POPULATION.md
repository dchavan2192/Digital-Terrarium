# Remembering increased births in one of 31 additional founders

At the close food distance, **1 of 31** previously evolved controllers produced more direct offspring with memory intact than after its memory was cleared immediately before choosing. Twenty tied and ten produced fewer offspring. All founders saw the same reliable cues and food schedule in this comparison. This cohort includes the selected-arm champion from every wiring-study founding population except seed 6226, which informed the earlier distance pilot. Inclusion was independent of earlier recall performance. [Frozen protocol](FOOD_CUE_POPULATION_PLAN.md) · [Prior distance study](FOOD_CUE_DISTANCE.md)

| Starting gap | Comparison | More offspring | Same | Fewer |
|---|---|---:|---:|---:|
| 0.0625 | Intact versus reset under predictive cues | 1 | 20 | 10 |
| 0.0625 | Intact versus reset under shuffled cues | 0 | 25 | 6 |
| 0.0625 | Predictive difference minus shuffled difference | 1 | 25 | 5 |
| 0.25 | Intact versus reset under predictive cues | 1 | 20 | 10 |
| 0.25 | Intact versus reset under shuffled cues | 0 | 24 | 7 |
| 0.25 | Predictive difference minus shuffled difference | 1 | 26 | 4 |

Seed 6200 was the sole beneficiary at both distances. With predictive cues it ate ten meals and produced four offspring. After a reset it ate eight meals and produced three offspring. Its two additional meals both followed a left cue and a two-step delay. Under shuffled cues both memory conditions produced three offspring. This founder had failed the earlier five-blank test.

## Where recall and feeding came apart

At both distances, the eight earlier five-blank passers had zero gains, three ties and five losses with predictive cues. The 23 earlier nonpassers had one gain, 17 ties and five losses. To understand the passers, we looked back at their intact, predictive trials at the close gap. Each row pools four scheduled opportunities from each of those eight founders. These event counts describe behavior within founders.

| Blank steps after the cue | Opportunities | Turned toward food | Landed on food | Ate |
|---:|---:|---:|---:|---:|
| 2 | 32 | 0 | 0 | 0 |
| 5 | 32 | 32 | 26 | 8 |
| 8 | 32 | 0 | 0 | 0 |
| 11 | 32 | 32 | 26 | 8 |

Two limits are visible. Turn direction depended on the delay. Even at the trained five-step delay, six correct turns fell short of the food cell and 18 correct landings yielded no meal. Correct recall alone was an incomplete measure of foraging competence. A next local comparison can test whether training across several delays improves the full sequence from turning through eating and reproduction, using independently seeded populations and the same controls.

The **1/31** is descriptive of these archived champions. They share a fixed-delay training regime and were evaluated on one food schedule, with solitary founders and direct newborns as the endpoint. The earlier three-controller pilot deliberately selected sustained recall. Its three cases should not be pooled with this cohort to estimate prevalence.

## Verification and replay

All 31 founders completed six lifetimes at each of two distances. The independent event audit checked 372 lifetimes and 5,952 food opportunities, including realized positions, landed cells, consumption, energy and direct newborns. The runner pinned the input archive and earlier recall classifications, verified matched schedules, and compared each donor’s turns and movement displacements across distances. The local single-worker release run used clean source `8940b62`. A second audit from the extracted output archive reproduced every donor outcome and cohort count. [Raw schedules, donor identities, per-founder contrasts and events](food_cue_population_raw.tar.gz) · [Study runner](../../analysis/run_food_cue_population.py)

The output archive has SHA-256 `9240a2a0d2fcbc8b27a7ff8ef7fceb3ac03c46e7efd0b13b7f03593fed326b05`. To reproduce the study, use a new local output directory.

```sh
cargo build --release -p sim-runner
RAYON_NUM_THREADS=1 python3 analysis/run_food_cue_population.py --output output/local-food-cue-population
```
