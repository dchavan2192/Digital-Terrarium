# Terrarium — Progress & Continuation Doc

**Purpose**: Living document that tracks all work, decisions, and next steps. Read this first at the start of every session so you can continue without re-explanation.

**Last updated**: 2026-11-15

---

## Audit and Roadmap (Fable 5.2, 2026-11-15): read this first

This section supersedes "Current State" below, which describes the Session 2 codebase and is kept only as history. Phases 9–11 further down remain as detailed item lists; the stages here set their order.

### Where the project stands

The engineering is in good shape. The science has no standing results yet.

**What is solid**
- A deterministic, config-driven core (`sim-core`, about 4,600 lines) with no rendering code, 52 tests, and CI plus the Pages deploy green at `f009ab0`.
- A headless runner with sweeps, a Pi workflow, and an explainable 3D web demo driven by the same engine through `sim-wasm`.
- A literature base: `docs/research/PRIOR_ART.md`, `OEE_SHADOW_DESIGN.md`, and the bibliography.

**What is not**

On 2026-11-15 three silent bugs were found. Each one invalidates earlier results.

| Bug | Effect | How it was found |
|---|---|---|
| Sharing paid the recipient 1.5× the donor's cost | Free energy; populations lived on it; the "87% cooperation" result was an exploit | The owner noticed creatures camping in the demo |
| `[food.fruit]` was never read | Fruit was off in every run | The front-end engineer noticed a config flag reading false |
| Newborns counted as stressed | Methylation saturated by tick 2,000 | Code audit |

Every number in `README.md`, `ALIFE_2027_OUTLINE.md`, `EXPERIMENT_FINDINGS.md` and the session logs below predates these fixes. Treat the count of established results as zero.

**First run with honest energetics** (Pi, commit `f009ab0`, `hard_cap = 2000`, 100k ticks, 3 seeds per rate, 12 runs):

| `food.regrowth_rate` | Outcome |
|---|---|
| 0.0002 | Extinct in 3 of 3 (between ticks 8.8k and 27.8k) |
| 0.0003 | Extinct in 2 of 3; the survivor held about 55 creatures and dipped to 1 |
| 0.0004 | Survived in 3 of 3, holding about 63–77 creatures (lows of 16, 29 and 1) |
| 0.0005 (current default) | Survived in 3 of 3, holding about 65–88 creatures (lows of 17, 7 and 24) |

- The cap never blocked a birth in any run.
- Sharing fell from about 350 per tick to well under 1 per tick.
- **Conclusion:** the old ecosystem of 500–1,500 creatures was sustained by the sharing bug. With real energetics the default world supports about 70 creatures and sits close to extinction. Every parameter was tuned against the bug, so the ecology must be retuned before any experiment. The live demo now runs this sparse world too.

### Findings, in priority order

1. **Bugs are being found by luck, not by tests.** No test asserted that energy comes only from food, or that a config flag has any effect. Stage 0 adds an energy ledger and mechanism-effect tests.
2. **No persistence.** `Genome` and `Organism` are not serialisable, and there is no checkpoint, resume or genome export. A week-long Pi run dies with a power cut. Evolved brains cannot be tested, compared across eras, or seeded from a connectome. Every later stage depends on this.
3. **The brain cannot support the stated goal.** It is feed-forward only, holds at most 64 nodes (38 hidden after 19 inputs and 7 outputs), keeps no state between ticks, and does not learn within a life. Nothing that any theory of consciousness treats as relevant can arise in it.
4. **The world does not pay for cognition.** Evolution builds only what the environment rewards. Today a reflex ("turn toward food, eat") is close to optimal, so brains have no reason to grow. Adding brain capacity without changing the ecology will produce nothing.
5. **Crossover only mixes weights** (audit item G). The sex arm of any experiment is meaningless until a child can inherit structure from both parents.
6. **Cross-platform determinism is untested.** `sim-core` makes 24 calls to platform maths functions (`powf`, `exp`, `sin` and others). "Same seed = same run" may hold only on one machine; Mac, Pi and browser runs may diverge.
7. **Documentation asserts things that are stale or false.** README: population about 1,500, 30 tests, about 4,400 ticks/s. Outline abstract: "satisfying Bedau's criterion". `PRIOR_ART.md` lists the novelty claims to withdraw.
8. **Statistical power.** With 3–5 seeds only very large effects are detectable. Plan for 10–20 seeds, with metrics chosen before the run.
9. **The Pi is at its thermal limit.** It reached 85.6 °C and hard-throttled (`throttled=0xe0006`) with four concurrent runs and the fan at maximum.
10. **Repo hygiene for open source.**
    - `.claude/settings.local.json` is tracked on `main`.
    - The owner's main checkout has diverged from `origin`; current work lives in `.worktrees/merge` on branch `merge-remote`.
    - There is no CONTRIBUTING.md or CITATION.cff.
    - `sim.rs` is a single 1,643-line file and will not absorb water, parasites and plasticity cleanly.

### The consciousness goal

**What cannot be done.** No simulation can prove that digital consciousness can be created. There is no accepted theory of consciousness and no agreed way to measure it, so no behaviour or network structure can settle the question. A project that claims otherwise will not be taken seriously, and the claim would be false.

**What can be done.** Show which consciousness-relevant capacities evolve under ecological selection *without being designed in*, and which environmental pressures produce them. Most machine-consciousness work builds an architecture by hand. The evolutionary studies I know of (Edlund et al. 2011; Joshi, Tononi & Koch 2013; Albantakis et al. 2014) evolved tiny agents on single tasks, not an ecosystem. Whether this niche is really open needs its own prior-art pass (item 4.0).

**Restated goal:** an open-source evolutionary testbed for the origins of consciousness-relevant capacities, with theory-derived tests anyone can run. A positive result is evidence about what it takes for those capacities to evolve. It is a step, and should be described as one.

**Which theories give testable targets** (citations are from memory; verify before using them in a paper):

| Framework | What it says matters | What Terrarium could measure |
|---|---|---|
| Unlimited Associative Learning (UAL): Ginsburg & Jablonka (2019); Birch, Ginsburg & Jablonka (2020) | Open-ended learning about novel, compound stimuli and actions, proposed as the evolutionary marker of minimal consciousness | A behavioural battery: trace conditioning, second-order conditioning, novel compound discrimination, reversal learning |
| Recurrent processing: Lamme | Feedback loops | The fraction of brains with recurrent edges, and whether lesioning them costs fitness |
| Global workspace: Baars; Dehaene, Lau & Kouider (2017) | A limited-capacity hub that broadcasts to many processes | Whether evolved networks form a bottleneck hub, and how broad the deficit is when it is lesioned |
| Higher-order theories / metacognition | Monitoring one's own states | An opt-out task: does a creature decline a risky choice when its information is poor? |
| Attention schema: Graziano; Wilterson & Graziano (2021) | A model of one's own attention | Later; needs an attention mechanism first |
| Integrated information: Tononi | Causal integration (zero for feed-forward networks) | Integration proxies tracked over evolutionary time |
| Indicator-properties report: Butlin, Long et al. (2023) | A checklist drawn from the theories above, plus agency and embodiment | A per-lineage scorecard |

UAL is the best first target. It is defined behaviourally, it was proposed as an *evolutionary* transition, and "UAL evolved in an open-ended ecosystem" can be defended without taking a side on what consciousness is.

**Ethics.** If indicators accumulate, the project needs a written precautionary policy before scaling up (Birch 2017, 2024). It is cheap to write now and awkward to retrofit.

### Plan

Each stage ends in a gate. Do not start a stage until the previous gate is met.

