# Baseline long-horizon recurrence-pressure plan — 2026-11-17

Frozen prospectively before this campaign starts. This is a structural and demographic diagnostic, separate from behavioral memory validation. The earlier eight-world demo-setting pilot used food regrowth 0.002 and cap 500 and encountered substantial birth blocking. This follow-up uses the current baseline (food 0.008, cap 2000) and a longer horizon. Prior baseline observations motivate lower cap pressure; they do not guarantee these new worlds will remain below the cap.

## Fixed intervention and unit

Eight fresh worlds, four paired seed labels 4000–4003, with recurrence-add mutation probability 0 versus 0.3. Each world starts from the same pinned baseline configuration, except its assigned recurrence probability, and runs to tick 500,000. Founders start feed-forward. No selection rule, neural cost, pruning rate, ecological fitness, food, season, drought, day/night or population-cap changes are introduced by this campaign. In particular, baseline pruning remains 0.02 and connection cost 0.001.

The fixed serial order balances which arm runs first within pairs:

| Job | Seed | Recurrence-add probability |
|---|---:|---:|
| 0 | 4000 | 0 |
| 1 | 4000 | 0.3 |
| 2 | 4001 | 0.3 |
| 3 | 4001 | 0 |
| 4 | 4002 | 0 |
| 5 | 4002 | 0.3 |
| 6 | 4003 | 0.3 |
| 7 | 4003 | 0 |

A world, not a saved tick or organism, is the observational unit. Pairing fixes seed labels and environmental initialization. The engine's independent environment RNG preserves its scheduled environmental random process; evolutionary/organism random histories can diverge when recurrence changes draws and descendants. Pairing does not create identical counterfactual organisms. No new seeds, horizon extensions, arm reordering or parameter tuning based on observed outcomes.

## Frozen provenance

Native engine: clean source `b40ae9c15ef5f90f17e7e2bbbc286c1c19c0b88c`; Pi executable SHA-256 `05e6fe23121818fb897332a53f314da11e2ea98fed6f61cfe52d607009833e63`. Use the exact bytes of `configs/default.toml` at that revision. `analysis/run_long_recurrence.py` verifies the whole base file against Git, not just food/cap values, and copies the base, executable, supervisor and guard into each job. The only per-job config override is `mutation.add_recurrent_probability`; seed and output cadence are explicit command arguments.

Before ticks, retain `campaign.json` with the exact eight jobs, source revision, binary/base/worker and per-job input SHA-256 values. The copied worker and all pinned inputs must pass verification before execution. Retain native session manifests/embedded config, logs, guard/supervisor state and source metadata. Later repository edits must not change a prepared campaign. The campaign worker itself may be newer than the pinned simulation engine; both identities are recorded separately. No legacy demo Bash worker is used.

## Prospective measurements and reporting

Log every 1,000 ticks: a completed nonextinct world should contain exactly 500 ordered snapshots, ticks 1,000 through 500,000. Preserve all trajectories and final recorded endpoints. The primary structural descriptions are late-window mean enabled recurrent connections per living organism and fraction of living organisms with at least one enabled recurrent edge. Primary demographic description is late-window mean population. The fixed late window is ticks 375,000–500,000 inclusive (126 scheduled observations); early descriptive window is 50,000–100,000 inclusive (51). Report per-world early, late, late-minus-early and final values, then paired on-minus-off contrasts for the four seed labels and their arithmetic mean and range. Provide all four pairs; no tick-level significance test or confirmatory claim from n=4.

Also report enabled total connections, total minus recurrent connections, raw mean brain activity, population minimum/maximum and variability, births, deaths and cumulative cap-blocked births. Give late population standard deviation, minimum and maximum, and the fraction of saved late snapshots at or above 90% of cap (1,800). Report whether any cap-blocked births occurred across the whole run and within the late window, using differences of cumulative counters at the appropriate boundaries. Snapshot proximity can miss inter-snapshot cap events; cap-block counters supply a separate measure. Do not call a cap-binding world uncapped or retune/exclude it.

In the pinned implementation, total and recurrent connection means both count enabled edges over the same post-update living population; their subtraction is therefore a valid descriptive feed-forward-edge mean. Recurrence fraction is a structural presence measure. Raw brain activity is summed absolute weighted source activation averaged over pre-action thinking agents, with different timing from post-update population; it is neither stored memory content nor energy use. If energy is summarized, label the separately recorded brain-activity energy ledger explicitly and do not substitute raw activity for metabolic cost.

