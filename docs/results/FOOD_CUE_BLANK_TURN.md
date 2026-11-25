# Did a default turn mask a usable cue response?

We measured each saved controller's turn after a blank cue at the same delay, then subtracted zero, half or all of that default turn from its response to each signed cue. This exploratory calculation changes the final motor command only. Each controller's recurrent trajectory is replayed intact. The correction uses a separate blank trial at the known delay and is supplied by the evaluator. It is not an evolved behavior. [Source experiment](FOOD_CUE_TIES.md)

| Random-tie training arm | Recurrent descendants | Correct opposite turns after full correction, both signed margins above 1e-8 at any training delay | Both margins at least 0.1 | Largest smaller margin after correction |
|---|---:|---:|---:|---:|
| Fixed delays | 904 | 115 | 0 | 0.0240 |
| Varied delays | 916 | 199 | 0 | 0.0525 |

A subset carried weak information with the correct sign in both directions. The existing 0.1 turn-margin criterion was met by no descendant at any training delay under any of the three corrections. That criterion measures strength of turn, not a necessary or sufficient threshold for eating.

We separately replayed the 16 locked random-tie champions on both held-out orders under predictive and shuffled cues. Before correction, and with half correction, each arm averaged eight correct turns at the 0.1 margin per 16 opportunities under either cue treatment. After full correction, neither arm had any such turns under either treatment. Every unmodified signed-cue output was checked against its native turn record. No new movements, meals or births were simulated here. Descendants share lineages; these counts describe two sets of eight matched populations.

The previous studies demonstrated a reproductive benefit in selected controllers and identified when it failed to generalize. This check closes the explanation that simply removing a default turn would reveal strong, correctly directed recall in this batch. It is a reasonable stopping point for the current one-step food arena. A future project would need a distinct question and success criterion. [Reproduce the check](../../analysis/food_cue_blank_turn.py)
