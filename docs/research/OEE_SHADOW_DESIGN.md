# Neutral Shadow Model for Terrarium's OEE Claim — Design Note

Status: proposal (research). Nothing here has been implemented or run.
Author role: research scientist. Date: 2026-11-15.
Code read for this note: `crates/sim-core/src/activity.rs`, `sim.rs` (`Simulation::new`, `step()` sections 6 and 6-Die, stats block around lines 1410–1475), `genome.rs` (`reproduce`, `crossover`), `config.rs` (`ActivityTrackingConfig`), `crates/sim-runner/src/run.rs`, `configs/default.toml`.

---

## 1. Why the current claim is not enough

Terrarium reports `mean_cumulative_activity` (labelled "D(t)") and treats its unbounded growth as evidence of open-ended evolution (PROGRESS.md item "Show D(t) grows unboundedly — Terrarium passes the ALife test"). That inference fails. Bedau and colleagues built evolutionary activity statistics to separate adaptive persistence from persistence that happens anyway, and they only interpret activity *after normalising it against a neutral "shadow" run*. A shadow has the same births, deaths and mutation process but chooses who reproduces and who dies at random (Bedau, Snyder & Packard 1998; Channon 2001, 2006). Neutral drift, inheritance and simple persistence build up activity on their own. In Bedau et al.'s Echo runs the raw mean activity was often indistinguishable from the shadow's (their Fig. 3). Without a shadow there is no null hypothesis.

Reading `activity.rs` turns up four more problems that make the current number hard to interpret even before normalisation:

1. **Wrong quantity under the wrong name.** In Bedau's notation D(t) means *diversity* (the number of extant components), not mean activity. More importantly, `mean_cumulative_activity` divides by `traits.len()`, which counts *every topology ever seen, extinct ones included*. Bedau's mean cumulative activity divides by extant diversity: Ā_cum = A_cum / D.
2. **Count-weighted increments grow trivially.** `a_i += count` adds population counts at each window, so one dominant topology held by about N organisms adds about N per window and the mean grows linearly with no novelty at all. Dolson et al. (2019, §2.1) point out that even with presence-only increments, a single component under *stabilising* selection makes mean activity grow without bound. That is why Channon (2003) moved to the median.
3. **The hard cap makes "unbounded diversity" impossible by construction.** Extant D ≤ population ≤ `creature.hard_cap` = 1500, so Bedau's Class 3 (Dolson's 4a/4c), which needs unbounded D, cannot occur. The only open-ended class Terrarium can show is Channon's **4b**: bounded diversity with unbounded per-component (median) activity above the shadow.
4. **The topology hash depends on gene order.** `topology_hash` folds connections in vector order, but `Genome::crossover` sorts connections by (from, to) while asexual `mutate_add_connection` appends to the end. The same wiring can therefore hash to different values depending on lineage history, which inflates `novel_traits` artificially, and more so as `crossover_rate` rises.

---

## 2. Recommended shadow algorithm for Terrarium

### 2.1 Principle (from the literature)

Bedau, Snyder & Packard (1998, "Echo's Neutral Shadow" section), verified against the PDF:
- A shadow "creature" has no position or energy and never interacts. It only has a genotype plus its birth and death.
- "The timing and number of birth and death events in the neutral shadow are directly copied from those in the normal Echo run, as is the neutral shadow's mutation rate."
- Each birth in the real run triggers one birth in the shadow from a parent chosen **uniformly at random**. Each death in the real run kills one shadow creature chosen **uniformly at random**.
- Changes in the real run drive the shadow, but the shadow never feeds back into the real run.

Channon (2001) states the general rule: mirror the real run in every detail, except that wherever selection operates in the real run, use random selection in the shadow.

### 2.2 What the shadow holds

```
ShadowPopulation {
    genomes:  Vec<Genome>,        // full NEAT genomes, no Brain, no position/energy
    rng:      ChaCha8Rng,         // independent stream, see 2.3
    tracker:  ActivityTracker,    // a second, independent tracker
    idx_buf:  Vec<usize>,         // reusable scratch for sampling parents without replacement
    newborns: Vec<Genome>,        // reusable scratch for this tick's births
}
```

