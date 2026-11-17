# Council path: evolved memory, task competence and neural costs

Independent research review, 2026-11-17. Status: **proposal for council debate; no experiment launched**. Primary sources were browsed for this review. The distinction between a peer-reviewed publication, an author manuscript and a preprint is explicit below. This is a focused search, not a systematic review or a novelty determination.

## Decision for this project

Keep experimental recurrence available without making an adaptive-memory claim. The passed ten-seed, 200k-tick population baseline establishes a useful engineering operating point, not a cognitive selection pressure. The donor's 898 controllers were feed-forward by construction. Under the visible-cue condition they made zero correct choices, 31 wrong choices and 1,765 timeouts across 1,796 trials. This demonstrates poor transfer to the imposed task, not a memory deficit: cue availability did not overcome the unfamiliar sensor-to-action and goal mapping. Recurrence in the ecosystem cannot reasonably be expected to teach that arbitrary mapping automatically.

A manufactured recurrent controller and its controlled lesions establish instrument feasibility. They do not show that the evolutionary operators can discover the solution, that natural selection in the ecology rewards it, or that the neural cost coefficients are biologically calibrated. Those are separate hypotheses.

## Five checked primary papers

### 1. NEAT: task-specific hidden-state control, not spontaneous transfer

Stanley, K. O. & Miikkulainen, R. (2002). *Evolving Neural Networks through Augmenting Topologies*. **Evolutionary Computation 10(2), 99–127**, peer-reviewed journal article. [DOI](https://doi.org/10.1162/106365602320169811), [author-hosted final journal PDF](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf), [publisher distribution metadata](https://ieeexplore.ieee.org/abstract/document/6790655/authors).

The experiments selected controllers explicitly for pole balancing. Omitting velocity observations required temporal inference. The paper reports 20 simulations for the non-velocity comparison, mean 33,184 NEAT evaluations, and tests generalization over 625 initial states; a solution had to succeed in at least 200. Ablations tested historical markings, speciation and growth from minimal structure. These results support testing search operators against an explicit task, multiple independent searches and declared initial-state generalization. They do not establish ecosystem intelligence or metabolic efficiency. Reported comparison significance also assumes similar variances for competing methods; do not transplant the quoted efficiency advantage as our expected effect. Terrarium's mutation/crossover policy and ecological fitness are not a complete replication of NEAT.

### 2. Polyworld: adaptive dynamics require a demographic null

Yaeger, L. S. (2009). *How evolution guides complexity*. **HFSP Journal 3(5), 328–339**, peer-reviewed journal article, accepted 2009-08-29. [DOI](https://doi.org/10.2976/1.3233712), [full primary article](https://pmc.ncbi.nlm.nih.gov/articles/PMC2801533/).

Ten naturally selected runs were paired with ten passive lockstep runs. Lockstep replayed birth/death counts and timing while choosing parents and deaths randomly; genetic operations and sensory experiences were comparable statistically, not identical. Neural-dynamics complexity rose initially under selection, while passive complexity later exceeded it. Resource-patch occupation supplied a separate behavioural-adaptation measure. Maximizing complexity directly did not establish that behavioural adaptation.

Crucial limitation for our cost story: these reported experiments **removed per-neuron and per-synapse costs**, retaining behaviour costs. Population-dependent energy costs also regulated numbers. The study therefore does not validate Terrarium's neuron/connection tax or its food-limited population dynamics. Use a shadow for selection-versus-drift complexity claims, not as a substitute for a within-controller memory intervention. The later arXiv posting of the related 2008 conference work is not a new 2011 experiment; this review relies on the verified journal article instead.

### 3. Representation: distinguish a measured information variable from cognition

Marstaller, L., Hintze, A. & Adami, C. (2013). *The Evolution of Representation in Simple Cognitive Networks*. **Neural Computation 25(8), 2079–2107**, peer-reviewed journal article. [DOI](https://doi.org/10.1162/NECO_a_00475), [publication record and abstract](https://pubmed.ncbi.nlm.nih.gov/23663146/), [authors' earlier manuscript](https://arxiv.org/html/1206.5771v1).

The published abstract describes evolving artificial neural networks and hidden-Markov-gate networks on categorization, quantifying environmental representation with an information-theoretic variable R. This measures a declared relationship between internal and environmental variables, rather than counting neurons. The earlier manuscript shows architecture-dependent starting distributions and warns that adaptation speed across differently encoded architectures is not directly comparable. The author-hosted final PDF returned HTTP 403 in this review; detailed manuscript results must not be silently attributed to an identical final text. Publication status and the broad method/results are independently verified by the journal-indexed abstract. For Terrarium, information in a state is a diagnostic; causal task dependence still needs intervention and behavioural success. No claim of consciousness follows from R.

### 4. Animats: task and sensor/motor capacities jointly determine memory demand

Albantakis, L., Hintze, A., Koch, C., Adami, C. & Tononi, G. (2014). *Evolution of Integrated Causal Structures in Animats Exposed to Environments of Increasing Complexity*. **PLOS Computational Biology 10(12), e1003966**, explicitly peer-reviewed research article, published 2014-12-18. [DOI and full article](https://doi.org/10.1371/journal.pcbi.1003966), [publisher text](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1003966).

Small binary hidden-Markov-gate controllers evolved on catching/avoiding falling blocks. Task conditions altered required temporal discrimination; extra conditions restricted sensors or motors. Each condition used 50 independent evolutionary histories, with 128 evaluation trials and 60,000 generations. Under the model's limited hidden-element capacity, tasks demanding longer sensor histories favoured more integrated architectures and higher integration measures. The paper itself notes that evolutionary difficulty is not straightforwardly predicted from task labels because agents can move and sample the environment. This supports separate sensor/motor competence checks and prevention of external-memory shortcuts. These are task-selected discrete networks, not open-world reproduction, Terrarium controllers or empirical demonstrations of consciousness. Their integration metric is not our recurrent-edge count, entropy or topology activity, and is not evidence that bigger brains must be fitter.

### 5. Connection costs: a specific selection mechanism, not an intelligence meter

Clune, J., Mouret, J.-B. & Lipson, H. (2013). *The evolutionary origins of modularity*. **Proceedings of the Royal Society B 280(1755), 20122863**, peer-reviewed research article, accepted 2013-01-09 and published online 2013-01-30. [DOI](https://doi.org/10.1098/rspb.2012.2863), [author-hosted published PDF with supplement](https://jeffclune.com/publications/2013-CluneEtAl-EvolutionaryOriginsModularity-RoyalSociety.pdf), [publication record](https://pubmed.ncbi.nlm.nih.gov/23363632/).

Computational evolution compared performance alone against performance plus connection-cost selection on pattern-recognition/logic tasks, then assessed adaptation to changed problems. The default cost was optimized summed squared connection length; a connection-count treatment gave qualitatively similar results. Cost-based selection promoted modularity and subsequent evolvability in those task families. The cost objective operated probabilistically in a multi-objective algorithm, not as food-reserve depletion. This supports explicitly separating performance, wiring and realized activity costs in a future experiment. It does not show that any fixed per-edge charge universally improves intelligence, or calibrate Terrarium's coefficients. Cost-free versus costly worlds may alter population size and search opportunity as well as controller design; those pathways must be measured rather than explained away as direct intelligence costs.

## What these papers collectively permit us to infer

These are my inferences, not claims made collectively by the papers:

- Hidden-state tasks can select recurrent solutions when their payoff is explicit and the search can acquire the relevant sensorimotor mapping. Mere ecological variability does not demonstrate that remembering helps.
- Recurrence, functional memory, lifetime learning and adaptive ecological benefit are four different measurements. Our engine currently provides the first and instrument-level examples of the second; the current donor-transfer results establish neither the third nor the fourth.
- A structural cost needs its own intervention. A rise in neural complexity without structural cost, or a result from geometric wiring minimization, is not evidence that our additive reserve tax is correct.
- A generic complexity target is a poor immediate objective. It risks optimizing a proxy without improving the behaviour we care about.
- The five papers establish substantial prior art. Combining recurrence and neural costs with a simulated ecology does not, by itself, establish novelty.

## One narrow proposed follow-up: can task selection discover usable memory?

**Question:** after a shared visible-cue motor-competence stage, does task-directed evolution acquire a delayed-choice policy whose performance causally depends on neural state? This is a bounded engineering/neuroevolution pilot, not an ecological selection study and not yet executable under the existing reproduction-only runner.

1. Implement a separate isolated task-selection runner; do not modify ecological reproduction, public defaults or the ongoing baseline. Use the saved arena geometry and rewards. Fix founder generation, mutation operators, per-generation evaluation budget, population size, generation horizon and seed lists before the pilot. Cost is **not** manipulated in this first study.
2. Use paired branches from each independently generated founder population. Both receive the same fixed visible-cue training budget; one branch allows recurrent mutation, the other remains feed-forward. Then both receive the same fixed hidden-cue training budget. Retain and report all branches, including failed visible-cue learners. No early stopping or extending budgets until success.
3. Define the final-controller sampling rule beforehand: one training-score champion per run, with a deterministic genotype/ID tie-break. This is a declared selection-study endpoint, unlike our all-record transfer report; never retrospectively present selected champions as the whole ecological population. Archive the full final population too.
4. Evaluate the selected controller on a frozen balanced trial set in four conditions: visible/intact, visible/state-cleared, hidden/intact, hidden/state-cleared. All task state resets per trial. Keep the same genome and snapshot weights for state clearing. Include the manufactured recurrent and feed-forward references, and cue-label-permutation checks. No evaluation result feeds back into selection.
5. **Primary endpoint:** per independent evolutionary run, hidden mean-reward difference between intact and state-cleared evaluation, then the mean of those differences across runs. The feed-forward branch supplies a zero-dependence control. Report visible-cue competence, hidden absolute reward, timeouts, structural/activity diagnostics and selection failures alongside this endpoint; a lesion difference alone is not sufficient when intact performance is poor.
6. **Independent unit:** the founder-population/search run. Paired branches, two cues, agents and generations are not independent replicates. Run a small fixed-budget pilot to estimate failure rates, runtime and run-level variation; choose confirmatory sample size from a stated precision target before fresh seeds. A convenient count such as ten or twenty is not a power calculation. Do not discard failed runs to form a competence-conditioned success cohort.

The strongest alternative explanation is general damage to dynamical motor control rather than loss of the stored cue: visible/intact versus visible/cleared is therefore essential. State clearing throughout the trial still cannot localize necessity to the delay specifically; a later, separately validated delay-only intervention would be needed for that narrower causal claim. Another limitation is evolution learning a fixed task rather than useful ecological memory. If this pilot cannot acquire visible-cue competence, stop at that diagnosis rather than adding recurrent mechanisms, costs or ecology until a favourable result appears.

## Immediate recommendation and council questions

Do not launch a large recurrence/cost ecology sweep next. First decide whether task-directed evolution is the project's intended intermediate objective, or whether a new ecological payoff for remembered information is preferred. The first is a cleaner and cheaper proof that the evolutionary search can acquire the behaviour; the second is a larger scientific design with energetic and demographic confounds. Neither has an experiment-ready frozen protocol yet.

Questions for peer review: Is the shared competence stage an acceptable scope for this project? Does the proposed run-level endpoint separate stored-cue use from general dynamical motor damage adequately? Should the fixed task stay an engineering benchmark, reserving novelty claims for an eventual ecological information-benefit interaction? No answer here warrants an adaptive-memory claim for the demo's optional recurrence setting.

## Round 2: cross-challenge and constrained agreement

Read [assay_validity.md](assay_validity.md) and [ecology_and_design.md](ecology_and_design.md). I endorse their competence-first recommendation and qualify my earlier primary endpoint: clearing state on every tick tests dependence on ongoing recurrent computation, not cue retention alone. It can disrupt movement/readout dynamics even when the initial cue is irrelevant. A visible-cue lesion control is necessary but does not fully remove that confound in the hidden task.

Before a retention-specific claim, add a **single delay-end reset**: after the final immobile blank tick and before the first movement inference, set previous activations to zero once, preserving pose, weights, methylation and all other task state. Then allow recurrence normally throughout movement. Apply the identical reset at the equivalent instant in the matched visible-cue condition. Keep the all-ticks-cleared condition as a broader dynamical-dependence diagnostic. The primary retention-use contrast should be intact versus this one-time reset in the hidden condition, accompanied by absolute intact competence and the corresponding visible-reset contrast. This still diagnoses use of earlier task information, not a unique biological storage mechanism. Phase-specific perturbation is supported methodologically by the [Bolkan study discussed in the assay memo](https://doi.org/10.1038/nn.4568); the exact software intervention is our design inference.

One qualification to the proposed held-out rotations: rigidly rotating the entire arrangement preserves the relative visual geometry and required turn sequence. With egocentric rays, it mostly tests robustness to the changed absolute-heading scalar; it is not transfer to a new relative navigation problem or ecological landscape. It is nevertheless a valid, modest nuisance/generalization check if rotation, heading normalization and the left/right goal labels are fixed beforehand. Do not retune on those results or count eight trials as eight independent learners.

Task-directed genetic optimization is **evolutionary acquisition across generations**. Calling it “learning” without that timescale would invite confusion with within-life weight plasticity or acquisition of a new association. NEAT's task-selected successes justify testing our optimizer, not assuming spontaneous ecological transfer or equating retained activation with associative learning.

I endorse these next three steps only:

1. Freeze a small visible-cue acquisition pilot, training/test distributions, run-level sampling and failure reporting. Keep the original 898-genome transfer result intact.
2. After competence, freeze delayed-task acquisition plus intact, all-ticks-cleared and delay-end-reset evaluations, with matched visible conditions and prespecified delays. Cue-label permutation is a fixed negative control, never permission to flip the scoring sign. Retain every search run.
3. Only after that causal gate, design a predictive-versus-shuffled ecological payoff study with equal resource exposure, competent stateless competitors and realized energy accounting. Structural/activity-cost manipulations belong to a separate declared hypothesis, not a rescue knob for failed transfer.
