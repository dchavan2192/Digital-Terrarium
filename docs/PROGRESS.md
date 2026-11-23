# Terrarium — Progress & Continuation Doc

**Purpose**: Living document that tracks all work, decisions, and next steps. Read this first at the start of every session so you can continue without re-explanation.

**Last updated**: 2026-11-23 UTC

---

## Local food-cue and offspring calibration

The delayed food-cue arena is implemented, tested and replayable. The engineered founder had seven direct offspring with predictive cues and three with shuffled cues. Clearing its memory gave three in both arms on the primary schedule. Sixteen additional order checks passed. The demo Task lab includes the paired meals and births. Next, verify archived evolved controllers retain their original responses when moved into this arena. No Pi run was launched. [Report, raw events and replay instructions](results/FOOD_CUE_PAYOFF_2026-11-23.md)

## Feedback subset replay

Exhaustive replay of the three sustained champions found one minimal sufficient feedback subset per controller. The turn self-loop sufficed alone in seed 6320 variable. The other two also required turn-to-hidden feedback. [Report and data](results/RETENTION_FEEDBACK_SUBSETS.md).

## Feedback connection replay

All three champions passing every delay from 0 through 32 depended on a positive turn self-loop. Removing it left 1, 2 and 0 passing delays respectively, with visible cue responses still correct. Three controllers span two founder seeds. Full single-edge interventions and intact controls are archived in the [lesion report](results/RETENTION_EDGE_LESIONS.md).

## Delay replay and paired transfer study — 2026-11-23

Six of the seven prior champions passing at 19 blanks reversed both cue choices at 20 and recovered at 21. Seed 6226 passed at every delay from 0 through 32. In 32 fresh matched populations, only seed 6320 passed all four unseen delays, in both training arms. Ranks and responses independently verified. Both studies ran locally. [Report and raw records](results/RETENTION_DELAY_TRAINING_2026-11-23.md).

## Recurrent wiring discovery — 2026-11-23

Nine of 32 fitness-selected populations starting with feed-forward controllers produced a final champion passing the five-blank retention criterion. Both controls scored 0/32. Adjacent delays show parity sensitivity. All runs completed locally. [Report and raw data](results/RETENTION_WIRING_2026-11-23.md).

## Paired retention selection and transfer — 2026-11-23

All 32 local matched-founder pairs completed. Selection increased final five-tick recall prevalence by 92.1875 percentage points versus random survival. Among ten founder-negative pairs, nine selected and five neutral populations discovered a solution. Only 11/32 selected champions passed at 20 blanks. Delay length and parity are confounded. Discovery of useful recurrent wiring remains open. [Report, protocol and raw histories](results/RETENTION_SELECTION_2026-11-23.md).

## Stationary retention pilot completed — 2026-11-23

Three of four final circuits retained a signed cue across five blanks and met the matched reset and visible-cue criteria. Two populations had successful founders. Seed 6003 first succeeded at generation one. Local native run, complete histories archived and independently reconstructed. Future work can compare selection against neutral search with matched founders and test fresh delays. [Full report](results/STATIONARY_RETENTION_2026-11-23.md).

## Current systems audit and retention-control development — 2026-11-23

**Current decision:** follow the [systems audit](research/SYSTEMS_AUDIT_2026-11-23.md). All older sections below are dated history; active-run labels, parallel-worker advice and unchecked roadmaps are not current launch instructions. The one-off pilot is complete; Pi schedules remain disabled. No new Pi experiment was launched during this audit.

- Three independent agents reviewed science, memory internals and demo/operations. No concrete recurrence propagation/newborn reset/checkpoint/accounting fault found in reviewed paths. Main gap: existing task-directed acquisition deliberately disables recurrence, so its results do not diagnose evolved memory.
- Corrected the decision path: navigation competence gates interpretation of navigation failures, not every stationary retention assay. Defer the new first-turn classifier; a successful archived policy initially turns right for both cues. Next is a prospectively specified stationary delayed-readout acquisition protocol after instrument validation, with all founder successes/partials retained and champions frozen before diagnostics. Recurrent topology initialization versus discovery must be explicit.
- Implemented opt-in `--delay-end-reset` on movement control/export CLIs. It zeros neural state once after held cue/delay, preserves pose/inputs/weights and allows recurrence during response. Both hidden and visible conditions supported; original default evaluations unchanged. Extra trials count toward existing sample/edge budgets. Whole-state reset also removes cue-independent context; require matched controls before a cue-retention claim.
- Reconciled README replay counts/research link, stale NIGHT_TASK instructions, historical paper/memory-rollout scope and an obsolete Results capability statement. Removed unsupported zero-mutation-drift claim from config comments; no numerical defaults or old data changed. Demo evidence navigation now links the audit.
- Validation: all187 post-change workspace release tests pass (baseline184), all-target Clippy/format pass, all25 Node tests pass with source-byte/pose checks active, and10 thermal plus10 supervisor tests pass. Independent code review found no blocking issue in reset timing, preserved state, resumed recurrence or workload accounting. Parsed default configuration exactly equals the previous commit; comments alone changed. CI/Pages publication follows this push. Read-only Pi check53.45°C, no matching runner/guard/supervisor; schedules remain disabled.

## Completed one-off pilot and demo evidence update — 2026-11-23

