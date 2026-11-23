# Systems audit and current research decision — 2026-11-23

Status: code, protocol and operations review; not an experimental result or exhaustive proof of correctness. Three agents independently reviewed scientific validity, memory implementation and demo/operations; the primary agent inspected their findings, ran checks and implemented the bounded corrections below. No new scientific campaign or Pi experiment was launched. Prior pinned data are unchanged.

## Finding: experimental alignment is the main gap

The evidence does not establish that evolved useful memory is absent, or identify a cause of its absence. We have not yet directly run task-directed recurrent acquisition with the controls needed for that inference.

- Both `acquisition.rs` and `diverse_acquisition.rs` deliberately disable recurrence and use visible cues. Their failures concern those search/task conditions, not memory capacity. The latest pilot retained all eight champions; only one seed pair solved all training responses.
- The long ecological contrast changed recurrence-add mutation pressure. More recurrent edges and lower population do not isolate retention, energy burden, harmful dynamics or mutation bias.
- Manufactured delayed-cue/movement controls demonstrate an implemented retention mechanism. They do not demonstrate evolution discovering useful memory.
- Poor visible navigation prevents attributing delayed **navigation** failures to memory. It does not make navigation or geometry generalization a prerequisite for every simpler retention assay.

The [latest pilot](../results/VISIBLE_DIVERSITY_PILOT_2026-11-22.md), [long ecological contrast](../results/BASELINE_RECURRENCE_LONG_2026-11-22.md) and [manufactured/donor competence check](../results/VISIBLE_CUE_CONTROL_2026-11-17.md) retain their original scope. No new biological, ecological, learning or consciousness claim follows from this audit.

## Engineering checks and corrections

| System | Evidence and finding | Action/status |
|---|---|---|
| Recurrent state | `brain.rs` reads previous activations; `sim.rs` stores state per organism, initializes newborn state empty and charges recurrent structure/activity. Memory tests cover inheritance, perturbation and exact checkpoint continuation. | No concrete fault found in reviewed paths; this is scoped evidence, not a proof over all configurations. |
| Retention intervention | Existing all-ticks clearing also removes response-phase recurrent dynamics. The [Nov17 council](SCIENCE_COUNCIL_2026-11-17.md) requested a single delay-end reset. | Implemented optional `--delay-end-reset` on `delayed-choice` and `evaluate-delayed-choice`, including matched visible-cue use. |
| Evaluation bounds | The added condition requires two extra trials per exported genome. | Opt-in budgets now count six trials/genome and eight manufactured-control trials; existing sample/edge limits remain. Defaults keep their old conditions/budgets. |
| Mutation interpretation | Nominal structural attempt rates do not prove zero realized drift. | Removed the default-config comment claiming any brain-size trend must reflect selection. Numeric parameters unchanged. |
| Ops handoff | NIGHT_TASK referred to missing DESIGN.md and forbade pushing despite later owner authorization. Old roadmap sections mix completed, proposed and active items. | NIGHT_TASK now points to this decision and current PROGRESS. Historical sections remain records, not launch instructions. Schedules stay disabled; the authorized one-off is complete. |
| Demo/evidence | Live UI distinguishes recurrence presence from useful recall, and saved native replays from live ecology. README still described only nine champions. | README now describes all17 across two studies. Task lab links this audit; no invented new-study replay or success metric was added. Browser/mobile interaction QA remains unverified. |
| Research direction | Earlier paper plan targets inherited methylation, distinct from recurrent activation state. | Marked the outline as a separate historical candidate; updated memory rollout status. Current sequence is below. |
| Reproducibility | Published complete pilot archive reproduces its audit from an empty directory. Ordinary CI lacks raw archives and skips source-byte replay tests; those checks passed locally with archives installed. | Keep that distinction explicit. Old pinned builds/configs are not silently replaced by current source. |

## Single delay-end reset: what changed

The optional condition zeros neural activations **once after the final held cue/blank tick and before the first movement inference**, including when blank delay is zero. It preserves position, heading, weights, connectivity and observations. New activations are stored normally afterward, so recurrence remains available throughout response. Existing intact and all-ticks-cleared conditions are preserved. This is an evaluator addition; frozen acquisition protocols and their selected champions are unchanged.

Whole-state zeroing removes cue-independent phase/context as well as cue information. A performance difference alone therefore establishes sensitivity to pre-response state. A cue-retention interpretation additionally needs balanced hidden-cue success, identical post-cue observations/pose, and matched visible/reset competence controls. Neither reset is an energy-matched intervention in an ecology.

Opt-in complete-export work is still bounded by100,000 retained samples. The old898-genome donor with the default arena exceeds that cap with six trials/genome; the command rejects it rather than sampling successes or increasing limits. Use a separately planned workload for future data collection.

## Current decision ladder

1. **Validate the retention instrument.** Manufactured recurrent, feed-forward and state-reset controls; frozen pose and identical hidden post-cue observations; boundary timing and response recurrence explicitly tested. This change advances that step.
2. **Prepare a small stationary delayed-readout acquisition protocol.** Specify representation, cue/readout mapping, recurrence/topology initialization and mutation, fixed delay/budgets/seeds, ranking and final champion rule before runs. Retain all searches, failures and generation-zero successes. Freeze champions before diagnostic evaluation. Include delayed intact/boundary reset, visible intact/same reset and broader all-ticks clearing. If recurrent topology is supplied initially, describe weight optimization on a recurrent architecture, not evolution discovering recurrence. No protocol or launch is authorized by this decision document alone.
3. **Test navigation transfer separately if claiming embodied performance.** The proposed stationary visible first-turn classifier is deferred: initial turn sign is not necessary for successful navigation (saved progress seed2102 initially turned right under both cues and still reached both assigned goals). Classification also leaves speed and later feedback unselected. Do not make that proxy another mandatory gate or infer why navigation fails from it.
4. **Test ecological information payoff separately.** A new ecological study needs calibration for the current diet/resource engine, temporal information with a specified benefit, controlled resource exposure and independent worlds. Earlier baseline studies used older engines. Generalization, ecological fitness, lifetime learning and consciousness remain different claims requiring different evidence.

This supersedes navigation/generalization-as-universal-gates in earlier handoffs. Preserve their dated experimental decisions; do not rewrite past protocols or reinterpret consumed test sets as fresh. The older broad roadmap and methylation paper outline are possible later branches, not an active unattended queue.

## Verification and remaining limits

Before edits, all184 workspace release tests passed. After the boundary-reset addition, all seven delayed-choice CLI tests passed: original controls/export paths, unchanged legacy evaluations, feed-forward equality, hold/input/pose identity, zero-delay timing and recurrence after reset. All187 post-change workspace release tests pass, along with all-target Clippy/format,25 Node tests (including installed raw-source checks),10 thermal-guard tests and10 supervisor tests. Parsed default configuration matches the previous commit exactly; comment corrections change no numeric setting. No optimizer training result is generated by these fixtures.

Read-only Pi check:53.45°C, no matching simulation/guard/supervisor processes. Thermal/supervisor safety tests are run locally; this does not continuously monitor hardware or establish that every failure mode is covered. The existing schedule policy remains disabled; this audit did not independently enumerate hosted scheduler state or change schedules. User authorization to commit/push verified development persists.
