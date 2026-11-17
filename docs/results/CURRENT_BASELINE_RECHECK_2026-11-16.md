# Current-engine baseline recheck — 2026-11-16

Completed: see [the final report](CURRENT_BASELINE_RECHECK_2026-11-17.md); all four criteria passed.

Plan fixed before launch. Engineering revalidation, not a new held-out selection
study or a feature-effect experiment.

The owner approved `food.regrowth_rate=0.008` and `creature.hard_cap=2000` after
the [older-build validation](PHASE9A_VALIDATION_2026-11-16.md). Both settings are
now in `configs/default.toml`. Recurrent memory remains off. The demo overlay
keeps its separate regrowth 0.002 and cap 500 for watchability. The current engine
includes independent drought randomness and corrected node identity, so the
older trajectories cannot stand in for a current-engine check.

- One candidate: 0.008; seeds 100–109, reusing the validation seeds to check the
  baseline after engine changes. These are not fresh held-out seeds.
- 200,000 ticks, log interval 500, checkpoints every 50,000 ticks, pinned binary
  and base configuration. Same endpoints and four acceptance criteria as the
  [original plan](PHASE9A_VALIDATION_PLAN.md). No alternative rate is selected
  from this batch. Failure reopens calibration.
- One job at a time, one Rayon thread, on the Pi to reduce thermal load after
  the previous two-worker batch. Record thermal history; no multi-day campaign.
- Separate batch directory; preserve all previous outputs and executables.
- Analyze with `analysis/summarize_validation.py` after completion. Do not
  interpret partial results as a pass. Report completion, survival, population,
  cap blocking and energy residuals. This does not test whether memory helps.

Launch command (after publishing the source and checking that the Pi is idle):

```bash
PI=pi@100.127.61.93 JOBS=1 THREADS_PER_JOB=1 VALUES=0.008 \
  bash analysis/launch_pi_validation.sh
```

The launch revision, batch path and observed status will be recorded in
`docs/PROGRESS.md` once launch is verified. Runtime is expected to exceed this
interactive session; completion must be checked before reporting results.

## Verified launch and thermal control

Launched at 17:18:43 UTC on 2026-11-16, pinned source
`0da954fa2911e5a0f075375b41196f8dafa6e4c8`, batch
`output/phase9a/validation_20261116T171843Z_28985`, ten jobs, one worker/thread.
The previous batch remains intact. The browser demo deployment and CI for the
published source passed.

A separate batch-specific thermal guard was attached at 17:40 UTC. It checks
the temperature every five seconds, pauses matching simulation processes at
75°C and resumes at 65°C. A missing/invalid sensor pauses work. Linux pidfds,
exact executable path and process birth identity protect unrelated processes.
Thermal pauses change wall time, not simulated time; the trajectory and sample
schedule are unchanged. The guard's transition log and status JSON live in
`thermal_guard/` within this batch. This is operational temperature control,
not a guarantee about hardware faults or other workloads.

Ten unit tests cover control policy and identity checks. A live test on
separate disposable sleep processes at
`output/guard_smoke_20261116T174204Z` verified pause/resume, ignoring unrelated
processes, re-pausing after an external resume while hot, hot-shutdown safety,
restart recovery and normal completion. The active simulation was untouched by
that test. Guard SHA-256:
`e25611f799fac2537df86738e99cd29ad801a87bf4f4408df0c6d20e56a6176c`.

Restart the guard from its pinned batch copy if necessary, keeping its default
output path so it can recover pauses it owns. A process stopped while hot is
intentionally left stopped until monitoring resumes and cooling is verified.
The finalization helper waits for completion and writes a strict summary in
`finalization/summary.json`; it never treats partial results as success or
starts another campaign.
