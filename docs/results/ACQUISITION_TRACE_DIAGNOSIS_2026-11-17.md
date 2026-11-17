# Saved acquisition trajectory diagnosis — 2026-11-17

## Scope and extraction

This is a descriptive audit of **all nine saved training-selected champions**, including every unsuccessful search in the original three-seed pilot and the paired six-search comparison. It introduces no evaluation, simulation, build, Pi work, new geometry, validation case or additional selection. The observations below come only from `summary.json → champion_training_trials → intact → trials`, using the two already recorded visible-cue trials. The separately saved state-cleared trials are not combined with these observations. Champions were selected on training performance; these traces are training replays, not held-out validation.

For each trace, extract the first movement action, first-contact outcome, movement tick count and final `pose_after`. Compute correct-goal surface distance as `max(0, hypot(x-goal_x, y-goal_y)-1.5)` and path length by summing Euclidean displacement over recorded movement segments, including the shortened contact segment. No neural policy is reevaluated. Numbers below are rounded to six decimals. Exact source files and checksums appear below.

The saved arena starts at `(0,0)`, heading `pi/2`, with goal centers `(-4,4)` and `(4,4)`, radius `1.5`, movement horizon `20`, maximum speed `1`, and maximum turn `pi/16`. Cue `-1` targets right; cue `+1` targets left. Initial surface distance to either correct goal is `sqrt(32)-1.5 = 4.156854249492381`. The cue remains visible during these movement trials. Positive first turn rotates toward left from the initial heading; negative rotates toward right. Turns are normalized outputs multiplied by `pi/16`; speeds below are normalized outputs multiplied by `1`.

## All champions

Every paired entry is **cue -1; cue +1**, respectively. `right / 20` means first right-goal contact on movement tick 20; `timeout / 20` means no goal contact through tick 20. The champion ID belongs to its own search and is not a cross-search identity.

| Search seed / champion ID | First turn (-1; +1) | First speed (-1; +1) | Choice / tick (-1; +1) | Final correct-goal surface distance (-1; +1) | Recorded path length (-1; +1) |
|---|---|---|---|---|---|
| Original 2000 / 117 | 0.219611; 0.324525 | 0.295466; 0.340805 | timeout / 20; left / 13 | 7.239760; 0.000000 | 8.155681; 5.755640 |
| Original 2001 / 17 | 0.256324; 0.255276 | 0.400651; 0.413740 | left / 16; left / 15 | 5.010440; 0.000000 | 5.118540; 5.211215 |
| Original 2002 / 3 | 0.317524; 0.294413 | 0.412060; 0.370257 | left / 15; left / 17 | 5.171913; 0.000000 | 5.538265; 5.611836 |
| Terminal only 2100 / 423 | 0.112880; -0.001363 | 0.487187; 0.493193 | right / 16; right / 14 | 0.000000; 5.027175 | 6.268620; 5.323478 |
| Terminal only 2101 / 182 | 0.500534; 0.433065 | 0.432852; 0.314789 | timeout / 20; left / 15 | 9.239681; 0.000000 | 9.895517; 5.269215 |
| Terminal only 2102 / 0 | 0.307386; 0.403722 | 0.324760; 0.330790 | timeout / 20; left / 17 | 5.468446; 0.000000 | 6.537401; 5.703059 |
| Terminal progress 2100 / 381 | -0.138335; 0.156466 | 0.274676; 0.345500 | right / 20; left / 20 | 0.000000; 0.000000 | 5.908404; 5.127456 |
| Terminal progress 2101 / 94 | -0.401004; -0.361870 | 0.538602; 0.585431 | right / 11; timeout / 20 | 0.000000; 10.289951 | 5.689201; 11.235802 |
| Terminal progress 2102 / 818 | -0.233687; -0.011438 | 0.422783; 0.485950 | right / 20; left / 15 | 0.000000; 0.000000 | 5.198869; 6.093145 |

## What the traces show

Each of the three original champions and each terminal-only comparison champion succeeds on one cue only. Their first-action cue contrasts vary: original seed 2001 turns `0.256324` versus `0.255276`, while original seed 2000 turns `0.219611` versus `0.324525`. Nonzero cue response alone does not establish correct action selection. Several policies initially turn left for both cues. Conversely, progress seed 2102 initially turns right for both cues, yet reaches the assigned goal on both trajectories. Initial turn sign alone cannot classify the complete feedback policy; later sensory inputs, headings and actions also change along the recorded path.

Progress seeds 2100 and 2102 achieve both correct training responses. Progress seed 2101 remains partial. Preserve the original pilot and the paired comparison as distinct searches; the two cues, repeated generation records, founders, champions and paired searches are not independent biological replicates. This nine-champion description is not a pooled effectiveness estimate or a causal optimizer comparison. It does not diagnose a memory deficit or demonstrate within-life learning.

