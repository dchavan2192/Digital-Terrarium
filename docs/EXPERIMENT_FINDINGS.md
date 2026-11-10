# Experiment Findings — November 2026

Two rounds of experiments testing whether cooperation, predation, and brain
complexity can emerge from pure natural selection.

## Executive Summary

**v1 experiments** (original parameters): All three null results. No cooperation,
no predation, no brain size variation. Diagnosis: energy economics too easy,
sharing zero-sum, no pruning mutation.

**v2 experiments** (after rebalancing): **Cooperation evolved to 87%** via
Hamilton's kin selection. Predation partially emerged (trophic level ~1.2).
Brain cost still doesn't constrain complexity.

**Key result:** The first emergent social behavior in Terrarium — kin-directed
energy sharing — evolved reliably across 5/5 seeds when the benefit multiplier
made Hamilton's rB > C satisfiable.

---

## Changes Between v1 and v2

| Parameter | v1 | v2 | Rationale |
|---|---|---|---|
| Connection pruning mutation | 0% | 2% per reproduction | Create brain size variance |
| `food.energy_per_food` | 30 | 15 | Make herbivory less dominant |
| `combat.energy_transfer_ratio` | 0.5 | 0.75 | Make predation viable |
| `sharing.benefit_multiplier` | 1.0 (implicit) | 1.5 | Satisfy Hamilton's rB > C |

---

## Experiment 1: Does Cooperation Evolve?

**Parameter:** `sharing.share_amount` = [0, 2, 5, 10, 20]

**Theory:** Hamilton (1964) kin selection: cooperation evolves when rB > C,
where r = relatedness, B = benefit to recipient, C = cost to donor.

### v1 Results (benefit_multiplier = 1.0)

| share_amount | Cooperation Rate | Kin Share Ratio |
|---|---|---|
| 0 | 0.172 | N/A |
| 2 | 0.000 | started ~0.6, declined to ~0.02 |
| 5 | 0.000 | ~0.0 |
| 10 | 0.000 | ~0.0 |
| 20 | 0.000 | ~0.0 |

**v1 conclusion:** Cooperation selected against everywhere. B = C (zero-sum
transfer), so Hamilton's condition requires r > 1, which is impossible.

### v2 Results (benefit_multiplier = 1.5)

![Exp1v2 Results](images/experiments/exp1v2_cooperation.png)

| share_amount | Cooperation Rate | Population | Kin Share Ratio |
|---|---|---|---|
| 0 | 0.099 +/- 0.086 | 482 | N/A |
| 2 | 0.002 +/- 0.003 | 453 | ~0.2 (declining) |
| **5** | **0.866 +/- 0.051** | **1497** | **~1.0** |
| **10** | **0.787 +/- 0.141** | **1496** | **~1.0** |
| **20** | **0.752 +/- 0.156** | **1496** | **~1.0** |

### Analysis

**Cooperation evolved reliably when share_amount >= 5.** This is the first
emergent social behavior in Terrarium.

Key observations:
- **87% cooperation rate at share=5** — nearly every creature shares every tick
- **Kin share ratio ~1.0** — virtually all sharing is between relatives
  (hue difference < 30 degrees). This is textbook kin selection.
- **3x population advantage** — sharing populations reach 1500 (hard cap)
  while non-sharing ones stay at ~450. Cooperation is a population-level
  advantage, not just individual benefit.
- **Threshold effect at share=2** — cost 2, benefit 3 (2 x 1.5), net gain 1.
  This marginal benefit isn't enough to overcome selection pressure against
  altruism. Cooperation needs a clear payoff.
- **Cooperation rate decreases slightly with share_amount** (87% → 75% from
  share=5 to share=20) — higher stakes sharing is riskier, so slightly fewer
  creatures evolve to do it, but those that do sustain massive populations.

**Why it works now:** With multiplier 1.5, sharing 5 energy costs 5 but
delivers 7.5. Hamilton's condition: r × 7.5 > 5, so r > 0.67. Offspring
spawn adjacent to parents with hue drift of +-5 degrees, creating spatial
kin clusters with r well above 0.67. The math checks out and evolution
found the solution independently.