**Stage 0: a trustworthy instrument** (about 1–2 weeks)
- [ ] 0.1 Retune the ecology on honest energetics. Pi sweep of `food.regrowth_rate` over 0.001–0.005 at `hard_cap = 2000`. Target: 800–1,300 creatures, zero cap-blocked births, 10 of 10 seeds alive at 200k ticks. Update `default.toml` and `demo.toml`.
- [ ] 0.2 Energy ledger. Account for every source and sink each tick (food, fruit, metabolism, movement, brain, crowding, temperature, combat loss, sharing loss, reproduction, deaths, clamping at the energy cap). In debug builds, assert that the change in total energy equals the ledger. Expose the ledger in `TickStats`.
- [ ] 0.3 Mechanism-effect tests. For every `enabled` flag and major parameter, an ON run and an OFF run must diverge within a set number of ticks. This catches the fruit class of bug.
- [ ] 0.4 Persistence. Serde for `Genome`, `Organism` and simulation state; `--checkpoint-every` and `--resume` in sim-runner; periodic genome export. Test: 2,000 ticks straight must equal 1,000 + save + load + 1,000, bit for bit.
- [ ] 0.5 Cross-platform determinism. Hash the state after 5,000 ticks on Linux in CI, in WASM under node, and on the Pi. If the hashes differ, move the affected maths to a portable implementation (the `libm` crate).
- [ ] 0.6 Truth pass on the docs. Fix README metrics and the outline abstract, and withdraw the claims `PRIOR_ART.md` lists. Start `docs/RESULTS.md`, where a result is listed only with its commit, config, seeds and analysis script. Nothing enters README unless it is in that file.
- [ ] 0.7 Repo. Decide whether to untrack `.claude/settings.local.json` (owner's call), resync the main checkout, add CONTRIBUTING.md and CITATION.cff.
- [ ] 0.8 Pi. Run 3 jobs at once or improve cooling. No multi-day run before 0.4.
- **Gate:** tests green including 0.2–0.5, and a 10-seed baseline that reproduces on two machines.

**Stage 1: first results that stand** (about 2–3 weeks of Pi time)
- [ ] 1.1 Crossover with global innovation numbers, validated against neat-python or TensorNEAT.
- [ ] 1.2 Neutral shadow model, following Polyworld's lockstep mode (Yaeger, Griffith & Sporns 2008) and `OEE_SHADOW_DESIGN.md`.
- [ ] 1.3 `docs/PREREG.md`: hypotheses, metrics, seeds and analysis fixed before the runs.
- [ ] 1.4 Factorial experiment: sex × niche construction × epigenetics, at least 10 seeds per cell, scored against the shadow, with a shuffled-methylation control.
- [ ] 1.5 Cooperation re-test under energy-conserving sharing. A null result is a result.
- [ ] 1.6 Model description in ODD format, using MODES metric names.
- **Gate:** at least one effect that survives the shadow control and replicates. That is the ALIFE 2027 paper (check the submission date).

**Stage 2: a brain that could support cognition** (about 3–4 weeks; can overlap Stage 1's Pi time)
- [ ] 2.1 Recurrent connections (Phase 9b).
- [ ] 2.2 Raise `MAX_NODES` from 64 to 256, so capacity is limited by energy cost and not by the buffer.
- [ ] 2.3 Evolved neuromodulated plasticity (Phase 9c; Soltoggio et al. 2008).
- [ ] 2.4 Cognitive test battery in sim-runner, run on exported genomes in fixed arenas: detour, delayed response, trace conditioning, reversal learning, second-order conditioning, novel compound discrimination. Report it as a UAL checklist.
- **Gate:** the battery runs in CI on a reference genome; feed-forward creatures fail the memory tasks; evolved recurrent creatures pass at least one.

**Stage 3: a world that pays for cognition** (about 4–6 weeks)
- [ ] 3.0 Split `sim.rs` into one module per tick phase before adding mechanics.
- [ ] 3.1 Water in low ground (Phase 10). Commuting between water and food rewards spatial memory.
- [ ] 3.2 Cue-predicted resources whose meaning changes within a lifetime, for example a food type that turns toxic for a season. Evolution cannot hard-wire the answer, so learning is rewarded (Hinton & Nowlan 1987).
- [ ] 3.3 Parasites (Phase 10).
- [ ] 3.4 Individual recognition and repeated encounters, so reciprocity becomes possible (Trivers 1971).
- [ ] 3.5 Predator–prey tournaments across eras (Cliff & Miller 1995).
- **Gate:** learning or recurrent genotypes outcompete reflex genotypes in the new ecology and not in the old one. That shows cognition is being selected for, and is publishable by itself.

**Stage 4: the indicator programme** (ongoing once Stages 2 and 3 are in)
- [ ] 4.0 Prior-art pass on evolved-agent consciousness research, in the style of `PRIOR_ART.md`, before claiming the niche.
- [ ] 4.1 Per-lineage scorecard over evolutionary time: recurrence, integration proxies, hub structure with lesion tests, the opt-out metacognition task, the UAL battery.
- [ ] 4.2 Pre-registered experiments that discriminate between theories. Examples: does UAL arrive together with a jump in integration? Does a workspace-like hub appear only under multi-task pressure?
- [ ] 4.3 `docs/ETHICS.md`: the precautionary policy and what would trigger a review.
- [ ] 4.4 Scale. Once brains pass about 1,000 neurons the Pi is the wrong machine; batch the brain forward pass for a GPU and compare against JaxLife.
- [ ] 4.5 Release the battery and scorecard as an open benchmark. Others are more likely to adopt that than the simulator.

### What not to do
- Do not claim consciousness, sentience or "fruit-fly level" anywhere.
- Do not add water, parasites or other mechanics before Stage 0 and the Stage 1 experiments. Each one changes every baseline.
- Do not start multi-day Pi runs before checkpointing exists.
- Do not let unattended sessions write results into README. Results go through `docs/RESULTS.md`.

### Next five actions
1. Launch the 0.1 regrowth sweep. This needs the owner's go-ahead, because the Pi is hard-throttling.
2. Build the 0.2 energy ledger and the 0.3 mechanism-effect tests.
3. Build 0.4 persistence.
4. Do the 0.6 truth pass on README and the outline.
5. Build 1.1 innovation-number crossover.

---

## Current State (historical: Session 2 codebase, superseded by the audit above)

### What's Built and Working
- [x] **sim-core**: World, food grid, spatial grid (counting-sort), brain (13->8->3 FF net), genome (evolvable weights + mutation rate + color hue), organism, deterministic tick loop
- [x] **sim-viewer**: macroquad + egui, pan/zoom camera, food/creature rendering, inspector panel
- [x] **sim-runner**: Headless CLI, single runs with CSV stats, parallel parameter sweeps via rayon
- [x] **CI pipeline**: GitHub Actions — clippy, test, fmt, WASM build check
- [x] **Tuned parameters**: 5/5 seeds survive 100k ticks, ~1100 stable pop, ~750 births/1000 ticks
- [x] **Cumulative stats**: total_births/total_deaths added to TickStats
- [x] **Autonomous workflow**: CLAUDE.md + .claude/settings.local.json with scoped permissions

### Key Metrics (current tuned config)
- Population: ~1100 stable equilibrium
- Turnover: ~75,000 births over 100k ticks (massive evolutionary churn)
- Mean energy: ~30 (threshold=60, so only good foragers reproduce)
- Speed: ~4400 ticks/s (rayon parallel + fast_tanh), up from ~1000 single-threaded
- Deterministic: same seed = same run (ChaCha8Rng)

### GitHub
- Repo: `dchavan2192/Digital-Terrarium`
- Branch: `main` (7 commits)
- All CI checks passing

---

## Active Work Items

### Phase 1: Foundation [COMPLETE]
- [x] Scaffold workspace (sim-core, sim-viewer, sim-runner)
- [x] Implement core simulation loop
- [x] Build visual renderer
- [x] Build headless CLI + sweep tool
- [x] Performance optimization (~50 -> ~900 ticks/s)
- [x] Parameter tuning for stable 100k-tick runs
- [x] Add cumulative birth/death tracking

### Phase 2: Biological Realism [IN PROGRESS]
- [x] **Probabilistic mortality** — Gompertz-Makeham mortality curve implemented
  - Per-tick death probability: h(age) = λ + α·exp(β·age)
  - β (aging_rate) is a genomic trait — longevity can now evolve
  - max_age retained as absolute safety cap
  - Backed by Gompertz (1825), Williams (1957), Kirkwood (1977)
- [ ] **Research-backed design decisions** — Every mechanic should cite why it exists
- [ ] **Define control environment** — Standard test config for reproducible comparisons

### Phase 3: Ecological Complexity [COMPLETE]
- [x] Attack/consume mechanic (predator-prey dynamics)
- [x] Body size as evolvable gene (affects speed, metabolism, attack power/defense)
- [x] Speciation detection and lineage tracking
- [x] Trophic level analysis
- [x] **Spatial resource gradients (biomes)** — ecological niches driving adaptive radiation
  - BiomeZone config: circular zones with regrowth_multiplier and smooth falloff
  - Per-cell regrowth rates computed at init, supports wraparound distance
  - Default config: 2 lush oases (3× regrowth), 1 moderate center (2×), 1 barren zone (0.3×)
  - Renderer shows biome tinting: bright green (lush) → brown (barren)
  - Backward compatible: no biomes = uniform regrowth (existing configs unchanged)
  - Citations: Schluter (2000) adaptive radiation, Rosenzweig (1995) species diversity

### Phase 4: Social Intelligence [COMPLETE]
- [x] Broadcast signal channel (brain output -> nearby brain input)
- [x] Energy transfer/sharing between creatures
- [x] Kin recognition via color hue + signal correlation
- [x] Cooperation rate measurement

### Phase 5: Cognitive Complexity [COMPLETE]
- [x] **Variable hidden layer size** — evolvable hidden neuron count (4-32)
  - hidden_size as genomic trait, 2% mutation rate (±1 neuron per reproduction)
  - Weight vector resized on topology change (new neurons get Xavier-scale init)
  - Brain cost scales naturally with actual neuron count (already brain_cost × neuron_count)
  - Stack-allocated forward pass preserved (uses MAX_HIDDEN=32 fixed buffer)
  - Genome distance accounts for topology difference (0.8× weight for speciation)
  - Viewer inspector shows full topology (e.g., 15→8→6)
  - Citation: Stanley & Miikkulainen (2002) NEAT topology evolution
- [x] **Full NEAT-style topology** — connection genes replace dense weight matrix
  - ConnectionGene: (from, to, weight, enabled) — arbitrary feed-forward connections
  - HiddenNode: (id, depth) — nodes at variable depths for topological ordering
  - Add-connection mutation (3%): new edge between unconnected feed-forward pair
  - Add-node mutation (1%): split existing connection A→B into A→C→B (classic NEAT)
  - Toggle-connection mutation (1%): flip enabled/disabled on random connection
  - Sparse forward pass: connections pre-sorted by eval order, single linear pass
  - Stack-allocated activation buffer [f64; MAX_NODES=64] — zero heap allocs per tick
  - genome_distance: merge-join on sorted connections (structure + weight + traits)
  - Skip connections emerge naturally (input→output, deep hidden paths)
  - Citations: Stanley & Miikkulainen (2002), Kassahun & Sommer (2005)
- [x] **Information-theoretic brain analysis** — Shannon entropy of brain outputs
  - Per-organism activation histograms: 6 outputs × 8 bins with exponential decay (τ≈100 ticks)
  - shannon_entropy(): normalized [0,1], 0=deterministic, 1=maximum entropy
  - activation_to_bin(): maps [-1,1] output to histogram bin index
  - Population behavioral diversity: entropy of peak-bin distributions across all organisms
  - New TickStats: mean_brain_entropy, behavioral_diversity, mean_connections
  - Viewer: stats panel shows entropy/diversity/connections; inspector shows per-organism entropy
  - CSV output includes all new fields automatically (serde::Serialize)
  - Citations: Shannon (1948), Beer & Williams (2015)

### Phase 2.5: Environmental Dynamics [IN PROGRESS]
- [x] **Seasonal/cyclical food variation** — regrowth oscillates sinusoidally over configurable period
  - `[food.seasons]` config: amplitude (0-1) and period (ticks)
  - Formula: `multiplier = 1.0 + amplitude * sin(2π * tick / period)`
  - Default: amplitude=0.3, period=10000 (regrowth swings 70%-130%)
  - Applied globally on top of biome multipliers
  - Backward compatible: amplitude=0 disables (existing configs unaffected)
  - Viewer shows current seasonal multiplier percentage
  - Citation: Botkin (1990) "Forest Dynamics: An Ecological Model"
- [x] **Population density pressure** — creatures in crowded cells have higher metabolism (Allee effect)
  - `[density_pressure]` config: `cost_per_neighbor = 0.01` (extra energy per tick per nearby creature)
  - Neighbor counting piggybacks on existing `for_each_nearby` spatial query — zero extra allocation
  - New TickStats field: `mean_crowding` (average neighbor count across living organisms)
  - Viewer stats panel shows "Mean crowding: X.X"
  - Backward compatible via `#[serde(default)]` — cost=0.0 disables (existing configs unaffected)
  - Two new integration tests: `density_pressure_increases_metabolism`, `density_pressure_no_effect_when_zero`
  - Citation: Allee (1931) "Animal Aggregations: A Study in General Sociology"
- [x] **Dynamic speciation threshold** — compute from population genome variance

### Testing & Validation [ONGOING]
- [x] **Biome-aware food regrowth integration test** — verifies FoodGrid with BiomeZone config produces per-cell regrowth rate variation; lush zones regrow faster than moderate zones, which regrow faster than neutral (no-zone) areas; also verifies uniform rates when no biomes configured
- [x] **Seasonal food variation tests** — `seasonal_food_variation` verifies sinusoidal multiplier at tick 0 (baseline), peak summer (1+amplitude), peak winter (1-amplitude), and actual food accumulation at each phase; `seasonal_no_effect_when_amplitude_zero` verifies backward compatibility

### Phase 7: Scientific Novelty — Publishable Mechanisms [COMPLETE]
**Goal**: Implement features no other ALife platform has, targeting ALIFE 2027 submission.
**Research council** (Nov 14, 2026) surveyed ALIFE 2023-2026 proceedings and identified 5 highest-impact features.

- [x] **Activity-dependent neural metabolism** — brain cost scales with ACTIVITY, not just structure
  - Effective cost = structure_cost + Σ|weight × activation| per tick
  - Selects for sparse coding: brains that activate few connections are cheaper
  - Citations: Attwell & Laughlin (2001) "An Energy Budget for Signaling in the Grey Matter of the Brain", Niven & Laughlin (2008), Lennie (2003)
- [x] **Sexual reproduction with NEAT crossover** — the first major evolutionary transition
  - Mate-finding, crossover of different-topology genomes, twofold cost of sex
  - Enables recombination: faster adaptation, Muller's ratchet prevention
  - Citations: Maynard Smith (1978) "The Evolution of Sex", Muller (1932), Stanley & Miikkulainen (2002) §3.2
- [x] **Open-ended evolution (OEE) metrics** — MODES toolbox (Dolson et al. 2019)
  - Evolutionary activity statistics: novelty rate, cumulative activity D(t), active traits
  - Lineage tracking: generation depth, lineage diversity, coalescence
  - THE measurement that proves the sim produces genuine open-ended evolution
  - Citations: Bedau & Packard (1991), Dolson et al. (2019) "The MODES Toolbox" *Artificial Life* 25(1):50-73, Taylor et al. (2016)
- [x] **Niche construction** — organisms modify their environment (ecological inheritance)
  - Construction grid: organisms deposit/remove material that alters local food regrowth
  - Creates a second inheritance system beyond the genome
  - No neural-agent ALife platform has this — completely novel
  - Citations: Odling-Smee, Laland & Feldman (2003) "Niche Construction: The Neglected Process in Evolution", Laland et al. (1999) PNAS 96:10242-10247
- [x] **Epigenetic inheritance** — heritable non-genetic state (methylation marks)
  - Per-connection methylation [0,1] silences connections; responds to environmental stress
  - Offspring inherit parental methylation with noise — transgenerational ecological memory
  - Almost completely absent from ALife literature (only EpiAL, Sousa & Costa 2010)
  - Citations: Jablonka & Lamb (2005) "Evolution in Four Dimensions", Heard & Martienssen (2014) Cell 157(1):95-109

**Planned experiments** (each: feature ON vs OFF, 5+ seeds, 500k+ ticks):
- [ ] OEE (REWORDED 2026-11-15): Show *excess* activity over a neutral shadow model is positive and unbounded — the target is Channon class 4b (unbounded median activity, bounded diversity, because hard_cap limits diversity). Raw D(t) growth alone proves nothing, since drift also grows. Design: docs/research/OEE_SHADOW_DESIGN.md (~1.5 days). Prerequisite: an order-independent topology_hash.
- [ ] Core ALIFE 2027 paper: factorial sweep (sex × niche × epigenetics × droughts), with each mechanism scored by its excess activity over the shadow (Channon 2006; Taylor et al. 2016)
- [ ] Follow-ups: shadow vs MODES persistence filter on the same runs (Dolson et al. 2019); hard_cap scaling sweep (does diversity saturate?)
- [ ] Niche construction: Do niche-constructing lineages survive droughts better?
- [ ] Epigenetics: Do stress-primed offspring outperform naïve offspring post-drought?
- [ ] Sexual vs asexual: Does sex accelerate adaptation speed and prevent Muller's ratchet?

**Future high-impact** (post Phase 7):
- [ ] Horizontal gene transfer (HGT) — connection gene exchange between organisms
- [ ] Major evolutionary transition to multicellularity — group-level selection emerging from cooperation

### Phase 7.5: Correctness Fixes Before Experiments [BLOCKING — from code audit 2026-11-15]
Experiments on the Pi are ON HOLD until these are fixed; otherwise the ON/OFF ablations measure bugs.
The on/off switches themselves were verified to disable each feature cleanly, and no determinism or allocation violations were found.
- [x] **A. Methylation saturates** — child_energy/max (0.15) < stress_threshold (0.3), so all newborns count as "stressed" (sim.rs:732); mean_methylation hits 0.78 by tick 2000. Fix: stress relative to reproduction threshold or a rolling energy trend; make the threshold configurable.
- [x] **B. Methylation never affects the living brain** — brain is only built at spawn/birth (sim.rs:237, 1006); per-tick methylation changes the genome only. Either rebuild the brain periodically or document it as an inheritance-only effect.
- [x] **C. inheritance_noise=0 panics** — empty gen_range (sim.rs:1001); uniform noise, not Gaussian as the docs say.
- [x] **D. Old 15/6 configs read a hidden node as `construct`** (brain.rs:186); validate inputs>=19 / outputs>=7, and update configs/intense.toml.
- [x] **E. Niche bonus ignores biomes/seasons/droughts** — additive global rate (sim.rs:1088-1101), so constructed cells regrow inside droughts. Docs say a multiplier; ConstructionGrid::regrowth_multiplier is dead code. Signed mean_construction_level cancels out.
- [x] **F. No twofold cost of sex** — only the initiator pays (sim.rs:1031); mates can be reused without limit; the 0.5 mate factor is hard-coded (sim.rs:958).
- [ ] **G. Crossover only mixes weights** — the child's edge set always equals the fitter parent's; hidden-node ids aren't global innovation numbers.
- [ ] **H. OEE metrics mislabeled** — "D(t)" averages over all traits ever seen (activity.rs:110), not active ones; lineage_diversity can only decrease (hits 1 by tick 8k); sexual/asexual births are per-tick, not cumulative. **Add a neutral shadow model** (design: docs/research/OEE_SHADOW_DESIGN.md). — metric fixes done (extant-only Ā(t), windowed lineage_diversity, cumulative birth counters, order-independent topology hash v2); shadow model still open.
- [x] **K. Hard-coded constants** → config: mate factor 0.5, construct dead zone 0.01, regrowth cutoff 0.001, structural mutation probabilities in genome.rs.
- [x] Update stale CLAUDE.md (still says 15→6 I/O, 22 tests; config table out of date).
- [x] Sweep tool: on/off switches for reproduction.sexual, niche_construction.enabled, epigenetics.enabled
- [x] Pi launch script: analysis/launch_pi_phase7.sh (sims run on the Pi only — never locally)
- [ ] README OEE section (uncommitted): its numbers and images are unverified (the images don't exist). Replace with real Pi results.

- [x] **Fruit was silently disabled in every run**: the sim read a top-level `fruit` table, but the config defines `[food.fruit]`. Fixed, with a regression test (2026-11-15). Earlier results ran without fruit.
- [ ] sim-core v2 data for the demo: per-organism last action + target, death causes, energy-flow history, hidden activations (see the ABI doc's v2 table)

### Phase 8: Explainable 3D Web Demo [IN PROGRESS — started 2026-11-15]
**Problem**: The current demo (macroquad triangles + egui panel) looks like a game; visitors can't tell what's happening, and the stats are hard to read.
**Goal**: A portfolio-grade, explainable visualization that makes the science legible to a non-expert in 10 seconds.

**Architecture decision (D7)**: The new web front end is **Three.js + HTML/CSS dashboard**, driving sim-core compiled to WASM through a thin new crate `crates/sim-wasm`.
It uses a plain `extern "C"` ABI and zero-copy Float32Array views into WASM memory, so it needs no wasm-bindgen tooling.
sim-core keeps zero rendering code. The macroquad viewer stays as the native dev tool, and the old WASM page stays available at `web/classic.html`.

**Visual design**:
- 2.5D/3D world: terrain mesh from the elevation map, ground coloured by food density / biome, drought zones as dry overlays, day/night lighting.
- Creatures as small instanced low-poly "mini agents" (body + eyes): size = body_size, colour = species, a glow/pulse for actions (eating, attacking, sharing, constructing), short motion trails.
- Orbit camera + top-down map toggle; click a creature → follow-cam + inspector (brain graph, energy budget, lineage, trophic level).
- Data layers toggle: food, temperature, pheromones, niche construction, elevation.

**Explainability / stats**:
- Live "what's happening" event feed in plain English (droughts, predation spikes, new species, population crashes).
- A few headline KPIs with sparklines (population, species, mean brain size, cooperation rate, trophic level), plus a full charts tab.
- Guided "tour" / legend overlay explaining creatures, colours, energy, evolution.

**Work items**:
- [x] Design spec — docs/research/DEMO_UX_DESIGN.md
- [x] crates/sim-wasm: C-ABI exports (ABI: docs/research/SIM_WASM_ABI.md), with stable species tracking + derived events
- [x] web/demo/ front end (Three.js 0.160.0): MVP done. v2 still open: charts tab, trails, share/construct cues, light theme
- [x] Update deploy-pages.yml + ci.yml to build sim-wasm + ship the new front end
- [ ] Portfolio card (/Users/dineshchavan/projects/portfolio-dhruv/ds.html): fix stale claims, link the new demo, add a screenshot
- [ ] Visual QA in a browser (needs the user's OK — the demo runs the sim locally)
- [ ] Deploy (push to main — needs the user's explicit OK)

### Phase 9.0: Energy Conservation Bug in Sharing [BLOCKING — found 2026-11-15]
- [x] **FIXED (2026-11-15): Sharing created energy from nothing.** The donor pays `share_amount` (5) but the recipient gets 5 × `benefit_multiplier` (1.5) = 7.5 (sim.rs, sharing step). Two neighbours sharing every tick each net +2.5/tick, about 8× base metabolism, so clusters can camp in place indefinitely (seen in the demo). Run data: shares rise to ~350/tick (≈ +875 free energy/tick population-wide) while mean energy climbs 65 → 117 at the cap.
  - Breaks design principle 4 ("food is the only source of energy").
  - Probably why the population pins at hard_cap; it will also confound the Phase 9a regrowth sweep.
  - **The "cooperation evolved (87%)" result is invalidated**: cooperation was selected as a free-energy exploit, not by Hamilton's rule.
  - Fix: conserve energy (recipient gets ≤ the donor's cost, with a transfer loss). Hamilton's B > C should come from the *marginal fitness value* of energy (a starving recipient gains more than a well-fed donor loses), not from creating energy. Consider restricting gifts to donors above the reproduction threshold.
  - Add an energy-conservation integration test: total energy change per tick = food eaten − all costs (no other sources).
  - Re-run the v2 cooperation experiments after the fix.
  - Fix landed: `sharing.transfer_efficiency` (default 0.9, alias of the legacy `benefit_multiplier`) is validated to (0, 1]; `combat.energy_transfer_ratio` is validated to [0, 1]. New tests: `energy_never_increases_without_food` and `energy_creating_share_efficiency_is_rejected`.
  - [ ] Still to do: re-run cooperation experiments; optionally restrict gifts to donors above the reproduction threshold.
- [ ] **Prior-art audit (docs/research/PRIOR_ART.md)**: withdraw the false claims in PROGRESS and docs/ALIFE_2027_OUTLINE.md ("no neural-agent platform has niche construction": JaxLife 2024 and Chiba et al. 2020 do; "EpiAL is the only epigenetics precedent": Turner et al. 2013, Brawer et al. 2017; the EpiAL citation is ICAART 2010). Cite Polyworld's lockstep mode (Yaeger, Griffith & Sporns 2008) as the reference design for the shadow model. Call our cost of sex a "twofold per-offspring investment cost". Adopt the ODD protocol (Grimm et al. 2020) and the MODES metric names. Narrow novelty claim: a factorial genetic × epigenetic × ecological inheritance test in evolved neural agents, scored against a neutral shadow.

### Phase 9: Food-Limited Ecology + Cognition [PLANNED — agreed 2026-11-15]
**Why**: (1) Population pins at hard_cap (500) by ~tick 1000 and births are blocked at the cap (sim.rs:993). The cap, not food competition, decides who reproduces, which weakens selection and hides drought/niche/epigenetics effects. (2) Brains are tiny (~11–16 evolved neurons, ~220 connections vs ~140k neurons / ~50M synapses in a fruit fly, FlyWire 2024; 302 neurons in C. elegans). More importantly, they are purely feed-forward with no memory and no lifetime learning: behaviour is reflexive, closer to Braitenberg vehicles or bacterial chemotaxis. Never claim "fruit-fly level".

**9a. Food-limited carrying capacity** (do FIRST, before the Phase 7 ablations)
- [ ] Pi sweep of `food.regrowth_rate` (e.g. 0.0002–0.0005) with `creature.hard_cap` 1500–2000; pick the value where mean population ≈ 60–70% of the cap and the cap almost never binds
- [x] New stats: `cap_blocked_births` / `total_cap_blocked_births` (should stay ~0); `sweep --set KEY=NUMBER` for fixed overrides
- [x] Ran `analysis/launch_pi_phase9a.sh` on the fixed code (hard_cap=2000; regrowth 0.0002–0.0005 × 3 seeds × 100k ticks). Result: the range was far too low. Populations of 55–88 at best, extinction at 0.0002–0.0003, zero cap-blocked births. See the audit table at the top. Results on the Pi in `output/phase9a/`; pre-fix partial runs in `output/phase9a_prebugfix/`.
- [ ] Follow-up sweep over 0.001–0.005 (audit item 0.1)
- [ ] Check runs don't overshoot into extinction (all 5 seeds survive); keep the cap only as a performance backstop
- [ ] Then run the Phase 7 ON/OFF ablations at this food-limited setting
- Citations: Verhulst (1838) logistic growth; Allee (1931) density dependence; Lotka (1925) / Volterra (1926)

**9b. Memory: recurrent NEAT connections**
- [ ] Allow recurrent edges (previous-tick activations as inputs), evolved by add-connection; stack-allocated `[f64; MAX_NODES]` prev-state buffer per organism, no per-tick allocs; determinism preserved
- [ ] Config: `brain.allow_recurrent`, `mutation.add_recurrent_probability`
- [ ] Stats: fraction of brains with recurrent edges, mean recurrent edges
- Citation: Stanley & Miikkulainen (2002) NEAT supports recurrence

**9c. Lifetime learning: evolved neuromodulated Hebbian plasticity**
- [ ] Per-connection evolvable plasticity coefficients (ABCD Hebbian rule) gated by a modulatory output/neuron; weights change within a lifetime and are NOT inherited (genome keeps the initial weights), distinct from epigenetic methylation
- [ ] Config: `[plasticity] enabled, learning_rate`, plus mutation rates for the rule coefficients
- Citations: Soltoggio et al. (2008) evolved neuromodulated plasticity; Risi & Stanley (2010) adaptive HyperNEAT; Hinton & Nowlan (1987) the Baldwin effect
- Novelty angle: evolution + in-lifetime learning + epigenetics in one open-ended ecosystem lets us study the Baldwin effect

**9d. Measure intelligence behaviourally (not by neuron count)**
- [ ] A "cognitive test battery" in sim-runner: freeze evolved genomes from saved snapshots and test them in standard mini-arenas: detour (food behind a barrier), delayed foraging (remember a patch after it vanishes from view), kin vs non-kin discrimination, predator avoidance
- [ ] Score per generation → a figure showing cognitive capability evolving over time; compare ON/OFF for recurrence/plasticity
- [ ] Requires genome snapshot export (also needed for arms-race CIAO plots, Cliff & Miller 1995)

### Phase 10: Real Ecosystem — Water, Nutrient Cycling, Parasites [PLANNED — agreed 2026-11-15]
Order: AFTER the Phase 7 ablations, so first results aren't mixed with new mechanics.
- [ ] **Water as a second essential resource**: water pools in low elevation (reuse ElevationMap); per-organism hydration drains each tick (faster when warm and for larger bodies); drinking at water cells. Droughts shrink water bodies.
  - Brain: hydration input + "water" ray type, so inputs 19 → ~20–21; update validate/demo.
  - Essential resources, Liebig's law of the minimum (Tilman 1982). Expect commuting between water and food, and waterhole ecology.
  - Tune with a Pi sweep (avoid instant extinction).
  - Demo: render real water in the terrain.
- [ ] **Nutrient recycling / decomposition**: corpses become a nutrient grid that boosts regrowth; optional decomposer niche. Closes the matter loop (Lindeman 1942).
- [ ] **Parasites / disease**: pathogens transmitted between neighbours, evolving against host resistance genes → Red Queen dynamics; gives sex a real advantage (Hamilton 1980; Van Valen 1973).
- [ ] **Multiple nutrients** (e.g. C/N) for resource-ratio coexistence (Tilman 1982) — optional, after water.

### Phase 11: Real Brains + Integration [IDEAS — 2026-11-15, check prior art first]
- [ ] **Connectome seeding**: initialise some creatures with the real C. elegans connectome (302 neurons, ~7k synapses; White et al. 1986; WormWiring / OpenWorm c302 data), mapping chemosensory neurons → food inputs and motor neurons → turn/speed. Experiment: connectome-seeded vs random-start populations — does real wiring give an evolutionary head start, or is it rewired? Requires recurrence (9b). Fly larva (~3k neurons, Winding et al. 2023) is a possible stretch goal; an adult fly (FlyWire 2024, ~140k neurons) is not feasible per creature on a Pi.
- [ ] **Integration over evolution**: measure integrated-information / complexity proxies of evolved brains over time (Edlund et al. 2011; Albantakis et al. 2014). Frame it as complexity measurement, NOT consciousness; feed-forward nets score Φ = 0 under IIT. Reference: Butlin, Long et al. (2023) indicator properties.

### Phase 6: Analysis & Distribution [PLANNED]
- [ ] Python analysis pipeline for experiment plots
- [x] WASM build + GitHub Pages live demo — https://dchavan2192.github.io/Digital-Terrarium/
- [ ] Paper-quality figures and reproducible experiment configs

### Infrastructure [ONGOING]
- [x] GPU acceleration investigation — NOT worth it at current scale (see Research section)
- [x] rayon within-tick parallelism (4.3x speedup achieved)
- [ ] SoA (struct-of-arrays) refactor for hot fields (1.5-3x speedup)
- [x] Fast approximate tanh (Pade approximant, part of 4.3x speedup)
- [ ] Define "Earth-like" primordial environment model

---

## Research & Citations

### Mortality Models [RESEARCHED]

**Current problem**: Fixed `max_age = 3000` means all creatures die at exactly the same age. This is biologically unrealistic and eliminates any selection pressure on longevity.

**Solution**: Replace with **Gompertz-Makeham mortality law**:
```
death_probability_per_tick = lambda + alpha * exp(beta * age)
```
- `lambda` — extrinsic mortality (accidents, starvation): ~0.0001
- `alpha` — initial senescence rate at birth: ~0.00001
- `beta` — aging rate (how fast mortality increases): ~0.001
- Make `beta` a genomic trait so longevity can evolve

**Key evolutionary biology theories backing this**:
1. **Mutation accumulation** (Medawar, 1952): Deleterious late-life mutations accumulate because selection is weak at old ages
2. **Antagonistic pleiotropy** (Williams, 1957): Genes beneficial early in life can cause late-life deterioration — aging is a trade-off
3. **Disposable soma** (Kirkwood, 1977): Organisms allocate energy between reproduction and somatic repair — aging results from under-investment in maintenance

**Implementation plan**: Encode `beta` in genome. Optionally add "repair cost" — creatures can spend energy to slow senescence (implements disposable soma). Antagonistic pleiotropy emerges naturally from the energy budget.

**ALife precedent**: Most sims (Avida, Tierra) use replacement-based death, not aging. Polyworld uses energy depletion. Explicit Gompertz aging is rare but scientifically motivated — this would be a differentiator.

**Citations**:
- Gompertz, B. (1825) "On the Nature of the Function Expressive of the Law of Human Mortality"
- Williams, G.C. (1957) "Pleiotropy, Natural Selection, and the Evolution of Senescence" *Evolution* 11(4):398-411
- Kirkwood, T.B.L. (1977) "Evolution of ageing" *Nature* 270:301-304
- Mitteldorf, J. (2006) "Chaotic Population Dynamics and the Evolution of Ageing" *Evol Ecol Res* 8:561-574
- Chow et al. (2004) "Adaptive Radiation from Resource Competition in Digital Organisms" *Science* 305:84-86

### Primordial Earth Environment [RESEARCHED]

**Early Earth conditions (~3.8-4.2 Bya)**:
- Reducing atmosphere: CO2, N2, H2O, CO, H2 (no free O2)
- Temperature: 50-80C oceans, hydrothermal vents 60-400C
- Energy: UV radiation, lightning, hydrothermal chemistry, volcanic activity
- Chemistry: simple organics (HCN, formaldehyde, amino acids), clay minerals as catalysts

**Relevant models**:
- Eigen's quasispecies (1971): error threshold for information maintenance
- Kauffman's autocatalytic sets (1986): random polymer networks achieving catalytic closure
- Ganti's chemoton (1971): minimal living cell model

**How to map to Terrarium** (future phases):
- Multiple resource types (not just one food) — chemical diversity
- Spatial energy gradients (e.g., hydrothermal vent zones with higher food density)
- Degradation rate — organisms break down over time (ties into aging model)
- Temperature zones affecting metabolism rates

**Citations**:
- Eigen, M. (1971) "Self-organization of matter" *Naturwissenschaften* 58:465-523
- Kauffman, S.A. (1986) "Autocatalytic sets of proteins" *J Theor Biol* 119:1-24
- Szostak et al. (2001) "Synthesizing life" *Nature* 409:387-390
- Martin & Russell (2003) "On the origins of cells" *Phil Trans R Soc B* 358:59-85

### GPU Acceleration [RESEARCHED — NOT RECOMMENDED YET]

**Verdict: GPU is not worth it at current scale (1100 creatures, 139 weights/brain).**

**Why**: GPU dispatch overhead (~30-100us per dispatch) dwarfs the actual compute (~1us for 1100 brain forward passes). Break-even is ~10k-50k creatures with current brain size.

**Recommended CPU optimizations instead** (ordered by ROI):

| Optimization | Effort | Expected Speedup |
|---|---|---|
| rayon within-tick parallelism | Low (hours) | 3-6x |
| SoA data layout for hot fields | Medium (1-2 days) | 1.5-3x |
| f32 instead of f64 | Medium (half day) | 1.5-2x |
| Fast approximate tanh | Low (30 min) | 1.1-1.3x |

**GPU becomes interesting** when: creature count hits 4000+ AND brain complexity increases (NEAT with 50-200+ neurons). That's a Phase 5 concern.

**Key insight on the 800% CPU**: That's rayon's parameter sweep parallelism — all 8 cores running different simulations. The actual single-tick loop is single-threaded. Adding rayon within the tick loop would parallelize a single run across cores.

---

## Design Decisions Log

Every non-trivial design decision should be recorded here with justification.

### D1: Deterministic simulation via seeded RNG
**Decision**: Use ChaCha8Rng seeded from config for all randomness
**Why**: Reproducibility is essential for scientific claims. Same config + seed = same run.
**Source**: Standard practice in computational biology; required for peer review

### D2: Energy as universal currency
**Decision**: All actions (moving, thinking, reproducing) cost energy; food is the only source
**Why**: Creates a single optimization target for evolution. Mirrors real metabolic costs.
**Source**: Thermodynamic constraints on biological systems (Lotka, 1922)

### D3: Intelligence tax (brain_cost per neuron per tick)
**Decision**: Larger brains cost more energy to maintain
**Why**: Prevents runaway brain growth. In nature, brains consume ~20% of metabolic budget despite being ~2% of body mass. Creates evolutionary tension between intelligence and efficiency.
**Source**: Aiello & Wheeler (1995) "The Expensive-Tissue Hypothesis"

### D4: Toroidal world (wraparound boundaries)
**Decision**: World wraps in both axes, no edges
**Why**: Eliminates edge effects that would create artificial niches. Every location is topologically equivalent.
**Source**: Standard in spatial ecology models (Durrett & Levin, 1994)

### D5: Fixed-topology brain (v1)
**Decision**: Start with fixed 13->8->3 feed-forward net, evolve only weights
**Why**: Simplest evolvable controller. Topology evolution (NEAT) is Phase 5 — need to prove the platform works with simple brains first.
**Source**: Stanley & Miikkulainen (2002) — NEAT showed topology matters, but weight-only evolution is the baseline

### D6: Parameter tuning — scarce food + cheap reproduction
**Decision**: regrowth=0.0015, threshold=60, parent_cost=30, metabolism=0.2
**Why**: Creates selective pressure where mean energy (~30) is below reproduction threshold (60). Only skilled foragers reproduce. Cheap reproduction (cost=30) enables high turnover (~750 births/1000 ticks) for rapid evolution.
**Validated**: 5/5 seeds survive 100k ticks with stable ~1100 population

---

## Environment Spec (Control Config)

For any experiment, the **control** is `configs/default.toml` at the commit tagged for that experiment. Key parameters:

```
World: 1000x1000, toroidal
Food: 10k cells, regrowth=0.0015/tick, 30 energy/food
Creatures: 300 initial, 1500 cap, max_age=3000
Energy: start=60, max=200, metabolism=0.2, movement=0.08, brain=0.002
Reproduction: threshold=60, parent_cost=30, child_energy=30
Brain: 13->8->3, tanh activation
Mutation: 10% per weight, +/-0.2 step
```

**Standard test protocol**: Run 5 seeds (0-4) x 100k ticks. Report survival rate, mean final population, total births.

---

## How to Continue a Session

1. Read this file (`docs/PROGRESS.md`)
2. Read `CLAUDE.md` for safety rules and conventions
3. Check the "Active Work Items" section for what's next
4. Check "Research & Citations" for pending research
5. Pick up the next unchecked item and work

---

## Session Log

### Session 1 (2026-11-07)
- Scaffolded entire project from design doc
- Built sim-core, sim-viewer, sim-runner
- GitHub repo created, CI pipeline, README, LICENSE
- Performance optimized 50->195 ticks/s (later ~900 with smaller pop)
- Set up autonomous agent workflow (CLAUDE.md + permissions)

### Session 2 (2026-11-07, continued)
- Phase 1 parameter tuning complete
- Added cumulative total_births/total_deaths to TickStats
- Validated stability: 5/5 seeds survive 100k ticks
- Pushed tuned config to GitHub
- Started Phase 2 research: mortality models, GPU acceleration, primordial Earth
- Created this progress doc

### Session 3 (2026-11-07, continued)
- **Implemented Gompertz-Makeham probabilistic mortality**
  - Added `[aging]` config section: `extrinsic_mortality` (λ=0.0001), `initial_senescence` (α=0.00001), `aging_rate` (β=0.002)
  - Added `aging_rate` as evolvable genomic trait (mutates 5% of the time, ±20%, clamped [0.0001, 0.01])
  - Replaced fixed `age >= max_age` death check with per-tick Gompertz probability
  - `max_age` retained as absolute safety cap
  - Added aging params to sweep.rs for parameter exploration
  - New test: `gompertz_mortality_kills_old_creatures` (11 tests total, all passing)
  - Full CI: clippy clean, fmt clean, all tests pass
- Pi 5 running 500k-tick baseline overnight as control data

### Session 4 (2026-11-08)
- **Rayon within-tick parallelism**: 4.3x speedup (1000 -> 4400 ticks/s)
  - Parallelized sense-think-act loop with `par_iter()` — read-only phase, no RNG, determinism preserved
  - Sequential phases (apply actions, reproduce, die) unchanged
- **Fast approximate tanh**: Pade approximant `x(27+x²)/(27+9x²)`, max error ~0.004
- Analyzed Pi baseline data (5 seeds x 100k + 1 seed x 500k ticks)
- Pi running Gompertz comparison overnight
- **Predator-prey combat mechanics**
  - New brain output: attack (outputs 3->4)
  - [combat] config: attack_range=8, attack_cost=5, attack_damage=20, energy_transfer_ratio=0.5
  - Attacker pays cost, nearest neighbor in range takes damage, 50% energy transfer (trophic loss)
  - ~7600 attacks in first 1000 ticks — creatures using combat immediately
  - Citations: Lotka (1925), Volterra (1926), Yaeger Polyworld (1994)
- **Evolvable body size**
  - body_size gene [0.5, 3.0], default 1.0, 5% mutation rate
  - Speed scales as max_speed/body_size (bigger = slower)
  - Metabolism scales as base_metabolism * body_size^0.75 (Kleiber's law)
  - Attack damage scales with attacker size, defense scales with victim size
  - Added as 14th brain input so creatures sense their own size
  - Citation: Kleiber (1932) body size allometry
- **Speciation detection & lineage tracking**
  - Added `parent_id` and `generation` to Organism (lineage tracking)
  - Genome distance function: weighted sum of brain weight divergence + trait differences
  - Greedy threshold-based species clustering (NEAT-style compatibility distance)
  - New stats: `species_count`, `mean_generation` in TickStats and CSV output
  - Viewer inspector shows generation and parent ID for selected creatures
  - New test: `lineage_tracking_and_speciation` (12 tests total, all passing)
  - Citation: Stanley & Miikkulainen (2002) NEAT compatibility distance
- **Trophic level analysis**
  - Per-organism `energy_from_food` and `energy_from_attacks` lifetime tracking
  - Trophic level = 1 + (attack_energy / total_energy): 1.0 = herbivore, 2.0 = pure predator
  - Classification: herbivore (<25% from attacks), predator (>50% from attacks), omnivore (between)
  - New stats: `mean_trophic_level`, `herbivore_count`, `predator_count` in TickStats/CSV
  - Viewer shows per-creature trophic level and energy breakdown in inspector
  - New test: `trophic_level_tracking` (13 tests total, all passing)
  - Citation: Lindeman (1942) "The Trophic-Dynamic Aspect of Ecology"
- **README plots and analysis**
  - Python plotting script (analysis/plot_readme.py) with dark GitHub theme
  - 5 plots: dashboard (hero), population/energy, birth/death, combat, evolution
  - README rewritten with real metrics, biology citations table, embedded plots
- **Phase 4: Social Intelligence — complete**
  - Broadcast signal channel: 5th brain output (signal [-1,1]), distance-weighted averaging of nearby signals as brain input. Citation: Werner & Dyer (1991)
  - Energy sharing: 6th brain output (share), voluntary energy transfer to nearest neighbor. Config: share_range=10, share_amount=5. Citation: Hamilton (1964), Trivers (1971)
  - Kin recognition: 15th brain input — color hue similarity to nearest creature [-1,1]. Enables kin-selective behavior
  - Cooperation measurement: cooperation_rate (shares/pop), kin_share_ratio (fraction of shares to kin, hue diff < 30 degrees)
  - Brain now 15 inputs, 6 outputs (turn, speed, eat, attack, signal, share)

### Session 5 (2026-11-08, continued)
- **Variable hidden layer size (Phase 5 start)**
  - `hidden_size` added to Genome as evolvable trait, range [4, 32], default 8
  - 2% mutation rate per reproduction: ±1 neuron, weight vector resized correctly
  - New neurons get Xavier-scale random weights; removed neurons simply dropped
  - `Brain::from_genome()` constructor for per-organism topology
  - `genome_distance()` updated for variable-size comparison + topology penalty
  - Viewer inspector shows full topology (15→N→6)
  - New test: `variable_hidden_size_evolves` (14 tests total, all passing)
  - Pi 5 safe: stack-allocated forward pass unchanged (MAX_HIDDEN=32 buffer)
  - Citation: Stanley & Miikkulainen (2002) NEAT topology evolution
- **Full NEAT-style topology (connection genes)**
  - Replaced dense weight matrix with ConnectionGene list + HiddenNode list
  - brain.rs rewritten: sparse forward pass with pre-sorted connections, [f64; MAX_NODES=64] buffer
  - genome.rs rewritten: NEAT mutations (add-connection 3%, add-node/split 1%, toggle 1%)
  - Add-node splits A→B into A→C(weight=1.0)→B(old weight) — preserves behavior while complexifying
  - genome_distance: zero-allocation merge-join on sorted connection genes (O(n+m))
  - Skip connections (input→output) emerge naturally from add-connection mutations
  - Genome stores num_inputs/num_outputs internally — reproduce() no longer needs external params
  - Updated tests: `brain_topology_matches_genome`, `neat_topology_evolves` (14 tests, all passing)
  - Citations: Stanley & Miikkulainen (2002), Kassahun & Sommer (2005)
- **Information-theoretic brain analysis (Phase 5 complete)**
  - Per-organism output histograms: 6 outputs × 8 bins, exponential decay (τ≈100 ticks)
  - Shannon entropy normalized to [0,1] — measures behavioral variability over recent history
  - Population behavioral diversity: entropy of peak-bin distributions across organisms
  - New stats: mean_brain_entropy, behavioral_diversity, mean_connections
  - Viewer updated: stats panel + inspector show entropy metrics
  - Citation: Shannon (1948), Beer & Williams (2015)
- **Spatial resource gradients (biomes)**
  - BiomeZone: circular zones with regrowth_multiplier and smooth falloff from center
  - Per-cell regrowth rates (FoodGrid stores regrowth_rates Vec alongside cells)
  - Default config: 4 biomes — 2 lush oases (3×), 1 moderate center (2×), 1 barren NE (0.3×)
  - Renderer: biome-aware food coloring (bright green → brown by richness)
  - Wraparound-safe distance calculation for biome zones
  - Backward compatible via `#[serde(default)]` — existing configs work unchanged
  - Motivation: Pi 500k runs showed 0 speciation — uniform environment = no niche pressure
  - Citations: Schluter (2000), Rosenzweig (1995)

### Session 7 (2026-11-09)
- **Integration test for biome-aware food regrowth** (16 tests total, all passing)
  - `biome_aware_food_regrowth`: constructs FoodGrid with two BiomeZone configs (5× lush, 2× moderate), verifies three-tier regrowth rates (lush > moderate > barren/neutral), tests actual food accumulation over 10 regrow cycles
  - `biome_no_zones_gives_uniform_regrowth`: verifies backward compatibility — no biomes configured means all cells get the uniform base regrowth rate
  - Discovered that "barren" zones (multiplier < 1.0) have no effect in current implementation due to "highest multiplier wins" logic starting at 1.0 — documented in test comments

### Session 8 (2026-11-09)
- **Seasonal/cyclical food variation** (18 tests total, all passing)
  - Added `SeasonsConfig` struct with `amplitude` and `period` fields to config.rs
  - Added `[food.seasons]` section to default.toml (amplitude=0.3, period=10000)
  - `FoodGrid::regrow()` now takes tick and SeasonsConfig, applies sinusoidal multiplier
  - `seasonal_multiplier()` public function: `1.0 + amplitude * sin(2π * tick / period)`, clamped ≥0
  - Applied globally on top of per-cell biome rates — creates boom-bust cycles
  - Viewer UI shows "Season: X%" when seasons are active
  - Backward compatible via `#[serde(default)]` — amplitude=0 means no effect
  - Two new integration tests: `seasonal_food_variation`, `seasonal_no_effect_when_amplitude_zero`
  - Citation: Botkin (1990) "Forest Dynamics: An Ecological Model"

### Session 9 (2026-11-09)
- **Population density pressure (Phase 2.5)**
  - Added `DensityPressureConfig` with `cost_per_neighbor` field (default 0.0 for backward compat)
  - Added `[density_pressure]` section to default.toml: `cost_per_neighbor = 0.01`
  - Neighbor counting integrated into existing parallel sense phase (`for_each_nearby` closure)
  - `neighbor_count: u32` added to Action struct, crowding cost applied during metabolism
  - `mean_crowding` added to TickStats — average neighbor count across living organisms
  - Viewer stats panel displays "Mean crowding: X.X"
  - Two new integration tests (20 total, all passing):
    - `density_pressure_increases_metabolism`: verifies crowding reduces mean energy
    - `density_pressure_no_effect_when_zero`: verifies backward compatibility (cost=0 identical to omitted)
  - Zero extra spatial queries or allocations — piggybacks on existing vision neighbor loop
  - Citation: Allee (1931) "Animal Aggregations: A Study in General Sociology"

### Session 10 (2026-11-09)
- **Dynamic speciation threshold (Phase 2.5 complete)**
  - Added `SpeciationConfig` with `dynamic`, `fixed_threshold`, `sample_size`, `factor`, `min_threshold`, `max_threshold`
  - Added `[speciation]` section to default.toml: `dynamic = true`, `factor = 0.75`, bounds [0.1, 2.0]
  - Algorithm: deterministic stride-based sampling of organisms, compute all pairwise genome distances among sample, set threshold = factor × mean_distance, clamped to [min, max]
  - Replaces hard-coded `species_threshold = 0.5` with adaptive computation
  - `speciation_threshold` added to TickStats for observation and CSV output
  - Viewer shows threshold alongside species count: "Species: N (t=X.XX)"
  - Backward compatible via `#[serde(default)]`: omitting `[speciation]` gives `dynamic=false`, `fixed_threshold=0.5` (identical to previous behavior)
  - Two new integration tests (22 total, all passing):
    - `dynamic_speciation_threshold_adapts_to_diversity`: verifies threshold is computed from genome variance, within configured bounds, and differs from fixed default
    - `speciation_fixed_threshold_backward_compatible`: verifies omitting config section gives identical behavior to explicit `dynamic=false`
  - Zero allocation: stride sampling is O(sample_size²) distance computations with no heap allocation beyond the existing genome distance function
  - Citation: Stanley & Miikkulainen (2002) "Evolving Neural Networks through Augmenting Topologies"

### Session 11 — Overnight autonomous (2026-11-15)
- **OEE 200k-tick experiment completed on Pi 5** (seed 42, ~15 min at 214 ticks/s)
  - **D(t) = 67.45** — grows unboundedly, fits D(t) ≈ 11.94·ln(t) − 81.5
  - Early growth rate: 0.0006/tick, late rate: 0.0001/tick (ratio 0.22 — sublinear but never zero)
  - Novel traits remain positive throughout (final: 8/window), active traits ~99
  - Lineage diversity collapses to 1 by tick ~5k — single dominant ancestor
  - Max generation reaches 150, mean generation ~143
  - Methylation decays from 0.65 → 0.10 (population adapts away from stress)
  - Construction level oscillates ±0.1 — organisms actively modifying environment
- **Analysis plots generated**: oee_activity.png, oee_lineage.png, oee_epigenetics.png in docs/images/
- **Phase 7 checklist**: All 5 features marked COMPLETE
- **Key finding**: D(t) follows logarithmic growth — consistent with Bedau's "class 2" unbounded
  activity. The system produces genuine open-ended evolution by the Bedau & Packard (1991) criterion.

### Session 12 — Overnight autonomous (2026-11-15, continued)
- **5-seed OEE replication experiment** on Pi 5 (seeds 0-4, 200k ticks each)
  - Seeds 0-3 complete, seed 4 in progress
  - D(t) log fits: seed 0 = 12.82·ln(t), seed 1 = 21.58·ln(t), seed 2 = 14.01·ln(t), seed 3 = 18.93·ln(t)
  - All seeds show unbounded D(t) growth — replication confirms OEE
  - Mean D(t) at 200k ≈ 82.2 across seeds 0-3
- **Created multi-seed analysis script**: `analysis/plot_multi_seed.py` (95% CI comparison plots)
- **Created ablation experiment configs**: `configs/no_epigenetics.toml`, `configs/no_niche_construction.toml`
- **Created ALIFE 2027 paper outline**: `docs/ALIFE_2027_OUTLINE.md`
- **Pushed**: `0ec0f20` — analysis tools and paper outline

### Session 13 — Overnight autonomous (2026-11-15, continued)
- **Epigenetics ablation experiment running** on Pi 5 (5 seeds × 200k ticks, no_epigenetics.toml)
  - Seed 0 complete (835.2s, faster than baseline — no methylation computation)
  - Seeds 1-4 in progress
- **Created ablation analysis script**: `analysis/plot_ablation.py`
  - D(t) comparison with 95% CI between baseline and ablation conditions
  - Population stability (CV) comparison
  - Mann-Whitney U statistical tests for significance
- **Updated ALIFE 2027 outline**: D(t) numbers updated to confirmed 5-seed data (~15.5·ln(t))
- **Updated README** with OEE results section — multi-seed plots and summary statistics
- **Pushed**: `f04f7e4` (ablation script + outline), `c4bbd95` (README OEE section)

> **Caveat (added 2026-11-15, Session 14):** the Session 11–13 OEE numbers were computed BEFORE the Phase 7.5 fixes. Back then `mean_cumulative_activity` averaged over every topology ever seen (not extant ones), the topology hash was order-dependent, and fruit was silently disabled. There was also no neutral shadow model, so 'genuine OEE' is NOT yet established. Treat these numbers as preliminary and re-run them with the fixed code plus the shadow model.
