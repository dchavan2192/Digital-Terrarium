# Prospective visible-cue training-diversity pilot — 2026-11-18

This design is frozen before any actual policy is evaluated on the six new held-out conditions below. It tests a bounded change in the training environments of a visible-cue genetic search. It does not test memory retention or alter ecological fitness. Previously examined layouts may be used for training; their old validation results do not become fresh test evidence. The parent approved the coordinates below before implementation/evaluation. Source, binary and exact configuration hashes must be recorded in a prelaunch addendum before the first search; a configuration or implementation change after test inspection requires a new study and fresh tests.

## Question, pairing and fixed search

Does training on three specified relative goal geometries yield better transfer to four untouched relative geometries than a reference-only search with the same maximum trial budget? Heading robustness is a separate secondary endpoint.

Eight searches: four paired seeds 5000–5003, each with a fixed-reference and a varied-geometry arm. Balanced serial order is (5000,fixed), (5000,varied), (5001,varied), (5001,fixed), (5002,fixed), (5002,varied), (5003,varied), (5003,fixed). Each pair starts from byte-canonical identical founder genomes and persistent IDs; save founder hashes to verify this. Pairing is at the search-seed level. Selection outcomes and subsequent evolutionary random histories may diverge. Deterministic trial evaluation must consume no evolutionary RNG.

Both arms: population 64, founder generation 0 plus 25 replacement generations, 8 elites, tournament size 3, 8 hidden nodes, the existing feed-forward founder procedure and persistent-ID tie resolution. Use native weight-only asexual mutation: probability 0.1 and step 0.2; add-connection, add-node, toggle, prune, add-recurrent and trait mutation probabilities all zero. No recurrent edges, crossover, within-trial learning, methylation update, ecological selection or new reward mechanism. Preserve existing founder/offspring/elite identity and lineage rules. Rescore all 64 candidates in all 26 generations, retaining every generation and every declared seed, including failures and partial successes.

Each candidate receives exactly six training trials: both signed cues in each of three fixed ordered layout slots. All trials reset neural state and pose. The fixed arm uses three identical copies of the reference layout; the varied arm uses reference, near and narrow once each. Repeated reference copies are deterministic budget controls, not independent observations or additional environmental diversity. Both arms have 1,664 candidate score evaluations and 9,984 training trials per search; the eight-search campaign has 13,312 score evaluations and 79,872 training trials. With cue1 + blank5 + movement horizon20, the maximum inference budget is 259,584 ticks/search and 2,076,672 campaign-wide. Actual executed movement steps and wall time can differ because trials terminate on contact.

Rank candidates lexicographically by total correct contacts out of six (descending), then mean clipped terminal progress across the same six trials (descending), then persistent ID (ascending). Correct count is primary; progress cannot outrank an extra correct response. Progress is computed from the assigned goal disc after termination using the already established normalization max(0,min(1,(start_surface_distance-terminal_surface_distance)/start_surface_distance)). Reward remains terminal-only. Neither assigned target coordinates nor progress, success or reward are added to controller observations or used for within-trial changes. Fixed-arm correct totals are multiples of three; its ordering is the same as reference-only correct/progress ordering, while varied totals can be any integer 0..6. This difference reflects the declared training distribution, not a statistical increase in replicate count.

## Arena and training layouts

Use existing 20-input visible-cue movement semantics: five rays, field of view 2.094 radians, range12, radius1.5, maximum turn pi/16, speed1, cue_ticks1, blank_delay5, movement horizon20, and existing fixed nonvisual scalars. Pose is held during cue/blank phases. The signed cue is present during movement; both goal markers have identical appearance, and only terminal contact reveals reward. Cue +1 maps to the egocentric left goal, cue -1 to right. Preserve this mapping in every layout and rotation; never flip mapping after results.

Coordinates below use a = lateral offset and b = forward offset. At release (0,0), heading pi/2, left/right centers are (-a,b)/(a,b). Training layouts, in order:

| Name | a (lateral) | b (forward) |
|---|---:|---:|
| reference | 4 | 4 |
| near | 3 | 3 |
| narrow | 2.5 | 4 |

Fixed arm slots are reference/reference/reference. Varied slots are reference/near/narrow. The physical arena and sensors within a slot are otherwise identical across arms. Keep duplicate slot IDs distinct in logs so all six trial outcomes remain auditable.

## Fresh held-out conditions

Freeze these six conditions without evaluating any saved or newly trained actual policy on them. No new test condition may be chosen based on policy scores or traces. Conditions are deterministic, hand-chosen engineering probes, not a random population of environments.

