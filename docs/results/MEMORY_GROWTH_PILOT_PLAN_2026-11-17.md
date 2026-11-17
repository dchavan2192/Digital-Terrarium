# Exploratory overnight recurrence-growth pilot — 2026-11-17

Frozen before launch. Motivation: owner observed rising average connections in the optional memory demo. This is an engineering/ecology diagnostic separate from the council's competence-first memory-benefit program. It cannot establish adaptive memory or distinguish selection from mutation bias.

## Fixed design

Eight serial runs: seeds 3000–3003, paired recurrence-add probability 0 and 0.3. Order: (3000,0), (3000,0.3), (3001,0.3), (3001,0), (3002,0), (3002,0.3), (3003,0.3), (3003,0). Same seed labels provide paired environmental initialization, not identical later random histories: enabling recurrence draws changes subsequent streams. All seeds/runs retained, including extinction and technical failures. No extensions based on outcomes.

Use the completed baseline batch's pinned native executable and full base config from `output/phase9a/validation_20261116T171843Z_28985`, source `0da954f`. Preserve its source-dirty/build caveat; archive hashes of actual executable/config and baseline source metadata. This intentionally reuses the validated engine rather than building the new acquisition runner. It supports recurrence and all required overrides.

Each run: target 200,000 ticks, log every 500, checkpoint every 50,000, final genome export. Apply all current demo overrides explicitly: food regrowth 0.002, cap 500, season period 3,600, drought spawn 0.0006/duration 1,500, day/night period 600, and assigned recurrence probability. Keep all other parameters at pinned base values. Founders begin feed-forward. This is a bounded job queue intended for unattended overnight operation; elapsed time is not promised and may extend beyond morning.

## Interpretation and planned summary

The 0.3 demo mutation chance is much larger than the default 0.02 per-birth pruning chance. Their effects are not symmetric: recurrent additions can fail, pruning samples any gene, expression/toggling, topology limits, reproduction and selection alter outcomes. A rising count is therefore compatible with mutation pressure; it is not evidence that useful memory has evolved.

Plot all trajectories: population, mean enabled total connections, mean recurrent connections, recurrent fraction, and brain activity, plus recorded births/deaths/cap blocking. Report per-run final values and means across saved ticks 150,000–200,000 only for completed runs, with extinction/technical status alongside; do not silently drop failed/extinct runs or relabel late windows as equilibrium. Pair run-level summaries by seed; n=4 is exploratory, not a powered confirmatory sample. If total minus recurrent counts is used, verify both columns have the same enabled-edge denominator first. Examine final genomes only as exploratory follow-up, with no champion selection.

No memory assay, task-learning claim, intelligence metric or causal cost-benefit estimate is an endpoint. No resource or mutation tuning after launch. A demographic/random-parent null would be needed to distinguish selection from drift for a complexity claim. Visible-task acquisition remains the next development gate.

## Operations and recovery

One native worker, `RAYON_NUM_THREADS=1`; no concurrent simulation. Existing `pi_thermal_guard.py` monitors only this batch's copied executable, pauses at 75°C, resumes at 65°C, polls every five seconds, and pauses if the sensor is invalid. Worker requires a monitoring guard before launch and leaves its runner paused if the guard dies. No throttling history is cleared. Initial Pi check: idle, 52.1°C; `0xe0000` contains historical flags, no low-bit active flags.

Batch contains pinned binary/config, script/guard, hashes, job list, runner logs, checkpoints and manifests. Never replace its executable or delete outputs. `status`, `exit_code`, `completed_jobs.txt`, `thermal_guard/status.json` and `launcher.log` are the first checks. A failed job stops the queue; do not blindly rerun the script (it would repeat completed jobs). Resume the saved checkpoint in a new session using the same pinned executable, embedded config and target tick after checking/restarting the guard; preserve the original session as partial. Record recovery explicitly before continuing remaining jobs.
