# Session handoff — 2026-11-15

## Resume update — 2026-11-17 UTC (supersedes active-Pi notes)

Baseline batch `validation_20261116T171843Z_28985` COMPLETED: ten of ten survived
200k ticks with zero cap blocking; late mean 893.629, minimum 620. All four fixed
criteria pass. Final report/raw release in RESULTS.md. Local analysis exactly
matches pinned Pi finalizer; 30 artifact hashes audited. Guard finished after 149
cooling cycles; sampled max 75.7°C, no active throttle flags. Pi now idle/cool,
source 0da954f pinned outputs preserved. Do not relaunch/resume a completed batch.

Next development is visible-cue competence control, now implemented default false
on the two movement CLI commands and targeted/full tested. Same arena, 898 donors,
hold phases/horizon/reward; only movement cue availability differs. Controls all
reward 1 when visible and 1/.5/.5 when hidden. All 153 Rust, 10 JS and 10 thermal tests
pass, Clippy/format pass; no overlapping Cargo runs. The actual comparison is complete on clean `bda6f35`: delayed 7 correct/7 wrong/
1,782 timeouts versus visible 0 correct/31 wrong/1,765 timeouts. All 898 pairs/hashes
and unchanged delayed traces audited. See VISIBLE_CUE_CONTROL_2026-11-17.md. Keep inference
limited to task competence; unfamiliar donor brains may time out even with cues.
Competence is still unestablished even with cues; validate a competence/training
protocol or ecological endpoint before any memory-benefit comparison. No new
evolutionary campaign started. User authorization to commit/push persists.

## Stop/resume record — 2026-11-16, 22:43 UTC (read first)

User requested a stopping point before usage limit. All new feature work stops here.
Movement arena source `0bc1be2` is saved/pushed. Final report, JSON/plot, audit and
raw dataset are linked from RESULTS.md and PROGRESS.md. Controls reward 1/0.5/0.5;
898 evolved feed-forward donors: 7 left for both cues, 891 timeouts. All paths
match between intact/reset and opposite cues; timeouts are not a memory-deficit
finding. Full serialized Rust suite: 151 pass; JS 10, thermal 10, Clippy/format pass.

Pi is still running baseline `validation_20261116T171843Z_28985`, pinned source
`0da954f`: six completed, seed 106 active. At last check 73.8°C, no active
hardware throttling, thermal guard working, finalizer waiting. Leave its pinned
executable and outputs intact. On resume inspect batch status, guard and
finalization; completed results still need mirroring/registration. No extra
campaign queued. Next coding priority is the visible-cue motor/competence
control described in the new arena report, then controlled recurrent donors.
Do not resume broad feature or experiment work without first reading this update.

## Development update — 2026-11-16, exported-genome assay

Source `aa6e844` adds `evaluate-delayed-cue`, public safe genome validation and
thermal policy tests in CI. Full workspace: 142 tests, JS: 10 and thermal: 10 pass.
The actual negative control evaluated all 898 seed-100 genomes (tick 200,001;
separate one-tick export bridge using pinned 0da954f). All lacked recurrence,
scored 1/2 and had identical intact/cleared outputs. Reports/protocol/checksums
are in RESULTS.md. No evolved-memory or fitness claim; next is controlled
recurrent-genotype evaluation and a memory-requiring task. The Pi recheck is
still active: four runs complete and seed 104 running at 21:27 UTC, guarded near
72°C with no active throttling. Keep its pinned executable intact. User dropped
the permissions-configuration task; continue under normal host permissions.

## Latest continuation — 2026-11-16 (supersedes earlier pending items)

- Calibration batch `validation_20261116T021704Z_23586` COMPLETED: 30/30 survived, no cap blocking; rate 0.008 selected by the fixed rule. Report, uncertainty, exploratory drift and raw archive links are in RESULTS.md and PROGRESS.md. Owner approved default food 0.008/cap2,000; demo keeps food0.002/cap500.
- Current-engine baseline ACTIVE on Pi: `output/phase9a/validation_20261116T171843Z_28985`, source 0da954f, ten seeds100–109 ×200k ticks, one worker/thread. Never replace its pinned executable. Its own thermal guard pauses at75°C/resumes65°C; check `thermal_guard/status.json` and guard.log. Its pinned finalizer waits to write `finalization/summary.json`; never summarize partial data as success.
- Source d35dd2c added batch-specific PID-safe thermal control and the synthetic delayed-cue instrument. Pi instrument output is `output/delayed_cue_20261116T175013Z`, with committed JSON/report. Manufactured controls passed; no evolved-cognition claim. Full Rust129 tests and thermal: 10 tests pass. Next research task: exported-genotype behavioural evaluation before a memory ecology comparison; no other evolutionary campaign queued.
- GitHub issue1 tracks results/recheck; raw dataset prerelease `calibration-food-2026-11-16`. Owner explicitly authorizes multiple agents, routine development, commits and pushes; preserve old runs and avoid destructive cleanup.
- Owner also requests project-only personal filesystem access plus internet/GitHub without prompts. `docs/CODEX_PROJECT_PERMISSIONS.toml` is a prepared template, NOT installed/active. Current managed host policy cannot be changed from inside the chat. Do not claim this boundary is enforced. No global settings were read/changed for that task.

