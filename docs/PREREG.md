# Draft study plan: inherited neural state and environmental predictability

**Status: DRAFT — NOT REGISTERED, NOT FROZEN, NOT LAUNCHED.**

Created 2026-11-15. This document proposes a study, not a completed preregistration. Pilot effect sizes, assay implementation, seed lists, sample size, observation horizon, primary endpoint, and final analysis are unresolved. No confirmatory results may be attributed to this draft. Before validation, resolve the checklist below, commit a frozen protocol, and record its revision and timestamp separately from later amendments.

## Question and scope

Does inherited connection-suppression state improve offspring performance because it retains useful information about parental conditions, or does any effect come from generic neural attenuation and its energetic consequences?

The model's “methylation” is a controller-weight suppression abstraction, not molecular epigenetics. The target is a bounded causal effect in this model. No intelligence, consciousness, or open-ended-evolution claim follows from the proposed endpoint.

Primary motivation comes from [Colicchio & Herman (2020)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7029079/), whose environmental-data/optimality study links transgenerational plasticity to predictability, and [Romero-Mujalli et al. (2024)](https://doi.org/10.1093/evlett/qrae012), whose stress-sensitive epigenetic plasticity model resets marks at reproduction. These motivate controls and questions; they do not establish a Terrarium result or its novelty. See the [council review](research/SCIENCE_COUNCIL_2026-11-15.md).

## Prerequisites

- [ ] Organism-reserve accounting and forced mechanism tests pass, including clipping, transfer loss, reproduction, and signed death removal.
- [ ] Source/build/config/seed/metric provenance is recorded and outputs cannot be overwritten.
- [ ] Genome and state export plus same-build checkpoint continuation are verified. Long Pi experiments require tested recovery.
- [ ] An assay can initialize genetically identical newborn copies, transform marks before brain construction, and control subsequent stress responses without changing other physiology.
- [x] (2026-11-16: droughts have their own random stream and an optional `food.droughts.schedule_seed`; tested in `environment_stream.rs`.) Evaluation uses independent environmental RNG or an explicit replay schedule. Same initial seed is not sufficient while organism/environment events share one RNG.
- [ ] Calibration and validation seed sets, environment schedules, and lineage-sampling rules are disjoint and committed.
- [ ] Primary endpoint, horizon, smallest effect of interest, sample-size rationale, analysis, and exclusions are fixed before validation.

Checkpoint rationale: [Cerutti et al. (2008)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2632580/) demonstrated artifacts from repeating PRNG sequences after molecular-simulation restarts and recommended retaining generator state. That is a transferable failure mode, not validation of this ecology. Complete-state continuation is our engineering acceptance criterion; seed-only reinitialization is insufficient.

## Part A: mechanistic common-garden assay

Sample newborn genomes and post-inheritance-noise marks from multiple independent evolutionary runs at prespecified times. The sampling rule must not select only the most successful lineages, most methylated organisms, or favorable outcomes. Record extinct donor populations and how the absence of samples is handled.

For each sampled newborn, construct assay copies with identical genome, body traits, age, starting energy, position/heading treatment, food/fruit fields, environmental schedule, and other initial state. Construct each newborn brain from the transformed marks; do not compare a lagging living brain in one arm with a freshly rebuilt brain in another.

| Arm | Initial marks | Within-lifetime response | Purpose |
|---|---|---|---|
| Inherited | Exported newborn marks | Same stress response and demethylation in all main arms | Full inherited-state condition |
| Reset at birth | Zero marks | Identical response begins after birth | Isolate inheritance from within-life response |
| Shuffled | Permute inherited values among enabled connections, preserving their exact distribution and sum | Identical response | Test location-specific information versus amount of suppression |
| Uniform attenuation | Replace enabled-connection marks by their inherited enabled-pool mean | Identical response | Test aggregate gain reduction rather than inherited pattern |

Inherited, shuffled, and uniform arms match total initial methylation over the enabled pool; reset intentionally removes that state. Keep disabled-connection marks and treatment rules explicit and identical across the non-reset controls. Shuffles must not move marks between enabled and disabled pools. Repeat permutations as nested assay trials, not independent evolved replicates.

Equal methylation totals do **not** guarantee equal effective signaling or energy expenditure: weights and activations differ across connections. Measure realized structural/activity costs and reserve balance in every arm. Do not describe these controls as perfectly energy-matched.

Evaluate every arm in both stressful and benign common-garden conditions. The same replayed environment and matched placement trials apply to copies of a genotype. Avoid uncontrolled interactions between treatment copies; use separate otherwise-identical assay worlds or a declared competition design. A survival assay must specify how reproduction is handled and whether offspring count toward its endpoint; do not silently mix survival and fecundity.

### Hypotheses and alternative explanations

- **Inherited-information hypothesis:** inherited marks outperform reset marks in environments predictable from parental exposure, and the advantage is reduced by shuffling. This is a proposed pattern, not a guaranteed expected result of the current mechanism.
- **Generic attenuation alternative:** uniform suppression performs similarly to inherited patterns, or performance differences track altered neural energy cost rather than mark location.
- **Within-life-only alternative:** reset copies catch up once exposed to the same stress; initial inheritance has little lasting effect.
- **Maladaptation/null:** inheritance harms performance after an environmental switch, or the effect is too small to matter at the declared precision.

## Part B: ecological follow-up

Only after Part A is interpretable, compare inherited-state policy × environmental predictability. Keep mean resource input, stress intensity distribution, and other settings comparable; distinguish changed temporal correlation from simply changing total drought exposure. Validate intended schedule statistics before organism runs.

Measure the correlation between each parent's relevant exposure window and its offspring's selective environment at actual birth/evaluation times. Agents have overlapping, variable generations and move between patches, so fixed tick-lag autocorrelation alone does not establish information available across generations. Prespecify exposure summaries and handle two-parent ancestry explicitly if sexual reproduction is included. A first mechanistic study can avoid this additional question by using sampled identical newborn genomes rather than interpreting sexual ancestry.

Include extinction, cap-blocked reproduction, local food availability, and energy-cost summaries as diagnostics. A cap that binds in one arm can change the causal comparison. Add niche-construction interactions only after this two-factor study is supported; do not launch a broad factorial merely to seek a significant combination.

## Endpoints, sample size, and analysis — unresolved

**Pilot-only choices:** compare candidate endpoints such as survival through a fixed horizon, or realized reserve change during a nonbreeding assay. Choose one primary endpoint and horizon after checking assay sensitivity and biological interpretation. The final protocol must specify handling of death, reproduction, and zero populations. No numerical effect size or required replicate count is asserted before this pilot.

Define the smallest effect worth detecting in endpoint units. Use pilot variation and a prespecified precision/power analysis to choose independent evolutionary runs and assay repetitions; do not assume that 5, 10, or 20 seeds is automatically sufficient. Freeze the validation sample size and stopping rule before observing validation outcomes.

The independent unit is the donor evolutionary run or prespecified independent lineage, depending on the frozen design. Offspring, timepoints, and permutations from one donor run are nested observations. Analyze matched differences or a hierarchical model with that dependence represented. Report effect sizes and uncertainty. Secondary outcomes and multiple comparisons must be identified in advance; exploratory analyses are labeled as such.

Use held-out seed sets and environment schedules for validation. Do not drop extinct or nonrecovering runs to equalize sample sizes. Recovery not reached before the observation limit is censored, not an observed recovery at the final tick. Record technical failures separately from biological extinction and specify rerun rules before launch.

## Interpretation and completion gate

A reproducible positive, harmful, or sufficiently precise null result is a valid outcome. An imprecise null does not establish absence of an effect. Stop according to the frozen rule; do not tune costs, stress, or mark dynamics until the desired effect appears.

Comparisons with neutral demographic shadows may answer a separate selection-versus-drift activity question; they are not substitutes for the controlled offspring assay. Finite-run topology activity cannot demonstrate unbounded adaptation. Novelty is assessed against primary prior work before submission, separately from whether an effect is statistically distinguishable from zero.

## Freeze record — fill before confirmatory execution

```text
Protocol revision and timestamp:
Code/build/metric/schema versions:
Assay implementation and tests:
Donor populations and sampling rule:
Pilot seeds/schedules and archived outputs:
Held-out validation seeds/schedules:
Primary endpoint and horizon:
Smallest effect of interest:
Power/precision analysis and sample size:
Matching, shuffling, and within-life update rules:
Reproduction, mortality, extinction, and censoring treatment:
Exclusions, technical failures, and rerun policy:
Analysis script revision and exact command:
Stopping rule and amendment policy:
Independent reviewer decision:
```

Accepted outcomes and their provenance belong in [RESULTS.md](RESULTS.md). Freeze and archive the protocol before validation, then record departures instead of rewriting the original plan retrospectively.
