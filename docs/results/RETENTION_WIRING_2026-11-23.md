# Nine populations evolved functional feedback from feed-forward founders

At the five-blank training delay, 9 of 32 fitness-selected populations produced a final champion meeting the prespecified functional retention criterion. Random survival with the same recurrent mutation operator yielded 0/32. Selection with recurrence disabled yielded 0/32. Against each control the paired tally was nine wins, 23 ties and zero losses. Founders had no feedback edges. Native mutations chose the endpoints and initial weights of new recurrent edges. [Protocol](RETENTION_WIRING_PLAN.md)

The criterion required an expressed feedback edge, correct answers to both hidden cues with signed turn margins of at least 0.1, identical hidden answers after erasing neural state, and correct answers to both cues when they were shown again after the same reset. Thirteen selected champions passed when the margin threshold was removed. The feed-forward arm supplies a state-independent control. The random-survival arm measures discovery under the same mutation opportunity and uses the same best-final ranking rule.

| Blank ticks | Selected champions passing /32 | Selected passing with sign alone /32 |
|---:|---:|---:|
| 4 | 1 | 3 |
| 5 (training) | 9 | 13 |
| 6 | 1 | 3 |
| 19 | 7 | 9 |
| 20 | 1 | 1 |
| 21 | 7 | 9 |

Both controls scored zero by both criteria at every tested delay. Success was more common at the odd delays, even across the adjacent 19, 20 and 21 tick probes. This timing sensitivity limits claims about recall at an arbitrary delay. Identifying a specific oscillating circuit would require inspecting individual state trajectories and perturbing its connections.

The result establishes discovery in this stationary cue task under a supplied recurrent mutation operator, with 24 generations and a fixed two-hidden-node founder layout. Survival and reproduction in the live ecology remain separate questions. The 32 paired populations are the independent units. Diagnostic cues and delays are repeated measures.

The protocol was committed at 7465128 before collection. Native macOS arm64 release source 641468a was clean. All 29,184 birth records, 2,400 ranked population snapshots, 96 locked champions and 576 champion-delay rows were retained. A separate Python forward pass reconstructed every birth training response and all four probe conditions per diagnostic row. The archive checksum is 52355d4dc66ecd6be7abd8a6b490fc2cf55b2d93b6809eb2450a62becfaf1dc7. [Raw histories](retention_wiring_raw.tar.gz) · [Per-seed outcomes](retention_wiring_pairs.json)

```bash
cargo run --release -p sim-runner -- retention-wiring
mkdir -p output/wiring-review
tar -xzf docs/results/retention_wiring_raw.tar.gz -C output/wiring-review
python3 analysis/check_wiring.py output/wiring-review > output/wiring-review/results.json
cmp output/wiring-review/results.json docs/results/retention_wiring_pairs.json
```
