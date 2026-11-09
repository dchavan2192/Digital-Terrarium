# Terrarium — Progress & Continuation Doc

**Purpose**: Living document that tracks all work, decisions, and next steps. Read this first at the start of every session so you can continue without re-explanation.

**Last updated**: 2026-11-08

---

## Current State

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

### Phase 6: Analysis & Distribution [PLANNED]
- [ ] Python analysis pipeline for experiment plots
- [ ] WASM build + GitHub Pages live demo
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
