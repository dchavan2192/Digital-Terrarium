# Council path: ecology, memory value and experimental design

Reviewed 2026-11-17. Independent literature review for the research council.
No code or experiment was changed. All five sources below are published,
peer-reviewed primary research; four are theoretical/computational studies,
not animal experiments. No verbatim passages are reproduced. Proposed design
choices below are our inferences, not results established in Terrarium.

## Findings that change our next step

Memory is valuable only when earlier information improves a consequential
decision beyond what current observations already provide. Environmental
variation alone is not enough: slow change can favor fixed preferences, while
unpredictable change can make history misleading. A stateless strategy can be
competitive even in a variable world. Longer memory is not automatically better.
These conclusions are supported in different model settings by
[Arehart and Adler](https://doi.org/10.1098/rspb.2023.1084),
[Robira et al.](https://doi.org/10.1098/rsos.210809), and
[Lee et al.](https://doi.org/10.1098/rspb.2024.1606).

For Terrarium, enabling recurrent mutations creates a capacity; it does not
create an ecological reason to use it. The current donor controllers were not
selected for our arbitrary cue-to-goal mapping. Their failure with the cue
visible therefore blocks interpretation of their delayed failures as a
retention deficit. This is a design inference from our own diagnostic, not an
animal-cognition finding.

Do not turn the timescale literature into a requirement to make food more
volatile. A stationary but partially observed world can still make history
useful for identifying its hidden current state. The papers below often model
learned value estimates, rather than the transient recurrent activations used
here. They do not establish that Terrarium's existing ecology has no selection
for memory; that remains an empirical question.

## Primary papers and their actual scope

| Verified publication | What was actually studied and found | Implication and boundary |
|---|---|---|
| **Arehart, E. and Adler, F. R. (2023).** *A minimal model of learning: quantifying the cost and benefit of learning in changing environments*. Proceedings of the Royal Society B **290**, 20231084. [DOI](https://doi.org/10.1098/rspb.2023.1084); [full text](https://pmc.ncbi.nlm.nih.gov/articles/PMC10465976/). Received 15 May, accepted 28 July 2023. | A mathematical forager updates estimates of two resource values with a memory parameter and an exploration parameter. The authors compare resource uptake with fixed strategies across environmental regimes. Benefits can be small, and even optimized versions of their learner can lose to a fixed strategy in some environments. | Always compare with competent fixed/stateless policies, not just a weak random controller. This is a value-estimation learning model, not evolving recurrent NEAT activation state. Its optimal coefficients are not Terrarium settings. |
| **Robira, B., Benhamou, S., Masi, S., Llaurens, V. and Riotte-Lambert, L. (2021).** *Foraging efficiency in temporally predictable environments: is a long-term temporal memory really advantageous?* Royal Society Open Science **8**, 210809. [DOI](https://doi.org/10.1098/rsos.210809); [published-version repository record, explicitly refereed](https://eprints.gla.ac.uk/252769/). Accepted 3 September, published 22 September 2021. | Simulated foragers differ in temporal cognition in renewable-resource landscapes. For everyday foraging, long-term temporal knowledge did not clearly outperform a simpler short-term estimate of current availability; anticipation was a more promising context. The model grants accurate knowledge of potential source locations/types. | A strong simple competitor matters, and current availability may make long memory redundant. This does not establish that spatial memory is useless: that ability was largely supplied rather than evolved. Publisher/PMC access was intermittent; the indexed primary article and institutional published abstract were checked. |
| **Turner, C. R., Morgan, T. J. H. and Griffiths, T. L. (2024).** *Environmental complexity and regularity shape the evolution of cognition*. Proceedings of the Royal Society B **291**, 20241524. [DOI](https://doi.org/10.1098/rspb.2024.1524); [full text](https://pmc.ncbi.nlm.nih.gov/articles/PMC11495953/). Accepted 7 August 2024. | Optimality/invasion analysis jointly models sensory reliability and decision rules. Information processing can compensate for unreliable sensing, and selection may favor biased behavior even when a reliable cue exists. The model backgrounds learning and detailed neural mechanisms. | Validate that the cue is detectable, worth acting on, and linked to payoff. More environmental complexity is not a sufficient manipulation. This is not evidence that adding recurrent edges causes cognition. |
| **Lee, E. D., Flack, J. C. and Krakauer, D. C. (2024).** *Constructing stability: optimal learning in noisy ecological niches*. Proceedings of the Royal Society B **291**, 20241606. [DOI](https://doi.org/10.1098/rspb.2024.1606); [published paper on author site](https://static1.squarespace.com/static/5f29a430a2b6a34680879cc0/t/67242aa246f20f1d35e4d238/1730423462813/lee-et-al-2024-constructing-stability-optimal-learning-in-noisy-ecological-niches-2.pdf); [bibliographic record](https://pubmed.ncbi.nlm.nih.gov/39471866/). Accepted 4 September 2024. | Analytical/numerical agents integrate noisy estimates of a switching environmental state. Optimal memory balances tracking against stale information; its duration scales sublinearly with environmental persistence in the model. Their niche-construction analysis depends on whether benefits can be monopolized or shared. | Control noise, persistence and cue usefulness separately. Keep niche construction fixed during the first memory study: it can alter the information environment itself. Their decay-weight aggregator and KL-loss model do not identify a universal RNN time constant or justify a Terrarium niche-construction benefit. |
| **Mery, F. and Kawecki, T. J. (2005).** *A cost of long-term memory in Drosophila*. Science **308**(5725), 1148. [DOI](https://doi.org/10.1126/science.1111331); [primary abstract record](https://pubmed.ncbi.nlm.nih.gov/15905396/). | An animal experiment distinguishes protein-synthesis-dependent long-term memory from anesthesia-resistant memory. Inducing long-term memory increased susceptibility to severe stress. This review verified the primary abstract and publication metadata; publisher full text returned 403. | Memory formation can have costs, but this does not calibrate our recurrent-edge price, demonstrate a cost of an activation buffer, or equate recurrent state with biological long-term memory. Do not copy a biological percentage into model parameters. |

The literature already contains extensive theory about memory timescales,
changing resources, costly information and niche feedback. Our credible future
contribution would need a narrow, controlled result in this implementation and
a careful prior-art comparison. Neither optional recurrence nor a successful
finite task supports consciousness, UAL, open-ended evolution or a broad
novelty claim.

## Recommended narrow path

**Immediate gate: establish competence before measuring retention.** Use the
already fixed visible-cue task as an acquisition/competence benchmark. Define a
separate pilot in which independently founded controller populations are
selected for the task, or define an assay that uses cues and actions already
meaningful in their ecology. Genetic optimization across generations is not
within-life learning. Do not choose a new cue channel, invert the answer map,
or extend the horizon after examining these validation donors. Declare any
new training and validation distributions first, and retain unsuccessful runs.

**First mechanistic hypothesis after competence:** a competent recurrent
controller should earn more task resource/reward with its state intact than
with its state reset, when a past cue predicts a later profitable patch and is
no longer present. This is a retention-use claim, not yet an evolutionary
fitness claim. The same genotype with reset state controls wiring and genetic
weights. It does not perfectly match realized neural activity or energy costs;
record those rather than asserting energy matching.

**Minimal ecological follow-up, only after that gate:** two otherwise identical
foraging patches have equal long-run resource input. A brief local cue gives
information about which patch will be profitable after a declared delay. Compare
a predictive cue schedule with a shuffled/uninformative schedule preserving
cue frequencies, reward frequencies and the external resource schedule. Keep
mean food, density cap, sensory range, reproduction and niche-construction
policy fixed. Charge existing movement, connection and activity costs and
measure net reserve or realized reproduction, rather than assigning a free
task score and calling it ecological fitness. This follow-up is a proposal;
the current arena does not implement that energy accounting.

The current input vector must not reveal the later answer through food rays,
latitude, heading, timing, pose or persistent environmental marks. If cues are
temporarily removed, body pose/actions should remain controlled as in the
current arena, or embodied state must be measured and treated explicitly as
an alternative memory mechanism. Vary cue predictiveness, not drought rate
alone: changing drought frequency can change food exposure and mortality as
well as predictability. Record predictiveness at actual encounters/decisions,
not just the world's autocorrelation.

## Replication, endpoints and decision rules

- **Unit:** independent evolutionary population/run. Genotypes, repeated cues,
  episodes and descendants within one run are nested observations.
- **Primary mechanistic endpoint:** within-genotype intact-minus-reset return
  over held-out task bouts, aggregated under a prespecified donor-sampling rule
  to one estimate per independent evolutionary run. Include failures/timeouts.
- **Primary ecological endpoint later:** declare either net reserve gain in a
  nonbreeding common-garden bout or realized reproductive output with an explicit
  mortality/reproduction policy; do not silently switch between them.
- **Controls:** visible cue, competent fixed policy, cue-independent policy,
  same-genotype reset, predictive versus shuffled cues, and realized energy
  ledger. Oracle knowledge is an upper bound, not a fair reflex competitor.
- **Power:** use a separate small throughput/variance pilot and a declared
  smallest effect of interest to choose precision/sample size. No paper here
  licenses an arbitrary five- or ten-seed confirmatory sample.
- **Freeze:** training/pilot and validation seeds/schedules must be disjoint.
  Pair external schedules where intended, preserve environment RNG separately,
  and record technical failures separately from extinction. No reruns because a
  result is unfavorable; no significance test on the 898 correlated donors.

## Council recommendation

Allow the requested memory-on demo as an explicitly experimental visualization.
Show recurrent-connection presence separately from any claim of useful memory.
Do not turn that convenience setting into the scientific default. The next
research development should solve the competence/selection mismatch with a
small declared protocol, not add water, parasites, plasticity and multiple new
cost factors simultaneously. A retained null or harmful result is valid;
tuning until memory wins is not a defensible stopping rule.

## Council round 2: cross-challenge and prioritized gates

I read [the evolved-memory review](evolved_memory.md) and
[the assay-validity review](assay_validity.md). I agree with their shared
competence-first decision and with keeping an isolated selection benchmark
distinct from ecological adaptation.

**Challenge one possible overreach:** rotated held-out arenas are a useful
generalization test, but rotation also changes the absolute heading input.
Success therefore requires more than the minimal cue–response rule. Failure
on those rotations must not become a verdict on retention or on the mutation
operators generally. Keep that endpoint explicitly separate from acquisition
on the original fixed task. The proposed eight-of-ten/seven-of-eight gate is
an engineering criterion, not a power-based scientific threshold. Likewise,
my two-patch ecological proposal is a later phase, not justification to skip
competence acquisition or to add environmental mechanics now.

**Cost correction:** recurrent structural maintenance, realized neural
activity, lifetime synaptic plasticity, and inherited/snapshot methylation
are distinct interventions. The first selection benchmark should keep their
settings fixed, as the evolved-memory review recommends. Clearing activations
retains the graph but can alter activity cost and downstream behavior; it is
not an energy-matched lesion. Zeroing weights is also not equivalent to
removing connections when maintenance is charged. Freeze marks for a task
comparison and report realized signaling/energy separately. None of the
biological long-term-memory findings calibrates an activation-buffer price.

The three prioritized gates are:

1. **Acquisition/competence.** A small, declared offline genetic-optimization
   pilot must acquire the fixed visible rule repeatedly. Preserve all seeds,
   failures and final populations, choose final candidates by training rules,
   and evaluate frozen policies on declared held-out conditions. Call this
   genetic optimization across generations, not learning within an organism's
   lifetime. Freeze budgets and a validation protocol after the pilot.
2. **Functional retention.** Select for the delayed task, then compare the
   same frozen controller intact, reset at the end of the immobile delay,
   and reset under a matched visible condition. Report absolute success and
   timeouts alongside lesion differences. Whole-trial clearing can damage
   dynamical motor control as well as storage; do not localize a whole-trial
   effect to memory maintenance alone.
3. **Ecological benefit.** Only after those gates, test an information-value
   manipulation with matched resource schedules and existing energy accounting.
   Use independent worlds, prespecified donor sampling and a pilot-based
   precision target. Keep niche construction and plasticity policy fixed
   initially; do not introduce a broad mechanism factorial.

No gate requires making the whole ecology nonstationary. A stationary,
partially observed world can reward memory, and embodiment can store history
outside recurrent activations. Compare with competent embodied reflex policies
rather than assuming that all history dependence is neural. Conversely,
variable food does not prove that remembered information is useful. This
distinction supports an experimental memory-on demo while leaving scientific
claims gated on the experiments above.