For complete worlds, cross-check final exported living-genome counts and expressed edge counts against final population/connection statistics. Retain every exported organism; no champion selection. Plots show all eight worlds and both arms, not only means. A rise over these windows is a finite-horizon trajectory, not evidence of stationarity, equilibrium or unbounded growth. Add-edge thresholds, expression, splitting, pruning and reproduction constrain topology; the nominal 512 add-edge threshold is not a universal maximum genome size.

Extinction is a biological outcome, not a failed technical replicate. Report its tick, final population zero and observed trajectory; do not fabricate missing post-extinction neural means or treat zero-population structural metrics as living-organism means. For an extinct or technically incomplete world, mark the fixed late-window summary unavailable unless all required observations exist. Preserve partial early/final observations and explicit completion status. Report complete-pair contrasts only where defined, alongside all four planned pairs and missingness reasons, rather than silently dropping worlds. No outcome-dependent restart; any later recovery is separately documented and cannot erase the original partial result.

## Bounded execution and resource policy

One copied native runner at a time, `RAYON_NUM_THREADS=1`, with no concurrent simulation. Each job has a fresh owned guard and hardened supervisor. Thermal hysteresis pauses at 75°C, resumes at 65°C, polls every five seconds, and fails closed on an invalid sensor or lost guard protection. Guard signals require verified matching executable identity; PID-only signaling is insufficient. Retain temperature/throttling samples and transitions. Sampling cannot rule out peaks between observations; historical throttle bits are not current active throttling.

Each job's watchdog is six hours (21,600 seconds), including guard startup and cooling, plus normal bounded cleanup recorded by the supervisor/worker (20 seconds per job). Eight jobs therefore have 48 hours of declared active watchdog allowance plus cleanup; this is not a promise of completion overnight or a hard guarantee against an unresponsive supervisor/kernel. A rough extrapolation from earlier measured runs is about 18 hours and is not a runtime guarantee. A timeout stops the queue and preserves the partial job; remaining jobs are marked unstarted. If a supervisor does not respond to termination, the worker records failure and deliberately preserves its guarded descendants for inspection rather than orphaning their protection. Its two exceptional cleanup waits can total40 seconds, and external inspection is required before any recovery. Do not claim a hard total wall ceiling in that exceptional state.

Before preparing and before starting each job require at least 20 GiB free disk and 3 GiB `MemAvailable`. The current preflight observed approximately 208 GiB disk free and 7.6 GiB available RAM. During execution the supervisor samples runner RSS every 0.5 seconds and terminates the owned runner if RSS exceeds 2 GiB; this is a sampled process limit, not a hard kernel allocation ceiling. Do not claim continuous free-disk or system-RAM monitoring: these two resource checks occur at job boundaries. A failed resource check, technical error, checkpoint error, thermal-supervision failure or unclean supervisor/guard completion stops the queue. Biological extinction with a clean exit may continue to the next fixed job.

Checkpoint every 50,000 ticks (10 per completed world, 80 total); export genomes at 500,000. Preserve the native 256 MiB per-checkpoint limit. Maximum successful checkpoint payload is 20 GiB across the campaign, plus atomic-write space, exports, binaries and logs. This is not a total archive-size cap. Serialization may allocate before a checkpoint-size rejection, which motivates the RSS watchdog. Previously measured final checkpoints were 43.9–50.08 MiB for ten baseline worlds and 12.32–13.73/25.69–28.14 MiB for demo recurrence-off/on worlds; these earlier sizes do not guarantee 500,000-tick sizes. No cap increase, tracker disabling, pruning of artifacts or checkpoint deletion to rescue a failed run.

The worker is single-attempt: exclusive campaign lock, prepared-status gate, pinned-input validation and no overwrite/resume of an attempted campaign. First inspect campaign status, `worker.json`, each `supervisor.json`, guard status/logs, native manifest and launcher logs. Preserve all completed/partial sessions and hashes. Any operator intervention, process cleanup or future recovery must be recorded, and a new attempt must not overwrite the frozen campaign.

## Interpretation limits

This intervention estimates finite-horizon structural and demographic contrasts under two mutation settings, not the benefit of memory. Recurrence-add 0.3 versus pruning 0.02 is asymmetric mutation pressure; recurrence can accumulate without adaptive retention, and outcomes also reflect expression, turnover, demographic feedback and drift. Lower population would not isolate a neural-cost mechanism or establish harm. Neither recurrence presence nor raw activity demonstrates memory use, learning, intelligence, cooperation, open-ended evolution or UAL. Distinguishing selection from drift/mutation bias requires a separately designed null; behavioral memory benefit requires competent policies and a controlled retention assay. Earlier demo and baseline runs differ in settings, horizon and possibly engine revision, so cross-study comparisons are contextual rather than clean single-factor estimates.
