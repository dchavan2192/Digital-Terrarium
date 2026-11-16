# Phase 9a held-out validation plan

Status: **plan, written before any validation data exists** (2026-11-15). This is an engineering calibration of the food-regrowth default, not a scientific result. It follows the pilot [CAL-2026-11-15-FOOD](PHASE9A_PILOT_2026-11-15.md) and roadmap item 0.1.

## Question

Which candidate `food.regrowth_rate` gives a population that is limited by food (not by the cap), survives long runs, and sits in the roadmap's working range of 800–1,300 creatures?

## Design

| Setting | Value |
|---|---|
| Candidate rates | 0.008, 0.010, 0.012 |
| Seeds | 100–109 (held out: never used in any earlier run; the pilot used 0 and 1) |
| Duration | 200,000 ticks per run |
| Base config | `configs/default.toml` at the launched commit, copied into the batch as `base.toml` |
| Overrides | `creature.hard_cap = 2000`, `food.regrowth_rate` = candidate |
| Logging | every 500 ticks |
| Checkpoints | every 50,000 ticks and at the end |
| Execution | Raspberry Pi 5, 2 concurrent single runs, 1 Rayon thread each, pinned executable |
| Runs | 30 (3 rates × 10 seeds) |

The candidates come from the pilot, where 0.004, 0.006 and 0.008 gave late populations of about 568, 738 and 915 over two seeds. 0.010 and 0.012 were not piloted; they are included so the batch also measures how population scales with food above 0.008.

Launch with `bash analysis/launch_pi_validation.sh`; continue an interrupted batch with `RESUME=<batch> bash analysis/launch_pi_validation.sh`. Each job resumes from its latest checkpoint using the batch's own executable.

## Measurements, fixed in advance

For each run, from its stats CSV (sessions of a resumed job joined on tick; repeated ticks are identical):

- **Survived:** the run reached tick 200,000 with a living population.
- **Late mean population:** mean of the 200 snapshots at ticks 100,500–200,000.
- **Minimum after burn-in:** smallest logged population at ticks 20,000–200,000. Logged every 500 ticks, so shorter dips can be missed.
- **Cap-blocked births:** `total_cap_blocked_births` at the final tick.
- **Reserve balance residual:** largest absolute `energy_balance_residual` over all snapshots (an instrument check).

The run is the independent unit. Snapshots within a run are not replicates.

## Acceptance criteria for a candidate rate

A rate is acceptable only if all four hold:

1. All 10 seeds survived to tick 200,000.
2. Cap-blocked births are zero in all 10 seeds.
3. The mean across seeds of the late mean population is between 800 and 1,300.
4. No seed has a minimum after burn-in below 200.

The 800–1,300 range and the floor of 200 are engineering choices for a watchable, statistically usable population. They are not biological claims.

## Decision rule

- The **smallest** acceptable rate becomes the proposed default: it is the scarcest world that still meets the criteria, so it keeps selection pressure highest.
- If no rate is acceptable, the default stays unchanged and the result is reported as is.
- A proposed default is applied to `configs/default.toml` only after the owner agrees. `configs/demo.toml` is tuned separately for watchability and is not part of this decision.
- Runs are not dropped. Extinct runs count against criterion 1. A run that fails for a technical reason (not extinction) is re-run from its checkpoint and the failure is reported.

## Known limits

- Environment events share the random stream with evolution, so droughts fall at different times in each run. That is acceptable for a calibration of average behaviour.
- 200,000 ticks is several hundred generations. It says nothing about multi-million-tick stability.
- Trajectories differ from the pilot's, because the engine's maths was made portable afterwards (CHK-2026-11-15-PORTABILITY). The model's rules are unchanged.
