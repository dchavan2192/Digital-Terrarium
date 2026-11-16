# Phase 9a food-regrowth pilot

Status: **calibration only**. Reviewed 2026-11-16 UTC (2026-11-15 local). All six runs completed 20,000 ticks with surviving populations; launcher exit code 0. No new default is selected from this pilot.

## Measurements

Late means use the 20 logged snapshots at ticks 10,500 through 20,000, every 500 ticks. Each row is one independent run; snapshots are not independent replicates. The minimum is the minimum *logged* population and can miss shorter dips.

| Food regrowth | Seed | Final population | Late mean population | Minimum logged population | Late mean reserve per creature | Cap-blocked births |
|---|---|---|---|---|---|---|
| 0.004 | 0 | 526 | 549.25 | 197 | 31.07 | 0 |
| 0.004 | 1 | 537 | 585.90 | 169 | 30.60 | 0 |
| 0.006 | 0 | 775 | 690.90 | 308 | 31.42 | 0 |
| 0.006 | 1 | 819 | 784.30 | 350 | 31.01 | 0 |
| 0.008 | 0 | 946 | 920.20 | 431 | 31.75 | 0 |
| 0.008 | 1 | 916 | 909.65 | 495 | 32.11 | 0 |

Across the two seeds, mean late populations were **567.58**, **737.60**, and **914.93**, respectively. The hard cap was 2,000; cumulative blocked births were zero in every run, so no eligible birth was rejected by the cap during these runs. This is useful evidence that the cap did not determine the observed population in this pilot. It does not establish indefinite stability or behavior with the cap removed.

Attacks persisted: late sampled attacks per tick averaged 10.60, 18.75, and 32.88 across seeds at the three rates. Sharing averaged 1.25, 1.075, and 2.525 transfers per sampled tick. These are event counts, not evidence of specialized predator strategies or adaptive altruism.

The maximum absolute organism-reserve balance residual across all logged snapshots was **1.15e-10 energy units**, consistent with floating-point summation noise. This checks the sampled ledger accounting; release builds do not execute the debug balance assertion on every tick, and CSV logging cannot certify every intermediate tick.

## Interpretation and next decision

Increasing food regrowth was associated with larger populations in both seeds. At 0.008, the late population was about 46% of the safety cap. It is the strongest candidate of these three for a larger follow-up, but still below the roadmap target of 800–1,300 across a broad validation set. The pilot covers only two seeds and two seasonal periods. It supplies no confidence about 200k-tick survival, rare crashes, or long-term evolutionary outcomes. Keep the current defaults until independent validation is complete.

Next: preserve a compatible executable, enable periodic checkpoints in a single-run batch (sweep checkpoint options are not implemented), then validate candidate rates at 200k ticks using ten independent seeds. Do not count these tuning seeds as held-out validation. All runs expected to exceed three minutes belong on the Pi.

## Provenance

- Compiled source revision: `2f49287c44642e0c4354dbf84682c9c82d3f137e`.
- All manifests explicitly say `source_dirty = "true"`. At inspection the Pi tracked tree was clean at that revision, with only untracked `nohup.out` and `run_overnight.sh`. Those operational files plausibly explain the flag, but later inspection cannot prove the exact tree at build time. Retain this limitation; these are calibration observations, not accepted scientific results.
- Platform: Raspberry Pi 5, `aarch64-unknown-linux-gnu`; release build; `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; two Rayon workers.
- Stats schema 2; topology hash version 2. Requested/completed duration: 20,000 ticks each. Seeds 0 and 1. Override `creature.hard_cap=2000`; varied `food.regrowth_rate=0.004,0.006,0.008`.
- Full effective base config: [phase9a_pilot_base.toml](phase9a_pilot_base.toml), seed 0/rate 0.004; other jobs vary only seed and food regrowth. Per-run manifests/configs remain alongside raw data.
- Remote launch: `/home/pi/Digital-Terrarium/output/phase9a/launch_20261115T230545Z_22098`.
- Session: `session_1794783945293292320_22229_0`.
- Local raw mirror (ignored): `output/pi-pilot-review/launch_20261115T230545Z_22098/`.
- Committed [machine-readable measurements and SHA-256 artifact checksums](phase9a_pilot_2026-11-15.json) cover all six CSV/config/manifest triplets. Raw artifacts remain on the Pi and local mirror; the repository does not contain a full immutable raw-data archive.
- No simulation processes remained at the first completion check. Temperature was 54.3 °C then, 52.7 °C at provenance inspection. `throttled=0xe0000` records historical throttle conditions; no current throttle bits were set at inspection. No continuous thermal trace was collected.

Reproduce this summary without third-party packages:

```bash
python3 analysis/summarize_ecology_pilot.py \
  output/pi-pilot-review/launch_20261115T230545Z_22098/session_1794783945293292320_22229_0 \
  docs/results/phase9a_pilot_2026-11-15.json
```

The script rejects incomplete runs, unexpected sample windows, missing experimental cells, and manifest/config mismatches. The raw CSVs and stored configurations were read directly; no new experiments were launched for this report.