| Name | Family | Specification |
|---|---|---|
| geometry_forward_shift | relative geometry | a3.5, b5 |
| geometry_wide_shift | relative geometry | a5, b3.5 |
| geometry_near_narrow | relative geometry | a3, b4.5 |
| geometry_far_diagonal | relative geometry | a4.5, b5.5 |
| heading_minus30 | heading only | rigidly rotate reference pose and goals by -pi/6 |
| heading_plus30 | heading only | rigidly rotate reference pose and goals by +pi/6 |

Rigid rotation changes absolute heading while preserving egocentric goal geometry and the required relative turn sequence. It therefore probes sensitivity to an absolute-heading nuisance, not new relative navigation geometry. Normalize heading consistently with the existing arena. Relative-geometry cases retain the reference release pose/heading. All six preserve radius, sensors, cue mapping, motion constraints and horizon20; neither family adds distractors or stochasticity.

Policy-free geometry checks precede launch: circles nonoverlapping, start outside both, mirror symmetry, positive forward displacement, initial visibility and a feasible legal-action path within20 movement ticks. With the frozen five ray angles, analytic angular intersection margins for the four changed geometries are approximately0.16109,0.16138,0.21653,0.05045 radians, respectively, so both discs initially intersect a ray. A legal geometric witness turns in place toward the assigned center then moves straight; conservative tick bounds are9,10,7,10, respectively. These calculations establish arena feasibility/visibility, not that the manufactured reference or an evolved network implements that witness. Rotations preserve the corresponding reference geometry checks. Do not reject a frozen case because a controller later fails it.

Synthetic/manufactured instrument checks may test implementation correctness before actual test evaluation, but must not be used to tune these conditions to actual policies. Save the instrument outputs and any failures. Previously used seven-case validation layouts are consumed development conditions; do not report them as new held-out evidence.

## Selection lock, endpoints and analysis

Select exactly one final-generation champion per search using training ranking alone. Include all eight champions, even zero-success/partial-success searches. Lock/export/hash all eight training-selected champions and complete training histories before any actual held-out evaluation. No best-seed selection, test-based checkpoint selection, test-based mutation tuning, alternate cue mapping, additional training or choice of champion using test scores. A technical failure remains a planned missing search; do not replace its seed after inspecting results.

Evaluate both cues on all six frozen cases for each available champion. Primary held-out endpoint is intact-controller correct contacts /8 on the four changed relative geometries. Secondary endpoint is intact-controller correct contacts /4 on the two heading cases. Report left/right/timeout and per-cue terminal progress by case; report both-cue-success cases alongside aggregate correct fractions. Progress is diagnostic and cannot rescue a wrong response or timeout. Keep the two families separate in headline results. If the evaluator also runs state-cleared controls, retain them as integrity diagnostics, not extra independent replicates; these feed-forward controllers should have exact intact/cleared equality.

For each search report founder and final training correct totals /6, training progress, first generation reaching6/6 (or never), and complete score trajectories. Scores /6 are measured on different training distributions, so report them as within-arm acquisition rather than an identical external test. Report the common reference score separately. For held-out endpoints report all four seedwise varied-minus-fixed differences, their arithmetic mean and range, explicit available-pair counts /4, and each arm's available searches /4. A search, not a trial, cue, layout, organism or generation, is the independent unit. Missing results stay explicit; no pooling thousands of candidate evaluations into apparent replication. Four pairs form an engineering pilot, not a powered confirmatory test. There is no arbitrary success-percentage gate or significance claim. A future replication requires a prespecified precision target and fresh seeds/tests as appropriate.

This experiment changes both diversity and weighting of the three chosen training conditions; it does not identify a universal benefit of variation, an optimal curriculum or a learning algorithm mechanism. Positive transfer is limited to these frozen families and selected feed-forward controllers. Negative transfer can reflect the finite budget, training difficulty, or poor acquisition. Neither result establishes adaptive memory, within-lifetime learning, ecological competence or UAL. After this test is opened its cases are consumed; subsequent development needs new untouched conditions for another generalization claim.

## Operations and provenance gate

Do not overlap this campaign with the active ecological long-run campaign. Queue preparation may occur, but Pi simulation/build work starts only after the old campaign is cleanly complete and its owned runner/guard descendants are absent. Pin the validated new source revision before building the Pi release executable; save source-clean state, compiler/target/core-build identity, binary SHA-256, full arena/protocol/training-suite/holdout-suite hashes, copied worker/supervisor/guard hashes, paired order and exact commands in the prelaunch addendum/campaign manifest. Validate all whole effective configurations and mutation operators before ticks.

