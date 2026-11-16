# Current-engine baseline recheck — 2026-11-16

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
