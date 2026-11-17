# Visible-cue competence control — 2026-11-17

Development protocol recorded before implementing or evaluating the visible
condition. This follows the delayed-choice report's declared next step, not a
new confirmatory memory-selection study.

Keep the exact default arena, reference weights, geometry, signal channel,
held-pose cue/blank phases, movement horizon and terminal reward rule from
`configs/delayed_choice.toml`. Change exactly one intervention: in the visible
condition the original signed cue remains present during every movement tick.
In the existing delayed condition it stays zero after release. All other sensor functions, constants and state-reset rules stay identical.
Pose-derived sensor values may diverge after different actions change the path.

Expose an explicit `--cue-visible` option and record the mode in JSON. Default
commands must retain delayed-cue semantics. The manufactured feed-forward and
state-cleared references must solve both cues when visible, showing that the
arena is feasible without retention if the relevant information is available.
The recurrent reference should also solve both. The default delayed controls
must still score 1 / 0.5 / 0.5, without timeouts.

For the actual donor check, evaluate **all 898 records** of the same seed-100,
tick-200,001 export, in the same order, with snapshot weights/marks frozen. No
new donor selection, geometry, ray convention, horizon or reward adjustment.
Run the delayed and visible conditions on the same saved evaluator revision.
Compare per-genotype choices, timeouts, rewards and intact/reset correspondence.
These deterministic genomes from one donor population are not independent
replicates. Report descriptive paired differences without significance tests.

The visible response inputs must differ across cues only at the declared signal
channel for the same pose; no hidden correct-answer label enters observations.
Feed-forward intact/reset results must remain identical within each visibility
condition. Opposite-cue paths need not match in the visible condition because
current sensory information now legitimately differs. All held poses remain
fixed, and wrong first contact still ends the trial.

Failure even with a visible cue can reflect unfamiliar sensing, cue mapping or
motor requirements. A visible-versus-delayed difference is an information-
availability diagnostic, not learned association or ecological adaptation.
Do not tune until these natural donors succeed, and do not launch memory ON/OFF
ecological studies from this one-donor instrument check.
