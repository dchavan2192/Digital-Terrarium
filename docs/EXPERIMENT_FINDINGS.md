# Experiment Findings — November 2026

Three parameter sweep experiments, each 5 values x 5 seeds x 200k ticks,
run on Raspberry Pi 5. All experiments used `configs/default.toml` as the
base config with one parameter varied.

## Summary

All three experiments produced **null results on the target variable** but
**informative results about the simulation's dynamics**. The sim sustains
stable populations across a wide parameter range, but three key evolutionary
behaviors — predation, cooperation, and brain simplification — fail to
emerge. This points to specific mechanistic gaps, not broken code.

---

## Experiment 1: Does Cooperation Evolve?

**Parameter:** `sharing.share_amount` = [0, 2, 5, 10, 20]

**Hypothesis:** Higher sharing capacity should enable kin-selected altruism
(Hamilton 1964: rB > C). Creatures cluster near relatives (offspring spawn
next to parents, hue drifts slowly), so relatedness r > 0. If the benefit B
to the recipient exceeds the cost C to the donor weighted by r, sharing
should evolve.

![Exp1 Results](images/experiments/exp1_cooperation.png)

### Results

| share_amount | Final Cooperation Rate | Kin Share Ratio |
|---|---|---|
| 0 (control) | 0.172 +/- 0.074 | N/A |
| 2 | 0.000 | started ~0.6, declined to ~0.02 |
| 5 | 0.000 | ~0.0 |
| 10 | 0.000 | ~0.0 |
| 20 | 0.000 | ~0.0 |

### Interpretation

**Cooperation is selected against in all conditions where sharing costs energy.**

- `share_amount=0`: "Sharing" persists at ~17% because the share output has
  zero cost — brain outputs that fire the share neuron face no penalty, so
  there's no selection pressure to silence it. This is the control confirming
  the mechanism works.

- `share_amount >= 2`: Cooperation drops to zero within ~20k ticks. Evolution
  reliably eliminates sharing behavior. Any creature that shares loses energy
  and is outcompeted by non-sharers.

- Kin share ratio for `share=2` briefly exceeds the random baseline (0.5),
  reaching ~0.8 early on — sharing IS initially kin-directed. But the benefit
  doesn't outweigh the cost, so it vanishes anyway.

### Why Hamilton's Rule Fails Here

Hamilton's condition rB > C requires:
- **r** (relatedness): Moderate. Offspring spawn nearby and hue drifts +-5 deg,
  so spatial clusters have r ~ 0.3-0.5.
- **B** (benefit): The energy received by the neighbor. Equal to share_amount.
- **C** (cost): The energy lost by the donor. Also equal to share_amount.

Since B = C and r < 1, the condition rB > C simplifies to r > 1, which is
impossible. **The sharing mechanism transfers energy 1:1 with no multiplier.**
In biology, cooperative behaviors often have B > C (e.g., alarm calls cost
little but save many). Our sharing is a zero-sum transfer.

### Fix Direction

To make cooperation evolvable:
1. **Add a benefit multiplier**: shared energy could be amplified (e.g., 1.5x)
   to model synergistic benefits (group hunting, shared warmth, etc.)
2. **Add group-level threats**: predation pressure or environmental disasters
   that are survivable in groups but not alone would make mutual aid beneficial
3. **Reduce sharing cost**: make share_amount smaller relative to income so
   the cost C is trivial compared to total energy budget

---

## Experiment 2: Does Food Scarcity Drive Predation?

**Parameter:** `food.regrowth_rate` = [0.0005, 0.001, 0.0015, 0.002, 0.003]

**Hypothesis:** When food is scarce, creatures should evolve predatory
strategies to survive. Lotka-Volterra dynamics predict predator-prey
oscillations when herbivory alone can't sustain populations.

![Exp2 Results](images/experiments/exp2_food_scarcity.png)

### Results