- **Full genomes, no brains.** Reproduction reuses `Genome::reproduce` and `Genome::crossover` unchanged, so the shadow keeps the *exact* mutation operator, including the 2% add-connection, 1% add-node, 1% toggle and 4% prune rates and the evolvable `mutation_rate` and `mutation_step`. `build_brain()` is never called and nothing is sensed or thought.
- Hashes alone are not enough: structural mutation depends on the current connection list, so the shadow needs the genome to produce the next topology.
- Methylation and epigenetic noise are skipped. The topology hash ignores them, and skipping them saves RNG draws.

### 2.3 Determinism: separate RNG stream

- Seed: `let mut rng = ChaCha8Rng::seed_from_u64(config.world.seed); rng.set_stream(config.activity_tracking.shadow_stream);` (default stream `1`, since the main RNG uses stream 0). The ChaCha stream parameter gives a statistically independent keystream from the same seed. *Unverified: confirm that `set_stream` exists on `ChaCha8Rng` in the pinned `rand_chacha 0.3` with `cargo doc -p rand_chacha`. If it does not, use `seed_from_u64(seed ^ 0x5348_4144_4F57_0001)`.*
- The shadow **never touches `self.rng`**. It only reads counters that `step()` already computes: `asexual_births`, `sexual_births` and `deaths`. So the real run is bit-identical with `shadow = true` or `false`. This becomes a test (§4.6).
- The initial shadow population is `organisms.iter().map(|o| o.genome.clone())`, taken right after the spawn loop in `Simulation::new`. Cloning uses no RNG.

### 2.4 Per-tick mirroring (exact order matches `step()`)

In the real run, section 6 (Reproduce) appends offspring, and section 6-Die then iterates over **all** organisms, newborns included, before `retain`. The shadow does the same:

```
mirror(asexual: u64, sexual: u64, deaths: usize):
    n0 = genomes.len()
    // Real parents are distinct pre-existing organisms (one child per parent per tick),
    // so sample parents WITHOUT replacement from the n0 pre-birth genomes.
    partial Fisher–Yates on idx_buf[0..n0] using shadow rng
    for each asexual birth k:  newborns.push(genomes[idx_buf[k]].reproduce(&mut rng))
    for each sexual birth k:   p = idx_buf[next]; m = uniform random index != p   // a mate may also be a parent
                               a_is_fitter = rng.gen::<bool>()                      // fitness_bias replaced by coin flip
                               newborns.push(Genome::crossover(&genomes[p], &genomes[m], a_is_fitter, &mut rng))
    genomes.append(&mut newborns)
    for _ in 0..deaths:  i = rng.gen_range(0..genomes.len()); genomes.swap_remove(i)
```

Invariant: `shadow.genomes.len() == organisms.len()` after every tick (this becomes a test). If the real population goes extinct, so does the shadow.

Sampling without replacement is used because a real parent can produce at most one child per tick (the loop at sim.rs:938). If births exceed `n0` (they cannot today), clamp and log.

### 2.5 Activity measurement in the shadow

The shadow gets its own `ActivityTracker`, updated in the same stats block and on the same window (`window_size`) as the real tracker, with the same (fixed) hash.

### 2.6 Cost on a Raspberry Pi 5 (estimates, not measured)

| Item | Estimate | Basis |
|---|---|---|
| `ConnectionGene` size | 24 B | u16+u16+bool+2×f64, aligned |
| Genome at ~180 connections | ~4.5 KB | 180×24 + hidden nodes + header |
| Shadow genomes at cap 1500 | **~7 MB** | likely smaller, because neutral pruning (4% > 2% add) shrinks shadow genomes |
| Tracker map (per tracker) | ~50 B/trait; 100k traits ≈ 5 MB | grows with *traits ever*; the shadow usually has more genotypes (Bedau et al. 1998, Fig. 1 discussion). Mitigate with eviction (§4.1). |
| CPU per shadow birth | clone + one RNG draw per connection ≈ 1–3 µs | `reproduce` iterates all connections |
| CPU per tick | births ≈ deaths ≈ N / mean lifespan ≈ 3–30/tick → **≤ ~0.1 ms** | |
| Hashing per window | 1500 × 180 connections ≈ 0.3 M mix ops ≈ <1 ms per window, ×2 trackers | amortised over 100 ticks |

A real tick with 1500 brain forward passes, vision rays and so on takes several milliseconds on a Pi 5, so expected overhead is **≈1–3% CPU and <20 MB RAM**. Confirm with one short timing run on the Pi (shadow on vs off, same seed).

