# Research Bibliography — Terrarium

Papers and sources informing design decisions, organized by system.
Each entry includes how it influenced the implementation.

## Core Evolutionary Mechanics

### Kin Selection & Cooperation
- **Hamilton, W.D. (1964)** "The Genetical Evolution of Social Behaviour" *J. Theoretical Biology* 7(1):1-16
  - The foundational rule: rB > C. Our benefit_multiplier=1.5 satisfies this with spatial relatedness r~0.67.
  - **Confirmed in Terrarium**: 87% cooperation rate at share_amount=5.

- **Trivers, R.L. (1971)** "The Evolution of Reciprocal Altruism" *Quarterly Review of Biology* 46(1):35-57
  - Reciprocal altruism without kin recognition. Our signal channel could enable this.

### Speciation
- **Stanley, K.O. & Miikkulainen, R. (2002)** "Evolving Neural Networks through Augmenting Topologies" *Evolutionary Computation* 10(2):99-127
  - NEAT: our brain architecture. Compatibility distance for species clustering.
  - Key insight: original NEAT has no deletion mutations — bloat controlled by speciation + fitness sharing.

- **Anderson, D. & Pennock, R. (2014)** "Ecological speciation in digital organisms" *American Naturalist* 183(3):405-419
  - Proved ecological speciation (divergent resources) produces stronger isolation than drift-based.
  - Depletable, non-fungible resources are the minimum for stable coexistence.
  - High niche dimensionality increases speciation strength monotonically.

- **Schluter, D. (2000)** *The Ecology of Adaptive Radiation* Oxford UP
  - Character displacement requires d/w ratio > 1.5 for stability.
  - Our two food types (algae/fruit) implement this.

- **Grant, P.R. & Grant, B.R. (2006)** "Evolution of Character Displacement in Darwin's Finches" *Science* 313(5784):224-226
  - Sympatric character ratios 1.29-1.54x. Food type energy difference must be >2x for specialization.
  - Trade-off cost of 25-30% needed for stable displacement.

## Neural Architecture & Brain Cost

### Metabolic Cost of Neural Information
- **Attwell, D. & Laughlin, S.B. (2001)** "An Energy Budget for Signaling in the Grey Matter" *J. Cerebral Blood Flow & Metabolism* 21(10):1133-1145
  - **Synapses cost 50-85x more than cell bodies.** ~85% of signaling energy goes to postsynaptic processes.
  - Directly informed our switch from per-neuron to per-connection brain cost.

- **Laughlin, S.B. et al. (1998)** "The Metabolic Cost of Neural Information" *Nature Neuroscience* 1(1):36-41
  - Neural information processing has measurable metabolic overhead.
  - Each synapse consumes ~7,000 glucose molecules/second.

### NEAT Complexity Control
- **Dolson, D. & Park, E. (2012)** "Applying Neural Pruning to NEAT" Swarthmore College
  - Prune rate must be 1.5-2x add rate for net-zero complexity drift without crossover.
  - Informed our ratio change: 3%/2% → 2%/4% (add/prune).

- **Green, C. (2004+)** SharpNEAT Phased Searching
  - Alternating complexification/simplification phases. Track Mean Population Complexity.
  - When MPC exceeds baseline+threshold, disable adds and increase pruning.

- **Hadjiivanov, A. & Blair, A. (2020)** "Complexity-based speciation" arXiv:2010.05176
  - Species formed by grouping organisms with same hidden node count, not compatibility distance.

### Synaptic Pruning Biology
- **PMC3704263** "The metabolic cost of maintaining a synapse during development"
  - Resting synapses are not free: vesicle V-ATPase consumes ~half of resting ATP.

- **PMC8584672** "Information theory of developmental pruning" *PLoS Comp Bio*
  - Activity-dependent pruning: remove synapses where |weight × activation| stays below threshold.
  - Superior to random pruning — produces compact, functional networks.

## Environmental Systems

### Temperature
- **Bergmann, C. (1847)** "Über die Verhältnisse der Wärmeökonomie der Thiere zu ihrer Größe"
  - Larger body mass in cold climates for heat retention. Temperature must compound resource scarcity.

- **Allen, J.A. (1877)** "The influence of Physical Conditions in the Genesis of Species"
  - Appendage size varies with temperature. Body size + temperature create linked trade-offs.

- **Sinclair, B.J. et al. (2016)** "Can we predict ectotherm responses using thermal performance curves?" *Ecology Letters*
  - Gaussian TPC model. Sigma of 5-8 units. 10°C deviation = 60-80% performance loss for specialists.

### Food Types & Adaptive Radiation
- **Wickman, J. et al. (2019)** *Ecology Letters*
  - Trade-off shape matters: intermediate strength needed. Too weak = generalists dominate.

- **PMC4067365** "Trade-offs drive resource specialization and establishment of ecotypes"
  - When resource influx is high, generalists dominate. Specialization requires scarcity.