- The owner-authorized one-off fixed/varied training pilot completed8/8 searches at pinned clean c7324b6; no schedules enabled. [Report](results/VISIBLE_DIVERSITY_PILOT_2026-11-22.md), [complete audited JSON](results/visible_diversity_pilot_2026-11-22.json), [raw-data prerelease](https://github.com/dchavan2192/Digital-Terrarium/releases/tag/visible-diversity-pilot-2026-11-22). Batch `output/visible_diversity/pilot_20261122T233000Z` remains intact on Pi; full local mirror is `output/visible-diversity/pilot_20261122T233000Z`.
- Mixed outcomes: geometry fixed12/32 versus varied14/32, four seedwise differences[−.25,0,+.375,+.125]; heading both10/16. Only one pair achieved6/6 training in each arm. All partials retained; no held-out-based selection or reruns. No reliable improvement, memory or ecological claim. Next study must first address training competence and prospectively reserve fresh tests; these six test cases are consumed. Do not launch another experiment from this handoff.
- Guarded one-thread batch finished in15.149seconds; all exits0. Three telemetry samples55.1–58.95°C with no active low-bit flags; no pauses. This is sampled evidence, not a continuous temperature maximum. Latest Pi read-only check54.0°C with no runner/guard/supervisor processes. Schedules remain disabled.
- Demo Task lab retains all9 earlier champions and adds all8 new champions, all6 new held-out cases and both cues. Switching studies updates geometry, evaluator revision, result report and frozen plan together. Static311KB export exactly matches archived poses and source hashes. Failures/timeouts remain visible. New study uses c7324b6; older replay uses b40ae9c; live ecosystem is separate.
- Validation:25 Node tests pass including exact raw-source/pose matching, study selectors, aggregate outcomes and deterministic export. Six auditor tests pass; independent full audit reproduces the archived summary exactly. Figure visually inspected. Browser automation remains unavailable, so desktop/mobile interaction QA is unverified. Prior c8e1b50 CI45085173316 and Pages45085173291 both passed; final publication status follows this update's push.
- Remaining product direction from the UI/business/science review: make the evidence journey understandable before adding more mechanisms. User observation of Pause→Energy→Resume and Task lab success/failure replays is still needed; no usability improvement has been measured.

### Publication verification — 2026-11-23

- Reporting/demo commit `2019920d0a0ddd70bc6834db17fd203c93b564eb` is pushed. CI45192837084 and Pages45192837149 both succeeded. Public `version.json` identifies that full SHA; served replay JSON and Task lab code match committed bytes exactly.
- [Pilot raw-data release](https://github.com/dchavan2192/Digital-Terrarium/releases/tag/visible-diversity-pilot-2026-11-22) is published. Use `visible_diversity_pilot_2026-11-22_complete.tar.gz` (7,880,521 bytes; SHA-256 `9d4cbab4d4cf8bd72d582792bc484829ae94bfe74857addafe4940c0e825c76f`). All98 payloads verified; downloaded public archive matches local bytes. The initial archive remains for traceability, but its standalone test exposed a missing transitive auditor import. Complete package adds that dependency only; extracted empty-directory audit and six tests pass, summary identical. No experiment artifacts changed or replaced.
- This handoff-only follow-up records verification of2019920 and corrects package selection; no further simulation or experiment was started. Browser/mobile interaction testing remains open. Continue with competence-first protocol design and novice observation, keeping schedules disabled and preserving the consumed held-outs.

## Reconciled development and audited completed campaign — 2026-11-22

- Implementation/reconciliation sourcec7324b6 is pushed. GitHub Pages45082917776 succeeded and the public version file matches fullc7324b6; Ecology/neutral measures now deploy with the repaired engine. The overall CI run found an additional Rust1.100 range-loop style lint absent from local1.99.1; an equivalent iterator rewrite addresses it without changing sampling or suppressing checks. Local full/all-target Clippy checks remain passing; final workflow status follows the next push.
- [Completed long-study raw-data prerelease](https://github.com/dchavan2192/Digital-Terrarium/releases/tag/baseline-long-recurrence-2026-11-22) is published:175 noncheckpoint payloads verified, all final genomes and thermal/supervision records preserved. Checkpoints remain on Pi. No earlier artifacts were replaced.
- Dedicated outsider-UI agent produced an [externally researched review](research/DEMO_OUTSIDER_UX_REVIEW_2026-11-22.md), distinguishing primary HCI/science-communication work, a design essay and W3C guidance from actual usability evidence. Applied its bounded first change: final tour step now gives Pause→Energy→Resume as a concrete observation task. No claim of measured usability improvement; browser/user testing remains outstanding.
- One-off pilot is prepared, not yet launched in this entry: `/home/pi/Digital-Terrarium/output/visible_diversity/pilot_20261122T233000Z`. [Committed prospective launch record](results/visible_diversity_launch_2026-11-22.json) pins clean sourcec7324b6, binarySHA682bb415…, core76b23bc91f012cd7, compiler/target, exact config/code hashes and resource checks. Pi clean one-worker/offline release build took63seconds; available RAM~7.5GiB/disk~202GiB, temperature57.3°C at preparation. New strict independent auditor and six pure tests are saved before any actual study search. User authorized this one-off; no schedule was re-enabled.
- Fetched/fast-forwarded GitHubf541b8a→2e28f08, preserving interrupted local diversity-pilot work. Incoming diet/resource/shadow changes are now reconciled with the isolated runner. The Pi campaign completed8/8 worlds onNov18; Pi idle54.3°C at resumption. [Completed report](results/BASELINE_RECURRENCE_LONG_2026-11-22.md), [full JSON](results/baseline_recurrence_long_2026-11-22.json), all-world figure and raw-data archive retain the frozenb40ae9c scope. Off/on late populations985.135/822.899 and enabled links187.512/441.254; no cap-blocked births. These are structural/demographic observations, not useful memory or validation of the new diet/shadow defaults.
- Found/fixed upstream CI/Pages blockers: neutral-shadow random index draws used pointer-width-dependent ranges; all three now use the established fixed-width helper. Removed an upstream no-effect replacement flagged by Clippy. No checks were disabled. Clarified that finite activity above a demographic-conditional shadow does not by itself establish OEE; corrected the diet-variance documentation to match the actual variance formula. Earlier pinned baseline evidence is now labeled separately from newer defaults in README.
- Completed `acquire-visible-diversity`: matched fresh seed pairs5000–5003, fixed-reference versus three-layout training, equal trial budgets, all eight training selections locked/fingerprinted before the six fresh held-out conditions. [Prospective plan](results/VISIBLE_DIVERSITY_PLAN_2026-11-18.md). Explicitly disable upstream's independent diet-mutation probability in both isolated weight-only searches; genealogy regression verifies final diets equal their founders. The owner authorized a validated one-off run while keeping schedules disabled; actual source/binary launch evidence follows after the clean build. No automation schedule was re-enabled.
- Demo now has an Ecology tab with diet mean/variance, hidden neurons and real/neutral topology activity. Optional neutral comparison restarts the same seed and composes with memory mode across resets/new seeds/sharing; off/unknown/warm-up values stay unavailable. Saved Task lab has a visible evaluator-version boundary and links to the completed older-engine study. Live model metrics are not memory/intelligence/OEE scores.
- Validation:184 workspace Rust tests pass, including the portability check; three diversity CLI tests pass after the genealogy regression. All-target Clippy/format pass.24 Node tests and real WASM baseline/memory/neutral smoke checks pass; native/WASM complete-state agreement measured with both memory+shadow on at ticks0,1,100,2000. New smoke runs in CI and Pages. Browser access is unavailable, so visual browser/mobile QA remains unverified; deployment must be verified by workflow and fetched artifact identity before claiming live publication.
- Product/business review joined science/engineering agents: prioritize reproducible evidence and the bounded competence pilot over more ecological mechanisms. Primary audience is owner/research collaborators, with a clear visitor journey from live organism/brain to measured Ecology fields, saved success/failure and raw reports. No monetization assumption. Next milestone is an auditable all-pair pilot and working evidence links, regardless of outcome.

## Completed — ALIFE 2027 science sprint, 2026-11-18 UTC

Automated runs paused at owner request. Four commits landed on `main`; all pass `cargo fmt`, `cargo clippy -D warnings`, and `cargo test -p sim-core`. Automated scheduled task "Terrarium Environment Systems Dev" is now **disabled**.

### What shipped

**Commit `38c8ba0` — Configurable diet slope + MacArthur competition kernel**
- Added `food.fruit.diet_slope` config (default 1.0 → genuine 50% generalist fitness valley vs old 65%)
- Implemented MacArthur (1972) competition kernel: crowding cost ×`exp(-(Δdiet)²/(2σ²))`, σ = `food.fruit.diet_competition_sigma`; niche separation now relieves crowding pressure
- Doubled diet trait mutation probability and step size so specialists can emerge on experiment timescales
- Experiment config `configs/experiments/diet_radiation.toml` created

**Commit `ab9008e` — Five ALIFE 2027 experiment configs**
- `configs/experiments/cambrian_transition.toml` — pre-Cambrian epoch then resource pulse + predation unlock
- `configs/experiments/cooperation_phase_boundary.toml` — sweep transfer_efficiency 0.5→1.0
- `configs/experiments/drought_coexistence.toml` — intermediate disturbance hypothesis (Connell 1978)
- `configs/experiments/predation_arms_race.toml` — track mean_connections vs predation pressure
- Updated `diet_radiation.toml` with diet_competition_sigma sweep values 0.0/0.2/0.3/0.6

**Commit `0d1be66` — R* resource competition (Tilman 1982)**
- Food consumption now scales with diet efficiency: specialists draw fewer resources per unit energy gained, enabling genuine competitive exclusion by resource exhaustion
- Added `mean_diet_preference` and `diet_variance` to `TickStats`; high variance = character displacement signal
- Fixed dead `config.fruit` path — `FruitConfig.enabled` now defaults to `true` in TOML when `[food.fruit]` is present

**Commit `6a8bcaa` — Neutral shadow model for OEE measurement (Bedau, Snyder & Packard 1998)**
- `activity.rs`: Fixed presence-based tracking (was count-weighted, now +=1 per window per Bedau eq.1). Added `median_extant_activity` (Channon 2003 M(t)) which is resistant to inflation by persistent topologies.
- `shadow.rs` (new): `ShadowPopulation` mirrors the real population's demographic events (births, deaths, mutation operator) but selects parents and victims at random rather than by fitness. Uses ChaCha8 stream 2 so the main RNG is bit-identical whether shadow is on or off.
- `sim.rs`: `shadow_population`, `shadow_novel_traits`, `shadow_active_traits`, `shadow_mean_cumulative_activity`, `shadow_median_extant_activity` exported in `TickStats`. Enable with `activity_tracking.shadow = true`.
- 14 new tests (11 unit + 3 integration). Verified: shadow_does_not_perturb_main_run, shadow_population_tracks_real, shadow_is_deterministic.

### Scientific meaning

The shadow supplies a demographic-conditional null comparison. Excess real activity over this shadow would be conditional finite-horizon evidence whose interpretation depends on validating the null and metrics. It does not by itself establish open-ended evolution, unbounded novelty or a Channon Class4b result. The earlier finite-run activity claims remain unestablished.

### Next priorities (when runs resume)

1. **Run diet radiation experiment** (`configs/experiments/diet_radiation.toml`) — does the MacArthur kernel produce stable bimodal diet distribution? Use `sim-runner sweep` on the Pi.
2. **Enable shadow in experiment configs** — set `activity_tracking.shadow = true` in experiment TOMLs so each run produces shadow_* columns in CSV for comparison.
3. **sim-runner CSV output** — currently doesn't emit shadow_* columns; add them before ALIFE 2027 experiments so analysis scripts can compare real vs shadow activity automatically.
4. **OEE shadow design doc** — `docs/research/OEE_SHADOW_DESIGN.md` describes the full ablation experiment; run once diet radiation is stable.

---

## Continued development — long-run auditor ready, 2026-11-18

- Checked the Pi first: five of eight prescribed worlds completed, `job_05_seed_4002_rate_0.3` active, two unstarted. No main-branch updates or local edits at resumption. Preserved the pinned queue and all outputs; no restart, new simulation, engine/default/demo change or protocol adjustment.
- Added `analysis/summarize_long_recurrence.py` and [analysis/collection handoff](research/LONG_RECURRENCE_ANALYSIS.md). Strict source/base/whole-config/input and terminal-export checks pass on all five completed worlds in the mirrored observation. The explicit incomplete snapshot retains8slots/4pairs:5completed,1active,2unstarted; it is not a final result. Generated progress JSON stays under ignored `output/long-recurrence/`, with no partial scientific claim added to the results register.
- Analysis preserves exact51/126-observation early/late windows, inclusive cumulative-event arithmetic, late-minus-early metrics, sampled population/cap pressure and all arm/pair missingness. Biological extinction is distinct from technical failure; post-extinction living edge means are null, while pre-action activity keeps its different timing. Live prefixes and partial trailing CSV records are labeled explicitly. SHA-256 inventory records bytes actually consumed; non-atomic multi-file snapshot limits are documented.
- Two independent agent reviews informed counter/window, status, export and snapshot-integrity checks. All14 synthetic auditor tests pass, including full8-world completion, extinct/active/failed/unstarted retention, source/config/export/CSV corruption, impossible counters/queue ordering and consumed-byte hashes. CI now runs them. No actual biological outcomes were used to revise frozen endpoints.
- Latest live check2026-11-18 14:21:42UTC (09:21:42Eastern): worker38495, supervisor47393, guard47394, runner47396; guard `monitoring`, one matching runner,72.7°C, no pause at that snapshot, throttle flags0xe0000 with no active low bits. Use refreshed state files for current identity. The campaign is still running; do not start another simulation. After all jobs terminate, refresh complete artifacts and run the auditor against all eight jobs before any report or archive. Current Task lab remains deployed fromfa0cd99.

## ACTIVE long Pi campaign and live Task lab — verified 2026-11-18 02:39 UTC

- **Do not start a second simulation or rerun this queue.** Long study is active at `/home/pi/Digital-Terrarium/output/long_recurrence/baseline_long_20261118T023800Z`. Worker38495, first supervisor38497, guard38498, runner38500; use current state files for identity rather than assuming these PIDs remain valid. Eight frozen jobs, seed4000/rate0 first; completed-jobs list currently empty. [Frozen plan](results/BASELINE_RECURRENCE_LONG_PLAN_2026-11-17.md), [launch/provenance snapshot](results/long_recurrence_launch_2026-11-17.json).
- Verified actual simulation progress through logged tick10,000, population1059 and recurrence0 in the off arm. This is startup evidence, not a scientific result. Clean engineb40ae9c/core297edc68fb18e05e, full effective config checked at food.008/cap2000/seed4000/rate0. Pinned native executable SHA-256 `05e6fe23121818fb897332a53f314da11e2ea98fed6f61cfe52d607009833e63`; operational source `fa0cd99783ecba28a1a59d8045c55bee38921ad2`. Whole-base SHA-256 `bcfd543bc75ed4127389c17c24e6282a8e735c96a903d0a55abd6e0fd165079d`, copied-worker SHA-256 `0f0d0953b5feedb951b949729adc7d5504ce3506dca5445f9bc5eb99edc729ae`. Campaign.json records all eight copied input inventories; old engines/checkpoints remain untouched.
- Guard verified `monitoring`, exactly one pinned runner,66.65°C at02:39:52UTC, no pause at that snapshot; sampledRSS23,199,744bytes. Hardware snapshot flags0xe0000 retain history with no active low bits at sampling.75/65°C thermal guard and6h/job watchdog/resource policies are active. These observations do not establish a continuous temperature maximum or an experiment outcome. Estimated duration may extend into tomorrow; no completion promise.
- First checks next session/automation: parent `status`, `worker.json`, `launcher.log`; current job `supervisor.json`, `thermal_guard/status.json`, `guard.log`, `thermal_supervisor.jsonl`, then native session manifest/CSV/checkpoints. An incomplete snapshot of metadata/CSV is mirrored under local `output/long-recurrence/`; final analysis must refresh all completed/failed jobs and preserve partials. Stop-on-failure is intentional. Never replace copied binaries, remove checkpoints, blindly resume the worker or launch another study alongside it. A later recovery requires a separate recorded attempt with the exact engine/checkpoint.
- **Demo deployed successfully at sourcefa0cd99:** [open demo → Task lab](https://dchavan2192.github.io/Digital-Terrarium/demo/). Pages run44224538761 passed Rust release tests, real WASM checks and publication. Public version.json matches full implementation revision; deployed HTML contains Task lab and fetched189,581-byte replay payload exactly matches committed SHA-256 `f904a5986577187335220f9ffb33c7b491c00c94a79d13c2694de998e39b2f4e`. Browser layout remains visually unverified. All18 local Node tests,13 new worker tests,9 existing supervisor tests and10 thermal tests pass; CI status is recorded at handoff, with no new engine changes.
- After campaign completion, audit all eight prescribed configs/source hashes/statuses and checkpoint/export identities, then implement the frozen early/late per-world and paired summaries. No automatic scientific summarizer is installed for this new schema; do not apply the old demo-pilot summarizer unchanged. This run studies structural recurrence pressure/population effects, not memory benefit. Continue competence-acquisition design separately without using the consumed robustness suite as untouched validation.

## Resumed development — Task lab and long Pi campaign preparation, 2026-11-18 UTC

- Owner requested continued development, a long Pi run, agents, commit/push and a demo update. Three agents independently implemented the replay UI, reviewed the prospective experiment and tested unattended process safety. No incoming main changes; Pi initially idle54.3°C,208GiB disk free and7.6GiB available RAM.
- Added **Task lab** in the demo toolbar: lazy-loaded saved native replays for all9 champions×7cases×2cues, plus three fixed controls. Exact recorded geometry/poses/outcomes and source hashes; selectors, play/step/restart/scrub, accessible native dialog and mobile layout. Clear separation from live ecology and memory-benefit claims. Static payload189,581bytes. All18 Node tests pass, including raw-source/export identity; HTTP preview served successfully. Browser UI QA unavailable, so visual layout remains unverified. Existing Pages whole-web deployment includes all assets.
- [Frozen long baseline recurrence-pressure plan](results/BASELINE_RECURRENCE_LONG_PLAN_2026-11-17.md): fresh paired seeds4000–4003, mutation probability0/.3, eight serial500k-tick worlds at baselinefood.008/cap2000. This is a longer structural/demographic diagnostic under reduced expected cap pressure, separate from useful-memory testing. No delayed-memory selection or changed ecological rules. Clean engineb40ae9c and existing pinned binary05e6fe23… retained.
- Replaced the legacy Bash lifecycle for this study with an explicit `ecology-long` mode in the owned-child Python supervisor and a single-attempt pinned queue. One thread,75/65°C guard,6h per-job startup/cooling-inclusive watchdog, sampled2GiB RSS cutoff,20GiB disk/3GiB RAM gates at job boundaries. Checkpoint50k/log1000/final export500k; existing256MiB checkpoint limit preserved. Prior final baseline checkpoints43.9–50.08MiB; longer runs may still hit resource limits. Any technical failure stops the queue and preserves partials. Unresponsive-supervisor recovery remains an explicitly documented external-inspection case.
- Validation before launch:13 harmless process/queue tests,9 existing supervisor tests and18 Node tests pass. Tests cover guard startup/loss/staleness, stopped-runner timeout without resume, sampledRSS, serialized eight-job queue, changed inputs, resource gates and post-spawn persistence-error cleanup. No Rust/engine change; previous171-test Rust implementation baseline retained. CI includes new worker tests. Record actual pinned campaign/guard/runner evidence in a new entry after launch; do not infer an active run from this preparation entry.

## Resumed development — completed saved-policy robustness diagnostic, 2026-11-18 UTC

- Owner resumed after the usage-limit stop. GitHub had no incoming source changes; Pi idle at53.2°C. Clean evaluator `b40ae9c` built in an isolated worktree with one Cargo worker; pinned binary SHA-256 `05e6fe23121818fb897332a53f314da11e2ea98fed6f61cfe52d607009833e63`. Old checkout/binaries/checkpoints preserved.
- Added committed guarded validation launch/audit path at `09226d3` before outcomes. All nine Pi champion-summary hashes matched the frozen source list before launch. Executed original, terminal-only and terminal-progress serially under unchanged seven-case suite; no selection, retuning or search extension. [Final report](results/VISIBLE_VALIDATION_2026-11-17.md), [JSON](results/visible_validation_2026-11-17.json), all-policy figure and [raw archive](https://github.com/dchavan2192/Digital-Terrarium/releases/tag/visible-validation-2026-11-17) preserve all63 policy/case combinations. All95 archived payload files verified against their SHA-256 inventory.
- **Finding:** progress policies reproduce2/3 full training successes, retain1/3 at−45° and2/3 at+45°, but0/3 solve both cues in each of the four changed relative geometries. Neither other cohort solves both cues on any case. Fixed manufactured feed-forward/recurrent references solve every case. Visible competence is brittle in these saved policies; this is not fresh-seed confirmation or a memory result. Near preserves center bearing while changing distance/apparent size.
- Baseline replay and all feed-forward intact/reset trajectories match exactly. Independent audit verified source hashes, outcomes, motion/contact accounting and clean guard completion, covering all522 trials. Final auditor tests: three pure plus three installed-dataset checks pass; nine supervisor and ten thermal tests pass. Rust remains the unchanged171-test validated evaluator. Figure visually checked.
- Pi batch `output/visible_validation/robustness_20261118T020500Z`, mirrored locally at `output/visible-validation/`. All supervisors/guards exit0 in about5.13seconds each; final readings55.1/57.85/57.3°C, no remaining paused or matching runner. Sparse telemetry is not continuous thermal/throttling assurance. Post-run Pi idle58.7°C. No additional experiment active.
- **Next:** preserve the negative transfer result; specify a fresh competence-acquisition intervention using varied training conditions and fresh validation conditions. The used suite is now diagnostic data, not an untouched test set. Diagnose trajectories before changing the objective; fresh-seed repeatability needs a precision target. Keep delayed-memory selection/ecological claims gated. Demo/defaults unchanged; diet-competition branch unmerged.

## Usage-limit stopping point — saved-policy validation harness, 2026-11-17

- Owner requested documentation, commit/push and a clean stop. **No new experiment launched; do not launch one from this wrap-up.** Pi checked idle at 53.8°C, hardware flags `0xe0000` (historical flags, no active low-bit throttling). Historical outputs, checkpoints, binaries and root checkout preserved.
- [Saved-trace diagnosis](results/ACQUISITION_TRACE_DIAGNOSIS_2026-11-17.md) covers all nine training champions without reevaluation. Three of four successful two-cue progress-champion responses contact on tick20. Partial seed2101's mean progress0.5 combines one success with a failed response clipped to0, not a halfway approach. First-turn sign alone does not classify the full policy. These remain descriptive training results, with no memory or generalization claim.
- Implemented bounded read-only `validate-visible`: strict saved-session selection by training rank, all failed seeds retained, source fingerprints/provenance, exact original-case regression before changed-case evaluation, fixed controls and policy-free geometry checks. [Prospective plan and resume checklist](results/VISIBLE_VALIDATION_PLAN_2026-11-17.md) freezes seven cases and suite SHA-256. **Actual archived policies have not been evaluated on these new cases.** Synthetic CLI fixtures are engineering checks only.
- Added loader/unit/CLI coverage for original/progress formats, exact baseline replay, all seven cases, source immutability, invalid suite/source rejection before output creation and unread failed partial populations. Corrected a test-fixture directory collision using an atomic counter; no production search behavior changed. Independent final read-only review found no substantive blocker. Final verification: all **171 workspace Rust tests** pass; all-target Clippy with warnings denied, format and whitespace checks pass. Local logs: `output/wrap-up-workspace-tests.log`, `output/wrap-up-clippy.log`.
- Next session: inspect Pi/GitHub, pin the committed evaluator and complete a tested thermal-supervised validation launcher before running the frozen diagnostic on all three cohorts. Existing acquisition supervisor cannot launch this new command unchanged. Keep original pilot separate from paired comparison; retain partial/failure outcomes and publish all cases. Fresh-seed repeatability precedes any delayed-memory selection. Demo/ecological defaults unchanged; diet-competition branch remains unmerged.

## Completed paired acquisition-method pilot — 2026-11-17

- Fixed source 72f7c5b and prospective analysis/operational commit a2971d5 ran the unchanged fresh 2100–2102 comparison: terminal-only controls0/3 acquired both visible responses; progress tiebreak2/3 (first full acquisition generations 6 and14), with seed 2101 retained as partial. [Audited report](results/TERMINAL_PROGRESS_COMPARISON_2026-11-17.md), [JSON](results/terminal_progress_comparison_2026-11-17.json), all-run plot and [raw archive](https://github.com/dchavan2192/Digital-Terrarium/releases/tag/terminal-progress-comparison-2026-11-17).
- All six searches completed their declared64/25 budgets, no extensions/extra seeds/tuning. Correctness stays primary; full progress gains are not substituted for competence. Independent raw audit verified all 156×64 generation scores, exact topology/ranks/parents/elite rules, matched founder genotypes by ID and terminal progress from actual geometry. Default terminal-only behavior remains intact. No ecological/default/demo-engine change.
- Parent `output/acquisition/progress_comparison_20261117T205542Z`, mirrored to `output/progress-comparison/`. Both supervisors/guards finished exit0 in about5.132 seconds each; final guard readings 55.65/56.75 °C, no pauses. Execution was shorter than a guard poll, so telemetry is sparse—not a continuous maximum/protection measurement. Pi idle after completion.
- Validation: 161 Rust tests, Clippy/format, seven supervisor tests and seven pure auditor tests pass; three post-collection corruption/dataset integration tests also pass. Plot visually verified; stable raw payload hashes checked before publication. Earlier pilots and all pinned binaries preserved. The separate diet-competition branch remains unmerged pending its recorded validation fixes.
- **Next development:** freeze fresh repeatability and held-out validation conditions for complete visible competence. Pilot n=3 pairs/training-only success does not establish robust transfer, adaptive memory or lifetime learning. Several successes contact on tick20, so do not infer delay tolerance. Do not reuse these pilot seeds as confirmatory data or launch hidden-memory selection before the competence gate. No further experiment launched after this comparison.
- User asked about the demo: UI metric-clarity changes are live (c3ba0c8), but the new tiebreak is isolated task-selection research and has not been added to ecological reproduction or the browser demo.

## Acquisition-method revision — 2026-11-17 (development, comparison not yet run)

- Continuing after the archived three-seed partial/null acquisition result. Added explicit opt-in `terminal_progress` fitness: correct count remains primary, mean clipped progress toward the assigned goal **after termination** breaks ties, then persistent ID. Original terminal-only method remains the default; old configs, controllers, sensor inputs, rewards and search random draws retain their behavior. No engine or demo setting change.
- New mode records per-candidate/mean/per-cue progress separately from actual correctness. Confirmed correct contacts normalize to1; finite-positive starting distance and finite terminal distances are guarded. No extra neural evaluations or online reward are introduced. [Design/research limits](research/TERMINAL_PROGRESS_ACQUISITION_2026-11-17.md).
- Fresh paired configs/seeds2100–2102 and [prospective comparison plan](results/TERMINAL_PROGRESS_COMPARISON_PLAN_2026-11-17.md) fix64/25 budgets, original geometry, all-failure retention, primary two-choice acquisition and guarded serial Pi execution. Execution source72f7c5b is pinned in a clean isolated Pi build; full validation and independent review pass. Paired auditor has 7 data-independent tests, with strict parent/child completion, frozen inputs, lineage/rank checks, founder identity and independently recomputed terminal progress. Prepared parent `output/acquisition/progress_comparison_20261117T205542Z` has two fresh sub-batches. Operational/analysis files are being saved before launch; no new pilot outcomes yet.
- Targeted runner tests pass, including default/explicit terminal-only byte-identical search artifacts, deterministic new-mode replay/ranking, correct-count priority, progress boundary/nonfinite guards, unchanged input/action/reward trajectories, and unknown-mode rejection. All161 workspace Rust tests, all-target Clippy and format checks pass; independent code review found no blocker. Comparison still requires saved audit-script readiness and a pinned clean build before launch.
- New remote ecology branch `science/diet-competition-kernel` is separate/unmerged. [Read-only review](research/DIET_COMPETITION_BRANCH_REVIEW_2026-11-17.md) found domain-validation and unsupported-sweep gaps; do not fold it into acquisition or launch its experiments yet.

## Demo metric clarity — 2026-11-17

- User asked whether the demo UI needed improvement. Focused changes make the observed connection growth interpretable: memory panel now separates measured average enabled feed-forward/recurrent/total links, discloses the running config’s mutation-attempt probability, and keeps tick-zero/missing metrics unavailable. Individual inspector shows the same decomposition and labels static cost as a baseline estimate.
- Brain KPI now says **Brain links**, with measured decomposition in its tooltip. Removed unsupported intelligence-payoff, carrying-capacity, genealogical-kin and adaptation claims from KPI/event wording. Sharing now reports events per creature rather than implying a donor percentage; hue similarity remains explicitly a proxy. UI design spec updated to prevent reintroducing old claims.
- Validation: 14 Node tests pass; real WASM memory smoke checks actual recurrent counts, population/individual link breakdowns and the configured 0.3 probability. JS syntax/diff checks pass. Browser host unavailable, so visual layout is not verified in this session. No engine/default-config/Pi experiment change; publication uses existing GitHub Pages.

## Resumed development — completed fixed acquisition pilot, 2026-11-17

- Owner resumed development after the stopping point. GitHub had no incoming source changes; Pi was idle at 51.6°C. Built exact `5504a20` in a clean isolated Pi worktree, one Cargo worker; all three frozen input/source hashes match. Old root checkout and prior binaries remain intact.
- Added/pushed tested guarded supervisor and audit/plot tooling at `400eea3`; [operational record](results/VISIBLE_ACQUISITION_OPERATIONS_2026-11-17.md) fixes active watchdog 1,800 seconds including startup/cooling before outcomes. Seven supervisor process tests and ten thermal tests pass; CI runs them. Guard loss/interruption leaves only the owned runner paused; timeout never resumes a hot stopped runner.
- Fixed [visible acquisition pilot](results/VISIBLE_ACQUISITION_PILOT_2026-11-17.md) completed all three seeds, no technical failures: seed 2000 best 0→0.5, seeds 2001/2002 0.5→0.5. No candidate scored both correct in any generation. Founders/final score counts and complete traces retained. This is partial/no acquisition under the fixed search budget, not memory failure or an architecture impossibility. No extra generations/seeds or post-outcome tuning.
- Batch `output/acquisition/pilot_20261117T163705Z`, mirrored locally under `output/visible-acquisition/`. Clean source5504/core297edc68fb18e05e, native release. Supervisor elapsed5.134seconds; guard finished at54.0°C without pauses. Short execution occurred between guard polls; thermal readings are sparse, not continuous assurance. No additional experiment active after completion.
- Strict summary plus independent audit verify all78 generation logs×64 records, elite/parent/ID rules, exact fixed topology and final visible replay. Three analysis integration tests pass, rejecting malformed sampling/topology/controls/cue replay. All data, hashes, plot and [raw archive](https://github.com/dchavan2192/Digital-Terrarium/releases/tag/visible-acquisition-pilot-2026-11-17) saved. No Rust mechanics or demo behavior changed this turn.
- Repeat safety checks caught a fixture-only empty-PID publication race; the fake runner now publishes its PID atomically. Three consecutive seven-test safety suites and the three real-dataset audit tests pass after the fix. Production supervisor and actual Pi experiment were unchanged; no scientific rerun was needed.
- **Next development:** diagnose and separately specify an acquisition-method revision before another search. Preserve this fixed-budget result; do not move to hidden-memory selection until visible competence is established. Changes to fitness/tie handling/budget require a new record and fresh declared evaluation, not silent continuation. Read the council and completed pilot reports first. Existing nightly schedule must check this entry before launching work.

## Requested stopping point — 2026-11-17, completed overnight analysis

- Owner requested a clean stop, complete documentation, commit and push. **Do not start another experiment from this stopping-point turn.** Pi overnight batch `output/memory_growth/overnight_20261117T040010Z` completed at 06:19 UTC: all eight runs survived 200k ticks, exit 0; guard finished. Fresh 14:40 UTC check: no runner, 52.7°C, no active low-bit throttling flags.
- [Final exploratory report](results/MEMORY_GROWTH_PILOT_2026-11-17.md), [JSON/provenance](results/memory_growth_pilot_2026-11-17.json), all-run plot and strict summarizer saved. Four run-level late means: connections 224.828 off/495.864 on; recurrence 0/264.926; population 336.426/277.406. Mostly recurrent structural accumulation under a high add-mutation setting, not proof of useful recall. Demo cap blocking and dirty-build caveats retained; no defaults changed.
- Independent agent reproduced all eight summaries and expressed-brain endpoint counts. Three analysis integration tests pass, including incomplete/duplicate/missing/nonfinite rejection; figure visually checked. No Rust/browser code changed in this turn, so the prior 157-test Rust/Clippy validation remains the implementation baseline.
- Completed dataset mirrored locally to `output/memory-growth/overnight_20261117T040010Z` (checkpoints omitted), raw archive under `output/memory-growth/`; GitHub prerelease `memory-growth-pilot-2026-11-17` holds public reproducibility assets. All checkpoints and failed zero-tick preflight retained on Pi. Scripts/JSON record checksums.
- [Next pilot plan](results/VISIBLE_ACQUISITION_PILOT_PLAN_2026-11-17.md) is fixed at implementation `5504a20`: 64 founders, 25 replacements, seeds 2000–2002, weight-only controller mutation, terminal fitness, no heldout validation. It has **not been launched**. Readiness review found no code blocker; next session must pin the Pi build/inputs and set a recorded operational watchdog/thermal guard before executing. Treat successful founders separately from acquisition and retain failed runs.
- Resume by reading this entry, checking Pi activity, fetching GitHub and reviewing the fixed acquisition plan. Do not recreate or extend the completed growth campaign. No further feature work is required to save this turn.

## Active unattended batch — verified 2026-11-17 04:00–04:02 UTC

- **Pi running:** `/home/pi/Digital-Terrarium/output/memory_growth/overnight_20261117T040010Z`. Eight fixed serial runs; seed 3000/rate 0 active, CSV verified through tick 5,500, population 343. This is progress evidence only; no outcomes interpreted. Read `launcher.pid` for the launcher identity; initial runner PID 32674 and guard PID 32663.
- Guard confirmed `monitoring`, one matching pinned executable, 56.75°C, pause/resume 75/65°C; hardware flags `0xe0000` have no active low-bit flags. Pin SHA256 binary `5b608d5f2002e74d4185e60c595e686d185362a13e89f4ba26aaa2dca9d274cb` is recorded in batch `pinned.sha256` (use that file as authoritative). Engine source `0da954f`, protocol source `f63de31`; old engine build caveats remain in archived metadata.
- Preflight attempt `overnight_20261117T035511Z` failed before any ticks because several demo settings are unsupported CLI overrides. Retained unchanged. Corrected operationally by generating the full TOML with whole-tree verification; fixed scientific settings/seeds/endpoints unchanged. New batch is active and the only simulation.
- Read `status`, `thermal_guard/status.json`, `completed_jobs.txt`, `launcher.log`, then job manifests/CSV. Checkpoint every 50k, target 200k, logs every 500, final genomes saved. Do not replace executable, run another simulation, or blindly restart the whole worker. Recovery instructions are in the frozen plan. Preserve all partial/failed runs.
- Development saved/pushed: `295efb2` acquisition runner (157 Rust tests + Clippy pass), `f63de31` verified full-config correction. CI/Pages passed `295efb2`; latest correction workflows were still in progress at handoff. No actual acquisition experiment launched.
- User is sleeping; continue only within existing host permissions and approved commands. Do not broaden security access. Pause approval-dependent actions if needed. The project permission template remains unactivated; user rescinded earlier permission-reconfiguration work. Next science: summarize all eight structural-growth runs after completion; then pin/freeze the separate visible acquisition pilot before executing it.

## Wrap-up — next development and overnight diagnostic, 2026-11-17 UTC

- Memory demo deployment for `646e374` completed successfully. User reports rising average connections. This observation is not yet a replicated result; memory mode's 0.3 add-recurrent probability creates a plausible structural mutation bias.
- Added a separate bounded `acquire-visible` CLI with random founders, weight-only controller optimization, sparse terminal fitness, fixed generation budget, deterministic champion rule, full final-population retention and trace-free shared arena scoring. [Implementation/protocol handoff](research/VISIBLE_ACQUISITION_RUNNER.md). No actual acquisition campaign yet; do not infer learning from implementation tests.
- Validation: all 157 workspace Rust tests pass, all-target Clippy passes; seeded CLI replay and invalid-design rejection pass, shared compact scoring agrees with full instrument. Shell syntax check passes for the overnight worker. Independent review found two blockers (founder config field and retained-trace work); both corrected.
- User requested an overnight Pi run and then usage-limit wrap-up. Pi was idle at 52.1°C. [Frozen exploratory structural-growth plan](results/MEMORY_GROWTH_PILOT_PLAN_2026-11-17.md): four fresh seeds × memory-off/on, eight serial 200k-tick runs at demo settings, existing pinned validated engine, guarded 75/65°C. Exact active batch location and launch evidence follow in the next saved entry.
- Next session: inspect the overnight batch status/guard/logs before any work; preserve every artifact. Summarize all runs under the saved plan once complete. Then finish acquisition-pilot readiness and freeze its source/protocol before launch. No broad memory-benefit claim or cost sweep is authorized by these diagnostics.

## Demo and research council — 2026-11-17 UTC

- Added optional **Memory on** demo mode (`demo/?seed=42&memory=1`), measured recurrent-creature count and an inspect button. Gold dotted brain edges retain previous-tick activations. Toggle restarts the same seed; reset/new seed retain mode, shared links carry mode. Recurrence arises through offspring mutation (demo-only probability 0.3), rather than being injected into founders. Experiment defaults remain off.
- Real WASM smoke: seed 42, tick 1,000, living recurrent creature #583; recurrent fraction 0.8661417322834646. Explicit off exactly preserves baseline stats trajectory, and reset reproduces candidate ID. This verifies recurrence presence and navigation, not adaptive recall. Browser overlay remains food 0.002/cap 500; checked-in demo overlay synchronized with deployment source.
- Validation: full workspace tests and all-target Clippy pass; 13 JS tests plus baseline and memory WASM smoke pass. New smoke runs in Pages deployment. Visual browser QA unavailable: computer-use permission is not granted, so layout has not been visually verified in this session.
- Three separate literature paths and a cross-challenge round produced the [council decision](research/SCIENCE_COUNCIL_2026-11-17.md). Next is a bounded visible-task genetic acquisition pilot, then delayed retention with a single delay-end reset, then ecological information-payoff testing. No new long Pi experiment launched.

## Resumed — 2026-11-17 UTC: baseline completed, competence control

- No incoming GitHub changes beyond saved `9d00b24`. Pi baseline completed at about 02:15 UTC; finalizer status `summarized`, guard `finished`, exit 0. All 10 seeds 100–109 survived 200k ticks, zero cap blocking; late mean 893.629 (SD 12.5436), post-burn-in minimum 620. All four fixed criteria pass. [Final report](results/CURRENT_BASELINE_RECHECK_2026-11-17.md), full JSON/plot and [raw-data release](https://github.com/dchavan2192/Digital-Terrarium/releases/tag/current-baseline-recheck-2026-11-17).
- Independent audit verified all 30 CSV/config/manifest hashes and 400 snapshots per seed. Local strict summary matches pinned Pi finalizer exactly. Source 0da954f/core abad322241615ca2, reuse of calibration seeds and dirty-build operational-file caveat retained. This is calibration, not equilibrium or memory-benefit evidence; full second-machine long-duration replication remains unmeasured.
- Guard logged 149 pause/resume cycles, 7,303 seconds cooling; max 75.7°C in 537 minute observations, no active throttling flags. Pi is now idle/cooled (55.4°C at check). No new long experiment launched; retain all pinned binaries/checkpoints.
- Implemented planned default-false `--cue-visible` on both movement commands. Only movement-phase cue changes; arena, donors, hold phases, horizon, rewards and state-reset rules are identical. Manufactured visible controls all reward 1; hidden controls remain 1/0.5/0.5. [Plan](results/VISIBLE_CUE_CONTROL_PLAN_2026-11-17.md) saved before actual donor evaluation. All 153 Rust tests, 10 JS and 10 thermal tests pass; all-target Clippy/format pass. Full workspace build/test serialized.
- Completed saved clean `bda6f35` paired evaluation of all 898 donors: delayed 7 correct/7 wrong/1,782 timeouts; visible 0 correct/31 wrong/1,765 timeouts. Independent audit verified all pairs/hashes and exact preservation of old delayed trials. [Report](results/VISIBLE_CUE_CONTROL_2026-11-17.md). Manufactured visible controls all 1; natural donor competence remains unestablished. Do not interpret delayed failure as memory deficit or tune mapping after scores. Next: validate task competence or an ecologically relevant endpoint before a memory-selection experiment. No new long run started.

## Stopping point — 2026-11-16, 22:43 UTC

User requested saving work before the usage limit. Stop new features/experiments here.

- Movement arena implementation saved at `0bc1be2`; manufactured controls and actual 898-genome negative control independently audited. [Final report](results/DELAYED_CHOICE_ARENA_2026-11-16.md), control JSON/plot and summary are committed with this stopping-point update; [public raw-data archive](https://github.com/dchavan2192/Digital-Terrarium/releases/tag/delayed-choice-instrument-2026-11-16) preserves the full donor and outputs.
- Controls reward 1 / 0.5 / 0.5. Actual donors: 7 left for both cues, 891 timeouts; no recurrence, identical intact/reset/opposite-cue paths. This is not a memory-deficit result. All 151 Rust, 10 JS and 10 thermal tests pass; final workspace run serialized; Clippy/format pass.
- Pi ACTIVE: six of ten baseline runs complete, seed 106 running. Batch `output/phase9a/validation_20261116T171843Z_28985`, pinned source `0da954f`. Guard at 73.8°C, no active throttling; finalizer waiting. Leave running; do not replace its executable. Cooling pauses remain 75°C/65°C, every 5 seconds.
- Resume by checking batch `status`, `thermal_guard/status.json`, and `finalization/status`. If completed, mirror logs/summary and apply the already fixed acceptance rule; do not treat partial data as success. No additional evolutionary campaign is queued.
- Next coding task: fixed visible-cue competence control before interpreting evolved-controller memory performance. Same arena/donors/order/horizon/reward; no tuning based on favorable scores. Then recurrent-genotype/state-reset comparisons, after baseline readiness. Defaults and browser demo are unchanged by this arena work.

## Development continuation — delayed-choice movement arena (2026-11-16)

- Added runner-only `delayed-choice` controls and `evaluate-delayed-choice` for all-record exports, configured in `configs/delayed_choice.toml`. Pose is held through cue/delay, all actions discarded, then turn/speed drives movement toward visually identical goals. Hidden correct-goal labels enter terminal scoring only. Wrong first contact ends the trial; timeout is explicit.
- Manufactured recurrent control chooses correctly for both cues (reward 1); feed-forward and same-network state-cleared controls choose the same goal for both (reward 1/2), all in five movement ticks. Tests check identical response inputs/poses and full no-memory trajectories, fixed hold pose, swept contact and rejection guards. This is a controlled memory-dependent task, not evolved cognition or ecological fitness.
- The clamp and arena evaluators share one validated export loader and bounded work. Arena traces are also bounded to 100,000 samples before evaluation. New CLI tests evaluate a genuine export and reject bad geometry/layouts without partial output.
- Independent review passed; serialized full workspace 151 Rust tests, all-target Clippy/format, 10 JS and 10 thermal tests pass. An earlier overlapping targeted build changed shared binary feature identity during one test; that artifact mismatch was resolved by serializing the final suite. No simulation fix or ecological change was required.
- [Arena protocol](results/DELAYED_CHOICE_ARENA_PLAN_2026-11-16.md) precedes the actual 898-genome negative-control evaluation. A plotter uses actual recorded movement traces. Pi remains on its pinned baseline; six of ten runs completed, seed 106 running at 22:26 UTC, guard monitoring at 65°C, no active throttling.

## Development continuation — exported-genome behavioural evaluation (2026-11-16)

- Added `sim-runner evaluate-delayed-cue --input FILE --cue-channel INDEX --blank-delay N`: evaluates every exported genome against the same network with state cleared each tick, using fresh trial state and identical probe inputs. Genetic weights and frozen snapshot methylation are rebuilt; expressed brains and live memory are ignored. Results carry donor/evaluator provenance and all individual responses. This is a synthetic clamp, not a fitness or learned-cognition assay.
- `Genome::validate()` shares the checkpoint validator; crafted dimension overflows now return errors before brain construction. Existing valid topologies remain accepted; 512 is an add-edge threshold, not a universal storage limit. Bounds and finite-response checks prevent partial or oversized evaluation work.
- Independent code review passed. Full workspace: 142 Rust tests; all-target Clippy/format, 10 JS tests and 10 thermal policy tests pass. Thermal tests are now in CI.
- [Pre-execution negative-control plan](results/EXPORTED_GENOME_ASSAY_PLAN_2026-11-16.md) chooses all 898 living genomes from seed100 at tick200,001. The original 200k run is untouched; its pinned binary resumed one tick into `output/exported_assay_donor_20261116T212115Z`. Actual negative-control evaluation follows the saved implementation revision.
- Actual negative control now passed on saved source `aa6e844`: all 898 genomes had zero recurrence, scored 1/2 and had exactly matching intact/state-cleared trial responses. [Report](results/EXPORTED_GENOME_ASSAY_2026-11-16.md) and [checksums](results/exported_genome_assay_2026-11-16.json). Before/after complete-state fingerprints match at ticks 0,1,100 (default config, seed42). This is an instrument check, not a memory or fitness result.
- Latest Pi check at 21:27 UTC: four recheck runs complete, seed104 active; thermal guard functioning, finalizer waiting, no active hardware throttling. No other sustained experiment was launched. The owner dropped the separate permissions-configuration task; its unused template remains inert.

## Continuation — 2026-11-16, validation completed

- Claude Code's latest source is `89e0992`: recurrent memory implemented and off by default. Existing cloud/Pi validation used its preserved older build; no active executable was replaced.
- At 17:06 UTC the Pi batch `validation_20261116T021704Z_23586` was confirmed completed. All 30 runs survived to 200k ticks; zero cap-blocked births and no technical failures. Logs/configurations mirrored into `output/pi-validation/`.
- Strict summary and provenance: [validation report](results/PHASE9A_VALIDATION_2026-11-16.md), [JSON with checksums](results/phase9a_validation_2026-11-16.json), result CAL-2026-11-16-FOOD-VALIDATION in RESULTS.md.
- All rates pass; the written rule proposes 0.008. The owner subsequently approved rate 0.008 and cap 2,000; both defaults are now applied for the current-engine recheck. Calibration used cap 2,000 and predates drought RNG/node identity/memory changes; recheck the latest engine before treating it as validated for the current engine.
- Independent review verified all 90 artifact checksums and all 30 complete 400-snapshot logs. Exploratory 95% CI for the selected rate: 884.02–943.81. README and results report publish this as calibration, not a memory or OEE result. [Current-engine recheck plan](results/CURRENT_BASELINE_RECHECK_2026-11-16.md) fixes one worker, the selected rate, seeds 100–109 and 200k ticks before launch.
- Exploratory ecology: selected-rate population rose 870.16→913.92 between early and late windows while connections fell 228.02→216.53; no equilibrium or adaptive-intelligence claim. Demo overlay explicitly retains cap 500 and food 0.002.
- Pi cooled to 57.1 °C and had no active throttle at the final check. Improve cooling before another long campaign. Next implementation priority remains the delayed-response behavioural test; memory stays off by default.

- Published calibration source `0da954f`, README/report/plot/data, [GitHub issue #1](https://github.com/dchavan2192/Digital-Terrarium/issues/1), and a [raw-data prerelease](https://github.com/dchavan2192/Digital-Terrarium/releases/tag/calibration-food-2026-11-16). Its CI and Pages passed.
- Current-engine recheck ACTIVE: `output/phase9a/validation_20261116T171843Z_28985` on Pi, pinned source `0da954f`, ten seeds 100–109 × 200k ticks at 0.008/cap 2,000. One worker/thread. Do not replace its executable or resume with another build. Separate batch-specific thermal guard added (pause 75°C/resume 65°C, 5s polling); live identity/pause/recovery checks passed. The strict completion summarizer is pinned separately under `finalization/`.
- Independent reviewers recommend completing this baseline and the delayed-cue behavioural instrument before any sustained memory/inheritance experiment. No additional evolutionary campaign is queued. User authorizes agents and commits/pushes after completed work; preserve all artifacts.

- Delayed-cue instrument now implemented: `sim-runner delayed-cue --blank-delay N`. Pi source `d35dd2c` passed all controls at delays 0,1,5,20,100. [Report](results/DELAYED_CUE_INSTRUMENT_2026-11-16.md) and actual responses registered as an implementation check; exported-genome behavioural evaluation remains next. Full Rust suite 129 tests, 10 thermal tests and all-target Clippy pass.
- At 19:07 UTC the current recheck had completed seeds100 and101, with seed102 running. The guard was monitoring near73°C and had performed real 75°C/65°C cooling cycles. No active hardware throttling at the follow-up check. Finalizer status remains waiting; no partial acceptance claim. Pi and local native instrument JSON match at all five delays. Source d35dd2c CI and Pages passed.
- Owner previously requested project-only files plus internet/GitHub without prompts; on returning, explicitly dropped that configuration task and requested normal development. Prepared [permission template](CODEX_PROJECT_PERMISSIONS.md); it is NOT active and cannot replace this session's managed policy. Host-side activation and boundary verification are still required. No global configuration was changed.

## Active development session — 2026-11-15 (stopping-point handoff)

Read [SESSION_HANDOFF_2026-11-15.md](SESSION_HANDOFF_2026-11-15.md) first to resume. The local checkout was reconciled with 14 incoming commits before development; earlier local sharing/negative-damage findings were already fixed upstream.

Completed this session:
- [x] Conflict-resolved clean baseline committed and pushed; portfolio pending source preserved separately.
- [x] 0.2 Measured per-tick organism-reserve ledger, adversarial energy tests, and live demo Energy tab.
- [x] 0.3 Ten forced mechanism fixtures for nine enabled flags, including OFF noninterference; scope in the scientific council note.
- [x] 0.4 Same-build checkpoint/resume, atomic checkpoint writes, genome export, exact split-run and live-brain-lag tests. Sweep checkpoint options remain unsupported.
- [x] Generated sweep validation, non-overwriting sessions, source/build/config/status manifests, failure-safe two-worker Pi launchers.
- [x] 0.6 Public claim corrections, results registry, primary-source council review and draft (not registered) study protocol.
- [x] Pages ALSA prerequisite fixed; portfolio card updated to capabilities and limitations; Energy-tab deployment verified green.
- [x] UI tour placement/focus and KPI width improvements; real WASM/energy smoke and presentation tests added to deployment gate.
- [x] Short six-run Pi ecology pilot completed: 6/6 survived 20k ticks; zero cap-blocked births. [Report](results/PHASE9A_PILOT_2026-11-15.md), CAL-2026-11-15-FOOD.
- [x] 0.1 Finite-duration baseline calibration completed 2026-11-17: approved 0.008/cap 2,000 passes 10/10 seeds×200k ticks; full second-machine replication remains unmeasured.
- [x] 0.5 Cross-platform state equivalence measured and fixed (2026-11-15, later session; see "Cross-platform determinism" below). Checkpoint files still require the build that wrote them.
- [x] Independent environment RNG/replayable drought schedules (2026-11-16, see below).
- [x] Node-history/crossover fixes (2026-11-16, see "Node identity" below).
- [ ] Broader UI visual QA (inspector, Follow camera).

### Cross-platform determinism (2026-11-15, session after the Codex handoff)

Reviewed the handoff's code (energy ledger, checkpoints, runner provenance, mechanism fixtures): all 94 tests passed on re-run, CI and Pages were green, and the Pi was idle. Then did handoff item 2.

- **Measured:** with identical initial state, the macOS build and the WASM build diverged after ONE tick. "Same seed = same run" held only on a single platform.
- **Cause 1:** `f64::sin`/`cos`/`exp`/`ln`/`powf` call the platform maths library; one creature's position differed in its last bit, which changed mate choice and every later random draw.
- **Cause 2:** five `gen_range(0..len)` draws over `usize` consumed 32 bits of randomness on WASM and 64 bits natively.
- **Fix:** `sim_core::math` wraps the pure-Rust `libm` crate and provides `random_index` (always a `u32` draw). `tests/portability.rs` scans sim-core for either pattern.
- **Result:** fingerprints of the complete state are identical at ticks 0, 1, 100, 2,000 and 20,000 on macOS (aarch64, rayon), Raspberry Pi 5 Linux (aarch64) and WASM (wasm32, single thread), default config, seed 42. CI now compares x86-64 Linux with WASM on every push.
- **Tools:** `sim-runner fingerprint` and `web/demo/tests/conformance.mjs`; `--dump-dir` writes full states for diffing. Single runs gained `--seed` and `--set` (needed for checkpointed validation batches).
- **Also fixed:** `wrap_coord` could return exactly the world width for a tiny negative remainder (out of range for grids and checkpoint validation).
- **Consequence:** trajectories changed on native builds, so earlier runs (including CAL-2026-11-15-FOOD) cannot be reproduced bit-for-bit by current code. They remain calibration observations. The WASM trajectory for a given seed did not change.
- **Limits:** one config and one seed were compared on three targets; x86-64 is covered by CI only. Equality is measured, not proven for every input.

### Held-out validation batch (launched 2026-11-16 02:17 UTC, in progress)

- **Plan, fixed before data:** [PHASE9A_VALIDATION_PLAN.md](results/PHASE9A_VALIDATION_PLAN.md). Rates 0.008, 0.010, 0.012 × seeds 100–109 × 200,000 ticks, cap 2,000. Four acceptance criteria; the smallest acceptable rate becomes the proposed default, applied only with the owner's agreement.
- **Batch on the Pi:** `~/Digital-Terrarium/output/phase9a/validation_20261116T021704Z_23586`, source `2b1c837`, pinned executable `sim-runner` and `base.toml` inside the batch. `source_dirty` is true only because of the untracked `nohup.out` and `run_overnight.sh`; the batch's `source_status` file records exactly that.
- **How it runs:** `analysis/launch_pi_validation.sh` + `analysis/pi_validation_jobs.sh`. Two single runs at a time, one thread each, checkpoint every 50,000 ticks. Re-running the launcher with `RESUME=<batch>` skips finished jobs and resumes the rest from their latest checkpoint. Tested locally: a resumed job ends byte-identical to an uninterrupted one.
- **Progress at 06:51 UTC:** 8 of 30 runs complete, none failed. Runs take 45–72 minutes each, so the batch needs roughly 12 more hours.
- **Thermal:** the batch logs temperature every minute (`thermal.log`). In 275 minutes the Pi was at its soft temperature limit for 217 and actively throttled for 38, peaking at 85.6 °C, even at two threads. Results are unaffected (trajectories do not depend on clock speed), but the Pi needs better cooling before anything longer.
- **Commits reached the Pi by git bundle**, because pushing to GitHub from the agent session was blocked. Local `main` is ahead of `origin/main` until the owner pushes. A copy of the binary that was in `target/release` before the update is kept as `output/phase9a/launch_20261115T230545Z_22098/sim-runner-2f49287`; its file time is later than the pilot launch, so it is not verified to be the pilot's executable.
- **Next:** when the batch finishes, write the summary script (join sessions on tick), apply the plan's criteria, register the outcome in RESULTS.md.

### Environment random stream (2026-11-16)

- **Problem:** droughts drew from the same random stream as evolution. Switching a feature off changed how many draws the organisms made, which moved every later drought. Two arms of an experiment therefore lived through different environments.
- **Fix:** `Simulation::env_rng`, ChaCha stream 1, used only for drought timing and placement. Optional `food.droughts.schedule_seed` replays one schedule across world seeds, or varies the schedule with the founders fixed. Checkpoint format version is now 2.
- **Tested** (`crates/sim-core/tests/environment_stream.rs`): the schedule is identical with sex, epigenetics or niche construction off and with a quarter of the food, while the organisms in those arms do diverge; the schedule seed replays across world seeds and does not change the founding population.
- **Still shared:** nothing else in the environment is random (seasons and terrain are deterministic). Mutation, mating and mortality share the main stream by design.
- **Consequence:** trajectories with droughts enabled changed again. The validation batch running on the Pi uses the build from before this change (`environmental_rng = "shared_with_evolution"` in its manifests); that is fine for a calibration of average behaviour, and its pinned executable is unaffected. Mac and WASM still match (re-checked through tick 12,000).

### Node identity (2026-11-16)

- **Problem 1, homology:** each genome numbered its own hidden nodes. Two lineages could give one id to unrelated neurons, and different ids to the same innovation, so crossover matched genes that did not share an origin. NEAT requires alignment by historical marking (Stanley & Miikkulainen 2002, §2.2 and §3.2).
- **Problem 2, lifetime cap (B3):** ids were also positions in the 64-slot activation buffer and only went up, so a lineage could add 29 neurons in its whole history.
- **Fix:** `genome::Innovations`, one registry per simulation. Splitting connection A→B yields the same node id in every lineage; a genome that already holds that node gets a fresh one. Ids are `u32` historical markers; `Genome::build_brain` maps them to local slots. Hidden nodes left without any gene are pruned at birth, so the limit (36 hidden with 20 inputs) is on neurons held at once. A node's depth is the midpoint of the split connection's ends, so depth is a function of the id and inherited genes always stay feed-forward.
- **What crossover does, stated accurately:** matching genes take their weight from either parent; all other genes come from the fitter parent. The child's wiring therefore equals the fitter parent's. That is the NEAT rule, not a defect (the scientific council's point), so audit item G is resolved as "homology fixed", not as "graph union added".
- **Tested** (`crates/sim-core/tests/innovation.rs`): same split gives the same id across lineages; re-splitting issues a fresh id; a shared innovation is inherited once in crossover; 200 add-and-prune rounds never hit a cap; a full genome with ids beyond 64 builds and runs; relabelling a node does not change brain output; 1,500 ticks of heavy structural turnover leave every genome and brain valid and checkpoint exactly.
- **Checkpoint format is version 3.** Mac and WASM still match through tick 12,000.
- **Not done:** the equal-fitness rule, a cross-check against a reference NEAT library, and raising `MAX_NODES` (roadmap 2.2).

### Species tracking in the demo (2026-11-16)

Presentation layer only (`crates/sim-wasm/src/species.rs`); sim-core's species statistics are unchanged.

- **Measured before:** 30 species announced as born and 20 as extinct in the first 6,000 ticks of the demo world (seed 42), about one every 120 ticks.
- **First hypothesis, mostly wrong:** that creatures were flipping between species because every creature was re-tested each update against a threshold that moves each tick, and because a species' reference genome was replaced whenever the reference creature died. Fixing that (species decided once at birth, fixed founding genome) only reduced the count to 25 and 18. Most of the turnover is real under the current definition: the threshold is 0.75 × the mean genome distance, so lineages keep drifting past it.
- **Second cause, the larger one:** a species was announced at 5 members, under 2% of the population. It is now announced only when it also holds 5% of the living population.
- **Measured after both:** 12 born and 6 extinct in the first 6,000 ticks (seed 42); over 16,000 ticks, 23 and 16 for seed 42, 38 and 34 for seed 7. At steady state that is one species event every 250–500 ticks.
- The lifetime-membership change is kept because it is the cleaner meaning (a creature never changes species or colour mid-life), not because it fixed the churn.
- Three unit tests cover lifetime membership, unannounced small lineages and the share rule.

### Where things stand at the end of the 2026-11-16 session

- Read `docs/SESSION_HANDOFF_2026-11-15.md`, last section, to resume.
- Pi validation batch at last check: `2026-11-16T14:53Z status=running done=25 failed=0 procs=2`. The summariser and the steps to take when it finishes are in the handoff.
- Memory (recurrent connections) is implemented and off by default (later the same day); see "Memory" below and `docs/research/MEMORY_DESIGN.md`.
- All 116 Rust tests and 10 demo tests passed at commit `389368f`.

### Memory (2026-11-16)

- **What it is:** a recurrent connection gives its target the source's activation from the previous tick, so a brain can hold state. One setting, `mutation.add_recurrent_probability`, default 0 (off).
- **Off is provably inert:** with the setting at 0 the engine reproduces saved pre-memory states exactly (two configurations), and no random number is drawn for the mechanism.
- **On works:** 8 tests, including exact continuation from a mid-run checkpoint; the energy ledger stays balanced; macOS and WASM match through tick 8,000, and CI checks Linux against WASM with memory on and off.
- **Not claimed:** that creatures use memory, or that it helps. Nothing in the current world requires remembering anything, so it may well not spread under selection (finding 4 of the audit).
- **Next:** a delayed-response behavioural test (roadmap 2.4), then an on/off comparison on the Pi at the validated food setting with the drought schedule held fixed.
- Details, decisions and limits: `docs/research/MEMORY_DESIGN.md`. 124 Rust tests and 10 demo tests pass.

### Requested for future development (owner, 2026-11-16)

- [x] **Expandable analysis tiles in the demo** (built 2026-11-16).
  - Clicking any headline tile (creatures alive, species, brain size, cooperation, food chain) opens a chart of that measure over time between the tiles and the 3D view; the view resizes to fit. Clicking the tile again, the close button or Esc collapses it.
  - Three time windows: last 2,000 ticks and last 10,000 ticks slide with the run at full resolution; "Whole run" grows with it.
  - Droughts are shaded, darker where more overlap. Hovering reads out the tick, value and droughts active.
  - Data: `web/demo/js/history.js` keeps a full-resolution ring for the sliding windows and the whole run in 1,024 points (neighbouring points are averaged and the stride doubles when it fills). History starts at the first measured tick.
  - Chart: `web/demo/js/ui/trend.js` (canvas). Seven tests in `web/demo/tests/history.test.mjs`. Checked with real screenshots on desktop and phone.
  - Not done: season bands on the time axis; comparing two measures on one chart; the same expansion for the Energy tab.

### Demo visual check (2026-11-16)

Real screenshots (desktop first view, tick ~5,400, Energy tab, map view, phone) of the current build confirmed the handoff's UI changes: the tour card no longer covers the legend, the species chip is fully visible, and the Energy tab shows measured values. Fixed one remaining flaw: KPI unit text ran under the sparkline; the unit now sits on its own line and the strip is 16 px taller. Still open: species churn dominates the feed; inspector and Follow camera not yet checked by eye.

No recurrence, plasticity, large factorial study or multi-day experiment started. Chrome access host remained unavailable despite connection attempts; no claim of visual browser verification. Existing historical stage lists below remain historical where this section supersedes them.

## Audit and Roadmap (Fable 5.2, 2026-11-15): read this first

This section supersedes "Current State" below, which describes the Session 2 codebase and is kept only as history. Phases 9–11 further down remain as detailed item lists; the stages here set their order.

### Where the project stands

The engineering is in good shape. The science has no standing results yet.

**What is solid**
- A deterministic, config-driven core (`sim-core`, about 4,600 lines) with no rendering code, 52 tests, and CI plus the Pages deploy green at `f009ab0`.
- A headless runner with sweeps, a Pi workflow, and an explainable 3D web demo driven by the same engine through `sim-wasm`.
- A literature base: `docs/research/PRIOR_ART.md`, `OEE_SHADOW_DESIGN.md`, and the bibliography.

**What is not**

On 2026-11-15 three silent bugs were found. Each one invalidates earlier results.

| Bug | Effect | How it was found |
|---|---|---|
| Sharing paid the recipient 1.5× the donor's cost | Free energy; populations lived on it; the "87% cooperation" result was an exploit | The owner noticed creatures camping in the demo |
| `[food.fruit]` was never read | Fruit was off in every run | The front-end engineer noticed a config flag reading false |
| Newborns counted as stressed | Methylation saturated by tick 2,000 | Code audit |

Every number in `README.md`, `ALIFE_2027_OUTLINE.md`, `EXPERIMENT_FINDINGS.md` and the session logs below predates these fixes. Treat the count of established results as zero.

**First run with honest energetics** (Pi, commit `f009ab0`, `hard_cap = 2000`, 100k ticks, 3 seeds per rate, 12 runs):

| `food.regrowth_rate` | Outcome |
|---|---|
| 0.0002 | Extinct in 3 of 3 (between ticks 8.8k and 27.8k) |
| 0.0003 | Extinct in 2 of 3; the survivor held about 55 creatures and dipped to 1 |
| 0.0004 | Survived in 3 of 3, holding about 63–77 creatures (lows of 16, 29 and 1) |
| 0.0005 (current default) | Survived in 3 of 3, holding about 65–88 creatures (lows of 17, 7 and 24) |

- The cap never blocked a birth in any run.
- Sharing fell from about 350 per tick to well under 1 per tick.
- **Conclusion:** the old ecosystem of 500–1,500 creatures was sustained by the sharing bug. With real energetics the default world supports about 70 creatures and sits close to extinction. Every parameter was tuned against the bug, so the ecology must be retuned before any experiment. The live demo now runs this sparse world too.

### Findings, in priority order

1. **Bugs are being found by luck, not by tests.** No test asserted that energy comes only from food, or that a config flag has any effect. Stage 0 adds an energy ledger and mechanism-effect tests.
2. **No persistence.** `Genome` and `Organism` are not serialisable, and there is no checkpoint, resume or genome export. A week-long Pi run dies with a power cut. Evolved brains cannot be tested, compared across eras, or seeded from a connectome. Every later stage depends on this.
3. **The brain cannot support the stated goal.** It is feed-forward only, holds at most 64 nodes (38 hidden after 19 inputs and 7 outputs), keeps no state between ticks, and does not learn within a life. Nothing that any theory of consciousness treats as relevant can arise in it.
4. **The world does not pay for cognition.** Evolution builds only what the environment rewards. Today a reflex ("turn toward food, eat") is close to optimal, so brains have no reason to grow. Adding brain capacity without changing the ecology will produce nothing.
5. **Crossover only mixes weights** (audit item G). The sex arm of any experiment is meaningless until a child can inherit structure from both parents.
6. **Cross-platform determinism is untested.** `sim-core` makes 24 calls to platform maths functions (`powf`, `exp`, `sin` and others). "Same seed = same run" may hold only on one machine; Mac, Pi and browser runs may diverge.
7. **Documentation asserts things that are stale or false.** README: population about 1,500, 30 tests, about 4,400 ticks/s. Outline abstract: "satisfying Bedau's criterion". `PRIOR_ART.md` lists the novelty claims to withdraw.
8. **Statistical power.** With 3–5 seeds only very large effects are detectable. Plan for 10–20 seeds, with metrics chosen before the run.
9. **The Pi is at its thermal limit.** It reached 85.6 °C and hard-throttled (`throttled=0xe0006`) with four concurrent runs and the fan at maximum.
10. **Repo hygiene for open source.**
    - `.claude/settings.local.json` is tracked on `main`.
    - The owner's main checkout has diverged from `origin`; current work lives in `.worktrees/merge` on branch `merge-remote`.
    - There is no CONTRIBUTING.md or CITATION.cff.
    - `sim.rs` is a single 1,643-line file and will not absorb water, parasites and plasticity cleanly.

### Second pass: line-by-line read of the core (2026-11-15)

**Covered:** the whole tick loop in `sim.rs` (sensing, actions, metabolism, combat, sharing, reproduction, death, regrowth), the brain forward pass, genome mutation and crossover, and food-grid construction. **Not yet read:** the spatial index, the pheromone, elevation and construction grids, the stats and species code, `sim-wasm`, the viewer and the runner.

**When each known bug entered** (first commit was 2026-11-07):
- Sharing multiplier: commit `9795f27`, 2026-11-09. Every run since then had free energy.
- Fruit never read: 2026-11-13, the day fruit was added.
- Methylation saturation: 2026-11-14, the day epigenetics was added.
- No code needs reverting; the fixes are forward. What resets is every result and every tuned parameter from 2026-11-09 on.

**New bugs (code does not do what the config and docs say):**
- [x] B1 (fixed 2026-11-15; fruit now grows only in zones with a multiplier above 1). The "barren" biome did nothing to algae. `best_multiplier` starts at 1.0 and takes the maximum, so a multiplier of 0.3 never applies (`world.rs`, FoodGrid::new). Fruit still grows there at 0.6× the base rate, so the barren zone is richer than open ground. Known since Session 7 and never fixed.
- [x] B2 (fixed 2026-11-15). Fruit did not regrow in a cell whose algae is full: the `continue` for full algae comes before the fruit block (`world.rs`, regrow_with_construction).
- [x] B3 (fixed 2026-11-16, see "Node identity"). A lineage could add at most 29 neurons, ever: `next_node_id` only went up and node ids doubled as buffer positions, so add-node stopped for good at `MAX_NODES` = 64.
- [x] B4 (fixed 2026-11-15). Attacking a victim whose energy is already negative gave negative damage: the attacker loses energy and its predation stats go down (`sim.rs:888`). Clamp at zero.
- [x] B5 (fixed 2026-11-15; `Config::validate` now rejects it). Nothing validated `child_energy <= parent_cost`, so a config could create energy at birth. The defaults (30 and 30) are fine.

**Design problems that distort results (decisions, not coding errors):**
- D1. An attack costs 5 energy on every tick the attack output is positive, whether or not anything is in range. In the fixed-code runs attacks are 0.0 per tick: predation is extinct. Earlier "predation emerged" findings were under free energy.
- D2. Construction costs up to 0.5 energy per tick (more than base metabolism, 0.3) whenever the output is outside ±0.01. That selects against the feature the niche-construction experiment is meant to study.
- D3. Mutation is biased toward smaller brains: prune 4% against add 2%. Connection counts drift down with no selection at all, which confounds every brain-size result.
- D4. Brain upkeep is about 0.25 energy per tick (215 connections) against base metabolism of 0.3, so thinking is over 40% of the resting budget.
- D5. Eating is free, and a creature cannot sense food in its own cell (vision rays start one cell out). "Always eat" is optimal, so the eat output carries no decision.
- D6. Hard-coded constants remain: diet efficiency 0.7, night vision ×0.5, the 30° kin threshold, offspring offset.

**Rule changes made on 2026-11-15 (owner asked for the design items to be implemented where the science supports them):**

| Item | What the literature supports | What changed | Old rule still available as |
|---|---|---|---|
| D1 attack cost | The energy cost of predation is in the pursuit and strike, not in readiness to attack (Holling 1959; Scantlebury et al. 2014) | `attack_cost` is paid only when a strike lands on a target in range | `combat.idle_attack_cost = 5` |
| D2 construction cost | Sustained work at 1–2× resting metabolism is physiologically plausible (Peterson, Nagy & Diamond 1990; Hammond & Diamond 1997), so the cost level was kept. The defect was that abstaining was nearly impossible | Dead zone widened from 0.01 to 0.2; effort and cost now rise smoothly from zero above it. The 0.2 is a modelling choice, not an empirical value | `niche_construction.activation_threshold = 0.01` |
| D3 mutation bias | Supported only in part. Real genomes can carry a deletion bias (Mira, Ochman & Moran 2001), so the bias is not wrong in itself. But the old comment's citation ("Dolson & Park 2012") could not be verified, and an unbiased default makes brain-size trends attributable to selection | `prune_probability` 0.04 → 0.02, which gives about zero neutral drift in connection count | `mutation.prune_probability = 0.04` |
| D5 free eating, no contact sense | Animals sense food on contact (Dethier 1976); handling food and searching for it compete for time (Holling 1959; Stephens & Krebs 1986) | New 20th brain input `food_here`; speed × `feeding_speed_factor` (0.5) while the eat output is on | `creature.feeding_speed_factor = 1` (the input stays) |

- D3 correction: the earlier note overstated this bias. The expected drift was about −0.017 connections per birth, roughly −10 per 100k ticks at 580 generations. The fixed-code runs lost 7.8 of 223 connections in 100k ticks, so the whole observed "pruning" is what mutation alone predicts. Selection was not visibly shrinking brains.
- D4 (brain upkeep is 40% of the resting budget) and D6 (remaining hard-coded constants) are unchanged.
- Citations in this table are from memory; verify before using them in a paper.

**Pi check of the new rules** (`hard_cap = 2000`, 20k ticks, 3 seeds, late-half means; old-rule runs at 0.006 were still running when this was written):

| Rules | Regrowth | Population | Lowest point | Attacks per tick | Predators |
|---|---|---|---|---|---|
| New | 0.003 | 510, 575, 511 | 135, 116, 144 | 2.0, 0.25, 5.2 | 8, 1, 15 |
| New | 0.006 | 691, 784, 759 | 308, 350, 409 | 18.7, 18.8, 20.2 | 49, 51, 49 |
| Old | 0.003 | 444, 460 (2 seeds) | 310, 315 | 1.05, 0.5 | 2, 1 |

- The new rules do not collapse into universal cannibalism. At 0.006, about 7% of creatures get most of their energy from hunting.
- The opening die-off is deeper under the new rules (lows of 116–144 against 310–315), because always-eating brains now travel at half speed until evolution gates eating on `food_here`.
- The cap never blocked a birth.
- Population is not proportional to food at high density: doubling regrowth from 0.003 to 0.006 raised it only about 1.4×.

**Why the world holds about 70 creatures.** It is arithmetic. Energy in: 10,000 cells × 0.0005 regrowth × about 1.3 (biomes) × 15 energy per food ≈ 100 energy per tick. Energy out: about 1.0–1.2 per creature per tick (0.3 metabolism, 0.25 brain, plus movement, temperature, crowding and construction). That gives 80–100 creatures; the runs show 65–88, with mean energy 28.7, below what a newborn starts with. That arithmetic fits the sparse world but breaks down at high density: 0.006 gives about 750 creatures, not 1,000, because crowding and predation costs grow. The 0.1 sweep should cover **0.002–0.012**.

### Demo state after visual check (2026-11-15)

Checked with real screenshots (desktop first view, after 5k ticks, map view, phone) driven through Chrome's DevTools protocol.
- [x] The demo world was nearly empty on the default food rate (5–15 creatures by tick 6,000). `configs/demo.toml` now sets `food.regrowth_rate = 0.002`: the population recovers from the opening die-off by about tick 600 and then swings between about 200 and 420 with the seasons, under the cap of 500.
- [x] Creatures were a few pixels wide. Default render scale raised from 1.0 to 2.2 (`web/demo/js/scene/creatures.js`).
- [x] Header said "500 creatures" regardless of the count; now "up to 500".
- [x] Deploy workflow runs the test suite before building, and stamps `version.json`; the demo header shows the commit it was built from.
- [ ] The tour card overlaps the Food legend on first view.
- [ ] The "Colour = species" chip is clipped at the right edge at 1440 px.
- [x] The feed was dominated by species churn ("Sp. 54" by day 6). Reduced on 2026-11-16; see "Species tracking in the demo".
- [ ] Not yet checked by eye: inspector drawer, Follow camera, the other data layers, tablet width.

### The consciousness goal

**What cannot be done.** No simulation can prove that digital consciousness can be created. There is no accepted theory of consciousness and no agreed way to measure it, so no behaviour or network structure can settle the question. A project that claims otherwise will not be taken seriously, and the claim would be false.

**What can be done.** Show which consciousness-relevant capacities evolve under ecological selection *without being designed in*, and which environmental pressures produce them. Most machine-consciousness work builds an architecture by hand. The evolutionary studies I know of (Edlund et al. 2011; Joshi, Tononi & Koch 2013; Albantakis et al. 2014) evolved tiny agents on single tasks, not an ecosystem. Whether this niche is really open needs its own prior-art pass (item 4.0).

**Restated goal:** an open-source evolutionary testbed for the origins of consciousness-relevant capacities, with theory-derived tests anyone can run. A positive result is evidence about what it takes for those capacities to evolve. It is a step, and should be described as one.

**Which theories give testable targets** (citations are from memory; verify before using them in a paper):

| Framework | What it says matters | What Terrarium could measure |
|---|---|---|
| Unlimited Associative Learning (UAL): Ginsburg & Jablonka (2019); Birch, Ginsburg & Jablonka (2020) | Open-ended learning about novel, compound stimuli and actions, proposed as the evolutionary marker of minimal consciousness | A behavioural battery: trace conditioning, second-order conditioning, novel compound discrimination, reversal learning |
| Recurrent processing: Lamme | Feedback loops | The fraction of brains with recurrent edges, and whether lesioning them costs fitness |
| Global workspace: Baars; Dehaene, Lau & Kouider (2017) | A limited-capacity hub that broadcasts to many processes | Whether evolved networks form a bottleneck hub, and how broad the deficit is when it is lesioned |
| Higher-order theories / metacognition | Monitoring one's own states | An opt-out task: does a creature decline a risky choice when its information is poor? |
| Attention schema: Graziano; Wilterson & Graziano (2021) | A model of one's own attention | Later; needs an attention mechanism first |
| Integrated information: Tononi | Causal integration (zero for feed-forward networks) | Integration proxies tracked over evolutionary time |
| Indicator-properties report: Butlin, Long et al. (2023) | A checklist drawn from the theories above, plus agency and embodiment | A per-lineage scorecard |

UAL is the best first target. It is defined behaviourally, it was proposed as an *evolutionary* transition, and "UAL evolved in an open-ended ecosystem" can be defended without taking a side on what consciousness is.

**Ethics.** If indicators accumulate, the project needs a written precautionary policy before scaling up (Birch 2017, 2024). It is cheap to write now and awkward to retrofit.

### Plan

Each stage ends in a gate. Do not start a stage until the previous gate is met.

**Stage 0: a trustworthy instrument** (about 1–2 weeks)
- [x] 0.1 Finite-duration ecology calibration: approved food 0.008/cap 2,000 passed ten seeds×200k ticks, all four criteria; report CAL-2026-11-17-BASELINE. B3 was fixed before this recheck. Demo retains its separate watchability overlay; broader two-machine replication remains unmeasured.
- [x] 0.2 Energy ledger (implemented this session; scope above). Account for every source and sink each tick (food, fruit, metabolism, movement, brain, crowding, temperature, combat loss, sharing loss, reproduction, deaths, clamping at the energy cap). In debug builds, assert that the change in total energy equals the ledger. Expose the ledger in `TickStats`.
- [x] 0.3 Mechanism-effect tests. Forced, matched fixtures exercise all nine enabled flags and test scoped OFF noninterference. Do not require arbitrary ecosystem divergence when a mechanism is inactive; see the council note for coverage limits.
- [x] 0.4 Persistence (same-build single-run checkpoint/resume implemented; sweeps pending). Serde for `Genome`, `Organism` and simulation state; `--checkpoint-every` and `--resume` in sim-runner; periodic genome export. Test: 2,000 ticks straight must equal 1,000 + save + load + 1,000, bit for bit.
- [ ] 0.5 Cross-platform determinism. Hash the state after 5,000 ticks on Linux in CI, in WASM under node, and on the Pi. If the hashes differ, move the affected maths to a portable implementation (the `libm` crate).
- [x] 0.6 Truth pass on the docs (completed this session). Fix README metrics and the outline abstract, and withdraw the claims `PRIOR_ART.md` lists. Start `docs/RESULTS.md`, where a result is listed only with its commit, config, seeds and analysis script. Nothing enters README unless it is in that file.
- [ ] 0.7 Repo. Decide whether to untrack `.claude/settings.local.json` (owner's call), resync the main checkout, add CONTRIBUTING.md and CITATION.cff.
- [ ] 0.8 Pi. Run 3 jobs at once or improve cooling. No multi-day run before 0.4.
- **Gate:** tests green including 0.2–0.5, and a 10-seed baseline that reproduces on two machines.

**Stage 1: first results that stand** (about 2–3 weeks of Pi time)
- [x] 1.1 Crossover with population-wide node identity (2026-11-16). Still open: validate against neat-python or TensorNEAT, and the equal-fitness rule (inherit both parents' extra genes).
- [ ] 1.2 Neutral shadow model, following Polyworld's lockstep mode (Yaeger, Griffith & Sporns 2008) and `OEE_SHADOW_DESIGN.md`.
- [ ] 1.3 `docs/PREREG.md`: hypotheses, metrics, seeds and analysis fixed before the runs.
- [ ] 1.4 Factorial experiment: sex × niche construction × epigenetics, at least 10 seeds per cell, scored against the shadow, with a shuffled-methylation control.
- [ ] 1.5 Cooperation re-test under energy-conserving sharing. A null result is a result.
- [ ] 1.6 Model description in ODD format, using MODES metric names.
- **Gate:** a controlled, reproducible estimate (including an informative null), appropriate uncertainty and independent validation. Neutral-shadow comparisons are required for activity selection-versus-drift claims, not every ecological endpoint.

**Stage 2: a brain that could support cognition** (about 3–4 weeks; can overlap Stage 1's Pi time)
- [x] 2.1 Recurrent connections (Phase 9b): implemented 2026-11-16, off by default. Not yet shown to be used by evolution.
- [ ] 2.2 Raise `MAX_NODES` from 64 to 256, so capacity is limited by energy cost and not by the buffer.
- [ ] 2.3 Evolved neuromodulated plasticity (Phase 9c; Soltoggio et al. 2008).
- [ ] 2.4 Cognitive test battery in sim-runner, run on exported genomes in fixed arenas: detour, delayed response, trace conditioning, reversal learning, second-order conditioning, novel compound discrimination. Report it as a UAL checklist.
- **Gate:** the battery runs in CI on a reference genome; feed-forward creatures fail the memory tasks; evolved recurrent creatures pass at least one.

**Stage 3: a world that pays for cognition** (about 4–6 weeks)
- [ ] 3.0 Split `sim.rs` into one module per tick phase before adding mechanics.
- [ ] 3.1 Water in low ground (Phase 10). Commuting between water and food rewards spatial memory.
- [ ] 3.2 Cue-predicted resources whose meaning changes within a lifetime, for example a food type that turns toxic for a season. Evolution cannot hard-wire the answer, so learning is rewarded (Hinton & Nowlan 1987).
- [ ] 3.3 Parasites (Phase 10).
- [ ] 3.4 Individual recognition and repeated encounters, so reciprocity becomes possible (Trivers 1971).
- [ ] 3.5 Predator–prey tournaments across eras (Cliff & Miller 1995).
- **Gate:** learning or recurrent genotypes outcompete reflex genotypes in the new ecology and not in the old one. That shows cognition is being selected for, and is publishable by itself.

**Stage 4: the indicator programme** (ongoing once Stages 2 and 3 are in)
- [ ] 4.0 Prior-art pass on evolved-agent consciousness research, in the style of `PRIOR_ART.md`, before claiming the niche.
- [ ] 4.1 Per-lineage scorecard over evolutionary time: recurrence, integration proxies, hub structure with lesion tests, the opt-out metacognition task, the UAL battery.
- [ ] 4.2 Pre-registered experiments that discriminate between theories. Examples: does UAL arrive together with a jump in integration? Does a workspace-like hub appear only under multi-task pressure?
- [ ] 4.3 `docs/ETHICS.md`: the precautionary policy and what would trigger a review.
- [ ] 4.4 Scale. Once brains pass about 1,000 neurons the Pi is the wrong machine; batch the brain forward pass for a GPU and compare against JaxLife.
- [ ] 4.5 Release the battery and scorecard as an open benchmark. Others are more likely to adopt that than the simulator.

### What not to do
- Do not claim consciousness, sentience or "fruit-fly level" anywhere.
- Do not add water, parasites or other mechanics before Stage 0 and the Stage 1 experiments. Each one changes every baseline.
- Do not start multi-day Pi runs before checkpointing exists.
- Do not let unattended sessions write results into README. Results go through `docs/RESULTS.md`.

### Next five actions
1. Launch the 0.1 regrowth sweep. This needs the owner's go-ahead, because the Pi is hard-throttling.
2. Build the 0.2 energy ledger and the 0.3 mechanism-effect tests.
3. Build 0.4 persistence.
4. Do the 0.6 truth pass on README and the outline.
5. Build 1.1 innovation-number crossover.

---

## Current State (historical: Session 2 codebase, superseded by the audit above)

### What's Built and Working
- [x] **sim-core**: World, food grid, spatial grid (counting-sort), brain (13->8->3 FF net), genome (evolvable weights + mutation rate + color hue), organism, deterministic tick loop
- [x] **sim-viewer**: macroquad + egui, pan/zoom camera, food/creature rendering, inspector panel
- [x] **sim-runner**: Headless CLI, single runs with CSV stats, parallel parameter sweeps via rayon
- [x] **CI pipeline**: GitHub Actions — clippy, test, fmt, WASM build check
- [x] **Tuned parameters**: 5/5 seeds survive 100k ticks, ~1100 stable pop, ~750 births/1000 ticks
- [x] **Cumulative stats**: total_births/total_deaths added to TickStats
- [x] **Autonomous workflow**: CLAUDE.md + .claude/settings.local.json with scoped permissions

### Key Metrics (current tuned config)
- Population: ~1100 stable equilibrium
- Turnover: ~75,000 births over 100k ticks (massive evolutionary churn)
- Mean energy: ~30 (threshold=60, so only good foragers reproduce)
- Speed: ~4400 ticks/s (rayon parallel + fast_tanh), up from ~1000 single-threaded
- Deterministic: same seed = same run (ChaCha8Rng)

### GitHub
- Repo: `dchavan2192/Digital-Terrarium`
- Branch: `main` (7 commits)
- All CI checks passing

---

## Active Work Items

### Phase 1: Foundation [COMPLETE]
- [x] Scaffold workspace (sim-core, sim-viewer, sim-runner)
- [x] Implement core simulation loop
- [x] Build visual renderer
- [x] Build headless CLI + sweep tool
- [x] Performance optimization (~50 -> ~900 ticks/s)
- [x] Parameter tuning for stable 100k-tick runs
- [x] Add cumulative birth/death tracking

### Phase 2: Biological Realism [IN PROGRESS]
- [x] **Probabilistic mortality** — Gompertz-Makeham mortality curve implemented
  - Per-tick death probability: h(age) = λ + α·exp(β·age)
  - β (aging_rate) is a genomic trait — longevity can now evolve
  - max_age retained as absolute safety cap
  - Backed by Gompertz (1825), Williams (1957), Kirkwood (1977)
- [ ] **Research-backed design decisions** — Every mechanic should cite why it exists
- [ ] **Define control environment** — Standard test config for reproducible comparisons

### Phase 3: Ecological Complexity [COMPLETE]
- [x] Attack/consume mechanic (predator-prey dynamics)
- [x] Body size as evolvable gene (affects speed, metabolism, attack power/defense)
- [x] Speciation detection and lineage tracking
- [x] Trophic level analysis
- [x] **Spatial resource gradients (biomes)** — ecological niches driving adaptive radiation
  - BiomeZone config: circular zones with regrowth_multiplier and smooth falloff
  - Per-cell regrowth rates computed at init, supports wraparound distance
  - Default config: 2 lush oases (3× regrowth), 1 moderate center (2×), 1 barren zone (0.3×)
  - Renderer shows biome tinting: bright green (lush) → brown (barren)
  - Backward compatible: no biomes = uniform regrowth (existing configs unchanged)
  - Citations: Schluter (2000) adaptive radiation, Rosenzweig (1995) species diversity

### Phase 4: Social Intelligence [COMPLETE]
- [x] Broadcast signal channel (brain output -> nearby brain input)
- [x] Energy transfer/sharing between creatures
- [x] Kin recognition via color hue + signal correlation
- [x] Cooperation rate measurement

### Phase 5: Cognitive Complexity [COMPLETE]
- [x] **Variable hidden layer size** — evolvable hidden neuron count (4-32)
  - hidden_size as genomic trait, 2% mutation rate (±1 neuron per reproduction)
  - Weight vector resized on topology change (new neurons get Xavier-scale init)
  - Brain cost scales naturally with actual neuron count (already brain_cost × neuron_count)
  - Stack-allocated forward pass preserved (uses MAX_HIDDEN=32 fixed buffer)
  - Genome distance accounts for topology difference (0.8× weight for speciation)
  - Viewer inspector shows full topology (e.g., 15→8→6)
  - Citation: Stanley & Miikkulainen (2002) NEAT topology evolution
- [x] **Full NEAT-style topology** — connection genes replace dense weight matrix
  - ConnectionGene: (from, to, weight, enabled) — arbitrary feed-forward connections
  - HiddenNode: (id, depth) — nodes at variable depths for topological ordering
  - Add-connection mutation (3%): new edge between unconnected feed-forward pair
  - Add-node mutation (1%): split existing connection A→B into A→C→B (classic NEAT)
  - Toggle-connection mutation (1%): flip enabled/disabled on random connection
  - Sparse forward pass: connections pre-sorted by eval order, single linear pass
  - Stack-allocated activation buffer [f64; MAX_NODES=64] — zero heap allocs per tick
  - genome_distance: merge-join on sorted connections (structure + weight + traits)
  - Skip connections emerge naturally (input→output, deep hidden paths)
  - Citations: Stanley & Miikkulainen (2002), Kassahun & Sommer (2005)
- [x] **Information-theoretic brain analysis** — Shannon entropy of brain outputs
  - Per-organism activation histograms: 6 outputs × 8 bins with exponential decay (τ≈100 ticks)
  - shannon_entropy(): normalized [0,1], 0=deterministic, 1=maximum entropy
  - activation_to_bin(): maps [-1,1] output to histogram bin index
  - Population behavioral diversity: entropy of peak-bin distributions across all organisms
  - New TickStats: mean_brain_entropy, behavioral_diversity, mean_connections
  - Viewer: stats panel shows entropy/diversity/connections; inspector shows per-organism entropy
  - CSV output includes all new fields automatically (serde::Serialize)
  - Citations: Shannon (1948), Beer & Williams (2015)

### Phase 2.5: Environmental Dynamics [IN PROGRESS]
- [x] **Seasonal/cyclical food variation** — regrowth oscillates sinusoidally over configurable period
  - `[food.seasons]` config: amplitude (0-1) and period (ticks)
  - Formula: `multiplier = 1.0 + amplitude * sin(2π * tick / period)`
  - Default: amplitude=0.3, period=10000 (regrowth swings 70%-130%)
  - Applied globally on top of biome multipliers
  - Backward compatible: amplitude=0 disables (existing configs unaffected)
  - Viewer shows current seasonal multiplier percentage
  - Citation: Botkin (1990) "Forest Dynamics: An Ecological Model"
- [x] **Population density pressure** — creatures in crowded cells have higher metabolism (Allee effect)
  - `[density_pressure]` config: `cost_per_neighbor = 0.01` (extra energy per tick per nearby creature)
  - Neighbor counting piggybacks on existing `for_each_nearby` spatial query — zero extra allocation
  - New TickStats field: `mean_crowding` (average neighbor count across living organisms)
  - Viewer stats panel shows "Mean crowding: X.X"
  - Backward compatible via `#[serde(default)]` — cost=0.0 disables (existing configs unaffected)
  - Two new integration tests: `density_pressure_increases_metabolism`, `density_pressure_no_effect_when_zero`
  - Citation: Allee (1931) "Animal Aggregations: A Study in General Sociology"
- [x] **Dynamic speciation threshold** — compute from population genome variance

### Testing & Validation [ONGOING]
- [x] **Biome-aware food regrowth integration test** — verifies FoodGrid with BiomeZone config produces per-cell regrowth rate variation; lush zones regrow faster than moderate zones, which regrow faster than neutral (no-zone) areas; also verifies uniform rates when no biomes configured
- [x] **Seasonal food variation tests** — `seasonal_food_variation` verifies sinusoidal multiplier at tick 0 (baseline), peak summer (1+amplitude), peak winter (1-amplitude), and actual food accumulation at each phase; `seasonal_no_effect_when_amplitude_zero` verifies backward compatibility

### Phase 7: Scientific Novelty — Publishable Mechanisms [COMPLETE]
**Goal**: Implement features no other ALife platform has, targeting ALIFE 2027 submission.
**Research council** (Nov 14, 2026) surveyed ALIFE 2023-2026 proceedings and identified 5 highest-impact features.

- [x] **Activity-dependent neural metabolism** — brain cost scales with ACTIVITY, not just structure
  - Effective cost = structure_cost + Σ|weight × activation| per tick
  - Selects for sparse coding: brains that activate few connections are cheaper
  - Citations: Attwell & Laughlin (2001) "An Energy Budget for Signaling in the Grey Matter of the Brain", Niven & Laughlin (2008), Lennie (2003)
- [x] **Sexual reproduction with NEAT crossover** — the first major evolutionary transition
  - Mate-finding, crossover of different-topology genomes, twofold cost of sex
  - Enables recombination: faster adaptation, Muller's ratchet prevention
  - Citations: Maynard Smith (1978) "The Evolution of Sex", Muller (1932), Stanley & Miikkulainen (2002) §3.2
- [x] **Open-ended evolution (OEE) metrics** — MODES toolbox (Dolson et al. 2019)
  - Evolutionary activity statistics: novelty rate, cumulative activity D(t), active traits
  - Lineage tracking: generation depth, lineage diversity, coalescence
  - THE measurement that proves the sim produces genuine open-ended evolution
  - Citations: Bedau & Packard (1991), Dolson et al. (2019) "The MODES Toolbox" *Artificial Life* 25(1):50-73, Taylor et al. (2016)
- [x] **Niche construction** — organisms modify their environment (ecological inheritance)
  - Construction grid: organisms deposit/remove material that alters local food regrowth
  - Creates a second inheritance system beyond the genome
  - No neural-agent ALife platform has this — completely novel
  - Citations: Odling-Smee, Laland & Feldman (2003) "Niche Construction: The Neglected Process in Evolution", Laland et al. (1999) PNAS 96:10242-10247
- [x] **Epigenetic inheritance** — heritable non-genetic state (methylation marks)
  - Per-connection methylation [0,1] silences connections; responds to environmental stress
  - Offspring inherit parental methylation with noise — transgenerational ecological memory
  - Almost completely absent from ALife literature (only EpiAL, Sousa & Costa 2010)
  - Citations: Jablonka & Lamb (2005) "Evolution in Four Dimensions", Heard & Martienssen (2014) Cell 157(1):95-109

**Planned experiments** (each: feature ON vs OFF, 5+ seeds, 500k+ ticks):
- [ ] OEE (REWORDED 2026-11-15): Show *excess* activity over a neutral shadow model is positive and unbounded — the target is Channon class 4b (unbounded median activity, bounded diversity, because hard_cap limits diversity). Raw D(t) growth alone proves nothing, since drift also grows. Design: docs/research/OEE_SHADOW_DESIGN.md (~1.5 days). Prerequisite: an order-independent topology_hash.
- [ ] Core ALIFE 2027 paper: factorial sweep (sex × niche × epigenetics × droughts), with each mechanism scored by its excess activity over the shadow (Channon 2006; Taylor et al. 2016)
- [ ] Follow-ups: shadow vs MODES persistence filter on the same runs (Dolson et al. 2019); hard_cap scaling sweep (does diversity saturate?)
- [ ] Niche construction: Do niche-constructing lineages survive droughts better?
- [ ] Epigenetics: Do stress-primed offspring outperform naïve offspring post-drought?
- [ ] Sexual vs asexual: Does sex accelerate adaptation speed and prevent Muller's ratchet?

**Future high-impact** (post Phase 7):
- [ ] Horizontal gene transfer (HGT) — connection gene exchange between organisms
- [ ] Major evolutionary transition to multicellularity — group-level selection emerging from cooperation

### Phase 7.5: Correctness Fixes Before Experiments [BLOCKING — from code audit 2026-11-15]
Experiments on the Pi are ON HOLD until these are fixed; otherwise the ON/OFF ablations measure bugs.
The on/off switches themselves were verified to disable each feature cleanly, and no determinism or allocation violations were found.
- [x] **A. Methylation saturates** — child_energy/max (0.15) < stress_threshold (0.3), so all newborns count as "stressed" (sim.rs:732); mean_methylation hits 0.78 by tick 2000. Fix: stress relative to reproduction threshold or a rolling energy trend; make the threshold configurable.
- [x] **B. Methylation never affects the living brain** — brain is only built at spawn/birth (sim.rs:237, 1006); per-tick methylation changes the genome only. Either rebuild the brain periodically or document it as an inheritance-only effect.
- [x] **C. inheritance_noise=0 panics** — empty gen_range (sim.rs:1001); uniform noise, not Gaussian as the docs say.
- [x] **D. Old 15/6 configs read a hidden node as `construct`** (brain.rs:186); validate inputs>=19 / outputs>=7, and update configs/intense.toml.
- [x] **E. Niche bonus ignores biomes/seasons/droughts** — additive global rate (sim.rs:1088-1101), so constructed cells regrow inside droughts. Docs say a multiplier; ConstructionGrid::regrowth_multiplier is dead code. Signed mean_construction_level cancels out.
- [x] **F. No twofold cost of sex** — only the initiator pays (sim.rs:1031); mates can be reused without limit; the 0.5 mate factor is hard-coded (sim.rs:958).
- [ ] **G. Crossover only mixes weights** — the child's edge set always equals the fitter parent's; hidden-node ids aren't global innovation numbers.
- [ ] **H. OEE metrics mislabeled** — "D(t)" averages over all traits ever seen (activity.rs:110), not active ones; lineage_diversity can only decrease (hits 1 by tick 8k); sexual/asexual births are per-tick, not cumulative. **Add a neutral shadow model** (design: docs/research/OEE_SHADOW_DESIGN.md). — metric fixes done (extant-only Ā(t), windowed lineage_diversity, cumulative birth counters, order-independent topology hash v2); shadow model still open.
- [x] **K. Hard-coded constants** → config: mate factor 0.5, construct dead zone 0.01, regrowth cutoff 0.001, structural mutation probabilities in genome.rs.
- [x] Update stale CLAUDE.md (still says 15→6 I/O, 22 tests; config table out of date).
- [x] Sweep tool: on/off switches for reproduction.sexual, niche_construction.enabled, epigenetics.enabled
- [x] Pi launch script: analysis/launch_pi_phase7.sh (sims run on the Pi only — never locally)
- [ ] README OEE section (uncommitted): its numbers and images are unverified (the images don't exist). Replace with real Pi results.

- [x] **Fruit was silently disabled in every run**: the sim read a top-level `fruit` table, but the config defines `[food.fruit]`. Fixed, with a regression test (2026-11-15). Earlier results ran without fruit.
- [ ] sim-core v2 data for the demo: per-organism last action + target, death causes, energy-flow history, hidden activations (see the ABI doc's v2 table)

### Phase 8: Explainable 3D Web Demo [IN PROGRESS — started 2026-11-15]
**Problem**: The current demo (macroquad triangles + egui panel) looks like a game; visitors can't tell what's happening, and the stats are hard to read.
**Goal**: A portfolio-grade, explainable visualization that makes the science legible to a non-expert in 10 seconds.

**Architecture decision (D7)**: The new web front end is **Three.js + HTML/CSS dashboard**, driving sim-core compiled to WASM through a thin new crate `crates/sim-wasm`.
It uses a plain `extern "C"` ABI and zero-copy Float32Array views into WASM memory, so it needs no wasm-bindgen tooling.
sim-core keeps zero rendering code. The macroquad viewer stays as the native dev tool, and the old WASM page stays available at `web/classic.html`.

**Visual design**:
- 2.5D/3D world: terrain mesh from the elevation map, ground coloured by food density / biome, drought zones as dry overlays, day/night lighting.
- Creatures as small instanced low-poly "mini agents" (body + eyes): size = body_size, colour = species, a glow/pulse for actions (eating, attacking, sharing, constructing), short motion trails.
- Orbit camera + top-down map toggle; click a creature → follow-cam + inspector (brain graph, energy budget, lineage, trophic level).
- Data layers toggle: food, temperature, pheromones, niche construction, elevation.

**Explainability / stats**:
- Live "what's happening" event feed in plain English (droughts, predation spikes, new species, population crashes).
- A few headline KPIs with sparklines (population, species, mean brain size, cooperation rate, trophic level), plus a full charts tab.
- Guided "tour" / legend overlay explaining creatures, colours, energy, evolution.

**Work items**:
- [x] Design spec — docs/research/DEMO_UX_DESIGN.md
- [x] crates/sim-wasm: C-ABI exports (ABI: docs/research/SIM_WASM_ABI.md), with stable species tracking + derived events
- [x] web/demo/ front end (Three.js 0.160.0): MVP done. v2 still open: charts tab, trails, share/construct cues, light theme
- [x] Update deploy-pages.yml + ci.yml to build sim-wasm + ship the new front end
- [ ] Portfolio card (/Users/dineshchavan/projects/portfolio-dhruv/ds.html): fix stale claims, link the new demo, add a screenshot
- [ ] Visual QA in a browser (needs the user's OK — the demo runs the sim locally)
- [ ] Deploy (push to main — needs the user's explicit OK)

### Phase 9.0: Energy Conservation Bug in Sharing [BLOCKING — found 2026-11-15]
- [x] **FIXED (2026-11-15): Sharing created energy from nothing.** The donor pays `share_amount` (5) but the recipient gets 5 × `benefit_multiplier` (1.5) = 7.5 (sim.rs, sharing step). Two neighbours sharing every tick each net +2.5/tick, about 8× base metabolism, so clusters can camp in place indefinitely (seen in the demo). Run data: shares rise to ~350/tick (≈ +875 free energy/tick population-wide) while mean energy climbs 65 → 117 at the cap.
  - Breaks design principle 4 ("food is the only source of energy").
  - Probably why the population pins at hard_cap; it will also confound the Phase 9a regrowth sweep.
  - **The "cooperation evolved (87%)" result is invalidated**: cooperation was selected as a free-energy exploit, not by Hamilton's rule.
  - Fix: conserve energy (recipient gets ≤ the donor's cost, with a transfer loss). Hamilton's B > C should come from the *marginal fitness value* of energy (a starving recipient gains more than a well-fed donor loses), not from creating energy. Consider restricting gifts to donors above the reproduction threshold.
  - Add an energy-conservation integration test: total energy change per tick = food eaten − all costs (no other sources).
  - Re-run the v2 cooperation experiments after the fix.
  - Fix landed: `sharing.transfer_efficiency` (default 0.9, alias of the legacy `benefit_multiplier`) is validated to (0, 1]; `combat.energy_transfer_ratio` is validated to [0, 1]. New tests: `energy_never_increases_without_food` and `energy_creating_share_efficiency_is_rejected`.
  - [ ] Still to do: re-run cooperation experiments; optionally restrict gifts to donors above the reproduction threshold.
- [ ] **Prior-art audit (docs/research/PRIOR_ART.md)**: withdraw the false claims in PROGRESS and docs/ALIFE_2027_OUTLINE.md ("no neural-agent platform has niche construction": JaxLife 2024 and Chiba et al. 2020 do; "EpiAL is the only epigenetics precedent": Turner et al. 2013, Brawer et al. 2017; the EpiAL citation is ICAART 2010). Cite Polyworld's lockstep mode (Yaeger, Griffith & Sporns 2008) as the reference design for the shadow model. Call our cost of sex a "twofold per-offspring investment cost". Adopt the ODD protocol (Grimm et al. 2020) and the MODES metric names. Narrow novelty claim: a factorial genetic × epigenetic × ecological inheritance test in evolved neural agents, scored against a neutral shadow.

### Phase 9: Food-Limited Ecology + Cognition [PLANNED — agreed 2026-11-15]
**Why**: (1) Population pins at hard_cap (500) by ~tick 1000 and births are blocked at the cap (sim.rs:993). The cap, not food competition, decides who reproduces, which weakens selection and hides drought/niche/epigenetics effects. (2) Brains are tiny (~11–16 evolved neurons, ~220 connections vs ~140k neurons / ~50M synapses in a fruit fly, FlyWire 2024; 302 neurons in C. elegans). More importantly, they are purely feed-forward with no memory and no lifetime learning: behaviour is reflexive, closer to Braitenberg vehicles or bacterial chemotaxis. Never claim "fruit-fly level".

**9a. Food-limited carrying capacity** (do FIRST, before the Phase 7 ablations)
- [ ] Pi sweep of `food.regrowth_rate` (e.g. 0.0002–0.0005) with `creature.hard_cap` 1500–2000; pick the value where mean population ≈ 60–70% of the cap and the cap almost never binds
- [x] New stats: `cap_blocked_births` / `total_cap_blocked_births` (should stay ~0); `sweep --set KEY=NUMBER` for fixed overrides
- [x] Ran `analysis/launch_pi_phase9a.sh` on the fixed code (hard_cap=2000; regrowth 0.0002–0.0005 × 3 seeds × 100k ticks). Result: the range was far too low. Populations of 55–88 at best, extinction at 0.0002–0.0003, zero cap-blocked births. See the audit table at the top. Results on the Pi in `output/phase9a/`; pre-fix partial runs in `output/phase9a_prebugfix/`.
- [ ] Follow-up sweep over 0.001–0.005 (audit item 0.1)
- [ ] Check runs don't overshoot into extinction (all 5 seeds survive); keep the cap only as a performance backstop
- [ ] Then run the Phase 7 ON/OFF ablations at this food-limited setting
- Citations: Verhulst (1838) logistic growth; Allee (1931) density dependence; Lotka (1925) / Volterra (1926)

**9b. Memory: recurrent NEAT connections**
- [ ] Allow recurrent edges (previous-tick activations as inputs), evolved by add-connection; stack-allocated `[f64; MAX_NODES]` prev-state buffer per organism, no per-tick allocs; determinism preserved
- [ ] Config: `brain.allow_recurrent`, `mutation.add_recurrent_probability`
- [ ] Stats: fraction of brains with recurrent edges, mean recurrent edges
- Citation: Stanley & Miikkulainen (2002) NEAT supports recurrence

**9c. Lifetime learning: evolved neuromodulated Hebbian plasticity**
- [ ] Per-connection evolvable plasticity coefficients (ABCD Hebbian rule) gated by a modulatory output/neuron; weights change within a lifetime and are NOT inherited (genome keeps the initial weights), distinct from epigenetic methylation
- [ ] Config: `[plasticity] enabled, learning_rate`, plus mutation rates for the rule coefficients
- Citations: Soltoggio et al. (2008) evolved neuromodulated plasticity; Risi & Stanley (2010) adaptive HyperNEAT; Hinton & Nowlan (1987) the Baldwin effect
- Novelty angle: evolution + in-lifetime learning + epigenetics in one open-ended ecosystem lets us study the Baldwin effect

**9d. Measure intelligence behaviourally (not by neuron count)**
- [ ] A "cognitive test battery" in sim-runner: freeze evolved genomes from saved snapshots and test them in standard mini-arenas: detour (food behind a barrier), delayed foraging (remember a patch after it vanishes from view), kin vs non-kin discrimination, predator avoidance
- [ ] Score per generation → a figure showing cognitive capability evolving over time; compare ON/OFF for recurrence/plasticity
- [ ] Requires genome snapshot export (also needed for arms-race CIAO plots, Cliff & Miller 1995)

### Phase 10: Real Ecosystem — Water, Nutrient Cycling, Parasites [PLANNED — agreed 2026-11-15]
Order: AFTER the Phase 7 ablations, so first results aren't mixed with new mechanics.
- [ ] **Water as a second essential resource**: water pools in low elevation (reuse ElevationMap); per-organism hydration drains each tick (faster when warm and for larger bodies); drinking at water cells. Droughts shrink water bodies.
  - Brain: hydration input + "water" ray type, so inputs 19 → ~20–21; update validate/demo.
  - Essential resources, Liebig's law of the minimum (Tilman 1982). Expect commuting between water and food, and waterhole ecology.
  - Tune with a Pi sweep (avoid instant extinction).
  - Demo: render real water in the terrain.
- [ ] **Nutrient recycling / decomposition**: corpses become a nutrient grid that boosts regrowth; optional decomposer niche. Closes the matter loop (Lindeman 1942).
- [ ] **Parasites / disease**: pathogens transmitted between neighbours, evolving against host resistance genes → Red Queen dynamics; gives sex a real advantage (Hamilton 1980; Van Valen 1973).
- [ ] **Multiple nutrients** (e.g. C/N) for resource-ratio coexistence (Tilman 1982) — optional, after water.

### Phase 11: Real Brains + Integration [IDEAS — 2026-11-15, check prior art first]
- [ ] **Connectome seeding**: initialise some creatures with the real C. elegans connectome (302 neurons, ~7k synapses; White et al. 1986; WormWiring / OpenWorm c302 data), mapping chemosensory neurons → food inputs and motor neurons → turn/speed. Experiment: connectome-seeded vs random-start populations — does real wiring give an evolutionary head start, or is it rewired? Requires recurrence (9b). Fly larva (~3k neurons, Winding et al. 2023) is a possible stretch goal; an adult fly (FlyWire 2024, ~140k neurons) is not feasible per creature on a Pi.
- [ ] **Integration over evolution**: measure integrated-information / complexity proxies of evolved brains over time (Edlund et al. 2011; Albantakis et al. 2014). Frame it as complexity measurement, NOT consciousness; feed-forward nets score Φ = 0 under IIT. Reference: Butlin, Long et al. (2023) indicator properties.

### Phase 6: Analysis & Distribution [PLANNED]
- [ ] Python analysis pipeline for experiment plots
- [x] WASM build + GitHub Pages live demo — https://dchavan2192.github.io/Digital-Terrarium/
- [ ] Paper-quality figures and reproducible experiment configs

### Infrastructure [ONGOING]
- [x] GPU acceleration investigation — NOT worth it at current scale (see Research section)
- [x] rayon within-tick parallelism (4.3x speedup achieved)
- [ ] SoA (struct-of-arrays) refactor for hot fields (1.5-3x speedup)
- [x] Fast approximate tanh (Pade approximant, part of 4.3x speedup)
- [ ] Define "Earth-like" primordial environment model

---

## Research & Citations

### Mortality Models [RESEARCHED]

**Current problem**: Fixed `max_age = 3000` means all creatures die at exactly the same age. This is biologically unrealistic and eliminates any selection pressure on longevity.

**Solution**: Replace with **Gompertz-Makeham mortality law**:
```
death_probability_per_tick = lambda + alpha * exp(beta * age)
```
- `lambda` — extrinsic mortality (accidents, starvation): ~0.0001
- `alpha` — initial senescence rate at birth: ~0.00001
- `beta` — aging rate (how fast mortality increases): ~0.001
- Make `beta` a genomic trait so longevity can evolve

**Key evolutionary biology theories backing this**:
1. **Mutation accumulation** (Medawar, 1952): Deleterious late-life mutations accumulate because selection is weak at old ages
2. **Antagonistic pleiotropy** (Williams, 1957): Genes beneficial early in life can cause late-life deterioration — aging is a trade-off
3. **Disposable soma** (Kirkwood, 1977): Organisms allocate energy between reproduction and somatic repair — aging results from under-investment in maintenance

**Implementation plan**: Encode `beta` in genome. Optionally add "repair cost" — creatures can spend energy to slow senescence (implements disposable soma). Antagonistic pleiotropy emerges naturally from the energy budget.

**ALife precedent**: Most sims (Avida, Tierra) use replacement-based death, not aging. Polyworld uses energy depletion. Explicit Gompertz aging is rare but scientifically motivated — this would be a differentiator.

**Citations**:
- Gompertz, B. (1825) "On the Nature of the Function Expressive of the Law of Human Mortality"
- Williams, G.C. (1957) "Pleiotropy, Natural Selection, and the Evolution of Senescence" *Evolution* 11(4):398-411
- Kirkwood, T.B.L. (1977) "Evolution of ageing" *Nature* 270:301-304
- Mitteldorf, J. (2006) "Chaotic Population Dynamics and the Evolution of Ageing" *Evol Ecol Res* 8:561-574
- Chow et al. (2004) "Adaptive Radiation from Resource Competition in Digital Organisms" *Science* 305:84-86

### Primordial Earth Environment [RESEARCHED]

**Early Earth conditions (~3.8-4.2 Bya)**:
- Reducing atmosphere: CO2, N2, H2O, CO, H2 (no free O2)
- Temperature: 50-80C oceans, hydrothermal vents 60-400C
- Energy: UV radiation, lightning, hydrothermal chemistry, volcanic activity
- Chemistry: simple organics (HCN, formaldehyde, amino acids), clay minerals as catalysts

**Relevant models**:
- Eigen's quasispecies (1971): error threshold for information maintenance
- Kauffman's autocatalytic sets (1986): random polymer networks achieving catalytic closure
- Ganti's chemoton (1971): minimal living cell model

**How to map to Terrarium** (future phases):
- Multiple resource types (not just one food) — chemical diversity
- Spatial energy gradients (e.g., hydrothermal vent zones with higher food density)
- Degradation rate — organisms break down over time (ties into aging model)
- Temperature zones affecting metabolism rates

**Citations**:
- Eigen, M. (1971) "Self-organization of matter" *Naturwissenschaften* 58:465-523
- Kauffman, S.A. (1986) "Autocatalytic sets of proteins" *J Theor Biol* 119:1-24
- Szostak et al. (2001) "Synthesizing life" *Nature* 409:387-390
- Martin & Russell (2003) "On the origins of cells" *Phil Trans R Soc B* 358:59-85

### GPU Acceleration [RESEARCHED — NOT RECOMMENDED YET]

**Verdict: GPU is not worth it at current scale (1100 creatures, 139 weights/brain).**

**Why**: GPU dispatch overhead (~30-100us per dispatch) dwarfs the actual compute (~1us for 1100 brain forward passes). Break-even is ~10k-50k creatures with current brain size.

**Recommended CPU optimizations instead** (ordered by ROI):

| Optimization | Effort | Expected Speedup |
|---|---|---|
| rayon within-tick parallelism | Low (hours) | 3-6x |
| SoA data layout for hot fields | Medium (1-2 days) | 1.5-3x |
| f32 instead of f64 | Medium (half day) | 1.5-2x |
| Fast approximate tanh | Low (30 min) | 1.1-1.3x |

**GPU becomes interesting** when: creature count hits 4000+ AND brain complexity increases (NEAT with 50-200+ neurons). That's a Phase 5 concern.

**Key insight on the 800% CPU**: That's rayon's parameter sweep parallelism — all 8 cores running different simulations. The actual single-tick loop is single-threaded. Adding rayon within the tick loop would parallelize a single run across cores.

---

## Design Decisions Log

Every non-trivial design decision should be recorded here with justification.

### D1: Deterministic simulation via seeded RNG
**Decision**: Use ChaCha8Rng seeded from config for all randomness
**Why**: Reproducibility is essential for scientific claims. Same config + seed = same run.
**Source**: Standard practice in computational biology; required for peer review

### D2: Energy as universal currency
**Decision**: All actions (moving, thinking, reproducing) cost energy; food is the only source
**Why**: Creates a single optimization target for evolution. Mirrors real metabolic costs.
**Source**: Thermodynamic constraints on biological systems (Lotka, 1922)

### D3: Intelligence tax (brain_cost per neuron per tick)
**Decision**: Larger brains cost more energy to maintain
**Why**: Prevents runaway brain growth. In nature, brains consume ~20% of metabolic budget despite being ~2% of body mass. Creates evolutionary tension between intelligence and efficiency.
**Source**: Aiello & Wheeler (1995) "The Expensive-Tissue Hypothesis"

### D4: Toroidal world (wraparound boundaries)
**Decision**: World wraps in both axes, no edges
**Why**: Eliminates edge effects that would create artificial niches. Every location is topologically equivalent.
**Source**: Standard in spatial ecology models (Durrett & Levin, 1994)

### D5: Fixed-topology brain (v1)
**Decision**: Start with fixed 13->8->3 feed-forward net, evolve only weights
**Why**: Simplest evolvable controller. Topology evolution (NEAT) is Phase 5 — need to prove the platform works with simple brains first.
**Source**: Stanley & Miikkulainen (2002) — NEAT showed topology matters, but weight-only evolution is the baseline

### D6: Parameter tuning — scarce food + cheap reproduction
**Decision**: regrowth=0.0015, threshold=60, parent_cost=30, metabolism=0.2
**Why**: Creates selective pressure where mean energy (~30) is below reproduction threshold (60). Only skilled foragers reproduce. Cheap reproduction (cost=30) enables high turnover (~750 births/1000 ticks) for rapid evolution.
**Validated**: 5/5 seeds survive 100k ticks with stable ~1100 population

---

## Environment Spec (Control Config)

For any experiment, the **control** is `configs/default.toml` at the commit tagged for that experiment. Key parameters:

```
World: 1000x1000, toroidal
Food: 10k cells, regrowth=0.0015/tick, 30 energy/food
Creatures: 300 initial, 1500 cap, max_age=3000
Energy: start=60, max=200, metabolism=0.2, movement=0.08, brain=0.002
Reproduction: threshold=60, parent_cost=30, child_energy=30
Brain: 13->8->3, tanh activation
Mutation: 10% per weight, +/-0.2 step
```

**Standard test protocol**: Run 5 seeds (0-4) x 100k ticks. Report survival rate, mean final population, total births.

---

## How to Continue a Session

1. Read this file (`docs/PROGRESS.md`)
2. Read `CLAUDE.md` for safety rules and conventions
3. Check the "Active Work Items" section for what's next
4. Check "Research & Citations" for pending research
5. Pick up the next unchecked item and work

---

## Session Log

### Session 1 (2026-11-07)
- Scaffolded entire project from design doc
- Built sim-core, sim-viewer, sim-runner
- GitHub repo created, CI pipeline, README, LICENSE
- Performance optimized 50->195 ticks/s (later ~900 with smaller pop)
- Set up autonomous agent workflow (CLAUDE.md + permissions)

### Session 2 (2026-11-07, continued)
- Phase 1 parameter tuning complete
- Added cumulative total_births/total_deaths to TickStats
- Validated stability: 5/5 seeds survive 100k ticks
- Pushed tuned config to GitHub
- Started Phase 2 research: mortality models, GPU acceleration, primordial Earth
- Created this progress doc

### Session 3 (2026-11-07, continued)
- **Implemented Gompertz-Makeham probabilistic mortality**
  - Added `[aging]` config section: `extrinsic_mortality` (λ=0.0001), `initial_senescence` (α=0.00001), `aging_rate` (β=0.002)
  - Added `aging_rate` as evolvable genomic trait (mutates 5% of the time, ±20%, clamped [0.0001, 0.01])
  - Replaced fixed `age >= max_age` death check with per-tick Gompertz probability
  - `max_age` retained as absolute safety cap
  - Added aging params to sweep.rs for parameter exploration
  - New test: `gompertz_mortality_kills_old_creatures` (11 tests total, all passing)
  - Full CI: clippy clean, fmt clean, all tests pass
- Pi 5 running 500k-tick baseline overnight as control data

### Session 4 (2026-11-08)
- **Rayon within-tick parallelism**: 4.3x speedup (1000 -> 4400 ticks/s)
  - Parallelized sense-think-act loop with `par_iter()` — read-only phase, no RNG, determinism preserved
  - Sequential phases (apply actions, reproduce, die) unchanged
- **Fast approximate tanh**: Pade approximant `x(27+x²)/(27+9x²)`, max error ~0.004
- Analyzed Pi baseline data (5 seeds x 100k + 1 seed x 500k ticks)
- Pi running Gompertz comparison overnight
- **Predator-prey combat mechanics**
  - New brain output: attack (outputs 3->4)
  - [combat] config: attack_range=8, attack_cost=5, attack_damage=20, energy_transfer_ratio=0.5
  - Attacker pays cost, nearest neighbor in range takes damage, 50% energy transfer (trophic loss)
  - ~7600 attacks in first 1000 ticks — creatures using combat immediately
  - Citations: Lotka (1925), Volterra (1926), Yaeger Polyworld (1994)
- **Evolvable body size**
  - body_size gene [0.5, 3.0], default 1.0, 5% mutation rate
  - Speed scales as max_speed/body_size (bigger = slower)
  - Metabolism scales as base_metabolism * body_size^0.75 (Kleiber's law)
  - Attack damage scales with attacker size, defense scales with victim size
  - Added as 14th brain input so creatures sense their own size
  - Citation: Kleiber (1932) body size allometry
- **Speciation detection & lineage tracking**
  - Added `parent_id` and `generation` to Organism (lineage tracking)
  - Genome distance function: weighted sum of brain weight divergence + trait differences
  - Greedy threshold-based species clustering (NEAT-style compatibility distance)
  - New stats: `species_count`, `mean_generation` in TickStats and CSV output
  - Viewer inspector shows generation and parent ID for selected creatures
  - New test: `lineage_tracking_and_speciation` (12 tests total, all passing)
  - Citation: Stanley & Miikkulainen (2002) NEAT compatibility distance
- **Trophic level analysis**
  - Per-organism `energy_from_food` and `energy_from_attacks` lifetime tracking
  - Trophic level = 1 + (attack_energy / total_energy): 1.0 = herbivore, 2.0 = pure predator
  - Classification: herbivore (<25% from attacks), predator (>50% from attacks), omnivore (between)
  - New stats: `mean_trophic_level`, `herbivore_count`, `predator_count` in TickStats/CSV
  - Viewer shows per-creature trophic level and energy breakdown in inspector
  - New test: `trophic_level_tracking` (13 tests total, all passing)
  - Citation: Lindeman (1942) "The Trophic-Dynamic Aspect of Ecology"
- **README plots and analysis**
  - Python plotting script (analysis/plot_readme.py) with dark GitHub theme
  - 5 plots: dashboard (hero), population/energy, birth/death, combat, evolution
  - README rewritten with real metrics, biology citations table, embedded plots
- **Phase 4: Social Intelligence — complete**
  - Broadcast signal channel: 5th brain output (signal [-1,1]), distance-weighted averaging of nearby signals as brain input. Citation: Werner & Dyer (1991)
  - Energy sharing: 6th brain output (share), voluntary energy transfer to nearest neighbor. Config: share_range=10, share_amount=5. Citation: Hamilton (1964), Trivers (1971)
  - Kin recognition: 15th brain input — color hue similarity to nearest creature [-1,1]. Enables kin-selective behavior
  - Cooperation measurement: cooperation_rate (shares/pop), kin_share_ratio (fraction of shares to kin, hue diff < 30 degrees)
  - Brain now 15 inputs, 6 outputs (turn, speed, eat, attack, signal, share)

### Session 5 (2026-11-08, continued)
- **Variable hidden layer size (Phase 5 start)**
  - `hidden_size` added to Genome as evolvable trait, range [4, 32], default 8
  - 2% mutation rate per reproduction: ±1 neuron, weight vector resized correctly
  - New neurons get Xavier-scale random weights; removed neurons simply dropped
  - `Brain::from_genome()` constructor for per-organism topology
  - `genome_distance()` updated for variable-size comparison + topology penalty
  - Viewer inspector shows full topology (15→N→6)
  - New test: `variable_hidden_size_evolves` (14 tests total, all passing)
  - Pi 5 safe: stack-allocated forward pass unchanged (MAX_HIDDEN=32 buffer)
  - Citation: Stanley & Miikkulainen (2002) NEAT topology evolution
- **Full NEAT-style topology (connection genes)**
  - Replaced dense weight matrix with ConnectionGene list + HiddenNode list
  - brain.rs rewritten: sparse forward pass with pre-sorted connections, [f64; MAX_NODES=64] buffer
  - genome.rs rewritten: NEAT mutations (add-connection 3%, add-node/split 1%, toggle 1%)
  - Add-node splits A→B into A→C(weight=1.0)→B(old weight) — preserves behavior while complexifying
  - genome_distance: zero-allocation merge-join on sorted connection genes (O(n+m))
  - Skip connections (input→output) emerge naturally from add-connection mutations
  - Genome stores num_inputs/num_outputs internally — reproduce() no longer needs external params
  - Updated tests: `brain_topology_matches_genome`, `neat_topology_evolves` (14 tests, all passing)
  - Citations: Stanley & Miikkulainen (2002), Kassahun & Sommer (2005)
- **Information-theoretic brain analysis (Phase 5 complete)**
  - Per-organism output histograms: 6 outputs × 8 bins, exponential decay (τ≈100 ticks)
  - Shannon entropy normalized to [0,1] — measures behavioral variability over recent history
  - Population behavioral diversity: entropy of peak-bin distributions across organisms
  - New stats: mean_brain_entropy, behavioral_diversity, mean_connections
  - Viewer updated: stats panel + inspector show entropy metrics
  - Citation: Shannon (1948), Beer & Williams (2015)
- **Spatial resource gradients (biomes)**
  - BiomeZone: circular zones with regrowth_multiplier and smooth falloff from center
  - Per-cell regrowth rates (FoodGrid stores regrowth_rates Vec alongside cells)
  - Default config: 4 biomes — 2 lush oases (3×), 1 moderate center (2×), 1 barren NE (0.3×)
  - Renderer: biome-aware food coloring (bright green → brown by richness)
  - Wraparound-safe distance calculation for biome zones
  - Backward compatible via `#[serde(default)]` — existing configs work unchanged
  - Motivation: Pi 500k runs showed 0 speciation — uniform environment = no niche pressure
  - Citations: Schluter (2000), Rosenzweig (1995)

### Session 7 (2026-11-09)
- **Integration test for biome-aware food regrowth** (16 tests total, all passing)
  - `biome_aware_food_regrowth`: constructs FoodGrid with two BiomeZone configs (5× lush, 2× moderate), verifies three-tier regrowth rates (lush > moderate > barren/neutral), tests actual food accumulation over 10 regrow cycles
  - `biome_no_zones_gives_uniform_regrowth`: verifies backward compatibility — no biomes configured means all cells get the uniform base regrowth rate
  - Discovered that "barren" zones (multiplier < 1.0) have no effect in current implementation due to "highest multiplier wins" logic starting at 1.0 — documented in test comments

### Session 8 (2026-11-09)
- **Seasonal/cyclical food variation** (18 tests total, all passing)
  - Added `SeasonsConfig` struct with `amplitude` and `period` fields to config.rs
  - Added `[food.seasons]` section to default.toml (amplitude=0.3, period=10000)
  - `FoodGrid::regrow()` now takes tick and SeasonsConfig, applies sinusoidal multiplier
  - `seasonal_multiplier()` public function: `1.0 + amplitude * sin(2π * tick / period)`, clamped ≥0
  - Applied globally on top of per-cell biome rates — creates boom-bust cycles
  - Viewer UI shows "Season: X%" when seasons are active
  - Backward compatible via `#[serde(default)]` — amplitude=0 means no effect
  - Two new integration tests: `seasonal_food_variation`, `seasonal_no_effect_when_amplitude_zero`
  - Citation: Botkin (1990) "Forest Dynamics: An Ecological Model"

### Session 9 (2026-11-09)
- **Population density pressure (Phase 2.5)**
  - Added `DensityPressureConfig` with `cost_per_neighbor` field (default 0.0 for backward compat)
  - Added `[density_pressure]` section to default.toml: `cost_per_neighbor = 0.01`
  - Neighbor counting integrated into existing parallel sense phase (`for_each_nearby` closure)
  - `neighbor_count: u32` added to Action struct, crowding cost applied during metabolism
  - `mean_crowding` added to TickStats — average neighbor count across living organisms
  - Viewer stats panel displays "Mean crowding: X.X"
  - Two new integration tests (20 total, all passing):
    - `density_pressure_increases_metabolism`: verifies crowding reduces mean energy
    - `density_pressure_no_effect_when_zero`: verifies backward compatibility (cost=0 identical to omitted)
  - Zero extra spatial queries or allocations — piggybacks on existing vision neighbor loop
  - Citation: Allee (1931) "Animal Aggregations: A Study in General Sociology"

### Session 10 (2026-11-09)
- **Dynamic speciation threshold (Phase 2.5 complete)**
  - Added `SpeciationConfig` with `dynamic`, `fixed_threshold`, `sample_size`, `factor`, `min_threshold`, `max_threshold`
  - Added `[speciation]` section to default.toml: `dynamic = true`, `factor = 0.75`, bounds [0.1, 2.0]
  - Algorithm: deterministic stride-based sampling of organisms, compute all pairwise genome distances among sample, set threshold = factor × mean_distance, clamped to [min, max]
  - Replaces hard-coded `species_threshold = 0.5` with adaptive computation
  - `speciation_threshold` added to TickStats for observation and CSV output
  - Viewer shows threshold alongside species count: "Species: N (t=X.XX)"
  - Backward compatible via `#[serde(default)]`: omitting `[speciation]` gives `dynamic=false`, `fixed_threshold=0.5` (identical to previous behavior)
  - Two new integration tests (22 total, all passing):
    - `dynamic_speciation_threshold_adapts_to_diversity`: verifies threshold is computed from genome variance, within configured bounds, and differs from fixed default
    - `speciation_fixed_threshold_backward_compatible`: verifies omitting config section gives identical behavior to explicit `dynamic=false`
  - Zero allocation: stride sampling is O(sample_size²) distance computations with no heap allocation beyond the existing genome distance function
  - Citation: Stanley & Miikkulainen (2002) "Evolving Neural Networks through Augmenting Topologies"

### Session 11 — Overnight autonomous (2026-11-15)
- **OEE 200k-tick experiment completed on Pi 5** (seed 42, ~15 min at 214 ticks/s)
  - **D(t) = 67.45** — grows unboundedly, fits D(t) ≈ 11.94·ln(t) − 81.5
  - Early growth rate: 0.0006/tick, late rate: 0.0001/tick (ratio 0.22 — sublinear but never zero)
  - Novel traits remain positive throughout (final: 8/window), active traits ~99
  - Lineage diversity collapses to 1 by tick ~5k — single dominant ancestor
  - Max generation reaches 150, mean generation ~143
  - Methylation decays from 0.65 → 0.10 (population adapts away from stress)
  - Construction level oscillates ±0.1 — organisms actively modifying environment
- **Analysis plots generated**: oee_activity.png, oee_lineage.png, oee_epigenetics.png in docs/images/
- **Phase 7 checklist**: All 5 features marked COMPLETE
- **Key finding**: D(t) follows logarithmic growth — consistent with Bedau's "class 2" unbounded
  activity. The system produces genuine open-ended evolution by the Bedau & Packard (1991) criterion.

### Session 12 — Overnight autonomous (2026-11-15, continued)
- **5-seed OEE replication experiment** on Pi 5 (seeds 0-4, 200k ticks each)
  - Seeds 0-3 complete, seed 4 in progress
  - D(t) log fits: seed 0 = 12.82·ln(t), seed 1 = 21.58·ln(t), seed 2 = 14.01·ln(t), seed 3 = 18.93·ln(t)
  - All seeds show unbounded D(t) growth — replication confirms OEE
  - Mean D(t) at 200k ≈ 82.2 across seeds 0-3
- **Created multi-seed analysis script**: `analysis/plot_multi_seed.py` (95% CI comparison plots)
- **Created ablation experiment configs**: `configs/no_epigenetics.toml`, `configs/no_niche_construction.toml`
- **Created ALIFE 2027 paper outline**: `docs/ALIFE_2027_OUTLINE.md`
- **Pushed**: `0ec0f20` — analysis tools and paper outline

### Session 13 — Overnight autonomous (2026-11-15, continued)
- **Epigenetics ablation experiment running** on Pi 5 (5 seeds × 200k ticks, no_epigenetics.toml)
  - Seed 0 complete (835.2s, faster than baseline — no methylation computation)
  - Seeds 1-4 in progress
- **Created ablation analysis script**: `analysis/plot_ablation.py`
  - D(t) comparison with 95% CI between baseline and ablation conditions
  - Population stability (CV) comparison
  - Mann-Whitney U statistical tests for significance
- **Updated ALIFE 2027 outline**: D(t) numbers updated to confirmed 5-seed data (~15.5·ln(t))
- **Updated README** with OEE results section — multi-seed plots and summary statistics
- **Pushed**: `f04f7e4` (ablation script + outline), `c4bbd95` (README OEE section)

> **Caveat (added 2026-11-15, Session 14):** the Session 11–13 OEE numbers were computed BEFORE the Phase 7.5 fixes. Back then `mean_cumulative_activity` averaged over every topology ever seen (not extant ones), the topology hash was order-dependent, and fruit was silently disabled. There was also no neutral shadow model, so 'genuine OEE' is NOT yet established. Treat these numbers as preliminary and re-run them with the fixed code plus the shadow model.

### Session 15 — Autonomous council (2026-11-18)

**Council of advisors** (Ecologist, Engineer, Strategist): 3 parallel reviews identified critical issues.

**Bug fixed — R* resource competition (Tilman 1982):** `eat_at` previously consumed 1.0 unit of food
regardless of diet efficiency, so a fruit specialist depleted algae the algae specialist needed even
while gaining zero energy. Fixed: `eat_at(pos, efficiency)` now scales consumption by diet efficiency,
enabling true competitive exclusion by resource exhaustion. Specialists now have lower R* for their own
resource. This was the missing piece for stable coexistence in the diet_radiation experiment.

**New stats added to TickStats:** `mean_diet_preference`, `diet_variance` (std_dev of diet_preference —
measures bimodality / character displacement), `mean_hidden_nodes` (hidden neuron count per brain,
tracks neural evolution). These are essential for diet_radiation and predation_arms_race experiments.

**Experiment configs updated:**
- `diet_radiation.toml`: sweep 0.1 → 0.2 (better threshold resolution); R* fix note added
- `drought_coexistence.toml`: removed 0.005 spawn_rate (too high — collapses ecosystem); capped at 0.001
- `predation_arms_race.toml`: removed ratio=0.0 (predation dies instantly); extend to 1M ticks for significance

**New experiment configs created:**
- `configs/experiments/cooperation_phase_boundary.toml` — sweep transfer_efficiency 0.5→1.0 in 11 steps.
  Tests Hamilton's rB > C phase transition: at what efficiency does cooperation emerge? Highest scientific
  impact: would be first quantitative match to Hamilton's equation in NEAT agents under energy conservation.
  Citation: Hamilton (1964), Nowak (2006).
- `configs/experiments/cambrian_transition.toml` — disable predation 200k ticks, then enable it; measure
  neural complexity step-change. Tests the Cambrian explosion hypothesis in NEAT agents. Citation:
  Aiello & Wheeler (1995), Dawkins & Krebs (1979).

**Comment updated:** config.rs FruitConfig docstring updated to reflect diet_slope=1.0 (50% generalist
efficiency, not 65%) and R* consumption semantics.
