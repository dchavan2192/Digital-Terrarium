# Terrarium: Testing Inherited Neural State in an Evolving Agent Ecosystem

## Historical candidate paper plan — not the active development protocol

Status reviewed 2026-11-23: this outline concerns inherited connection suppression, a different mechanism from within-life recurrent activation memory. It remains a candidate, not a committed paper direction or evidence claim. Use the [current systems audit](research/SYSTEMS_AUDIT_2026-11-23.md) and [PROGRESS.md](PROGRESS.md) for active work. Recurrence is now implemented; older feed-forward descriptions below describe the original candidate baseline.

Updated: 2026-11-15. The earlier outline's numerical abstract and open-ended-evolution conclusion are withdrawn. Its historical version remains in Git history. Submission venue, date, novelty, and feasibility require verification before committing to a deadline.

### Working abstract

Terrarium is an artificial-life platform in which energy-limited agents inherit neural-network genomes, stress-responsive connection-suppression state, and an environment altered by prior agents. We propose controlled tests separating the effects of inherited neural state from within-lifetime response, generic neural attenuation, and changes in resource availability. Versioned experiment records, energy accounting, and replayable evaluation are prerequisites. The initial study will ask when parental environmental information changes offspring performance, using frozen-genotype controls and independently replicated ecological runs. Comparisons with neutral demographic controls may assess selection-related topology persistence, but finite-run activity will not be presented as proof of open-ended evolution. No effect or novelty is asserted before data and prior-art review support it.

### 1. Question and contribution

- Primary candidate: under which parent-to-offspring environmental correlations does inherited connection-suppression state improve or impair offspring performance?
- Separate inherited state, within-lifetime suppression, and ecological inheritance causally before combining them.
- Contribution must be an interpretable empirical finding or an informative bound, with reproducible artifacts. Combining familiar mechanisms is not sufficient by itself.
- Do not infer consciousness, intelligence, adaptive altruism, or reproductive isolation from activity, network size, hue similarity, or genome clusters.

### 2. Related work and novelty boundaries

- [Stanley & Miikkulainen (2002), NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf): historical markings support gene alignment. Fitter-parent disjoint/excess inheritance is part of the method; locally reused node identities are the relevant implementation risk.
- [Yaeger, Griffith & Sporns, Polyworld lockstep](https://arxiv.org/html/1112.4906): selection-versus-drift comparisons using matched birth/death schedules are prior art, not a new Terrarium method.
- [Dolson et al. (2019), MODES](https://doi.org/10.1162/artl_a_00280): adopt published metric definitions when implemented; do not label custom activity counts as the complete toolbox.
- [Romero-Mujalli et al. (2024)](https://doi.org/10.1093/evlett/qrae012): stress-sensitive epigenetic plasticity is modeled already; their marks reset between generations. This motivates a within-life-only comparison.
- [Colicchio & Herman (2020)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7029079/): environmental predictability matters to transgenerational plasticity; variability alone is insufficient motivation.
- [Nisioti & Moulin-Frier (2023)](https://arxiv.org/html/2305.09369v1): niche construction and adaptability have already been studied together across diverse niches.
- [JaxLife (2024)](https://arxiv.org/abs/2409.00853): neural-agent ecological modification and complex behaviors already have substantial precedents.

A potentially distinctive scope is controlled inherited neural-state effects in an energy-accounted ecology, followed by an interaction with ecological inheritance. This is a candidate contribution, not an established priority claim. Extend the search before using “first” or “unique.”

### 3. Model and reproducibility

Use the [ODD protocol](https://www.jasss.org/23/2/7.html) for purpose, state, scheduling, design rationale, initialization, and submodels.

- Current controller: 20 inputs with five vision rays, variable hidden nodes, seven actions; feed-forward, without lifetime synaptic learning.
- Explicit organism-reserve accounting boundary, including overflow, transfer losses, reproductive investment, and death removal.
- Configurable ecological dynamics and food-limited calibration; the hard cap is a performance backstop.
- Crossover historical identity and node-cap limitations documented and tested before sex-related inference.
- Full run provenance, checkpoint equivalence, fixed evaluation environments, and independent environment randomness or replayed schedules.
- Determinism claims restricted to verified platforms/builds. Same seed is not automatically a shared drought schedule across ablations.

### 4. Initial experiment

**Mechanistic pilot, before evolution-wide factorials:** export genomes and methylation state from multiple independent evolved runs. For each sampled genotype, create offspring copies with identical initial resources, age, position distribution, and evaluation schedule.

Compare inherited marks, reset-at-birth marks with the same subsequent stress response, shuffled marks, and uniform attenuation matched to mean methylation. Use benign and stressful common-garden environments. Mark permutations are repeated nested controls, not extra independent evolutionary replicates.

Choose the primary performance endpoint and observation horizon in pilot work, then preregister both before validation. Candidate endpoints are offspring survival through a fixed horizon or realized energy balance before reproduction. Report uncertainty for differences and include failures/extinctions.

**Ecological follow-up:** inherited-state policy × environmental predictability, with matched mean resource input and held-out environmental schedules. Measure the actual correlation between parental exposure and offspring selective conditions. Only add niche-construction interaction after the two-factor effect is interpretable. A shadow comparison is needed for a selection-versus-drift activity endpoint, not as a prerequisite for every ecological comparison.

### 5. Results

Populate only from accepted entries in [RESULTS.md](RESULTS.md). Report model checks and calibration separately from scientific outcomes. State nulls, harmful effects, censored recovery, and heterogeneity between independent runs.

### 6. Discussion

- Does inherited state carry useful environmental information, merely alter network gain/energy costs, or do neither?
- How sensitive are conclusions to stress threshold, decay rate, mutation operator, neural costs, population size, and environmental timescale?
- What does the model omit about molecular epigenetics and biological fitness?
- Which observations survive independent seeds and alternative implementations?
- No inference from finite topology counts to unbounded evolution, or from task performance to subjective experience.

### 7. Completion gate

The study is complete when the preregistered comparison has adequate precision for its declared effect size and a reproducible interpretation, whether positive, negative, or null. Publication is a separate assessment of evidence, novelty, and venue fit. Do not tune until a desired positive result appears.
