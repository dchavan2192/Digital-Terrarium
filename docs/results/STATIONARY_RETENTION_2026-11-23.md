# A cue survives five blank ticks in three of four selected circuits

The pilot supplied a recurrent turn self-loop and selected its weight together with a bias and cue weight. Each trial presented one of two signed cues and then five blank ticks. With the cue hidden at the probe, three selected circuits answered both cues correctly. Erasing neural state once before the probe made their two responses identical. When the cue was presented again at the probe, all three answered both correctly after the same reset. [Prospective protocol](STATIONARY_RETENTION_PLAN_2026-11-23.md)

| Population seed | Successful founders /16 | First successful generation | Hidden intact /2 | Hidden reset /2 | Visible intact /2 | Visible reset /2 |
|---|---:|---:|---:|---:|---:|---:|
| 6000 | 3 | 0 | 2 | 1 | 2 | 2 |
| 6001 | 1 | 0 | 2 | 1 | 2 | 2 |
| 6002 | 0 | none | 1 | 1 | 0 | 0 |
| 6003 | 0 | 1 | 2 | 1 | 2 | 2 |

Seed 6003 is the instance in which full task performance first appeared during weight search. Two populations already contained a solution at initialization. Seed 6002 responded with a negative turn to both hidden cues and reversed both visible answers. At this single response tick, clearing on every tick gave the same readouts as clearing once at the probe for every winner.

These observations demonstrate functional retention at the training delay in a supplied circuit. The four independent populations give a limited estimate of repeatability. The design cannot estimate the advantage of selection over neutral weight search, discovery of recurrent wiring, performance at other delays, or ecological fitness. The native rational activation approximation produced some raw magnitudes slightly above one. Signs determine accuracy and raw magnitudes enter the prespecified margin tiebreak.

The clean native macOS release build used source 84ea1578ca3c74e42218667e60f269b462fae3e8 and protocol commit 97a7200. Every generation and diagnostic is archived in the repository. The independent checker reconstructed all 1,088 candidate records and 24 diagnostic response pairs from saved weights. [Raw histories and diagnostics](stationary_retention_2026-11-23/)

```bash
cargo run -p sim-runner --release -- stationary-retention --output output/stationary-reproduction
python3 analysis/check_stationary_retention.py docs/results/stationary_retention_2026-11-23
```
