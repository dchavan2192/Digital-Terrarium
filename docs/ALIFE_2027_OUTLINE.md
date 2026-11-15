# Terrarium: Open-Ended Evolution Through Niche Construction and Epigenetic Inheritance in a Neural-Agent Ecosystem

## ALIFE 2027 Paper Outline

### Abstract (~150 words)
We present Terrarium, a research-grade artificial life platform where neural-agent organisms with NEAT-topology brains live, compete, cooperate, and evolve in a sealed 2D environment. Terrarium integrates five mechanisms rarely combined in ALife: (1) activity-dependent neural metabolism, (2) sexual reproduction with NEAT crossover, (3) niche construction via environmental modification, (4) epigenetic inheritance through stress-responsive methylation, and (5) open-ended evolution (OEE) metrics via the MODES toolbox. We demonstrate that cumulative activity D(t) grows as ~12·ln(t) over 200k ticks across 5 seeds (p < 0.01), satisfying Bedau's criterion for unbounded evolutionary activity. Epigenetic markers respond to environmental stress and are inherited transgenerationally, creating a second channel of inheritance beyond the genome. Niche construction creates positive feedback loops where organisms modify food availability, altering selective pressures for future generations.

### 1. Introduction
- Gap: most ALife platforms lack multiple inheritance channels (genetic + epigenetic + ecological)
- No neural-agent ALife system integrates niche construction with epigenetic inheritance
- Contribution: Terrarium unifies these in a single deterministic, reproducible platform
- Key question: does combining these mechanisms produce qualitatively different evolutionary dynamics than genetic evolution alone?

### 2. Related Work
- **Neural-agent ALife**: Polyworld (Yaeger 1994), NEAT in ALife (Stanley & Miikkulainen 2002)
- **Open-ended evolution**: Bedau & Packard (1991), MODES toolbox (Dolson et al. 2019), Taylor et al. (2016) open problems
- **Niche construction in models**: Odling-Smee et al. (2003), Laland et al. (1999) — rarely implemented in neural-agent systems
- **Epigenetics in ALife**: EpiAL (Sousa & Costa 2010) — only precedent, no neural agents
- **Activity-dependent metabolism**: Attwell & Laughlin (2001), Niven & Laughlin (2008) — not yet applied to ALife neural costs

### 3. System Architecture
- 3.1 World model (toroidal grid, food dynamics, seasonal variation, biomes)
- 3.2 Organisms (energy budget, Gompertz mortality, body size allometry)
- 3.3 NEAT brains (19 inputs, 7 outputs, topology evolution, sparse forward pass)
- 3.4 Activity-dependent metabolism (structure cost + Σ|w·a| per tick)
- 3.5 Sexual reproduction (mate finding, NEAT crossover, twofold cost of sex)
- 3.6 Niche construction (construction grid, food regrowth modification)
- 3.7 Epigenetic inheritance (per-connection methylation, stress response, transgenerational)
- 3.8 OEE measurement (topology hashing, cumulative activity, MODES metrics)

### 4. Experiments
- **Exp 1: OEE validation** — 5 seeds × 200k ticks, measure D(t) growth
  - Hypothesis: D(t) grows unboundedly (Bedau class 2+)
  - Metric: log fit slope, 95% CI across seeds
- **Exp 2: Epigenetics ablation** — ON vs OFF, 5 seeds × 200k ticks
  - Hypothesis: epigenetics improves post-stress recovery
  - Metric: population recovery time after seasonal trough
- **Exp 3: Niche construction ablation** — ON vs OFF, 5 seeds × 200k ticks
  - Hypothesis: niche constructors survive environmental perturbation better
  - Metric: population stability (CV of population over time)
- **Exp 4: Sexual vs asexual** — compare D(t) and adaptation speed
  - Hypothesis: sex accelerates adaptation at cost of population size
  - Metric: D(t) growth rate, Muller's ratchet indicators

### 5. Results
- 5.1 D(t) grows as ~12·ln(t) across all seeds (preliminary data)
- 5.2 Epigenetics effect on stress response
- 5.3 Niche construction effect on ecosystem stability
- 5.4 Sexual reproduction effect on evolutionary rate
- 5.5 Emergent phenomena observed

### 6. Discussion
- Logarithmic D(t) growth: better than most ALife (which plateau) but sublinear
- What drives the log shape? Single-lineage dominance?
- Interaction between epigenetics and niche construction
- Toward consciousness emergence: what's missing?

### 7. Conclusion & Future Work
- NEAT topology evolution + niche construction + epigenetics in one platform
- Open-source: https://github.com/dchavan2192/Digital-Terrarium
- Live demo: https://dchavan2192.github.io/Digital-Terrarium/
- Future: horizontal gene transfer, multicellularity, group-level selection

### References
- Attwell & Laughlin (2001) J Cereb Blood Flow Metab 21(10):1133-45
- Bedau & Packard (1991) Artificial Life II, 431-462
- Dolson et al. (2019) Artificial Life 25(1):50-73
- Heard & Martienssen (2014) Cell 157(1):95-109
- Jablonka & Lamb (2005) Evolution in Four Dimensions, MIT Press
- Laland et al. (1999) PNAS 96:10242-10247
- Maynard Smith (1978) The Evolution of Sex, Cambridge UP
- Muller (1932) Am Nat 66:118-138
- Niven & Laughlin (2008) J Exp Biol 211:1792-1804
- Odling-Smee, Laland & Feldman (2003) Niche Construction, Princeton UP
- Sousa & Costa (2010) Proc ALIFE XII, 131-138
- Stanley & Miikkulainen (2002) Evol Comput 10(2):99-127
- Taylor et al. (2016) Artificial Life 22(3):408-423
- Yaeger (1994) Artificial Life III, 263-298
