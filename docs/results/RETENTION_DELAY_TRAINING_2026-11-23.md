# When can an evolved controller still read the cue?

**Replay of the previous wiring study.** We replayed all 96 locked champions at every blank delay from 0 through 32. At delay zero the probe immediately follows the cue. All 576 previously archived champion-delay combinations matched on hidden turn outputs, boundary-reset hidden outputs and visible reset outputs. Among the seven selected champions passing the original retention criterion at delay 19, six reversed both cue choices at delay 20 and recovered at 21. The seventh, seed 6226, passed at every tested delay. Two other five-tick successes (6215 and 6218) alternated early but lost the required margin before delay 19. These are output trajectories. The role of particular recurrent edges remains untested. [All replay rows](retention_delay_replay.csv)

**New paired comparison.** We trained fresh populations with equal budgets. One arm repeated five blanks. The other used three, four, six and seven blanks. Both started from the same 16 feed-forward genomes within each seed and could acquire feedback edges. After locking the final training winners, both arms faced four unseen delays. Passing required a signed margin of at least 0.1 on both hidden answers, identical hidden responses after erasing state, correct visible answers after the same reset, and an expressed feedback edge. [Prospective protocol](RETENTION_DELAY_TRAINING_PLAN.md)

| Previously unseen blank delay | Fixed training, champions passing /32 | Variable training, champions passing /32 |
|---|---:|---:|
| 8 | 1 | 1 |
| 9 | 3 | 1 |
| 18 | 1 | 1 |
| 19 | 3 | 1 |
| All four (primary) | 1 | 1 |

Seed 6320 passed all four in both arms. The paired tally was one both, 31 neither, zero variable-only and zero fixed-only. The observed difference was zero percentage points. Success was rare, so these data give limited precision about the difference between schedules. Each seed pair is one independent unit. The original replay is descriptive and uses a different cohort.

**Reproduction.** Protocol revision 8242537, clean evaluator cfdc39c. A separate Python forward pass reproduced all 19,456 birth training records and 256 champion-delay rows and checked founder equality, ranking, lineage and the final winner locks. The raw archive contains all 66 result files. SHA-256 is 5ce48820f9fa7cd7da872fbd5e51fc5476d709625fb1f69ebedb525e03b415c1. [Archive](delay_training_raw.tar.gz) · [Per-seed results](delay_training_pairs.json)

```bash
python3 analysis/replay_wiring_delays.py docs/results/retention_wiring_raw.tar.gz > output/replay.csv
cmp output/replay.csv docs/results/retention_delay_replay.csv
cargo run -p sim-runner --release -- retention-delay-training
mkdir -p output/delay-review
tar -xzf docs/results/delay_training_raw.tar.gz -C output/delay-review
python3 analysis/check_delay_training.py output/delay-review > output/delay-review/results.json
cmp output/delay-review/results.json docs/results/delay_training_pairs.json
```
