# How much feedback circuitry sustained recall?

We replayed every subset of expressed recurrent edges in the three saved champions previously found to pass all 33 delays. Each intervention preserves the original feed-forward network, all retained weights, cue mapping and reset controls. Sufficiency here means passing at every blank delay from 0 to 32 within that particular trained controller. [Previous single-edge study](RETENTION_EDGE_LESIONS.md)

| Champion | Subsets tested | Passing subsets | Delays passed with turn self-loop as sole feedback | Smallest sufficient feedback set |
|---|---:|---:|---:|---|
| 6226 selected | 32 | 8 | 5/33 (0–4) | turn → turn and turn → hidden node 10 |
| 6320 fixed | 32 | 8 | 3/33 (0–2) | turn → turn and turn → hidden node 9 |
| 6320 variable | 16 | 8 | 33/33 | turn → turn |

Each champion has exactly one inclusion-minimal sufficient subset. Removing every recurrent connection made hidden outputs identical across cue signs in all three. Restoring all connections reproduced 33/33 passes in each. The study exhausts feedback subsets for these fixed weights and this finite delay window. The three cases were selected for sustained performance and represent two independent founding populations. The two seed-6320 outcomes alone cannot establish a training-schedule effect.

Run `python3 analysis/retention_feedback_subsets.py` from the repository root to reproduce the [complete 80-subset results](retention_feedback_subsets.csv). The existing stationary evaluator supplies the response and control logic. An ecological test of fitness would additionally require a cue predicting future resources, a matched shuffled-cue condition and a reproductive endpoint.
