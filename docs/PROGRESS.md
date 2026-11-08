# Terrarium — Progress & Continuation Doc

**Purpose**: Living document that tracks all work, decisions, and next steps. Read this first at the start of every session so you can continue without re-explanation.

**Last updated**: 2026-11-07

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
- Speed: ~900 ticks/s single run, ~600 ticks/s with stats logging
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
- [ ] **Probabilistic mortality** — Replace fixed max_age=3000 with Gompertz-like mortality curve
  - After a certain age, probability of death increases exponentially each tick
  - Backed by Gompertz-Makeham law (see Research section)
  - Makes lifespan a distribution, not a constant — more biologically realistic
  - Enables evolution of longevity genes (future)
- [ ] **Research-backed design decisions** — Every mechanic should cite why it exists
- [ ] **Define control environment** — Standard test config for reproducible comparisons

### Phase 3: Ecological Complexity [PLANNED]
- [ ] Attack/consume mechanic (predator-prey dynamics)
- [ ] Body size as evolvable gene (affects speed, energy cost, attack power)
- [ ] Speciation detection and lineage tracking
- [ ] Trophic level analysis

### Phase 4: Social Intelligence [PLANNED]
- [ ] Broadcast signal channel (brain output -> nearby brain input)
- [ ] Energy transfer/sharing between creatures
- [ ] Kin recognition via color hue + signal correlation
- [ ] Cooperation rate measurement

### Phase 5: Cognitive Complexity [PLANNED]
- [ ] NEAT-style topology evolution (add/remove neurons and connections)
- [ ] Brain cost scales with actual complexity, not just neuron count
- [ ] Information-theoretic analysis of brain activations

### Phase 6: Analysis & Distribution [PLANNED]
- [ ] Python analysis pipeline for experiment plots
- [ ] WASM build + GitHub Pages live demo
- [ ] Paper-quality figures and reproducible experiment configs

### Infrastructure [ONGOING]
- [x] GPU acceleration investigation — NOT worth it at current scale (see Research section)
- [ ] rayon within-tick parallelism (3-6x speedup, low effort)
- [ ] SoA (struct-of-arrays) refactor for hot fields (1.5-3x speedup)
- [ ] Fast approximate tanh (1.1-1.3x speedup, trivial)
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