| regrowth_rate | Final Pop | Mean Trophic Level | Predator Count |
|---|---|---|---|
| 0.0005 | 314 | 1.0001 | 0 |
| 0.001 | 576 | 1.0004 | 0 |
| 0.0015 | 784 | 1.008 | ~1 |
| 0.002 | 928 | 1.003 | 0 |
| 0.003 | 1269 | 1.0001 | 0 |

### Interpretation

**Food scarcity does not drive predation. Pure herbivory dominates at all
scarcity levels.**

- Population scales linearly with food availability — the carrying capacity
  relationship is clean and well-behaved.
- Attacks spike in the first ~20k ticks (random brain weights produce
  aggression) but are rapidly selected against.
- `regrowth=0.003` shows beautiful seasonal population oscillations, confirming
  the Botkin (1990) seasonal mechanic works correctly.
- Trophic level is essentially 1.0 everywhere — no meaningful predation evolves.

### Why Predation Can't Compete

The energetics make herbivory strictly dominant:

| Strategy | Energy per action |
|---|---|
| **Eat food** | +30 energy, 0 cost |
| **Attack** | -5 cost + (damage × 0.5 transfer) |
| | Best case: -5 + (20 × 0.5) = +5 net |

Eating food yields **6x more energy** than a successful attack, with zero
risk and zero cost. Even under extreme scarcity, a creature is better off
searching for food than hunting. And when prey are starving (low energy),
attacks yield even less.

Additionally:
- Attacking costs energy even on a miss (5 per attempt)
- The victim might die, removing a future food source
- Attack range (8) is much smaller than vision range (60), so creatures
  must approach prey directly

### Fix Direction

To enable predator-prey dynamics:
1. **Increase trophic transfer ratio** from 0.5 to 0.7-0.8 (more realistic
   for single-prey consumption vs ecosystem-level Lindeman efficiency)
2. **Reduce food.energy_per_food** so herbivory is less overpowered
3. **Add attack speed bonus**: predators could lunge, making attacks more
   reliable
4. **Scale attack damage with body size more aggressively**: large predators
   should be able to one-shot small prey
5. **Consider area-of-effect attacks** or pack hunting mechanics

---

## Experiment 3: Does Brain Cost Constrain Complexity?

**Parameter:** `energy.brain_cost` = [0.0, 0.001, 0.002, 0.005, 0.01]

**Hypothesis:** Higher brain cost should select for simpler neural networks.
Creatures with expensive brains should evolve fewer connections as selection
balances behavioral capability against metabolic drain (Laughlin et al. 1998).

![Exp3 Results](images/experiments/exp3_brain_cost.png)

### Results

| brain_cost | Final Pop | Mean Connections | Brain Entropy |
|---|---|---|---|
| 0.000 | 795 | ~190 | ~0.44 |
| 0.001 | 767 | ~190 | ~0.42 |
| 0.002 | 758 | ~190 | ~0.42 |
| 0.005 | 717 | ~190 | ~0.50 |
| 0.010 | 694 | ~185 | ~0.42 |

### Interpretation

**Brain cost does NOT constrain neural complexity. All conditions converge
to ~190 connections regardless of cost.**

- Brain complexity grows identically across all conditions for the first
  ~50k ticks, then plateaus at the same level.
- Higher brain cost reduces population (fewer creatures can afford to live)
  but doesn't simplify individual brains.
- Brain entropy (behavioral complexity) is roughly constant across conditions.
- `brain_cost=0.01` (5x default) reduces population by ~13% vs the control
  but connections only drop by ~3%.

### Why Selection Doesn't Prune Brains

The NEAT mutation rates are the culprit:
- **Add connection**: 3% per reproduction
- **Add node**: 1% per reproduction
- **Remove connection**: 0% (no pruning mutation exists!)
- **Toggle connection**: 1% (disabled connections still count structurally)

There is no downward pressure on topology. Connections accumulate
monotonically. Even with high brain cost, extra connections with small
weights are near-neutral — they cost energy but don't hurt behavior.
Selection can only prune brains by killing creatures with too many
connections, but since the metabolic cost per connection is tiny compared
to base metabolism, the selection coefficient is negligible.

