# Research council: behavioural validity of the memory assay

Independent path: acquisition, retention and response competence. Written
2026-11-17 from primary-source browsing and the project's archived diagnostics.
This memo recommends a development experiment; no training, simulation or new
donor evaluation was run for it.

## Recommendation

Keep the present delayed-choice arena as a checked instrument. Do not interpret
its untrained ecological donors as a memory-capacity test yet. Establish task
acquisition on the fixed visible-cue version, evaluate frozen policies on
prespecified held-out poses, and only then develop a separately specified
retention experiment. Archive every independent training run. Preserve the
original cue mapping and report failures; do not flip labels, change the horizon
until donors succeed, or publish a winner selected using test scores.

This is an inference from task-design research, not a protocol copied verbatim
from an animal study. Training would establish competence in an artificial task,
not ecological adaptation or learning within an organism's simulated lifetime.

## What our existing evidence says

The same 898 feed-forward snapshot genomes from one seed-100 world received
the same fixed two-goal task. Each condition contains 1,796 balanced cue trials:

| Condition | Correct first contact | Wrong first contact | Timeout |
|---|---:|---:|---:|
| Cue absent during movement | 7 | 7 | 1,782 |
| Cue visible during movement | 0 | 31 | 1,765 |

All contacts were with the left marker. All 31 visible-condition contacts
occurred under the negative cue, whose prespecified correct marker is right.
Every within-condition intact/state-cleared trial record matched exactly, as
expected without recurrent edges. The default delayed traces matched the prior
saved implementation. The manufactured recurrent, feed-forward and state-cleared
controls scored 1/.5/.5 when delayed and 1/1/1 when visible. Inputs, poses,
metadata and artifact hashes were independently audited; see
[the paired diagnostic](../../results/visible_cue_summary_2026-11-17.json).

The manufactured result establishes implementation feasibility. It does not
establish that ecological genotypes acquired the arbitrary cue-to-goal rule or
can use the artificial visual/motor interface effectively. Approximately
99.22% of delayed trials and 98.27% of visible trials timed out. Thus the current
observations are dominated by poor task engagement/transfer. The visible
intervention changes these controllers' behaviour in this arena, but the reward
contrast does not establish a general cost of information or a memory deficit.
The 898 genomes and 1,796 trials are correlated observations from one donor,
not independent evolutionary replications. No significance test is warranted
for their deterministic paired diagnostic.

## Verified primary literature and the methodological inference

All five works below are peer-reviewed journal publications. The first four
report original animal experiments; the fifth is the authors' theoretical
framework, not an empirical demonstration of consciousness. No preprint is used
as evidence. Bibliographic metadata/DOIs were checked against publisher or
PubMed records. No verbatim passages are reproduced.

