# Fixed terminal-progress tiebreak comparison — 2026-11-17

Status: prospective paired engineering pilot, frozen before either arm is run. This is a new method comparison, not an extension of the archived seeds2000–2002 pilot. No held-out validation, memory selection, ecology or within-life learning is tested.

## Question and independent unit

Does a declared terminal-distance tiebreak help the existing weight-only search acquire both fixed visible-cue responses under the same finite budget? Primary per-run outcome: final champion's true correct count (0,1,2), with explicit two-cue acquisition indicator. Report founder correct count, final population counts, first acquisition generation and every failed/partial search. Distance improvement without two correct is not competence.

Use fresh paired seeds **2100,2101,2102**, each evaluated in both modes. The independent comparison unit is the paired founder-population/search seed (n=3), not controllers, generations or cue trials. Verify identical founder genotypes by ID across arms, irrespective of rank order. Later parent and genotype histories may differ because selection differs. Do not claim n=6 independent initializations or calculate a confirmatory significance test from this small engineering pilot.

## Frozen implementation and inputs

Build the committed revision containing the hashes below in a clean isolated Pi worktree. Record that full source SHA in each batch before execution; it must be identical across arms. A later documentation commit may identify the exact code commit, but must not change code or inputs after any outcomes are seen. Pin executable/input/script hashes before launch and preserve them.

| File | SHA-256 |
|---|---|
| `crates/sim-runner/src/acquisition.rs` | `c1b4bf7b38e5c34be704431d193b46c4587e8e07c41d47c4d3aee75f9a1643ba` |
| `crates/sim-runner/src/delayed_choice.rs` | `35d48f791ae18c7610397c0d6459934fc6dce977cb2465cbf398318a02586301` |
| `configs/visible_progress_control.toml` | `bf979d245a715c86a5160dc15b5f67245e0bf3654ecb718dba7783bef495ce46` |
| `configs/visible_progress.toml` | `244decdcccbfb190501a979ff954d3be7a37dce431c9f9fe87892447085dea15` |
| `configs/delayed_choice.toml` | `7a47f36167419d92c0ab7414d11b4c399526dc90196fd64d7d938e1c6372f213` |

Scientific settings: 64 random20→8→7 feed-forward founders,25 replacements plus generation0,8 elites,tournament3 with replacement; native weight mutation probability0.1/step0.2. Topology, recurrence and scalar-trait mutation off; methylation zero/fixed. Native neutral colour drift remains unused. No injected solutions. Arena, sensor mapping, holds, visible cue, first-contact terminal rule, horizon20 and primary terminal rewards remain identical to the earlier pilot. No within-trial weight or reward updates.

## Only intervention: ranking among equal correct counts

- Control (`fitness="terminal_only"`): higher true correct count, then lower persistent ID.
- Treatment (`fitness="terminal_progress"`): higher true correct count, then higher mean terminal progress across the two cues, then lower persistent ID.

After motion terminates at first contact or horizon, measure distance from the terminal pose to the assigned correct goal's surface. Starting surface distance must be finite/positive. Per-cue progress is `clamp((start_surface_distance-terminal_surface_distance)/start_surface_distance,0,1)`; confirmed correct contact gives exactly1 to avoid floating boundary residues breaking ties among full successes. Use only terminal position, not closest approach or path-integrated progress. The goal answer enters this diagnostic only after termination, never sensors, neural state or an online reward stream. Report mean and both per-cue values separately from true rewards.

A higher distance score cannot compensate for fewer correct contacts. This teacher-informed genetic-selection objective is not policy-invariant reward shaping. [Research/design review](../research/TERMINAL_PROGRESS_ACQUISITION_2026-11-17.md) records limitations and primary literature. Distance guidance may mislead search; success is not predicted by the design.

## Execution, budget and failure rules

Execute control's three seeds first, then treatment's three seeds, using the same pinned binary. Order is declared now; timing is descriptive, not an algorithmic speed comparison. Each arm has its own fresh unique batch and full copied inputs, with its protocol copied as `visible_acquisition.toml` for the existing guarded supervisor. No other simulation may run concurrently. Preserve all old experiment binaries/checkpoints.

Each arm:64×26×3=4,992 candidate evaluations (including reevaluated elites),9,984 training trials plus18 control/champion replay trials, at most260,052 neural steps and486 retained trace samples. Two arms total9,984 candidate evaluations/520,104 neural steps. These are work bounds, not independent observations or a runtime guarantee. Existing validation caps each command at2M neural steps,1B conservative edge-steps and100k retained trace samples. Extra post-terminal distance arithmetic introduces no new neural evaluations.

One serial runner with `RAYON_NUM_THREADS=1`; existing batch-specific thermal guard pauses75°C/resumes65°C, polls5seconds and pauses for invalid sensor data. Tested supervisor requires a fresh matching cool guard, leaves its owned runner paused on guard loss/interruption, and imposes a1,800-second active-command watchdog including startup/cooling for each arm. Bounded guard cleanup is separately recorded. Thermal logs can be sparse for short runs; never infer an unsampled maximum or continuous throttling coverage. Record actual times, readings, flags and all safety events.

No early stopping on success, generation extensions, changed seeds, mapping flips, resource changes, topology additions or post-outcome tuning. A technical timeout/failure remains a technical incomplete run, not biological failure. Do not drop it from a successful cohort. Preserve partials, record any separately approved technical rerun, and do not declare the paired pilot complete without both full arms. Failed readiness/safety checks must not be bypassed.

## Analysis and artifacts

Verify all six per-seed summaries, root completion files,26×64 generation records per search, fixed genotype topology, rank/elite/parent/ID rules, champion consistency, identical paired founders, and independently recomputed per-cue progress from recorded terminal trajectories. Final intact/state-cleared trials must match for these feed-forward networks. Freeze the analysis script alongside the launch records before execution; keep true acquisition, partial training improvement and secondary progress distinct.

Report every search's founder/final correct count and final score distribution; first generation with two correct if any; treatment founder/final progress; all paired true-score/acquisition differences. Plot all six training curves without tick/gen-level confidence intervals. No test set is held out here and no acquired-memory claim follows. Champion sampling uses the declared training rank, never a post-hoc trial score.

Retain binary, clean source/build/compiler/target, exact protocols/arena, supervisor/guard/scripts, launch command/limits, thermal/status/exit records, all founders/final genotypes, every generation's IDs/parents/scores, and final visible traces. Intermediate full genomes are not claimed to be archived. Hash stable final files and publish the all-arm report/raw archive with exclusions stated. Any revised next method or generalization study requires a separate frozen design and fresh validation conditions. Do not silently replace the original null/partial pilot.
