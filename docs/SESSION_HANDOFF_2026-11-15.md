# Session handoff — 2026-11-15

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