Use one native worker and RAYON_NUM_THREADS=1; reuse the hardened owned-process supervisor and thermal guard75/65 degrees C, poll5 seconds, fail closed on bad sensor/guard loss. The implementation uses one command with two strictly ordered phases: all eight searches, then the locked-champion held-out pass. Its single1,800-second watchdog covers the entire command, including startup/cooling and both phases, plus bounded guard cleanup. This replaces the earlier provisional per-search operational budget before any study execution; no score/trial/generation budget changes. Require20GiB disk/3GiB available RAM before work and record actual checks; do not claim continuous resource monitoring. Stop on technical/supervision failure, preserve all partials, and do not overwrite or rerun based on outcomes.

Archive raw training scores, genomes/lineages, champion trials, test traces, manifests, thermal records and hashes. Audit exact64x26 score matrices, rank/elite/parent rules, shared founder identities, six training trial slots per score, source/config identities, finite progress recomputation from terminal poses, cue/pose/hold semantics, all eight planned denominators and all six test conditions. Preserve the full result matrix regardless of outcome. This document specifies design only; no experiment has been run or outcome inferred here.

## Resumption and source reconciliation — 2026-11-22, before launch

The older ecological campaign finished8/8 cleanly and the Pi is idle. Upstream source2e28f08 added ecological diet mutation parameters and a neutral shadow while this pilot implementation was unfinished. The isolated search explicitly sets the separate diet-mutation probability to0 as well as protocol trait/topology/recurrence probabilities, preserving its weight-only contract. The embedded defaults and new clean source/core/compiler will be pinned in the actual launch record; this is a within-revision paired comparison, not a bit-identical continuation of the earlier pilot. No ecological simulation is instantiated by this command.

Default complete-command bounds, including fixed controls and selected-champion diagnostics:13,312 candidate scores,79,872 training trials,294 diagnostic trials,2,084,316 maximum neural steps,481,476,996 conservative edge passes and7,938 retained trace samples, below caps3M/1B/100k. Training artifacts and all selected champions are fingerprinted before any held-out evaluation. Four paired seeds and all six untouched held-out definitions remain unchanged. The owner authorized this one-off run on2026-11-22 and requested that scheduled automations remain disabled.

## Primary research basis and limits

These are peer-reviewed primary studies, not evidence that the proposed pilot will succeed; no direct quotations are used.

- Cobbe, Klimov, Hesse, Kim and Schulman (2019), “Quantifying Generalization in Reinforcement Learning,” ICML, PMLR97:1282–1289. Their CoinRun experiments separate training/test environments and find overfitting even with large training sets; several regularization/architecture choices improve generalization in that benchmark. This supports explicit untouched tests, not an extrapolation that three training geometries suffice here. Official proceedings: https://proceedings.mlr.press/v97/cobbe19a.html (no DOI listed there).
- Milano and Nolfi (2021), “Automated curriculum learning for embodied agents a neuroevolutionary approach,” Scientific Reports11:8985, DOI10.1038/s41598-021-88464-5. They adapt evaluation conditions to evolving agents' difficulty/weaknesses and report improvements on two embodied-control benchmarks. Our fixed three-layout treatment is not their adaptive curriculum and cannot inherit its reported advantages. Publisher: https://www.nature.com/articles/s41598-021-88464-5 ; author-institution full text: https://iris.cnr.it/retrieve/3f38f4af-26ed-43c6-ad5b-6be1170895c5/2021-Milano-Nolfi-Automated-Curriculum-Learning.pdf .
- Fortunato, Tan, Faulkner, Hansen, Badia, Buttimore, Deck, Leibo and Blundell (2019), “Generalization of Reinforcement Learners with Working and Episodic Memory,” NeurIPS32. They construct a diverse memory-task suite, evaluate held-out variations and perform architectural ablations to study memory-specific generalization. This motivates separating generalization dimensions and memory claims; a visible-cue feed-forward task here does not reproduce their memory experiment or establish working/episodic memory. Official proceedings: https://papers.neurips.cc/paper_files/paper/2019/hash/02ed812220b0705fabb868ddbf17ea20-Abstract.html ; full text: https://proceedings.neurips.cc/paper/2019/file/02ed812220b0705fabb868ddbf17ea20-Paper.pdf (no DOI listed on the proceedings page).