### Pheromone Trails
- **Wilson, E.O. (1975)** *Sociobiology: The New Synthesis* Harvard UP
  - Chemical communication as the basis for eusocial organization.

- **Deneubourg, J.L. et al. (1990)** *J. Insect Behavior* 3(2):159-168
  - Double-bridge experiment: pheromone trail half-life should be ~0.5-2x foraging trip duration.
  - Optimal decay: 0.003-0.007/tick for ALife timescales (we use 0.005 = 0.995 retention).

- **arXiv:0811.0080** "A Deterministic Model for Pheromone Deposition"
  - Deposit η=0.07, evaporation k=0.015 produced coherent stigmergic behavior.

### Day/Night Cycle
- **Kronfeld-Schor, N. & Dayan, T. (2003)** "Partitioning of Time as an Ecological Resource" *Annual Review of Ecology* 34:153-181
  - Temporal niche partitioning requires cycle:lifespan ratio of 1:50 to 1:150.
  - Arrhythmic organisms cannot coexist via temporal separation.
  - Informed our period change: 1000 → 40 ticks (ratio 1:62 at max_age=2500).

### Elevation & Geographic Isolation
- **MacArthur, R.H. & Wilson, E.O. (1967)** *The Theory of Island Biogeography* Princeton UP
  - Isolated patches drive allopatric speciation. Barrier resistance 2-20x = soft barriers.

- **Wright, S. (1943)** "Isolation by Distance" *Genetics* 28(2):114-138
  - Gene flow decreases with geographic distance. Elevation creates effective distance.

- **McRae, B.H. (2006)** "Isolation by resistance" *Evolution* 60(8):1551-1561
  - Resistance ratios: 2-5x = soft barriers (gene flow slowed), >50x = hard barriers.

### Droughts & Disturbance
- **Holling, C.S. (1973)** "Resilience and Stability of Ecological Systems" *Annual Review of Ecology and Systematics* 4:1-23
  - Resilience vs stability distinction. Ecosystems can be resilient to disturbance without being stable.

- **Connell, J.H. (1978)** "Diversity in Tropical Rain Forests and Coral Reefs" *Science* 199(4335):1302-1310
  - Intermediate disturbance hypothesis. Moderate disturbance maximizes species diversity.

- **Furness, E.N. et al. (2021)** "Evolutionary simulations clarify biodiversity-disturbance models" *Proc. Royal Society B* 288:20210240
  - IDH holds conditionally. Small-scale disturbance supports biodiversity better than large-scale.

## Primordial Earth & Cambrian Explosion

- **Sperling, E.A. & Stockey, R.G. (2018)** *Integrative and Comparative Biology*
  - Oxygen increase was episodic pulses, not gradual. Each pulse triggered diversification burst.

- **Stanford/Southampton (2024)** "Small change in Earth's oxygen levels sparked evolutionary leap"
  - Only a small incremental oxygen increase was needed to trigger the Cambrian explosion.

- **arXiv:2212.00899** "On the evolutionary emergence of predation" (2022)
  - Predation as ecological catalyst: first predators created selection pressure across entire biome.

- **Olson, R.S. et al. (2016)** "Exploring the coevolution of predator and prey morphology and behavior" MSU/Adami Lab
  - Neural-network-brained agents: co-evolutionary arms races genuinely escalated complexity.

- **Nolfi, S. & Floreano, D. (1998)** "Co-evolving predator and prey robots"
  - Stable co-escalation emerged when populations maintained sufficient diversity.

## ALife Platforms

- **Ray, T. (1996)** Tierra: Self-replicating programs. Parasites doubled species richness (8→16 species).
- **Ofria, C. & Wilke, C. (2004)** Avida: Depletable resources → stable coexistence via frequency-dependent selection.
- **Yaeger, L. (1994)** Polyworld: Heterogeneous neural architectures + spatial barriers → behavioral diversity.
- **Heinemann, C. (2024)** ALIEN2: Particle-based physics, GPU-accelerated.
- **Lu et al. (2024)** JaxLife: JAX-accelerated, demonstrated rudimentary agriculture and tool use.

## Major Evolutionary Transitions

- **Maynard Smith, J. & Szathmáry, E. (1995)** *The Major Transitions in Evolution* Oxford UP
  - Framework: each transition creates new replicating units at a higher level.
  - Most tractable for Terrarium: solitary → colonial (builds on existing kin cooperation).

## Open-Ended Evolution (OEE) & Activity Metrics

- **Bedau, M.A. & Packard, N.H. (1991)** "Measurement of Evolutionary Activity, Teleology, and Life" *ALife II* pp.431-461
  - Defined evolutionary activity statistics: cumulative activity D(t) as OEE indicator.
  - If D(t) grows unboundedly, evolution is open-ended. Plateau = stagnation.