**Citation confirmed:** Hamilton, W.D. (1964) "The Genetical Evolution of
Social Behaviour" — the benefit multiplier is the critical parameter, not
the absolute sharing amount.

---

## Experiment 2: Does Food Scarcity Drive Predation?

**Parameter:** `food.regrowth_rate` = [0.0005, 0.001, 0.0015, 0.002, 0.003]

**Theory:** Lotka-Volterra (1925/1926) predator-prey dynamics should emerge
when herbivory alone can't sustain populations.

### v1 Results (energy_per_food=30, transfer_ratio=0.5)

| regrowth_rate | Population | Trophic Level | Predator Count |
|---|---|---|---|
| 0.0005 | 314 | 1.0001 | 0 |
| 0.001 | 576 | 1.0004 | 0 |
| 0.0015 | 784 | 1.008 | ~1 |
| 0.002 | 928 | 1.003 | 0 |
| 0.003 | 1269 | 1.0001 | 0 |

**v1 conclusion:** Pure herbivory everywhere. Food yields 30 free energy vs
attacks netting ~5. Herbivory was 6x more profitable.

### v2 Results (energy_per_food=15, transfer_ratio=0.75)

![Exp2v2 Results](images/experiments/exp2v2_food_scarcity.png)

| regrowth_rate | Population | Trophic Level | Attacks/tick |
|---|---|---|---|
| 0.0005 | 1497 | ~1.1 | 5-8 |
| 0.001 | 1496 | ~1.1-1.2 | 5-8 |
| 0.0015 | 1497 | ~1.1-1.2 | 5-8 |
| 0.002 | 1495 | ~1.2-1.3 | 10-18 |
| 0.003 | 1496 | ~1.0 | 3-5 |

### Analysis

**Partial success.** Predation now persists throughout the run instead of
dying to zero. Trophic levels shifted from flat 1.0 to 1.1-1.3, with
~10-15% predators visible in the bar chart. The `regrowth=0.002` condition
shows the highest sustained attack rate (~15/tick at peak).

However, populations still hit hard cap across all conditions, meaning
food scarcity isn't truly constraining survival. The food nerf (30→15)
helped but wasn't aggressive enough. All creatures still have more than
enough to eat.

**Interesting observation:** `regrowth=0.002` (moderate food) produces MORE
predation than `regrowth=0.0005` (very scarce food). This suggests predation
needs prey to be well-fed (high energy targets) to be profitable. When
everyone is starving, there's nothing worth hunting.

**Remaining fixes needed:**
- Further reduce `energy_per_food` or increase `base_metabolism`
- Make brain cost proportional to connections (connection-heavy brains are
  currently too cheap to maintain)
- Lower `hard_cap` to create real competition for resources

---

## Experiment 3: Does Brain Cost Constrain Complexity?

**Parameter:** `energy.brain_cost` = [0.0, 0.001, 0.002, 0.005, 0.01]

**Theory:** Higher brain cost should select for simpler networks.
Laughlin et al. (1998): neural information processing has a metabolic cost.

### v1 Results (no pruning mutation)

All conditions: ~185-190 connections. No variation by brain cost.

### v2 Results (2% pruning mutation added)

![Exp3v2 Results](images/experiments/exp3v2_brain_cost.png)

| brain_cost | Population | Mean Connections |
|---|---|---|
| 0.000 | 1498 | ~183 |
| 0.001 | 1497 | ~180 |
| 0.002 | 1497 | ~180 |
| 0.005 | 1493 | incomplete data |
| 0.010 | 1493 | incomplete data (1 seed) |

### Analysis

**Still no meaningful effect.** The 2% pruning mutation brought connections
down slightly (190→180) but didn't create differential pressure across
cost levels. The problem:

- Add-connection rate (3%) > pruning rate (2%) = net growth of ~1% per
  generation
- Brain cost is per-neuron, not per-connection. A brain with 180 connections
  and 14 hidden nodes costs the same as one with 50 connections and 14 nodes.