## Stop here

User requested a stopping point and conservation of remaining session budget. No new features or experiment campaigns should start in this session. Resume by reading PROGRESS.md, this file, then the scientific council and results register. Do not repeat the stale-checkout audit.

## What changed

- Local baseline was 14 commits behind upstream; preserved local source/artifacts, merged three document conflicts, pushed a clean baseline. Upstream already fixed energy-creating sharing, negative combat damage, fruit and other world mechanics.
- Added a flat, serializable organism-reserve energy ledger with all harvest/cost/transfer/reproduction/death/overflow components and a debug balance assertion. Ten adversarial fixtures; no ecological rule changes in ledger work.
- Added measured Energy demo tab, clarified cluster/evolution tour language, focus trapping, better card placement and flexible KPI widths. Added real-WASM smoke and JS presentation checks to CI/deploy. Native dependency omission had broken Pages; ALSA install fixed it.
- Public config validation now covers generated jobs, finite values, capacities, resource/cost/probability constraints. Unique session directories, build/source/config/compiler/status manifests and final-tick logging avoid overwrite and false completion.
- Checkpoints preserve RNG, expressed live brains (including methylation refresh lag), activity archive, species history, all world grids, IDs and lineage. Checksums, build/target guard, integrity validation and atomic adjacent-file replacement. CLI --resume, --checkpoint-every, --export-genomes-every use absolute --ticks target. Same-build only; preserve the executable. Sweeps do NOT checkpoint yet.
- Ten forced tests cover fruit, temperature, pheromone, day/night, elevation, niche construction, epigenetics, sexual reproduction and activity tracking, including scoped disabled-mode invariance.
- Research council challenged design using primary papers and corrected claims. See research/SCIENCE_COUNCIL_2026-11-15.md, PREREG.md (draft/unregistered/unlaunched), RESULTS.md and corrected ALIFE_2027_OUTLINE.md. No established scientific current-model results.
- Portfolio repository /Users/dineshchavan/projects/portfolio-dhruv: baseline preserved and pushed; local agent settings left on disk but untracked. ds.html card now describes actual capabilities and research limits, links demo/source/progress. Latest portfolio commit 1f4e73d.

## Pi inventory and completed pilot

Completion update: all six runs completed successfully; no simulation remained at inspection. Read [the calibration report](results/PHASE9A_PILOT_2026-11-15.md) for per-seed measurements, source-dirty caveat and next steps. Earlier in-flight observations below are historical.

SSH: pi@dchavan. Main repository: /home/pi/Digital-Terrarium. Existing /home/pi/Digital-Terrarium-wip is a non-Git copy containing earlier rule-comparison outputs; preserve it. Main repo has untracked nohup.out and run_overnight.sh; preserve them too. Output folders are organized by experiment; never overwrite old runs.

This session launched only a short calibration pilot at source commit 2f49287:

- food.regrowth_rate = 0.004, 0.006, 0.008; seeds 0,1; 20,000 ticks each; hard_cap=2000; two workers.
- Launch root: /home/pi/Digital-Terrarium/output/phase9a/launch_20261115T230545Z_22098
- Session: session_1794783945293292320_22229_0
- PID observed: 22229. Detached via nohup; leave running. Last observed running, two runs completed; temperature 76.3 C, throttled=0xe0000 (historical flags, no current low-bit throttle).
- This pilot predates checkpoint support and is not resumable. Its manifests may flag dirty source because preserved untracked operational files exist; inspect exact manifest before registering any observation.

Status commands:

```bash
ssh pi@dchavan 'cat ~/Digital-Terrarium/output/phase9a/launch_20261115T230545Z_22098/status; tail -c 1600 ~/Digital-Terrarium/output/phase9a/launch_20261115T230545Z_22098/launcher.log; vcgencmd measure_temp; vcgencmd get_throttled'
```

Once completed, read sweep_summary.csv and all manifests, report late-window population/energy, minimum population, extinction and cap-blocked births. Do not select/publish a default from two short pilot seeds alone. Use independent validation seeds for the planned 200k-tick/10-seed gate. All simulations >3 minutes run on Pi. Avoid multi-day runs until sweep checkpoints or a pinned-binary single-run batch exists. Do not rebuild/pull over active experimental work without preserving its exact source/binary.

## Checks and verification limits

Prior to final combined checks: all energy and mechanism fixtures passed; six checkpoint tests passed including 2,000 uninterrupted vs 1,000+restore+1,000 plus future RNG; six runner tests passed including end-to-end checkpoint byte equality. Real WASM smoke at tick25 had population278 and reserve residual -5.60e-12. Three JS presentation tests passed. Final full-workspace format, all-target Clippy, and all 94 Rust tests (including the doctest) passed. Three JS tests, actual WASM smoke, shell syntax, analysis session-discovery/incomplete/duplicate guards, and whitespace checks passed.

