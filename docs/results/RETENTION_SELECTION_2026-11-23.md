# Selecting for recall spread five-tick retention across matched populations

At generation 16, 487/512 controllers in the selected populations answered both hidden cues correctly, compared with 15/512 under random survival. The prespecified mean paired difference in population prevalence was **+92.1875 percentage points**. Of 32 matched founder populations, 31 favored selection and one tied. Each seed pair is one experimental unit. The result applies to this supplied three-weight circuit and the specified survivor policy. [Frozen protocol](RETENTION_SELECTION_PLAN.md)

**Acquisition and descendants.** Twenty-two founder populations already contained a successful controller. Among the ten initially without one, selection first produced success in generation 1 for six pairs and generation 2 for three. One search finished without success. Under random survival three first succeeded in generation 1, one in generation 3 and one in generation 12. Five finished without success. Among newly generated children of successful parents, 5,163/5,517 succeeded under selection and 80/125 under random survival. Those conditional counts reflect different parent populations and repeated ancestry. [Every seed and generation](retention_selection_pairs.json)

**Change of delay.** After freezing the best final training-ranked member of each population, we counted a champion as passing when it got both hidden cues correct intact, one correct after erasing state, and both visible cues correct after the same reset. All counts below are out of 32 champions per arm.

| Blank ticks | Selected | Random survival | Paired selected wins / ties / losses |
|---|---:|---:|---:|
| 1 | 31 | 4 | 27 / 5 / 0 |
| 5 (training) | 31 | 4 | 27 / 5 / 0 |
| 10 | 11 | 1 | 10 / 22 / 0 |
| 20 | 11 | 1 | 10 / 22 / 0 |

Fewer champions passed at the two longer delays. Both longer delays are even, while both shorter delays are odd. The data leave duration and odd/even timing confounded. Recurrent wiring was supplied to every founder. How often mutation discovers useful wiring in an initially feed-forward population remains open.

**Reproduction.** Clean native source c6960f2af2aacd3fe7e2d4b31056806416914631, plan commit a466f82, macOS arm64, rustc 1.99.1. All 64 histories, the pre-diagnostic winner list and all 1,536 diagnostic pairs are archived. The Python checker reconstructed raw responses from saved weights, checked founder equality, ranks, ancestry and survivor counts, and reproduced the published seed summaries. [Raw archive](retention_selection_raw.tar.gz) (SHA-256 d179e80009031967ec39fc8e5eb6a7e986b4796fbb50662e970d42b56189bc7e).

```bash
cargo run -p sim-runner --release -- retention-selection
mkdir -p output/retention-review
tar -xzf docs/results/retention_selection_raw.tar.gz -C output/retention-review
python3 analysis/check_retention_selection.py output/retention-review > output/retention-review/summary.json
cmp output/retention-review/summary.json docs/results/retention_selection_pairs.json
```
