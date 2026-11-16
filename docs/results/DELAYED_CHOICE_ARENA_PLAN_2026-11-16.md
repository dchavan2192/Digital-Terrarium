# Delayed-choice movement arena — 2026-11-16

Development protocol recorded before evolved-donor evaluation. The manufactured
reference and geometry were deliberately designed to provide a feasible positive
control; this is not an independent statistical preregistration. This controlled
behavioural task is an instrument check, not training, a natural ecology or
evidence of adaptive evolved memory. Arena tunables live in
`configs/delayed_choice.toml`.

## Task

A 20-input, seven-output controller starts at (0,0), heading pi/2. Two identical
circular goal markers are centered at (-4,4) and (4,4), radius 1.5. Five vision
rays observe marker distance/type; the correct-answer label never enters vision
or any scalar sensor. Marker appearance and geometry are identical for both cues.

Present a +1 or -1 signal cue for one tick, then five blank ticks. Throughout
cue and delay, hold position and heading fixed and discard **all** actions.
There is no velocity, signal deposit, pheromone, construction, changing reserve,
methylation update, moving marker, or other external state carrying the cue.
Only the controller's previous neural activations may retain it.

Release at exactly the same pose in both cue conditions. Signal input is now
zero and remains zero. Turn then move using the current action, max turn pi/16
and max speed 1. Use swept segment-circle contact to avoid skipping a marker
between samples. The first marker reached ends the trial, even if wrong; the
correct-answer label is consulted only for this terminal reward. Cue +1 rewards
left; cue -1 rewards right. Reward 1 means correct, 0 means wrong or timeout.
Timeout is 20 movement ticks and is reported separately from a wrong choice.
No reward signal is supplied before termination.

Neural state, pose and all task state reset before every trial. Both balanced
cue trials run for the intact controller and the **same** controller with
previous activations cleared before every tick. Full decision inputs/actions
and pose trajectories are retained. Heading feedback after release can carry
information from an earlier decision action; this is allowed. The claim is
retention through the immobile delay, not exclusively internal storage during
all later navigation.

## Instrument validation

Use a manufactured reference: cue channel 12 to turn weight 5, turn bias 2,
recurrent turn self-loop 5, speed bias 2. It must reach the matching marker for
both cues. The otherwise identical feed-forward reference and same-network
state-cleared control must take the same path for both cues and reach one
marker, scoring 1/2 rather than achieving a nominal chance score via timeouts.

Verify initial pose and first decision inputs are identical across cues; verify
every decision input/action/pose is identical across cues for feed-forward and
state-cleared controls. Check trial-order reset, goal visual symmetry, wrong
contact termination, explicit timeouts, swept collision and malformed configs.
A failure is an instrument failure to fix, not a biological null finding.

## Exported controllers and limits

Reuse the validated all-record export loader; do not silently select genotypes.
Require input layout to match arena ray count. Rebuild weights with frozen
snapshot methylation, ignoring living expressed brains and neural state. The
marks may include lifetime history; no epigenetic inheritance effect is isolated.
Bound total forward/edge work and trace samples before starting evaluation.
Record arena configuration, export/evaluator identities and every trial.

After the manufactured gates pass, evaluate the already prespecified 898-genome
feed-forward donor export from seed 100, tick 200,001, as a negative control.
Each genotype's two cue paths must match in each condition and intact/reset
results must match. Rewards may be 0 (timeout) or 1/2 (same goal for both cues);
no feed-forward genotype can exceed 1/2 under the balanced hidden-answer task.
These correlated genomes are not independent evolutionary replicates. No
significance test applies to these deterministic checks.

A low score on this fixed task does not rule out memory used elsewhere. A
successful manufactured reference does not establish learning, UAL or useful
memory in the ecology. Evolutionary ON/OFF studies remain gated on baseline
validation, recurrent-donor evaluation and a frozen study protocol. The active
Pi batch and its pinned executable remain unchanged during development.