Energy-tab source 2f49287 CI and Pages deployment succeeded. Follow later deployment runs after final commit. Chrome browser host remained unavailable and native Computer Use permissions were absent; visual desktop/mobile QA is pending, despite HTTP200 local preview and actual WASM verification. Do not substitute a mock for real-data validation.

## Next work, in order

1. Confirm final CI/Pages and clean Git states; inspect completed Pi pilot and record it as calibration with provenance.
2. Finish Stage0 cross-platform conformance measurement; current state_json API exposes canonical state without build metadata, but no equality claim has been tested across Mac/Pi/WASM.
3. Independent environmental RNG or replayable schedules, checkpointable Pi batches with pinned executable, holdout baseline validation.
4. Fix historical node identity/crossover semantics before recurrence. Fitter-parent disjoint-edge policy alone is not a bug.
5. Freeze a narrow hypothesis and controls from PREREG after power pilots. Informative nulls count; no requirement for positive findings.
6. Behavioral evaluation before memory/plasticity. Do not expand mechanisms until instrument gates are met.

## Working agreements

User authorizes commits/pushes per completed task, portfolio/demo updates, primary-paper research, peer council debate and parallel agents. No destructive cleanup or force pushes. Long simulations on Pi. Preserve ignored local artifacts. Existing GitHub Pages is the hosting path; no new Sites hosting was registered. Skill guidance used: Sites local implementation/preview workflow. Research and testable novelty take priority over feature count; do not claim open-ended evolution or consciousness from current evidence.

## Update — 2026-11-16 (session after this handoff)

Read the two newest sections of PROGRESS.md ("Cross-platform determinism" and "Held-out validation batch") first.

- Item 2 done: platforms diverged after one tick; fixed with `sim_core::math`; macOS, Pi and WASM now match through tick 20,000; CI compares x86-64 Linux with WASM. Registered as CHK-2026-11-15-PORTABILITY.
- Item 3 partly done: checkpointed, pinned-executable Pi batches exist and the held-out validation is running (`validation_20261116T021704Z_23586`). Independent environmental RNG was added on 2026-11-16 (`env_rng`, `food.droughts.schedule_seed`).
- A validation batch is ACTIVE on the Pi. Do not pull, rebuild or launch anything else there until `status` reads `completed`. To continue after an interruption: `RESUME=output/phase9a/validation_20261116T021704Z_23586 bash analysis/launch_pi_validation.sh`.
- Local `main` is ahead of `origin/main`; the owner must push.
- Item 4 done on 2026-11-16: population-wide node identity (`genome::Innovations`), no lifetime neuron cap. Recurrence can now be built on it.
- Items 5–6 unchanged.

## Update — 2026-11-16, end of session (read this first)

The session stopped because the usage limit was near. Everything below is committed; the last commit may still need pushing by the owner (`git push origin main`; the agent's own push is blocked).

**Pi, at last check:** `2026-11-16T14:53Z status=running done=25 failed=0 procs=2`. Batch `output/phase9a/validation_20261116T021704Z_23586`. Do not pull, rebuild or launch anything else on the Pi until its `status` file reads `completed`. It has been at its temperature limit (about 83–85 °C) for most of the run.

**When the batch finishes, in this order:**
1. Mirror the logs (CSV, manifests, settings; not the checkpoints) into `output/pi-validation/`, as the rsync command in PROGRESS history shows.
2. `python3 analysis/summarize_validation.py output/pi-validation/validation_20261116T021704Z_23586 docs/results/phase9a_validation_2026-11-16.json`. It refuses an unfinished batch and applies the criteria fixed in `docs/results/PHASE9A_VALIDATION_PLAN.md`.
3. Write `docs/results/PHASE9A_VALIDATION_2026-11-16.md`, register the outcome in `docs/RESULTS.md`, and ask the owner before changing `food.regrowth_rate` in `configs/default.toml`.
4. Note in the report that the batch ran on commit `2b1c837`, before droughts got their own random stream and before node identity changed. That is acceptable for a calibration of average behaviour.

**Done this session, all committed:**
- Cross-platform determinism measured and fixed (`sim_core::math`); CI compares Linux with WASM.
- Checkpointed Pi batches with a pinned executable; validation plan and summariser.
- Droughts on their own random stream (`env_rng`, `food.droughts.schedule_seed`).
- Population-wide node identity (`genome::Innovations`); no lifetime neuron cap. Checkpoint version 3.
- Demo: KPI unit text fixed, species announcements at 5% of the population, expandable trend charts on the headline tiles.

**Memory (recurrent connections) was implemented later on 2026-11-16, off by default.** Outcome and open points are at the end of `docs/research/MEMORY_DESIGN.md`. Do not switch it on by default; the next step is the behavioural test. Reference states for the "off means identical" check are in `output/conformance/ref-pre-memory*/` (local, ignored by git; the design says how to regenerate them).

**Open after that:** behavioural test battery before switching memory on (roadmap 2.4); the neutral shadow model; freezing the study plan in `docs/PREREG.md`; inspector and Follow-camera visual check; better Pi cooling before any longer run.
