# Scientific council — 2026-11-15

Status: design review, not experimental evidence. Reviewed the reconciled upstream code/roadmap after baseline `a9ddf87`; earlier local-only audit findings may already be fixed upstream. Primary-source checks below support decisions, not a claim of exhaustive literature coverage.

## Decision

Build a trustworthy instrument before expanding the mechanism set. The next useful scientific contribution should be a causal result about an inheritance mechanism, not a claim that an attractive ecosystem has evolved intelligence or unbounded novelty.

A candidate question is: **When does inherited neural suppression carry useful parental environmental information, rather than simply lowering neural gain or energy use?** This is narrower than the original three-feature factorial and uses existing methylation machinery once export/evaluation exists.

## Papers checked and consequences

| Primary source | What was checked | Consequence for Terrarium |
|---|---|---|
| [Grimm et al. 2020, ODD second update](https://www.jasss.org/23/2/7.html), especially purpose/patterns and rationale | Model descriptions should explain what a model is for, its evaluation patterns, process scheduling, and design rationale. | Document state and tick order explicitly. A citation motivates a mechanic; it does not validate its numeric default. |
| [Stanley & Miikkulainen 2002, NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf), sections 2.2 and 3.2 | Historical markings align homologous genes; disjoint/excess genes normally come from the fitter parent, with random treatment under equal fitness. | Repair historical node identity. Do not treat fitter-parent structure inheritance alone as a defect or implement arbitrary graph union and call it NEAT. |
| [Yaeger, Griffith & Sporns, Passive and Driven Trends](https://arxiv.org/html/1112.4906), Natural Selection vs. Random Drift | Polyworld lockstep replays demographic events with random parents and deaths. Mutation/crossover statistics are comparable rather than literally identical because genotypes differ. | Shadow comparison is established prior art. Match the null to the inference and preserve independence from the real simulation RNG. |
| [Dolson et al. 2019, MODES](https://doi.org/10.1162/artl_a_00280), paper abstract/metric description | The toolbox distinguishes change, novelty, complexity, and ecological potentials. | Name custom activity summaries accurately. A single growing statistic is not the whole toolbox or proof of open-endedness. |
| [Glasserman & Yao 1992, Common Random Numbers](https://business.columbia.edu/sites/default/files-efs/pubfiles/4261/glasserman_yao_guidelines.pdf), introduction and comparison framework | Coupling stochastic inputs can reduce comparison variance, subject to conditions; it is not a universal guarantee. | Separate/replay environmental randomness before controlled drought comparisons. Same seeds alone do not synchronize event meaning across code paths. |
| [Romero-Mujalli et al. 2024, Epigenetic Plasticity](https://doi.org/10.1093/evlett/qrae012), model/methods | Stress-sensitive methylation-like changes can alter plasticity; marks reset between generations in that model. | Within-life-only plasticity is an important control. Stress-responsive epigenetic modeling itself is not novel. |
| [Colicchio & Herman 2020, Environmental Variation and Transgenerational Plasticity](https://pmc.ncbi.nlm.nih.gov/articles/PMC7029079/), introduction/model/results | Parent-offspring environmental predictability changes the expected value of transgenerational information. | Vary predictability as well as drought severity, and measure experienced parent-to-offspring correlation. Do not assume any stressful environment rewards inheritance. |
| [Nisioti & Moulin-Frier 2023, Niche Construction in Diverse Environments](https://arxiv.org/html/2305.09369v1), model and results | Niche construction, adaptability, and environmental heterogeneity have already been studied together. | Seek a specific causal interaction, not a blanket novelty claim for combining plasticity and construction. |
| [Waibel, Floreano & Keller 2011, Hamilton's Rule](https://journals.plos.org/plosbiology/article?id=10.1371/journal.pbio.1000615), abstract/model | Simulated foraging robots provide a prior quantitative kin-selection experiment; costs and benefits concern fitness. | Energy-transfer amounts are not automatically fitness costs/benefits. Hue similarity does not quantify genealogical relatedness. |

The [JaxLife paper](https://arxiv.org/abs/2409.00853) is another direct neural-agent ecology precedent. The current search does not establish priority for Terrarium's complete mechanism combination. Closest-work comparisons must be extended before submission.

## Council challenges and resolutions

These are design positions and actual cross-agent review topics, not experimental findings.

**Mechanism tests.** The roadmap asks every ON/OFF pair to diverge within a fixed number of ticks. The scientific objection is that a correctly implemented mechanism can remain inactive. Resolution: force a state that exercises its causal pathway, test the predicted local effect, and separately verify OFF behavior. Random ecosystem divergence is insufficient as a correctness oracle.

**Energy boundary.** Reviewed the instrumentation agent's organism-reserve ledger proposal. Agreement: account metabolizable food inputs before storage clamping, separate overflow and transfer inefficiency, retain all cost categories, and avoid changing ecological rules in the instrumentation patch. Challenge: current late death processing permits reserve overdrafts, so death removal must remain signed and clearly documented. Use numerical tolerance scaled to absolute flows; test saturated and depleted fixtures. This is an accounting identity over organism reserves, not a whole-world thermodynamic model.

**Run provenance.** Reviewed the runner agent's proposed config/source/build/metric/completion manifests. Agreement: validate all generated jobs before execution and prevent output replacement. Explicit challenge: a same-seed comparison still has differing drought schedules when organism events consume the same RNG. Defer a causal drought study until an independent environmental stream or replayed schedule exists. Provenance records this limitation rather than hiding it.

**What counts as progress.** A gate demanding a positive effect encourages repeated tuning until significance. Resolution: accept a reproducible positive, negative, or sufficiently precise null result. Shadow-normalized activity is relevant to selection-versus-drift questions; it need not gate unrelated ecological endpoints.

**What counts as novelty.** Recurrence, plasticity, neural costs, niche construction, and neutral comparisons all have prior art. Potential impact comes from a sharply controlled empirical interaction, reusable validation/evaluation tools, and independently reproducible evidence. “No exact prior found” remains a search limitation, not evidence of being first.

## Immediate acceptance criteria

1. **Energy instrument:** per-tick reserve-change residual remains within declared numerical tolerance; each major source/sink has a forced fixture, including clipping and depleted-energy cases. Instrumentation alone does not change trajectories.
2. **Mechanism effects:** forced activation plus OFF noninterference for each flag. Keep life-history, neural, and environmental effects distinguishable.
3. **Runner:** effective configurations validated after every override; no nonfinite inputs; unique immutable run paths; source/build/config/seed/metric versions and completion status recorded.
4. **Calibration:** choose settings on pilot seeds, validate on held-out seeds, report extinction and cap-blocked reproduction in every arm. Do not select only surviving runs. The target population range is an engineering criterion, not biological validation.
5. **Persistence:** exact continuation test within a declared build/platform, including RNG, food/environment, methylation, activity history, counters, and lineage/species state. Test checkpoint failure handling before multi-day Pi jobs.
6. **Claims:** numerical scientific claims enter [RESULTS.md](../RESULTS.md) with provenance and analysis before README/portfolio. Historical pre-fix measurements remain clearly labeled.

## Narrow experiment proposal

This is a design to preregister after the instrument and export/evaluation paths are ready; no runs have been performed for it.

**Mechanistic pilot:** freeze genomes from multiple independent evolutionary runs. Make otherwise identical offspring copies with: inherited marks; reset-at-birth marks with the same subsequent stress response; shuffled marks preserving their distribution; uniform attenuation matched to mean methylation. Test benign and stressful common-garden environments with identical starting resources and held-out environment schedules.

This separates inheritance from within-life response, pattern-specific effects from aggregate attenuation, and genotype differences from marks. Matching mean methylation does not guarantee identical neural energy use, so measure realized energy flows rather than treating this control as perfectly energy-matched.

**Primary endpoint:** select one in pilot work, such as survival through a fixed horizon or realized energy balance before reproduction. Then fix it, the horizon, exclusion rules, smallest effect of interest, sample size, and analysis before independent validation. Run-level or lineage-level differences are the replicates; repeated permutations and offspring trials are nested observations.

**Ecological follow-up:** inherited-state policy × environmental predictability, holding mean resource supply comparable. Measure parent-to-offspring exposure correlations using actual lifespans and experienced local environments. Report extinction and failures to recover; recovery at the observation boundary is censored. Add niche construction only after the first interaction is understood. A null or harmful inherited-state effect is informative.

**Interpretation limit:** a benefit in this abstraction would demonstrate a property of inherited controller state under specified ecology. It would not establish molecular epigenetic realism, intelligence, or consciousness.

## Continuing review rule

For each new mechanism, record the primary papers, supported design choice, numerical assumptions, strongest alternative explanation, and a falsifying test. Ask another agent to challenge the design before merging. Revisit the closest prior work before claiming novelty, not after generating favorable plots.

## Implemented fixture review: Stage 0.3 scope

`crates/sim-core/tests/mechanism_effects.rs` adds ten short forced-action tests. This follows the council's resolution above: activate the specified causal pathway and compare with an OFF counterfactual. The ODD scheduling/rationale guidance motivates documenting what each intervention measures; these implementation tests are our engineering criteria, not claims from that paper.

| Switch | Forced pathway checked |
|---|---|
| Fruit | A fruit specialist with eating enabled consumes a known fruit quantity only when the layer is enabled. Changing disabled fruit yield is inert. |
| Temperature | Known latitude/preference mismatch incurs the predicted extra expense. Changing the disabled mismatch-cost factor is inert. |
| Pheromones | Known deposit and decay affect the grid and then a directly connected sensor-driven turn. Disabled deposition changes neither the grid nor action. |
| Day/night | A night-phase light sensor changes a directly connected turn; a distant food cue outside the half-range cannot trigger eating. Disabled cycle period is inert. |
| Elevation | Forced movement follows the terrain speed multiplier. Disabled slope cost does not affect movement. |
| Niche construction | Forced building incurs the configured expense and changes local regrowth. Disabled regrowth bonus is inert. |
| Epigenetics | Forced low reserves produce marks and living-brain weight suppression. Disabled methylation rate is inert. |
| Sexual reproduction | Nearby eligible parents produce the predicted sexual/asexual birth modes and energy investments. Disabled crossover rate is inert. |
| Activity tracking | Enabled tracking observes a known topology while leaving physical state and the next RNG draw unchanged. Disabled window size is inert. |

The OFF comparisons inspect specific physical/grid endpoints and the next RNG draw, rather than merely noticing divergence. Existing integration and energy-ledger fixtures additionally cover transfers, saturation, combat, reproduction, mortality, construction thresholds, seasonal/biome regrowth, and feeding tradeoffs. This verifies scoped switch pathways and major energy flows; it does not exhaust every parameter, interactions, long-term dynamics, or evolutionary benefit. Cross-platform and checkpoint continuation remain separate checks.
