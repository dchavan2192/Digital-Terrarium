# Delayed-cue instrument check — 2026-11-16

Protocol written before the instrument's Pi execution. This is a controlled
check of a manufactured network, not an evolutionary experiment or evidence
that organisms have learned, use memory adaptively, or meet a UAL criterion.

## Inputs, reference and controls

Use the engine's `Brain` evaluator directly, with one cue input and seven
outputs. The turn output is the decision variable. The reference has a
feed-forward cue→turn connection of weight 2 and a recurrent turn→turn
connection of weight 2. No weights change during a trial.

A trial presents cue -1 or +1 for one tick, then N blank ticks, then an all-zero
probe tick. Read the turn output only at the probe. The input at every blank and
probe tick is identical. There is no time/probe input or environmental state
that reveals the cue. Start every trial with fresh zero brain state.

Score a nonnegative probe response as a positive cue; score a negative response
as a negative cue. The fixed zero tie predicts positive, so a cue-independent
zero output scores 1/2 on the balanced pair. Accuracy is correct trials / 2.

Run these controls with identical cue order and delay:

1. Intact recurrent reference: expect 2/2 correct.
2. Feed-forward control, same weights with the recurrent edge removed: expect 1/2.
3. Same recurrent reference, previous activations cleared before every tick:
   expect 1/2. This isolates stored state from network topology and weights.

The low-compute Pi check covers delays 0, 1, 5, 20 and 100. Report actual probe
responses and all control accuracies, not only a pass flag. These deterministic
trials are not independent evolutionary replicates, so no significance test or
sampling confidence interval applies. Failure is an instrument failure to fix,
not a negative evolutionary result. The reference demonstrates persistence,
not learned temporal gating or associative learning.

## Execution and limits

Run the instrument in a separate pinned build directory, with a unique output
folder and source/compiler/command records. Do not rebuild the active ecological
batch executable. Keep CPU use to one thread. The short command should finish
in seconds; no additional sustained evolutionary load is added to the Pi.

This check is a first instrument step. Applying an assay to exported evolved
genomes, preventing answer leakage through all world sensors, and assessing
within-genome memory-reset controls still require implementation and validation
before a memory-selection study can be interpreted. The current-engine ecology
baseline must also pass its full-duration acceptance gate.
