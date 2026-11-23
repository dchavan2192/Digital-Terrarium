# Can remembering where food will appear lead to more offspring?

A hand-built recurrent founder produced **seven direct offspring** when an early cue reliably predicted the next food patch and **three** when cues were shuffled. Clearing the same founder's neural state immediately before each choice gave **three offspring in both conditions**. Showing the cue again after clearing state restored the four-birth difference. This is a local calibration of a reproductive endpoint. Whether evolved organisms benefit remains open. [Prospective engineering plan](FOOD_CUE_ASSAY_PLAN.md)

| Controller intervention | Meals with predictive cues | Meals with shuffled cues | Direct offspring with predictive cues | Direct offspring with shuffled cues |
|---|---:|---:|---:|---:|
| Remember through the delay | 16 | 8 | 7 | 3 |
| Clear memory before choosing | 8 | 8 | 3 | 3 |
| Clear memory and show cue again at choice | 16 | 8 | 7 | 3 |
| Remove the feedback connection | 8 | 8 | 3 | 3 |

## What happened in the simulation

Each founder faced 16 meals over 152 native steps. A cue lasted one step, followed by 2, 5, 8 or 11 blank steps. The founder then moved once on an empty grid. One unit of algae appeared in the scheduled cell after that move. Pose was locked during the delay and meal. The two arms received food in the same cell, quantity and step at every opportunity. Within each delay the shuffled cue correctly identified the food twice in four events. Only the cue and a constant bias had sensory connections to the reference brain.

The founder kept its age and energy throughout. Each meal supplied 15 energy. Native reproduction required reaching 60 energy and transferred 30 to a newborn. We counted unique newborns directly from the simulation, checked their parent and birth tick, then removed them. On the primary schedule, eight additional meals supplied 120 extra energy and the founder produced four additional offspring. The paired birth difference after clearing state was zero. These are exact counts from one founder design on a predetermined schedule. Meals and offspring from the same founder are repeated observations.

## Verification and replay

The seed-6400 run used one local worker on Apple Silicon and a clean release build of `a5ef1308465823ddb78313e5b2ec2a6cb369e4ce`. The full configuration, controller overrides, compiler and target are in the manifest. Each event records the actual landing position, scheduled and consumed food, founder reserves and costs, and native newborn identities. The standalone auditor recomputes the meals, parent energy ledger, threshold crossings, birth counts and event timing. All 128 event records passed. [Manifest](food_cue_payoff_seed6400/manifest.json) · [Schedule](food_cue_payoff_seed6400/schedule.json) · [Event and birth records](food_cue_payoff_seed6400/result.json) · [Independent auditor](../../analysis/check_food_cue_payoff.py)

From the repository root:

```sh
python3 analysis/check_food_cue_payoff.py docs/results/food_cue_payoff_seed6400
RAYON_NUM_THREADS=1 cargo run --release -p sim-runner -- food-cue-payoff --seed 6400 --output output/local-food-cue
```

The primary result file has SHA-256 `1468bec6d5c7b2bb66856b6c437eb89a4172172692646ed98d5d8937d599806a`. A two-worker repeat reproduced it byte for byte.

We also checked 16 additional event orders (seeds 6401–6416) with the same founder and weights. Each audit passed. On all 16 orders the intact founder produced seven versus three offspring and clearing state gave a zero paired difference. The absolute count in both cleared-state arms rose to four on seeds 6407 and 6416. On those schedules the founder ate its eighth meal by step 117 and crossed the reproduction threshold before later maintenance costs were paid. These checks show why both food timing and quantity must be matched. They vary event order only and do not provide independent evolutionary replicates. The archive contains the complete schedule, manifest and results for each order and the two-worker repeat. [Order-check archive](food_cue_order_checks.tar.gz)

## Next scientific comparison

Transfer previously evolved controllers into the same arena locally. First verify that the transferred circuit reproduces its archived cue choices across the four delays. Preserve all feedback weights and record the movement and feeding rules supplied by the evaluator. Include failures to reach food and failures to eat. Measure each founder's predictive-minus-shuffled direct births with memory intact and cleared. Report paired changes by independent founding population, including controllers that failed the earlier recall task. The three sustained champions represent two founding populations and are suitable for checking transfer. Generalization calls for independent founders, and fitness in the open ecosystem requires competition and surviving descendants. Pi schedules remain disabled.