### 2.7 Config keys (`configs/default.toml` + `ActivityTrackingConfig`)

```toml
[activity_tracking]
enabled = true
window_size = 100
shadow = false            # Bedau/Channon neutral shadow (set true for OEE experiments)
shadow_stream = 1         # ChaCha stream id for the shadow RNG; must differ from main (0)
hash_version = 2          # 1 = legacy order-dependent hash, 2 = order-independent (recommended)
new_activity_min = 0      # a0 for online A_new; 0/0 = disabled (compute post hoc instead)
new_activity_max = 0      # a1
evict_extinct = true      # move extinct traits into a histogram to bound memory
histogram_bins = 32       # log2 bins for the component activity distribution C(t,a)
```

All fields use `#[serde(default)]` so old configs still parse. `validate()` should reject `shadow = true` with `enabled = false` and `shadow_stream == 0`.

---

## 3. TickStats fields and the analysis

### 3.1 Definitions (per sampling window k, tick t_k = k·w)

Component i is a topology hash. n_i(t_k) is the number of living organisms with hash i.

| Symbol | Formula | Notes |
|---|---|---|
| a_i(t) (count) | Σ_{t_k ≤ t} n_i(t_k) | current Terrarium increment |
| p_i(t) (presence) | Σ_{t_k ≤ t} 1[n_i(t_k) > 0] | Bedau et al. 1998 eq. (1); **recommended primary** |
| D(t) | #{i : n_i(t) > 0} | diversity (extant) = existing `active_traits` |
| A_cum(t) | Σ_{i extant} p_i(t) | total cumulative activity, eq. (4) |
| Ā_cum(t) | A_cum(t) / D(t) | mean cumulative activity, eq. (6) |
| M(t) | median_{i extant} p_i(t) | Channon (2003) component-normalised / median form |
| A_new(t) | (1/D) Σ_{i extant, a0 ≤ p_i ≤ a1} p_i(t) | new activity, eq. (7) |

Choosing a0 and a1 (Bedau et al. 1998): collapse the component activity distributions of real and shadow over all time, then normalise each by its total count. Let a′ be the lowest activity at which the two curves cross, and a_max the highest activity at which either is positive. Then a0, a1 = a′ ± 0.05·(a_max − a′). This needs the histogram output (§3.3), so A_new is computed **post hoc** by default.

### 3.2 New `TickStats` fields (sim.rs, after `mean_cumulative_activity`)

Real run:
- `total_traits_ever: u64` (already computed in `ActivityStats`, not exported yet)
- `total_cumulative_activity: f64` (A_cum, presence-based)
- `mean_extant_activity: f64` (Ā_cum, Bedau)
- `median_extant_activity: f64` (M, Channon)
- `new_activity: f64` (A_new if a0/a1 configured, else 0)
- `mean_mutation_rate: f64` (confound monitor)

Shadow (all 0 when `shadow = false`):
- `shadow_population: usize` (must equal `population`; acts as a sanity column)
- `shadow_novel_traits: u64`, `shadow_active_traits: u64`, `shadow_total_traits_ever: u64`
- `shadow_total_cumulative_activity: f64`, `shadow_mean_extant_activity: f64`, `shadow_median_extant_activity: f64`, `shadow_new_activity: f64`
- `shadow_mean_cumulative_activity: f64` (legacy definition, so old plots can be compared directly)
- `shadow_mean_connections: f64`, `shadow_mean_mutation_rate: f64` (confound monitors)

Keep the existing `mean_cumulative_activity` column for backward compatibility, but fix its doc comment: it is not D(t).

### 3.3 Separate histogram output (sim-runner)

The serde CSV writer cannot carry arrays, so `run.rs` should write `activity_hist_seed{S}.csv` in long format with columns `tick, run (real|shadow), bin, count`. Bin b covers p ∈ [2^b, 2^{b+1}). Rows are written every `log_interval`, from the getter `Simulation::activity_histograms()`. Extinct components are added to a lifetime histogram at eviction, which yields the collapsed distribution needed for a′.

### 3.4 Analysis (new `analysis/analyze_shadow.py`, to be written later)

1. **Excess (normalised) statistics**, as in Bedau et al. 1998 ("we normalize Echo's activity data by subtracting the neutral shadow's"):
   - X_D(t) = D^real − D^shadow
   - X_mean(t) = Ā_cum^real − Ā_cum^shadow
   - X_med(t) = M^real − M^shadow
   - X_new(t) = A_new^real − A_new^shadow
   - X_novel(t) = novel^real − novel^shadow (per window)
