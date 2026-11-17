# Fixed visible-cue acquisition feasibility pilot — 2026-11-17

Status: **plan fixed before the first actual pilot; not launched by this document**.
This is a bounded engineering search-feasibility pilot. It is not a confirmatory
memory experiment, held-out validation, ecological selection result or test of
learning within an organism's lifetime. The earlier 898-controller transfer
results remain unchanged and are not used to select training founders.

## Question and fixed source

Can the existing native weight-mutation/search procedure discover the fixed
visible-cue movement response within the declared finite budget, and does its
best final training score improve on the best randomly generated founder?
A successful random founder is reported separately from acquisition through
selection. Failure to improve, failure to acquire both responses, and technical
interruption are valid distinguishable outcomes; none licenses retuning this
pilot or declaring that the controller architecture cannot solve the task.

Execution source: **`5504a20746bfe0dfcff54b95546324c02db1058d`** (`5504a20`).
The implementation is `crates/sim-runner/src/acquisition.rs`, with SHA-256
`765055f3aeed2a3e2a7b72f707a67815e655d29dd80d35bbc68eb50b76a589ef`.
Pin and retain the Pi executable built from that source before execution; do
not replace an executable from any earlier experiment. Tracked source must
match this revision. Record the complete Git status and build dirty flag;
untracked operational files, if present, must be identified rather than
silently relabeling a dirty build as clean. The plan's later documentation
commit does not change the declared implementation revision.

## Unchanged protocol and arena

| Parameter | Fixed value |
|---|---|
| Protocol file | `configs/visible_acquisition.toml` |
| Protocol SHA-256 | `77e2a98d02c77a7eee05f6d571c9ad80f599e5bc5d0fac41d36a16a698407fce` |
| Arena file | `configs/delayed_choice.toml` |
| Arena SHA-256 | `7a47f36167419d92c0ab7414d11b4c399526dc90196fd64d7d938e1c6372f213` |
| Seeds, in execution order | 2000, 2001, 2002 |
| Population | 64 |
| Replacement generations | 25, after the separately recorded founder generation 0 |
| Elites | 8 |
| Tournament | 3 samples with replacement |
| Controller | 20 inputs, 8 hidden nodes, 7 outputs; native random founders |
| Weight mutation | Per-gene probability 0.1, uniform perturbation step 0.2 |
| Structural/trait mutation | Add-edge, split-node, toggle, prune, recurrence and scalar-trait probabilities all zero |
| Methylation | Zero initially and fixed; no within-life updates |
| Execution | Serial, one declared pilot at a time on the Pi |

Native reproduction retains neutral colour-hue drift, which is not sensed by
this arena. Thus this is weight-only **controller** optimization, not a claim
that every other serialized genotype field is constant. There is no crossover,
speciation, metabolism, body scaling, energy selection or ecological simulation.
No manufactured reference weights are injected into the candidate population.

Keep the existing arena, signed signal cue channel 12, left/right convention,
goal appearance, geometry, scalar constants, movement limits, horizon and
terminal reward unchanged. Each evaluation starts two fresh trials, cues -1
and +1. Cue presentation lasts one immobile tick, followed by five immobile
blank ticks. At release the signed cue is restored and stays visible throughout
movement. Position and heading are fixed during the hold phases, whose actions
are discarded. First swept contact ends the trial; +1 rewards left and -1
rewards right. Correct contact earns 1; wrong contact or timeout earns 0.
The movement horizon is 20 ticks. There is no distance shaping or hidden-cue
training/evaluation in this pilot.

## Selection, endpoints and stopping

Fitness is the integer number of correct first contacts across the two visible
trials: 0, 1 or 2. Selection ranks higher fitness first, then lower persistent
candidate ID. Tournaments choose the best rank among three samples with
replacement. Copy the eight elites and create 56 mutated offspring per
replacement generation; elites are evaluated again. The final champion is the
best member of the final population under the same score/ID rule.

For **each** seed report founder best and final best mean reward, whether both
cues were correct at founding and at the end, improvement over founders, final
population score distribution, all generation scores, and champion contacts,
wrong choices, timeouts and movement traces. Final champion trials replay the
training conditions; they are not a test set. A final mean reward of 1 (integer fitness 2) establishes
only success on these two fixed training trials. There is no prespecified
population success-rate threshold or significance test for three pilot runs.