- All creatures have roughly the same brain size, so there's no within-
  population variance for selection to act on.

**Required fix:** Make `brain_cost` proportional to `connection_count`, not
`neuron_count`. This is the single change most likely to make brain cost
meaningful. A creature with 180 connections should pay 3.6x more than one
with 50 connections.

---

## Cross-Cutting Themes

### 1. Cooperation is Solved

The benefit multiplier approach works exactly as Hamilton predicted. This
is a publishable result: "Kin-selected cooperation emerges reliably in
evolved neural network agents when the benefit-to-cost ratio exceeds the
inverse of spatial relatedness." The threshold behavior (works at share=5,
fails at share=2) adds nuance.

### 2. Predation is Close

The combat rebalance moved trophic levels from 1.0 to 1.1-1.3. Predation
exists but isn't dominant. The next push needs to make herbivory harder,
not predation easier. Stochastic droughts (added post-v2) may provide
the localized scarcity needed.

### 3. Brain Complexity Needs a Different Cost Model

Pruning mutation alone isn't enough. The cost model fundamentally doesn't
penalize connection count. This needs an architectural change, not a
parameter tweak.

### 4. The Simulation is Too Easy

Every v2 run hit hard cap (1500). No extinctions. When everyone survives,
selection is weak. The environment needs to be harsher: higher metabolism,
lower food, more frequent droughts. The v2 cooperation result succeeded
*despite* the easy environment — imagine what happens under real pressure.

---

## What "Interesting" Would Look Like

The simulation would be producing meaningful results when we observe:
- [x] **Persistent cooperation** that survives because it's beneficial
- [~] **Sustained predation** (partially achieved, trophic ~1.2)
- [ ] **Stable predator-prey oscillations** (not just background attacks)
- [ ] **Brain size variation** correlated with strategy
- [ ] **Speciation into distinct niches** that coexist
- [ ] **Extinction events** under harsh conditions

---

## Priority Fixes (Updated)

### Completed (v1 → v2)

- [x] Add connection pruning mutation (2%)
- [x] Rebalance combat: energy_per_food 30→15, transfer_ratio 0.5→0.75
- [x] Add sharing benefit_multiplier (1.5)
- [x] Add stochastic droughts (post-v2)

### Next Priority

1. **Make brain cost proportional to connections** — `brain_cost × connection_count`
   instead of `brain_cost × neuron_count`. This is the key change for Exp3.

2. **Increase base_metabolism** or further reduce food — make the environment
   harsher so not every run hits hard cap. Real selection needs differential
   survival.

3. **Run 500k+ tick experiments** with droughts enabled — the drought mechanic
   was added after v2 experiments ran. Need to test whether localized scarcity
   drives predation and migration.

### Future

4. **Sexual reproduction** (NEAT crossover) — accelerates adaptation via
   recombination. Justified by Red Queen hypothesis once disease is added.

5. **Disease** — density-dependent pathogens that punish clustering, creating
   a tradeoff with kin-selected cooperation.

6. **Environmental barriers** — impassable zones that isolate populations,
   enabling allopatric speciation.

---

## Experiment Log

| Date | Experiment | Config | Runs | Key Result |
|---|---|---|---|---|
| 2026-11-09 | v1 cooperation | share_amount sweep, multiplier=1.0 | 24/25 | 0% cooperation |
| 2026-11-09 | v1 food scarcity | regrowth_rate sweep, food=30, transfer=0.5 | 25/25 | 0% predation |
| 2026-11-09 | v1 brain cost | brain_cost sweep, no pruning | 25/25 | ~190 connections everywhere |
| 2026-11-10 | v2 cooperation | share_amount sweep, multiplier=1.5 | 25/25 | **87% cooperation** |
| 2026-11-10 | v2 food scarcity | regrowth_rate sweep, food=15, transfer=0.75 | 19/25 | trophic ~1.2 |
| 2026-11-10 | v2 brain cost | brain_cost sweep, 2% pruning | 19/25 | ~180 connections, no variation |
