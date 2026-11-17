# Visible acquisition pilot — operational launch record

Prepared before examining any actual pilot outcomes. Scientific settings and endpoints remain in [the fixed plan](VISIBLE_ACQUISITION_PILOT_PLAN_2026-11-17.md).

- Build in isolated clean Pi worktree `output/builds/visible_acquisition_5504a20`, checked-out exact source `5504a20746bfe0dfcff54b95546324c02db1058d`. Original root checkout/untracked files and all prior experiment binaries remain intact.
- Verify the three frozen source/input hashes from the plan, then copy the release executable and inputs into a new unique batch. Build uses one Cargo worker; observed compile duration 52.54 seconds and ending temperature 59.3°C. This compile duration is not a pilot runtime estimate.
- One serial runner, `RAYON_NUM_THREADS=1`; exact population, seed list, generations, fitness and selection unchanged. No local pilot or prior look at these seeds. All three searches attempted under the one command; retain all technical failures.
- Active-command wall-clock watchdog **1,800 seconds**, including guard startup and cooling pauses. Guard finalization/termination is a separate bounded cleanup after the active command has stopped; record that time in the supervisor elapsed duration. Deadline or interruption is a technical incomplete/censored run, not biological failure. No added generations or replacement seeds. Preserve all partial files and report any technical rerun separately.
- Supervisor owns the guard and direct runner subprocesses. Before starting, require a fresh cool monitoring status matching this guard PID and pinned executable. Established thresholds remain pause 75°C, resume 65°C, poll five seconds; stale/missing guard heartbeat or guard exit must leave the owned runner safely paused and mark failure. Record sensor readings and throttling flags during execution. Never continue a hot runner merely to terminate it: timeout may escalate to killing the owned stopped process.
- Successful command completion is established by its final summary and all three per-seed summaries, not the immutable `manifest.json` launch status (`started`). Final thermal guard must finish before hashing the complete dataset.

No study duration or outcome is promised by this record. Exact batch path, operational-script revision, executable/input hashes, observed runtime, guard history and outcomes are recorded with the completed dataset. Read-only analysis checks every generation, elite/parent/ID rule, fixed topology, founder/final scores and final visible replay; it does not add new trials or a held-out evaluation.

Prepared batch (not yet started at this operational commit):
`/home/pi/Digital-Terrarium/output/acquisition/pilot_20261117T163705Z`.
The supervisor's seven harmless process-fixture tests and the ten existing
thermal policy tests pass locally. Tests include stale/dead guard rejection,
startup identity checks, interruption pausing, fresh-batch refusal, and watchdog
termination of a stopped runner without resuming it; an unrelated sentinel
process remains untouched. CI now runs these supervisor tests as well.
