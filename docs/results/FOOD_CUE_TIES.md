# Random tie-breaking let recurrent lineages survive. They still chose one side.

In eight matched founding populations, randomizing survival among equal birth scores changed which controllers became parents. No candidate exceeded four training births under either tie rule or either delay regime. All 32 final champions chose a single patch on the held-out predictive schedules. Clearing state changed no champion's mean birth count on reliable or shuffled cues. [Protocol committed before scoring](FOOD_CUE_TIE_PLAN.md) · [Earlier selection pilot](FOOD_CUE_SELECTION.md)

| Training delays | Parental tie rule | Recurrent parent slots out of 384 | Populations exceeding four training births | Mean held-out offspring, reliable cues | Mean gain over clearing memory |
|---|---|---:|---:|---:|---:|
| Fixed | Oldest first | 0 | 0/8 | 3.75 | 0 |
| Fixed | Random | 261 | 0/8 | 3.625 | 0 |
| Varied | Oldest first | 0 | 0/8 | 3.75 | 0 |
| Varied | Random | 265 | 0/8 | 3.625 | 0 |

There were four parent slots per generation, 12 parental selections and eight populations per arm. Slots include repeat appearances. The independent units are the eight founding populations. The final champion lock used the same oldest-first rule for all four arms. Each held-out champion had 16 food opportunities per lifetime on each of two previously unused event orders. With intact state and reliable cues every champion ate eight meals per order. The engineered memory control produced seven offspring with reliable cues and three after reset on both orders. A constant-side reference produced three.

## Did the cue survive long enough to affect movement?

An exploratory replay examined every new descendant in the random-tie arms with at least one expressed recurrent edge. For each distinct training delay we presented the same genome with opposite cues and compared its two signed turn outputs after the delay. We counted a detectable difference when the outputs differed by more than 1e-8. We also counted whether their signs were strictly opposite, which would direct the two choices toward different patches.

| Random-tie descendants | With expressed recurrence | Detectable turn difference at any training delay | Opposite turn directions at any training delay |
|---|---:|---:|---:|
| Fixed delays | 904 | 269 | 0 |
| Varied delays | 916 | 414 | 0 |

In hundreds of these descendants, the cue still changed how much the controller turned. It never switched the descendant from turning left to turning right, or vice versa, at a tested training delay. The study therefore narrows the immediate problem to converting cue-dependent activity into different actions. It does not establish why the signed turn stayed on one side or how frequently future independent populations could escape. A next diagnostic is to compare each controller’s turn after positive, negative and blank cues, then perturb the default turn contribution in replay while retaining the recurrent pathway. Score both patch choices and meals before investing in a longer evolution campaign. [Reproduce the exploratory replay](../../analysis/check_food_cue_tie_signal.py)

## Records and verification

The local single-worker run used clean source `5af9c40`. A separate random stream permuted the parental candidates before a stable sort by descending births. Mutation and parent sampling used the original stream. We retained every pre-sort permutation. All 16 oldest-first histories matched the previous archive byte for byte. The audit reconstructed every ranking and final lock. The archive includes 5,120 candidate records across the four arms, all held-out events, reference runs and paired per-seed results. Extraction and re-audit reproduced the held-out outcomes. [Complete archive](food_cue_tie_raw.tar.gz) · [Runner and auditor](../../analysis/run_food_cue_selection.py)

Archive SHA-256 is `8142edbfa813fa55516e886ffc5b43efa34fea244a9c1d7e60ed668d2d5bbf5a`. Use a fresh output directory.

```sh
cargo build --release -p sim-runner
python3 analysis/run_food_cue_selection.py --tie-study --output output/local-tie-study
python3 analysis/check_food_cue_tie_signal.py output/local-tie-study/training/session_*
```
