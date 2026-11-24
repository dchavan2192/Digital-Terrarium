# Selecting for births favored a one-sided forager

Selecting for direct offspring produced champions that ate more and reproduced more on fresh food schedules. **None of the 32 champions gained offspring from retaining the cue.** Eight independently founded populations each supplied four matched training arms. Within each population, the two birth-selected arms chose the same original founder as champion. [Frozen protocol](FOOD_CUE_SELECTION_PLAN.md)

## Held-out performance

Means below are per 16-opportunity lifetime, with intact state and reliable cues. Each locked champion faced both fresh event orders at four unseen delays. The final column compares the same champion with its own memory cleared immediately before each choice. All eight paired differences were zero in every arm. The birth differences under shuffled cues and the predictive-minus-shuffled interactions were also zero for every champion, averaged across the two orders.

| Training delays | Ranked by | Correct turns | Food-cell landings | Meals | Direct offspring | Birth gain over reset |
|---|---|---:|---:|---:|---:|---:|
| Fixed | Turn | 8.25 | 8.25 | 2 | 1 | 0 |
| Varied | Turn | 7.75 | 7.75 | 2 | 1 | 0 |
| Fixed | Offspring | 8 | 8 | 8 | 3.8125 | 0 |
| Varied | Offspring | 8 | 8 | 8 | 3.8125 | 0 |

At either timing regime, birth-selected champions produced more offspring than turn-selected champions in six populations, tied in one and produced fewer in one. The mean paired difference was +2.8125 offspring. Fixed versus varied training gave identical per-population birth outcomes within each objective. The engineered memory reference produced seven offspring with reliable cues and three after reset on both held-out orders. The constant-side reference produced three. That control confirms that the environment could reward recall.

## Why higher reproduction did not require recall

The founder score check passed for all eight populations in both timing regimes. Founders ranged from zero to four direct births. In every population at least one initial feed-forward founder already scored four. The birth-selected winners consistently chose one side, where a meal appeared half the time, and ate on all eight rewarded landings.

New recurrent connections did arise. In **each** birth-selected timing arm, 1,118 of 1,152 descendants matched the best training score of four births, including 346 descendants with an expressed recurrent connection. No descendant exceeded it. The specified tie-break favored older candidate IDs. All eight birth-selected champions therefore came from the initial population, and the two timing arms selected the same eight genomes. This trial shows that the birth endpoint distinguishes feeding founders, but separates very few of their descendants once the one-sided policy is common. It does not establish that longer searches or other tie-breaking rules would also fail.

The next informative test is a matched comparison of the existing oldest-first tie rule with random survival among equally reproductive candidates. Preserve the primary offspring score, independent founding seeds, and held-out reset and shuffle comparisons. Measure how often recurrent descendants survive a tie and later surpass four training births. The current pilot supplies no evidence for expanding the same configuration on the Pi.

## Records and reproduction

The single-worker local study used clean source `3e9ae02`. Each arm retained 13 population rankings (founders and 12 generations), with all 160 candidate records and parental identities. The archived evaluation contains all 32 genomes, both held-out orders and all six cue/state conditions per champion (6,144 opportunities), plus reference controls. The independent audit checked founder matching, gate calculations, native turn scores, rankings and champion locks. It reconstructed held-out meals, energy balances and parent-linked births from the event records. The complete archive was extracted and rechecked. SHA-256 `30d2d1d18aa72c87a218c9265d245c7e9a48aeeca729c295369759e91cd54843`. [Study runner](../../analysis/run_food_cue_selection.py) · [Complete archive, including per-seed contrasts](food_cue_selection_raw.tar.gz)

```sh
cargo build --release -p sim-runner
python3 analysis/run_food_cue_selection.py --output output/local-food-cue-selection
```