2. **Positivity** (Bedau's Appendix eq. 10: f is positive iff lim (1/t)∫f dt > 0). Operational test: compute the time-average of X over the second half of each run, then take a 95% bootstrap CI across ≥ 5 seeds. "Positive" means the CI excludes 0.
3. **Boundedness.** Bedau's eq. 9 (lim sup f / t > 0) cannot be checked on finite data. Following Dolson et al. (2019, §3.1.3) and Wiser, Ribeck & Lenski (2013), fit competing models to M^real(t) and to D^real(t):
   - bounded: hyperbolic y = a·t/(b + t), logistic
   - unbounded: power law y = a·t^b, linear
   Compare with AIC/BIC per seed. Report the fraction of seeds where an unbounded model wins.
4. **Classification** (Dolson et al. 2019, Table 1, which merges Bedau 1998, Skusa & Bedau 2002 and Channon):

| Class | Ā_cum / M (excess) | A_new (excess) | D | Label |
|---|---|---|---|---|
| 1 | zero | zero | bounded | none |
| 2 | unbounded | zero | bounded | "uncreative" (ecological persistence only) |
| 3 | bounded | positive | bounded | bounded adaptive |
| 4a | bounded | positive | unbounded | unbounded (diversity-driven) |
| 4b | **unbounded (median)** | positive | bounded | unbounded (per-component) |
| 4c | unbounded | positive | unbounded | unbounded (both) |

   Because of the hard cap, Terrarium's target claim is **"Class 4b against a neutral shadow, using median activity."** Dolson et al. warn that 4b can still come from stabilising selection on more than 50% of components. Pair it with the MODES change metric or persistence filter (§5, item 2).
5. **Control checks:** (i) shadow-of-shadow: excess statistics of a shadow against itself must be ≈ 0, which can be emulated by running with selection weakened (e.g., `reproduction.threshold` tiny and `hard_cap` binding). (ii) Report the confound monitors: if `shadow_mean_mutation_rate` drifts far from the real value, note the "shadow drifts from the real run" critique (raised in the Channon/Geb line of work) and optionally add `shadow_mutation = "matched"` in a v2.

---

## 4. Implementation plan (file / function level)

### 4.1 `crates/sim-core/src/activity.rs`
- `topology_hash` → `topology_hash_v2`: an order-independent combine. For each connection, compute `splitmix64((from as u64) << 17 | (to as u64) << 1 | enabled as u64)` and `wrapping_add` it into an accumulator; then mix in `hidden_nodes.len()`. It needs no allocation and is commutative, so it fixes the crossover-order issue. Keep the v1 function and select with `hash_version`.
- `ActivityRecord`: add `presence: u64` next to `cumulative_count`, and make `last_seen` live (drop its `#[allow(dead_code)]`).
- `ActivityTracker`:
  - make `active_this_window` a struct field (`clear()` each window) to stop the per-window `HashMap` re-allocation, and add a `scratch: Vec<u64>` for the median (`select_nth_unstable`)
  - in `update()`: increment `presence` by 1 for extant hashes; compute `A_cum`, `Ā_cum`, `M` over **extant** hashes only; compute `A_new` if a band is configured
  - eviction: when `evict_extinct` is set and `last_seen < tick`, add the record's final `presence` to `lifetime_hist[log2 bin]` and remove it from the map. Keep a `u64` counter so `total_traits_ever` stays correct. Note: `HashMap::retain` iteration order is random (SipHash), but every quantity derived from it (sums, counts, histogram bins, median of a multiset) is order-independent, so determinism holds.
  - `extant_hist()` and `lifetime_hist()` getters
- `ActivityStats`: add `total_cumulative_activity`, `mean_extant_activity`, `median_extant_activity`, `new_activity`.
- Unit tests: extant-only mean, median, eviction preserving `total_traits_ever`, and hash order-invariance (permute connections, expect the same hash).

### 4.2 New `crates/sim-core/src/shadow.rs` (+ `pub mod shadow;` in `lib.rs`)
- `pub struct ShadowPopulation` as in §2.2.
- `pub fn new<'a>(genomes: impl Iterator<Item = &'a Genome>, cfg: &ActivityTrackingConfig, seed: u64) -> Self`
- `pub fn mirror(&mut self, asexual: u64, sexual: u64, deaths: usize)`, as in §2.4.
- `pub fn update_activity(&mut self, tick: u64)`, which feeds `genomes.iter().map(hash)` to its tracker.
- `pub fn mean_connections()`, `mean_mutation_rate()`, `len()`.
- Unit tests: population accounting; that it never consumes an externally passed RNG (only its own); same seed gives the same shadow hashes.

### 4.3 `crates/sim-core/src/config.rs`
- Extend `ActivityTrackingConfig` (line 579) with the §2.7 fields (`#[serde(default = ...)]`) and update `impl Default` (line 659).
- In `validate()` (line 707), add the checks from §2.7.

### 4.4 `crates/sim-core/src/sim.rs`
- `struct Simulation` (around line 66): add `shadow: Option<crate::shadow::ShadowPopulation>`.
- `Simulation::new` (after the spawn loop, around line 260): build the shadow if `activity_tracking.enabled && shadow`, from `organisms.iter().map(|o| &o.genome)`.
- `step()`: right after `self.organisms.retain(|o| o.alive);` (line 1072), call `if let Some(s) = &mut self.shadow { s.mirror(asexual_births, sexual_births, deaths); }`. The locals `asexual_births`, `sexual_births` (declared line 931) and `deaths` are in scope there. Do **not** move any `self.rng` call.
- Stats block (around line 1410): after the real `activity_tracker.update(...)`, call `shadow.update_activity(self.tick)`; then populate the new `TickStats` fields (struct at around line 151, literal at around line 1471).
- Fix the `mean_cumulative_activity` doc comment (line 150).

### 4.5 `configs/default.toml` (line 239)
Add the §2.7 keys (with `shadow = false` in the default; OEE experiment configs set it to `true`).

### 4.6 `crates/sim-core/tests/integration.rs` (run on the Pi, not locally)
1. `shadow_does_not_perturb_main_run`: same seed, shadow off vs on, 2,000 ticks. Every non-shadow `TickStats` field must be equal at every tick.
2. `shadow_is_deterministic`: two runs with shadow on give identical shadow fields.
3. `shadow_population_tracks_real`: `shadow_population == population` every tick.
4. `shadow_with_sexual_reproduction`: `reproduction.sexual = true` produces no panic and keeps the invariant.

### 4.7 `crates/sim-runner/src/run.rs` (line 65 / 71 / 76) and `sweep.rs`
Write the histogram CSV alongside each `wtr.serialize`. Note: `sweep.rs` currently has uncommitted local changes, so coordinate before editing it.

### 4.8 `analysis/`
Add `analyze_shadow.py` (§3.4), extend `plot_oee.py` with real-vs-shadow overlays, and plot excess statistics with seed CIs.

Estimated effort: about 1 day of Rust and 0.5 day of Python, plus one Pi validation run (5 seeds × 200k ticks, shadow on).

---

## 5. Other novelty opportunities (ranked)

1. **Mechanism ablation scored by excess activity (highest value, low cost).** Terrarium already has sexual reproduction, niche construction, epigenetic inheritance, pheromones, droughts and seasons behind config flags. A factorial sweep scored by *X_med* and *X_new* against a neutral shadow answers "which ecological and inheritance mechanisms produce non-neutral open-endedness?" inside one platform. Channon (2006) attributes Geb's unbounded dynamics to biotic selection and coevolution, and Taylor et al. (2016) ask for hallmarks of OEE to be separated from hypothesised mechanisms. This would be a direct, shadow-controlled test of mechanism hypotheses, and to our knowledge it is rare. This looks like the core ALIFE 2027 paper.
2. **Shadow vs persistence filter: a methods comparison.** Dolson et al. (2019, §3.1.1) present the MODES persistence filter (lineages that have descendants t generations later) as an alternative to shadows and note that the two filter neutral processes differently, but they do not compare them empirically in a single system. Terrarium already records `parent_id`, `generation` and `root_ancestor_id`, so both filters can run on the same data, and the MODES change, novelty and ecology metrics can be computed over topologies as well. That is a methodological contribution, and it also addresses the stabilising-selection caveat on Class 4b.
3. **Population-size scaling of diversity and complexity under a cap.** Because D ≤ `hard_cap`, sweep the cap (e.g., 500 → 4000) and test whether excess diversity and the maximum complexity of persistent lineages scale with N or saturate. This connects to formal definitions of unbounded evolution in Adams et al. (2017) and to Channon's later claim that maximum individual complexity is indefinitely scalable in Geb (Channon 2019). It needs more Pi compute.

---

## References

Verified in this session (primary PDF or publisher/bibliographic records):
- Bedau, M. A., Snyder, E., & Packard, N. H. (1998). A classification of long-term evolutionary dynamics. In C. Adami, R. Belew, H. Kitano & C. Taylor (Eds.), *Artificial Life VI* (pp. 228–237). MIT Press. **Read in full.** The PDF header gives pp. 228–237, but its own title page says pp. 189–198, so check the page range before citing. Class definitions, eq. (1)–(10), the shadow construction and the a0/a1 rule all come from this PDF.
- Rechtsteiner, A., & Bedau, M. A. (1999). A generic neutral model for quantitative comparison of genotypic evolutionary activity. *ECAL'99*, LNCS 1674, pp. 109–118. Springer. (A companion paper appears in GECCO 1999, pp. 1366–1373.) Bibliographic record only.
- Channon, A. (2001). Passing the ALife test: Activity statistics classify evolution in Geb as unbounded. *ECAL 2001*, LNCS 2159. Springer. doi:10.1007/3-540-44811-X_45. Abstract-level only; the "random selection wherever selection operates" quote is from the abstract.
- Channon, A. (2003). Improving and still passing the ALife test: Component-normalised activity statistics classify evolution in Geb as unbounded. *Artificial Life VIII*. MIT Press. The median and component normalisation details come from Dolson et al.'s summary; full text not read.
- Channon, A. (2006). Unbounded evolutionary dynamics in a system of agents that actively process and transform their environment. *Genetic Programming and Evolvable Machines*, 7(3), 253–281. doi:10.1007/s10710-006-9009-3.
- Dolson, E. L., Vostinar, A. E., Wiser, M. J., & Ofria, C. (2019). The MODES toolbox: Measurements of open-ended dynamics in evolving systems. *Artificial Life*, 25(1), 50–73. doi:10.1162/artl_a_00280. **Sections 2–3.1 read** (Table 1, persistence filter, boundedness via model fitting).
- Taylor, T., Bedau, M., Channon, A., Ackley, D., Banzhaf, W., et al. (2016). Open-ended evolution: Perspectives from the OEE workshop in York. *Artificial Life*, 22(3), 408–423. doi:10.1162/ARTL_a_00210.
- Droop, A., & Hickinbotham, S. (2012). A quantitative measure of non-neutral evolutionary activity for systems that exhibit intrinsic fitness. *Artificial Life 13*, pp. 45–52. MIT Press. An alternative to shadows for intrinsic-fitness systems; abstract not retrieved.

Cited from memory or secondary sources, **not verified this session**:
- Bedau, M. A., McCaskill, J. S., Packard, N. H., Rasmussen, S., et al. (2000). Open problems in artificial life. *Artificial Life*, 6(4), 363–376.
- Skusa, A., & Bedau, M. A. (2002). Towards a comparison of evolutionary creativity in biological and cultural evolution. *Artificial Life VIII*. (Known only via Dolson et al. Table 1.)
- Wiser, M. J., Ribeck, N., & Lenski, R. E. (2013). Long-term dynamics of adaptation in asexual populations. *Science*, 342, 1364–1367.
- Adams, A., Zenil, H., Davies, P. C. W., & Walker, S. I. (2017). Formal definitions of unbounded evolution and innovation reveal universal mechanisms for open-ended evolution in dynamical systems. *Scientific Reports*, 7. (Title confirmed via PMC; volume/article number from memory.)
- Channon, A. (2019). Maximum individual complexity is indefinitely scalable in Geb. *Artificial Life*, 25(2), 134–144.
- Andras, P. (2017). ECAL 2017 paper reporting cellular-automata worlds with more evolutionary activity than a random shadow world (Keele eprint 3673). Exact title not confirmed.
- Plantec et al. (2025), Flow-Lenia (arXiv:2506.08569): a recent ALife system applying activity statistics. Seen only in a snippet.

I could not find a clearly relevant ALIFE 2020–2026 paper proposing a *new* neutral model for OEE in this session's searches. The shadow and persistence-filter pair (Bedau 1998 / Dolson 2019) still appears to be the standard, which supports novelty opportunity 2.