At `brain_cost=0.01` with 190 connections and 14 hidden nodes:
- Brain cost per tick: 0.01 × (14 + 6) = 0.2
- Base metabolism per tick: 0.2
- So brain cost doubles metabolism — significant! But still not enough to
  select for simpler brains because ALL creatures have ~190 connections,
  so there's no within-population variance to select on.

### Fix Direction

To make brain cost a real evolutionary constraint:
1. **Add a connection pruning mutation** (e.g., 2% chance to delete a
   random connection per reproduction) — this creates variance in brain
   size for selection to act on
2. **Make brain cost scale with connections, not just neurons** — currently
   `brain_cost × neuron_count` ignores the number of connections. A brain
   with 190 connections costs the same as one with 50 if they have the
   same number of hidden nodes
3. **Increase brain_cost significantly** or make it exponential rather
   than linear
4. **Cap initial connections**: the fully-connected initial topology starts
   creatures with ~100+ connections from tick 0, which sets a high floor

---

## Cross-Cutting Themes

### 1. The Simulation is Stable but Boring

All 75 runs across all conditions survived 200k ticks. No extinctions. The
ecosystem is robust — but nothing interesting evolves. Creatures learn to
eat food and avoid wasting energy, then stop changing. This is a **local
optimum trap**: simple herbivory works well enough that there's no gradient
toward more complex strategies.

### 2. Mutation Rates Create Structural Drift

NEAT add-connection (3%) and add-node (1%) mutations are one-directional.
Without pruning mutations, topology grows monotonically. This means brain
complexity is driven by mutation pressure, not selection. Adding pruning
mutations would let selection determine the optimal brain size.

### 3. Energy Economics are Unbalanced

Food is too cheap (30 energy/cell, 0 cost), attacks are too expensive (5
cost for maybe 10 gain), and sharing is zero-sum. The energy balance needs
recalibration so that alternative strategies (predation, cooperation) can
compete with simple foraging.

### 4. What "Interesting" Would Look Like

The simulation would be producing meaningful results when we observe:
- **Stable predator-prey oscillations** (not just transient initial aggression)
- **Persistent cooperation** that survives because it's beneficial, not neutral
- **Brain size variation** within the population that correlates with strategy
- **Speciation into distinct niches** (predators, herbivores, cooperators) that
  coexist rather than one strategy dominating
- **Extinction events** under some parameter combinations but not others —
  currently everything survives everywhere

---

## Priority Fixes (Ranked)

### High Priority — Enable Core Dynamics

1. **Add connection pruning mutation (2-3%)** — unlocks brain cost as a
   selective force. Without this, Exp3 can never produce meaningful results.

2. **Rebalance combat energetics** — increase `energy_transfer_ratio` to
   0.7-0.8, reduce `food.energy_per_food` to 15-20, or add attack lunge
   mechanics. Goal: make predation viable but not dominant.

3. **Add sharing benefit multiplier** — shared energy arrives at 1.2-1.5x,
   modeling synergistic group benefits. This satisfies Hamilton's rule when
   r > C/B.

### Medium Priority — Enrich Dynamics

4. **Make brain cost proportional to connections, not neurons** — a brain with
   190 connections should cost much more than one with 50.

5. **Reduce initial network connectivity** — start with sparser networks
   (e.g., 30% connectivity instead of fully connected) so there's room to
   grow OR shrink.

6. **Add environmental catastrophes** — periodic food crashes that favor
   groups (cooperative survival) and large-bodied creatures (energy reserves).

### Low Priority — Analysis Infrastructure

7. **Track per-species stats in CSV output** — currently only population-level
   stats are logged. Per-species data would enable lineage analysis.

8. **Add lineage tree reconstruction** — track parent-child relationships
   across the full run for phylogenetic analysis.

---

## Next Session Plan

1. Implement fixes #1-3 (pruning mutation, combat rebalance, sharing multiplier)
2. Re-run all three experiments with the fixes
3. Compare before/after — do the dynamics change qualitatively?
4. If predation or cooperation emerge, run longer (500k+ tick) experiments
   to study stability and speciation
