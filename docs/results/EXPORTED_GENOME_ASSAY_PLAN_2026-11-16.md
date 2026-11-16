# Exported-genome delayed-cue evaluation — 2026-11-16

Implementation and negative-control plan, fixed before inspecting the selected
export. This is an analysis instrument, not a memory-selection study.

## Evaluation semantics

Accept a version-1 `terrarium-genomes` export with its original configuration,
source core build, target and tick. Validate configuration, dimensions, unique
organism IDs and every inherited genome before building a controller. Bound
file size and record/topology counts; malformed data fails without a partial
result. Keep a content fingerprint in the result and archive an external SHA-256.

Rebuild each inherited controller from its connection genes and their exported
snapshot methylation. These marks may reflect the living donor's lifetime
history; they are not assumed to be pristine birth or inherited-only marks. Ignore `expressed_brain` and any live organism activation state:
those may reflect a refresh lag and a different point in the creature's life.
Do not alter weights or marks during evaluation.

Require a declared zero-based cue channel. For the actual negative control,
use the signed broadcast-signal input, index `2 * ray_count + 2` (12 with five
rays). Clamp all other inputs to zero. Cue -1 or +1 lasts one tick, followed by
five zero-input blank ticks and one zero-input probe tick. Every trial starts
with zero state. Score only turn at the probe; negative predicts -1 and zero or
positive predicts +1. There is no probe/time sensor, movement, energy change,
stress update, reproduction, or input feedback. The zero-input vector is an
artificial clamp, not a natural ecological observation.

Evaluate the same rebuilt controller both with intact previous-tick activations
and with those activations cleared on every tick. Report all records in export
order, including organism/generation/root identifiers, enabled recurrent count,
both balanced-trial accuracies and actual cue/probe responses. No best-genome
selection, post-hoc sign inversion or cue-channel search is allowed for this
check. Genomes in one donor world are correlated; neither creatures nor repeated
deterministic trials are independent evolutionary replicates.

## Validation gates

Test a 20-input manufactured persistent controller, its same-network state-reset
control, and a feed-forward exported controller. Test methylation suppressing
the memory edge, fresh state and malformed formats/IDs/edges/dimensions. Preserve
the existing manufactured one-input assay's output and the ecological dynamics.

For one actual evolved negative control, choose seed 100, the first declared
recheck seed, from completed batch `validation_20261116T171843Z_28985`. Use its
pinned executable and final 200,000-tick checkpoint, resume exactly one tick into
a fresh output directory and export **all living genomes at tick 200,001**.
This extra tick is an export bridge because the pinned runner requires a resume
target greater than checkpoint tick; it does not alter the original run or its
validation measurements. Record checkpoint/export SHA-256 and source metadata.
The world has memory mutation off, so enabled recurrent counts must be zero,
intact/state-cleared outputs must match and balanced cue accuracy must be 1/2.
A difference is an instrument failure, not evidence of memory.

This single-donor check validates parsing and state-reset controls on evolved
feed-forward networks. It does not estimate how often memory evolves or whether
it helps. Applying the assay to memory-enabled evolved donors and designing a
memory-requiring environment remain separate gated tasks after the current
baseline recheck.
