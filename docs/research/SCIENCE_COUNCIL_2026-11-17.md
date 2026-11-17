# Science council — 2026-11-17 UTC

Status: literature review and development decision, not a preregistration or new experiment. Three independent agent paths reviewed five primary papers each, then read one another's memos and challenged their proposals. This focused review does not establish exhaustive coverage or novelty. Access restrictions and manuscript/publication distinctions are recorded in the individual reviews.

## Evidence and separate research paths

| Path | Review | Finding for Terrarium |
|---|---|---|
| Evolved memory and costs | [Evolved-memory review](council_2026-11-17/evolved_memory.md) | Task-selected evolution demonstrates a method for acquisition; it does not predict spontaneous transfer from ecological reproduction. Structural complexity alone does not establish useful memory. |
| Assay and causal validity | [Assay review](council_2026-11-17/assay_validity.md) | Establish sensor/motor competence first; localize state interventions to the delay boundary and include matched visible controls. |
| Ecology and experiment design | [Ecology review](council_2026-11-17/ecology_and_design.md) | Manipulate predictive information and account for realized costs; resource variability alone is not evidence that memory pays. |

The completed baseline passes its population calibration criteria. In contrast, all 898 feed-forward donor controllers fail to establish visible-task competence: 0 correct, 31 wrong, 1,765 timeouts across 1,796 trials. Their delayed failures cannot diagnose memory deficits. Manufactured controllers establish instrument feasibility only. See the [actual visible-cue results](../results/VISIBLE_CUE_CONTROL_2026-11-17.md).

Primary literature motivating the recommendations includes [NEAT's task-selected control experiments](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf), [Polyworld's selection versus passive lockstep experiments](https://pmc.ncbi.nlm.nih.gov/articles/PMC2801533/), [phase-specific thalamic interventions](https://doi.org/10.1038/nn.4568), and [a mathematical model comparing learning and fixed foraging strategies](https://pmc.ncbi.nlm.nih.gov/articles/PMC10465976/). These are peer-reviewed primary papers; the foraging work is a model, not animal validation of our energy coefficients. The UAL paper discussed in the assay review is theoretical framing, not an experiment demonstrating cognition in Terrarium.

## Challenges and resolutions

1. **All-ticks reset is too broad for a retention claim.** It can disrupt response-phase recurrent motor dynamics. Retain it as a broad diagnostic, but add a single reset after the last fixed-pose delay tick and before first movement inference; allow recurrence thereafter. Apply the same boundary reset in the visible condition. Preserve weights, pose and other task state. This exact software intervention is the council's design inference, not a protocol copied from a biological paper.
2. **Rotations are a modest holdout.** Rotating egocentric geometry preserves relative turn sequences and mostly changes absolute heading. It cannot establish navigation transfer to a new ecological problem. Freeze fresh validation distributions before evaluation and replace holdouts exposed during tuning.
3. **Neural costs require distinct hypotheses.** The cited Polyworld experiments removed neuron/synapse structural costs while retaining behavior costs. They cannot validate our structural tax. Geometric wiring cost, edge count, realized neural activity, methylation and within-life plasticity are distinct. State resetting is not an energy-matched intervention.
4. **Stationarity does not settle memory demand.** A stationary but partially observed environment may contain useful history. Greater volatility is neither necessary nor sufficient. Compare predictive versus shuffled information while controlling resource exposure and competent embodied stateless policies.
5. **Acquisition timescale matters.** Genetic optimization across generations is separate from within-life learning. Neither recurrence presence nor benchmark acquisition establishes associative learning, UAL or consciousness.

## Agreed next development sequence

1. **Visible-task acquisition pilot.** Build a separate bounded offline genetic-selection runner around the existing arena. Before running, fix founder generation, genetic operators, population/generation/evaluation budgets, seed list, training fitness and training-only champion/tie rule. Save all run failures, acquisition curves and final populations. Start with the fixed pose/mapping to test acquisition feasibility. Do not select delayed performance or tune ecological mechanisms in this unit.
2. **Delayed retention gate.** After competence, freeze delayed acquisition and fresh evaluation. Report absolute success and timeouts together with intact versus single delay-end reset, matched visible reset, and all-ticks-cleared diagnostics. Retain every independent search run. Trial pairs, organisms and generations are not independent replicates. Use pilot variation and an explicit precision target for later confirmatory sample size; a convenient seed count is not a power calculation.
3. **Ecological information payoff.** Only after functional retention, design predictive versus shuffled cue conditions with matched resource exposure, existing energy accounting and independent worlds. Keep niche construction/plasticity fixed initially. A broad recurrence/cost sweep is premature.

The next implementation unit is step 1, followed by a frozen pilot protocol and runtime estimate. None of these proposals is yet an executable confirmatory design. No new sustained Pi run was started. Future Pi work retains the existing single-worker thermal guard and must be budgeted from measured pilot runtime.

## User-visible demonstration

The demo has an optional experimental recurrence mode, enabled by **Memory on** or `?seed=42&memory=1`. It starts a new world with the same seed, permits recurrent mutation in offspring, reports measured presence, and offers an inspect button with gold dotted recurrent edges. The mode uses a demo-only mutation probability of 0.3; the scientific defaults remain off. This makes the mechanism inspectable without presenting it as evidence of adaptive recall.

Local real-WASM checks at seed 42/tick 1,000 found living recurrent creature #583 and recurrent fraction 0.8661417322834646. Baseline/off stats match exactly; reset reproduces the candidate. These are engineering smoke checks, not replicated biological findings. Visual QA could not run because computer-use access was unavailable.