1. **Morris, R. G. M., Garrud, P., Rawlins, J. N. P. & O'Keefe, J. (1982).
   Place navigation impaired in rats with hippocampal lesions. Nature 297,
   681–683.** DOI: [10.1038/297681a0](https://doi.org/10.1038/297681a0).
   [Primary publisher record/abstract](https://www.nature.com/articles/297681a0);
   [verified bibliographic record](https://pubmed.ncbi.nlm.nih.gov/7088155/).
   The original hidden-goal procedure separated navigational impairment from
   correlated motor, motivational and reinforcement components. **Inference for
   Terrarium:** a navigation score conflates several capacities unless a
   non-memory condition establishes usable perception, movement and goal-directed
   responding. Our visible-cue test is informative, but its failure does not
   identify which of those components is missing. Access limit: the full Nature
   page redirected to an inaccessible identity endpoint; this memo relies only
   on the verified primary abstract, not uninspected procedural details.

2. **Miller, E. K., Erickson, C. A. & Desimone, R. (1996). Neural mechanisms
   of visual working memory in prefrontal cortex of the macaque. Journal of
   Neuroscience 16(16), 5154–5167.** DOI:
   [10.1523/JNEUROSCI.16-16-05154.1996](https://doi.org/10.1523/JNEUROSCI.16-16-05154.1996).
   [Author-hosted full article](https://bpb-us-e1.wpmucdn.com/sites.mit.edu/dist/7/1739/files/2025/06/Miller-et-al-1996.pdf).
   Macaques performed delayed matching to sample with intervening test stimuli;
   the experiment distinguished stimulus responses, sample-selective delay
   activity and match-related responses, including a repetition control.
   **Inference:** encoding, retention and choice/readout must be distinguished.
   A recurrent edge or persisting activation alone is not a behavioural
   demonstration that a learned rule uses remembered information. Our sign latch
   is an engineering positive control, not an analogue of all these capacities.

3. **Bolkan, S. S. et al. (2017). Thalamic projections sustain prefrontal
   activity during working memory maintenance. Nature Neuroscience 20(7),
   987–996.** DOI: [10.1038/nn.4568](https://doi.org/10.1038/nn.4568).
   [Primary abstract and figure descriptions](https://pubmed.ncbi.nlm.nih.gov/28481349/).
   In a mouse delayed nonmatching-to-sample T-maze, pathway/phase-specific
   perturbations distinguished delay maintenance from subsequent choice;
   performance was also examined at different delays. **Inference:** after
   competence exists, manipulate storage during the immobile delay separately
   from response execution and measure a delay function. Clearing state on every
   tick changes both phases; a delay-end reset plus matched visible condition
   would better localize a retention dependency. PubMed lists a 2018 publisher
   correction, DOI [10.1038/s41593-018-0132-2](https://doi.org/10.1038/s41593-018-0132-2).
   The full PMC page was access-blocked, so no uninspected training criterion or
   numerical effect estimate is asserted here.

4. **Dudchenko, P. A., Wood, E. R. & Eichenbaum, H. (2000). Neurotoxic
   hippocampal lesions have no effect on odor span… [title abbreviated].
   Journal of Neuroscience 20(8), 2964–2977.**
   DOI: [10.1523/JNEUROSCI.20-08-02964.2000](https://doi.org/10.1523/JNEUROSCI.20-08-02964.2000).
   [Full original article](https://people.uncw.edu/galizio/website/Articles/Dudchenko.pdf);
   [bibliographic record](https://pubmed.ncbi.nlm.nih.gov/10751449/).
   Rats were trained to explicit performance criteria and tested with probes
   against reward scent/cup marking. Lesion outcomes differed across odor,
   spatial and delay manipulations; some apparent spatial impairment also
   implicated response inhibition. **Inference:** establish the rule before
   attributing poor delayed performance to retention, and test alternative
   information channels. Task-specific deficits do not license a broad verdict
   about an agent's memory generally.

5. **Birch, J., Ginsburg, S. & Jablonka, E. (2020). Unlimited Associative
   Learning and the origins of consciousness: a primer and some predictions.
   Biology & Philosophy 35, 56.** DOI:
   [10.1007/s10539-020-09772-0](https://doi.org/10.1007/s10539-020-09772-0).
   [Full theoretical article](https://pmc.ncbi.nlm.nih.gov/articles/PMC7116763/).
   UAL is proposed as a cluster involving novel compound associations,
   second-order and trace conditioning, and flexible revaluation, with explicit
   empirical predictions. **Boundary:** a fixed-weight manufactured latch, a
   two-cue motor rule or even task-trained recurrent retention is not this
   package. The proposed relation to consciousness is a theoretical hypothesis,
   not a validated consciousness test for software.

## One competence-establishing development method

Use **fixed-mapping visible-cue acquisition followed by frozen held-out probes**.
Keep the current goal geometry, ray conventions, horizon20, scalar constants,
held cue/blank phases and first-contact terminal reward. Positive cue continues
to mean left, negative cue right. Use the signal channel already declared.
During acquisition the cue is available during movement; all other sensor
functions are unchanged. Pose-dependent values may naturally diverge after
actions differ.

Train new task-specific policies, rather than silently editing the ecological
898-genome sample. A small offline evolutionary optimizer over fixed-topology
weights is sufficient for this development step; it need not alter sim-core or
claim lifetime learning. Record the optimizer, initialization, mutation rules,
fitness and fixed evaluation budget before running it. If terminal reward is
too sparse, a prespecified terminal distance term may be used for *training
fitness only*: it is never supplied as an input, never alters the terminal
evaluation score, and must be disclosed. Choose that rule in development pilots,
then freeze it before the validation runs; do not revise it after held-out tests.

Proposed validation specification, to be frozen after a budget pilot:

- Ten independent optimizer seeds per architecture; identical visible-task
  budgets for feed-forward and recurrent policies. A proposal of ten is an
  engineering starting point, not a power calculation. Record every seed,
  unsuccessful run and final policy. Choose each run's final candidate using a
  fixed training-fitness/tie rule; never select the best test-performing seed.
- Training poses: rigidly rotate the entire release/goal arrangement by
  0, pi/2, pi and 3pi/2. Held-out poses: pi/4, 3pi/4, 5pi/4 and 7pi/4. Both cue
  signs occur at each pose. This preserves relative geometry and rule while
  testing new absolute headings. It is a restricted generalization check, not
  evidence of transfer to the ecological world.
- Freeze weights and snapshot marks before testing. Holdout evaluation uses
  only original terminal reward. Primary competence endpoint per optimizer run:
  fraction of all eight held-out trials reaching the correct first goal within
  horizon. Report wrong contacts, timeouts and each cue's accuracy separately.
  Do not report only accuracy conditional on making contact.
- Proposed development gate: at least eight of ten runs in *each* architecture
  achieve at least seven correct held-out trials out of eight. These thresholds
  are stated engineering choices. If the gate fails, report a failed competence
  development attempt and do not reinterpret it as retention failure. Do not drop
  failed runs or replace their seeds. Any revised curriculum requires a new
  development record and fresh held-out validation.
- The independent unit for optimizer repeatability is the training seed/run;
  poses and trials are repeated measurements within that run. These units are
  not ecological evolutionary replicates. A later ecological claim needs
  independently evolved worlds and a prespecified sampling policy within them.

Passing visible acquisition would establish only that the trained policies can
execute the cue-conditioned response using these sensors and motors. It would
not show that the original ecological donors possessed this competence, or that
recurrence helps. A subsequent delayed-task acquisition stage must be planned
separately; there is no reason to expect training solely with a visible cue to
select long retention. After that stage, freeze the same policy and compare
intact state, delay-end state reset and matched visible probes at prespecified
trained and unseen delays. Include all validation runs in the outcome analysis,
not a post-hoc subset that happened to be competent.

## Claims that remain unready

- **Adaptive memory in the ecology:** requires a controlled ecological benefit
  or a task shown relevant to that ecology, beyond an artificial training score.
- **General working-memory capacity:** one fixed sensory/motor task is too narrow;
  include transfer across cues, layouts and delays without readout retuning.
- **Associative learning within a lifetime:** frozen inherited weights and a
  retained activation do not demonstrate acquisition or revaluation of an
  association during that organism's life. Offline weight selection is a
  different timescale and must be named as such.
- **UAL or consciousness:** the present evidence tests neither the proposed
  package nor the theory's empirical predictions. Avoid converting a roadmap
  checklist into a diagnostic claim.

## Cross-challenge questions for the council

1. What exact experience could have taught the ecological donor our fixed
   cue-to-left/right convention? If none, why would its test score diagnose
   retention rather than absence of task acquisition?
2. Will a proposed memory perturbation impair movement/readout as well as storage?
   What visible and phase-specific controls would distinguish these effects?
3. Can the protocol fail honestly, with all seeds and timeouts retained, or will
   geometry, mapping, horizon or tested genotypes be changed until it succeeds?
4. What links a positive artificial-task result to selection in the food ecology,
   and what new independent units would be needed for that claim?

## Council round 2: qualification and the next development unit

I read [evolved_memory.md](evolved_memory.md) and
[ecology_and_design.md](ecology_and_design.md). We agree that recurrence is a
capacity, visible-task failure blocks a retention diagnosis, and an ecological
memory/cost sweep is premature. I support the ecology path's predictive-versus-
shuffled information comparison later; equal mean resource input alone would
not establish matched sensory opportunity or demographic/search conditions.

**Qualification of my rotation proposal:** rigidly rotating the entire arena
preserves egocentric ray geometry and the required turn sequence. The reference
latch ignores absolute heading, so this is neither impossible nor a demanding
new task for it. For an exported controller it mainly probes robustness to the
absolute-heading input. It is a narrow nuisance-input check, not new spatial
planning or ecological transfer. Freeze relative goal labels, heading wrapping
and test poses before execution. Introduce genuinely novel relative geometry
only as a separately planned generalization test after acquisition works.

**Revision of my numerical competence gate:** eight-of-ten runs and seven-of-
eight trials were illustrative engineering choices, not defensible biological
thresholds or power calculations. Do not elevate them into a research finding.
For the first deterministic two-cue benchmark, a policy must demonstrate both
prespecified cue-to-goal responses before its delayed failures can diagnose a
retention problem. That is branch coverage of this task, not a population
success-rate estimate. Report the full acquisition distribution and every
failed run. Never select a passing cohort retrospectively for a favourable
architecture comparison. Confirmatory precision/sample size requires a separate
pilot and a declared smallest effect, with fresh validation seeds.

**State intervention:** I challenged the evolved-memory proposal's all-ticks
reset because it also disrupts response-phase dynamical coordination. Its author
agreed to add one reset at the end of the fixed-pose delay, immediately before
first movement inference, then permit recurrence normally. Apply an equivalent
reset with a visible cue. A hidden impairment with preserved visible competence
better supports dependence on stored information. It still does not identify
a unique neural code; all-ticks reset remains a broad state-dependence check.
Realized activity costs may also differ, as the ecology review notes.

**Exact next unit:** specify and implement a bounded, separate visible-task
selection runner using the existing arena and declared genetic operators. Fix
the founder distribution, training fitness, evaluation budget, seeds and
training-only champion/tie rule; save every final population and acquisition
curve. First use the original pose and mapping to test acquisition feasibility,
without an ecological or generalization claim. A fixed-budget pilot may fail
honestly. Freeze held-out conditions before a subsequent validation; do not keep
the same holdout after tuning against its failures. No hidden-delay selection,
ecological mechanism or extra cost factor belongs in this first unit.

Genetic selection across generations would produce task-selected policies.
It is not acquisition during an organism's lifetime, associative revaluation
within life, UAL, or evidence of consciousness. Neither a benchmark success nor
a demo memory toggle changes that boundary.