### Partial seed 2101 and clipping

The terminal-progress champion (ID 94) contacts right correctly for cue -1 on tick 11; its stored progress is exactly `1`. For cue +1 it also starts turning right (`-0.361870`, speed `0.585431`) and times out on tick 20 at `(7.606867, 6.069686)`, heading `6.080696` radians. Its correct goal is left. Its final correct-goal surface distance is `10.289951`, beyond the starting `4.156854`; raw normalized progress is approximately `-1.475418`, clipped to `0`. Thus the saved mean terminal progress is `(1 + 0)/2 = 0.5`. This value does not mean that the failing response is halfway to its goal.

That failing trace travels `11.235802` units. Its nearest recorded movement endpoint to the correct-goal surface is approximately `3.244712` units, but ends much farther away. This endpoint minimum is a descriptive path statistic, not a continuous segment minimum and not the selection objective. The objective uses only terminal distance. Clipping also makes all failing responses that end at or farther from their target than the start share progress `0`, regardless of how far they overshoot; this identifies a score plateau, not its causal role in search failure.

### Horizon boundary

Three of the four successful responses belonging to the two fully successful progress champions first contact on the final permitted movement tick: seed 2100 cues -1 and +1, and seed 2102 cue -1. Their final segment contact fractions are `0.464629`, `0.027769` and `0.812257`, respectively. They are genuine contacts under the saved first-contact rule, which tests tick 20 before ending the trial, not distances merely rounded to zero. Seed 2102 cue +1 contacts left on tick 15.

These outcomes establish success for the exact saved arena and horizon, with no spare whole movement tick in those three cases. They do not establish timing robustness, generalization, retention or success under a changed horizon. Likewise, a timeout identifies no contact during these 20 ticks; it does not identify what would happen later. Longer horizons or perturbed geometries would be new evaluations and are not performed or selected here. Wrong-goal trials terminate immediately on that contact, so their shorter traces provide no post-contact behavior.

## Reproduction from existing records

Parse each file below, take the intact two-cue traces, and apply the formulas in Scope and extraction. The table retains all requested searches; no source is dropped for its outcome. The parent experiment reports remain the authority for prospective budgets, primary endpoints and between-search interpretation.

- `output/visible-acquisition/pilot_20261117T163705Z/runs/session_1794933573406315818_37169_0/seed_2000/summary.json` — SHA-256 `7904b7ae7bd8af5bb0fa4780cd100b82627c956237fdc1e6fe2191a13cdda373`
- `output/visible-acquisition/pilot_20261117T163705Z/runs/session_1794933573406315818_37169_0/seed_2001/summary.json` — SHA-256 `fb8ab37be7c69d74d9889bccbaad07202e2cc4303b8ebffd83f5e4e575d8404d`
- `output/visible-acquisition/pilot_20261117T163705Z/runs/session_1794933573406315818_37169_0/seed_2002/summary.json` — SHA-256 `ab7bd01ee89163211cd43ea4e0c3f078942ca33f5ef0f8d08e6eda7c0b79d68f`
- `output/progress-comparison/progress_comparison_20261117T205542Z/terminal_only/runs/session_1794949602917027560_37741_0/seed_2100/summary.json` — SHA-256 `e3728bd0f1eb372d9939d7033f598dfec900eaad25e1cb773c5138bab7e3ecb3`
- `output/progress-comparison/progress_comparison_20261117T205542Z/terminal_only/runs/session_1794949602917027560_37741_0/seed_2101/summary.json` — SHA-256 `73b347b14b4c83fc8f69a1b5213075257cf2fff7a89fdc50472a2966559f83c9`
- `output/progress-comparison/progress_comparison_20261117T205542Z/terminal_only/runs/session_1794949602917027560_37741_0/seed_2102/summary.json` — SHA-256 `1881904ff05be72d6ac83d25673f8b5c2b2c99a5b2fc91dbd6ffa2865064ef26`
- `output/progress-comparison/progress_comparison_20261117T205542Z/terminal_progress/runs/session_1794949608116550471_37747_0/seed_2100/summary.json` — SHA-256 `9826a8be7b838722daf3b1c6288531e570c17c81d73aa87258f2a874f0b2f045`
- `output/progress-comparison/progress_comparison_20261117T205542Z/terminal_progress/runs/session_1794949608116550471_37747_0/seed_2101/summary.json` — SHA-256 `67f2da7b6a10449f10745ba21b1c39c1b2269d7d2e117d8c838f1300ba0f230f`
- `output/progress-comparison/progress_comparison_20261117T205542Z/terminal_progress/runs/session_1794949608116550471_37747_0/seed_2102/summary.json` — SHA-256 `f8cbb20adabcbe1d59dc4e8447486c4612b21658282eed571d85d18685e5b2c0`