- **Bedau, M.A. et al. (2000)** "Open Problems in Artificial Life" *Artificial Life* 6(4):363-376
  - Defined the OEE research agenda: the 14 open problems. #1: Generate open-ended evolution.

- **Taylor, T. et al. (2016)** "Open-Ended Evolution: Perspectives from the OEE Workshop"
  - No existing embodied ALife system has achieved sustained complexity growth.
  - Requirements: unbounded novelty generation, persistent innovations, ongoing complexity increase.

- **Dolson, E. et al. (2019)** "The MODES Toolbox" *Artificial Life* 25(1):50-73
  - Four metrics: change, novelty, ecology, complexity. Multi-task environments produce indefinitely increasing complexity.
  - First standardized framework for measuring OEE — our target instrumentation.

- **Channon, A. & Bedau, M.A. (2024)** Editorial, *Artificial Life* special issue on OEE
  - Calls for system-specific OEE analysis beyond generic EAS. Rich eco-evo systems are ideal candidates.

## Niche Construction

- **Odling-Smee, F.J., Laland, K.N. & Feldman, M.W. (2003)** *Niche Construction: The Neglected Process in Evolution* Princeton UP
  - Organisms don't just adapt TO environments — they modify them, creating feedback loops.
  - Ecological inheritance: environmental modifications persist across generations.

- **Laland, K.N., Odling-Smee, F.J. & Feldman, M.W. (1999)** "Evolutionary consequences of niche construction" *PNAS* 96:10242-10247
  - Niche construction can drive fixation of otherwise-neutral or deleterious alleles.
  - Creates positive feedback loops that accelerate or redirect evolution.

## Epigenetic Inheritance

- **Jablonka, E. & Lamb, M.J. (2005)** *Evolution in Four Dimensions* MIT Press
  - Four inheritance systems: genetic, epigenetic, behavioral, symbolic.
  - Epigenetic inheritance enables rapid phenotypic response to environment without genome change.

- **Heard, E. & Martienssen, R.A. (2014)** "Transgenerational Epigenetic Inheritance: Myths and Mechanisms" *Cell* 157(1):95-109
  - Methylation marks can persist 3-14+ generations in plants, 2-3 in mammals.
  - Stress-responsive methylation: drought, starvation, temperature shock all trigger heritable marks.

- **Sousa, A. & Costa, E. (2010)** "EpiAL: An Epigenetic Approach for an Artificial Life Model"
  - Only prior ALife work on epigenetics — simple model, no neural brains or ecology.
  - Terrarium's implementation would be the first in an embodied, ecologically rich system.

- **Day, T. & Bonduriansky, R. (2011)** "A unified approach to evolutionary consequences of genetic and nongenetic inheritance" *Am Nat* 178(2):E18-E36
  - Nongenetic inheritance can maintain maladaptive traits or accelerate adaptation depending on context.

## Sexual Reproduction & Recombination

- **Maynard Smith, J. (1978)** *The Evolution of Sex* Cambridge UP
  - The twofold cost of sex: why produce males? Recombination must provide >2x advantage.

- **Muller, H.J. (1932)** "Some Genetic Aspects of Sex" *Am Nat* 66:118-138
  - Muller's ratchet: asexual populations irreversibly accumulate deleterious mutations.
  - Sexual recombination purges harmful mutations by combining good alleles from both parents.

- **Weismann, A. (1889)** "The significance of sexual reproduction in the theory of natural selection"
  - Recombination generates variation faster than mutation alone — the original argument for sex.

## Horizontal Gene Transfer

- **Vogan, A.A. & Higgs, P.G. (2011)** "The advantages and disadvantages of horizontal gene transfer" *Biology Direct* 6:1
  - HGT accelerates innovation but can spread selfish genetic elements.

- **Goldenfeld, N. & Woese, C.R. (2007)** "Biology's next revolution" *Nature* 445:369
  - Early life was communal — HGT was the dominant mode of evolution before vertical inheritance.

## Parameter Guidance Summary

| Parameter | Value | Source |
|---|---|---|
| Brain cost: connection | 0.001/tick | Attwell & Laughlin 2001 |
| Brain cost: neuron | 0.001/tick | Attwell & Laughlin 2001 |
| Add connection rate | 2% | Dolson & Park 2012 |
| Prune connection rate | 4% | Dolson & Park 2012 |
| Sharing multiplier | 1.5x | Hamilton 1964 |
| Day/night period | 40 ticks | Kronfeld-Schor & Dayan 2003 |
| Pheromone decay | 0.995/tick | Deneubourg 1990 |
| Food type energy ratio | >2x | Grant & Grant 2006 |
| Diet trade-off penalty | 25-30% | Wickman 2019 |
| Elevation slope factor | 2-5x | McRae 2006 |
| Drought frequency | intermediate | Connell 1978, Furness 2021 |