The independent repeatability unit is the separately seeded founder-population
search run. Candidates, descendants, generations and cue trials are nested,
correlated observations. Report all three runs and all technical failures;
do not form a post-hoc successful cohort, select the best test-performing seed,
flip cue labels, extend generations, change weights/geometry/horizon, add
shaping or replace unfavourable seeds. No early success stopping. Any revised
acquisition method requires a separate development record and fresh planned
validation, rather than rewriting this protocol.

## Work bounds and operational safety

The fixed settings entail:

- **1,664 candidate evaluations per seed**, including founders and repeated
  elites; **4,992 total**.
- Two visible trials per candidate: **9,984 training trials**.
- Six manufactured-control trials before search and four final champion replay
  trials per seed: **18 additional trials**, included in work validation.
- At most 26 neural steps per trial; **260,052 neural steps in total**, including
  controls and champion replay. Contacts may terminate trials earlier.
- **486 retained trace samples** under the runner's conservative
  `(26 + 1) × 18` accounting; training evaluations allocate no step traces.
- The runner's conservative stored-gene allowance is 306 genes for these
  settings, giving **79,575,912 edge-steps**. This intentionally overestimates
  the unchanged weight-only topology. The generic command rejects over 2M
  neural steps, 1B edge-steps or 100k retained trace samples before artifacts.

These operation bounds are **not a measured wall-clock estimate**. No duration
in seconds or minutes is asserted here; runtime depends on the pinned release
build, hardware and thermal pauses. Before launch, record the chosen operational
wall-clock/watchdog limit and whether thermal pauses count toward it. A safety
limit ending the run is a technical incomplete/censored execution, not a failed
biological search result or permission to append extra generations. Preserve
partial files and original IDs; any technical rerun must be logged separately
with the same source and unchanged settings.

Confirm the overnight batch is complete and no simulation is active before
starting; preserve all old runs. Run serially with one thread and a batch-specific
thermal guard targeting only this pinned executable. Retain the established
75°C pause / 65°C resume policy and thermal log; missing/invalid temperature
must pause safely. Record real observed start/end times, pause history, active
throttling flags and measured elapsed time. Stop or leave safely paused if the
sensor/guard fails. Do not run an additional evolutionary campaign alongside
this pilot or perform destructive cleanup to make space.

## Artifact retention and reproducibility

Create a new experiment root containing the pinned executable, exact protocol
and arena copies, source revision/status, build/compiler/target metadata,
launcher command and operational limits, launcher stdout/stderr, exit status,
thermal guard/status and thermal readings. Run the saved binary with:

```bash
RAYON_NUM_THREADS=1 ./sim-runner acquire-visible \
  --protocol visible_acquisition.toml \
  --arena delayed_choice.toml \
  --output runs
```

The command creates a unique session. Preserve its `manifest.json` (resolved
protocol/arena, embedded founder defaults and provenance), `controls.json`,
root `summary.json`, and all `seed_2000`, `seed_2001`, `seed_2002` directories.
Each seed retains `founders.json`, `generations.jsonl`,
`final_population.json`, `innovations.json` and `summary.json` when completed.
All founder/final genomes and generation score/parent records remain available;
intermediate full genomes are not claimed to be archived. Preserve incomplete
files and error records for technical failures. The runner attempts remaining
declared seeds after a caught failure and exits nonzero if any fail.

Require all three seed summaries and a root completed summary before reporting
full completion. `manifest.json` retains its initial `started` status even after
success: it is a launch record, not the completion oracle. An absent, malformed,
failed or incomplete final summary is not a completed pilot. Validate 26
logged generations per completed seed and 64 score records per generation;
check scores and declared evaluation counts against the full population files.

After artifact writes finish, produce a SHA-256 inventory over the pinned
executable, copied inputs, source/build/launch records, all session JSON/JSONL,
logs and thermal records. Store the inventory beside, rather than inside, any
archive whose hash it records. Preserve a reproducible raw archive and its
SHA-256 with the report; never hash an actively changing file as though final.
Any published numerical claim must link the all-seed report and provenance in
`docs/RESULTS.md`. No successful pilot score alone establishes generalization,
retention, adaptive ecological memory, consciousness or within-life learning.

## Readiness assessment

Read-only review at source `5504a20` finds the previously identified fruit-field
compile error and controls/champion trace-budget omission corrected. The fixed
weight-only configuration, trace-free scoring equivalence tests and retained
failure semantics are suitable for this bounded feasibility pilot. This review
did not rebuild or execute the experiment. Launch still requires the operator
to verify the idle Pi, pinned build/inputs, watchdog choice, available storage
and working batch-specific thermal guard. No new scientific design blocker was
identified; these operational checks must not be mistaken for completed work.
