//! Neutral Shadow Population for Open-Ended Evolution measurement.
//!
//! The neutral shadow mirrors the real simulation's demographic events (births and
//! deaths) but selects parents and victims at **random** rather than by fitness.
//! Comparing activity statistics supplies a demographic-conditional reference;
//! a finite difference alone does not establish selection or open-ended evolution.
//!
//! Without a shadow there is no null hypothesis: even a population under pure
//! stabilising selection will grow its `mean_cumulative_activity` monotonically,
//! which cannot be distinguished from open-ended evolution (Dolson et al. 2019 §2.1).
//!
//! ## Algorithm (Bedau, Snyder & Packard 1998, §"Echo's Neutral Shadow")
//!
//! 1. At `Simulation::new`, clone the founding genomes into the shadow.
//! 2. Each tick, after the real population's births and deaths are finalized:
//!    - For every real birth, the shadow picks a random parent (without replacement
//!      among the pre-birth population) and applies the identical mutation operator.
//!    - For every real death, the shadow removes a random organism.
//! 3. The shadow's own `ActivityTracker` accumulates topology statistics.
//! 4. `sim.rs` exports `shadow_*` TickStats fields alongside the real run's fields.
//!
//! The shadow **never** reads or writes `Simulation.rng`; it uses its own
//! `ChaCha8Rng` stream so the real run is bit-identical whether the shadow is
//! enabled or not. This is verified by `shadow_does_not_perturb_main_run` in
//! `tests/integration.rs`.
//!
//! Citations:
//! - Bedau, M.A., Snyder, E. & Packard, N.H. (1998) "A Classification of
//!   Long-Term Evolutionary Dynamics" *ALife VI* pp.228-237
//! - Channon, A. (2001) "Passing the ALife Test: Activity statistics classify
//!   evolution in Geb as unbounded" *ECAL 2001* LNCS 2159
//! - Dolson, E. et al. (2019) "The MODES Toolbox" *Artificial Life* 25(1):50-73

use crate::activity::{ActivityStats, ActivityTracker, topology_hash_versioned};
use crate::config::{ActivityTrackingConfig, FruitConfig, MutationConfig};
use crate::genome::{Genome, Innovations};
use crate::math::random_index;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Neutral shadow — a population of genomes that evolves by random drift only.
///
/// The shadow mirrors the real population's demographic structure (size, birth count,
/// death count, mutation operator) but selects parents and victims at random rather
/// than by fitness. Excess activity is conditional on this null's assumptions;
/// it is not by itself evidence of open-ended evolution.
///
/// Only genomes are stored (no brains, positions, or energy): reproduction calls
/// `Genome::reproduce`/`Genome::crossover` unchanged, but `build_brain()` is never
/// called. This keeps memory at ~4–7 MB for a 1500-organism population with ~180
/// connections per genome.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ShadowPopulation {
    /// All shadow genomes (no brains).
    genomes: Vec<Genome>,
    /// Independent RNG — never touches `Simulation::rng`.
    rng: ChaCha8Rng,
    /// Population-wide node-id registry for NEAT crossover in the shadow.
    /// Separate from the real run's `Innovations` to avoid side effects.
    innovations: Innovations,
    /// Mutation config reference (cloned at construction; must stay in sync with
    /// the real run — the shadow uses the *same* mutation operator).
    mutation_cfg: MutationConfig,
    /// Fruit config reference (cloned at construction).
    fruit_cfg: FruitConfig,
    /// Activity tracker for the shadow population.
    tracker: ActivityTracker,
    /// Hash version (must match the real run's `hash_version`).
    hash_version: u32,
    /// Reusable scratch for partial Fisher–Yates parent sampling.
    #[serde(skip)]
    idx_buf: Vec<usize>,
    /// Reusable buffer for this-tick newborns.
    #[serde(skip)]
    newborns: Vec<Genome>,
}

impl ShadowPopulation {
    /// Create a shadow from the founding genomes.
    ///
    /// `seed` is typically `config.world.seed`; `stream` is `config.activity_tracking.shadow_stream`.
    /// Using a different stream from the main RNG (stream 0) and the env RNG (stream 1) ensures
    /// statistical independence (ChaCha stream parameter, Bernstein 2008).
    pub fn new(
        founding_genomes: impl Iterator<Item = Genome>,
        cfg: &ActivityTrackingConfig,
        brain_cfg: &crate::config::BrainConfig,
        mutation_cfg: &MutationConfig,
        fruit_cfg: &FruitConfig,
        seed: u64,
    ) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        rng.set_stream(cfg.shadow_stream);
        ShadowPopulation {
            genomes: founding_genomes.collect(),
            rng,
            innovations: Innovations::new(brain_cfg),
            mutation_cfg: mutation_cfg.clone(),
            fruit_cfg: fruit_cfg.clone(),
            tracker: ActivityTracker::new(cfg.window_size),
            hash_version: cfg.hash_version,
            idx_buf: Vec::new(),
            newborns: Vec::new(),
        }
    }

    /// Mirror the real run's demographic events with random selection.
    ///
    /// `asexual` and `sexual` are the real run's birth counts this tick.
    /// `deaths` is the real run's death count this tick.
    ///
    /// Parents are sampled **without replacement** from the pre-birth population
    /// (a real parent can produce at most one child per tick).
    /// For sexual births, the second parent (mate) is any other random organism.
    pub fn mirror(&mut self, asexual: u64, sexual: u64, deaths: usize) {
        let n0 = self.genomes.len();
        if n0 == 0 {
            return;
        }

        let total_births = (asexual + sexual) as usize;
        self.newborns.clear();

        // Partial Fisher–Yates to sample `total_births` distinct parents from
        // the n0 pre-birth genomes. If more births than organisms, clamp.
        let sample = total_births.min(n0);
        self.idx_buf.resize(n0, 0);
        for i in 0..n0 {
            self.idx_buf[i] = i;
        }
        for k in 0..sample {
            let j = k + random_index(&mut self.rng, n0 - k);
            self.idx_buf.swap(k, j);
        }

        let mut parent_cursor = 0usize;

        // Asexual births: each parent reproduces once
        let asexual_count = (asexual as usize).min(sample);
        for _ in 0..asexual_count {
            let p = self.idx_buf[parent_cursor];
            parent_cursor += 1;
            let child = self.genomes[p].reproduce(
                &self.mutation_cfg,
                &self.fruit_cfg,
                &mut self.innovations,
                &mut self.rng,
            );
            self.newborns.push(child);
        }

        // Sexual births: parent + random mate (may overlap)
        let sexual_count = (sexual as usize).min(sample.saturating_sub(asexual_count));
        for _ in 0..sexual_count {
            let p = self.idx_buf[parent_cursor.min(sample - 1)];
            parent_cursor += 1;
            // Pick a random mate (may be any index including parent)
            let m_idx = random_index(&mut self.rng, n0);
            let a_is_fitter = self.rng.r#gen::<bool>();
            let child = Genome::crossover(
                &self.genomes[p],
                &self.genomes[m_idx],
                a_is_fitter,
                &self.mutation_cfg,
                &self.fruit_cfg,
                &mut self.innovations,
                &mut self.rng,
            );
            self.newborns.push(child);
        }

        self.genomes.append(&mut self.newborns);

        // Deaths: remove random organisms — random removal is neutral selection.
        // Use swap_remove for O(1) and no allocation.
        let kill_count = deaths.min(self.genomes.len());
        for _ in 0..kill_count {
            if self.genomes.is_empty() {
                break;
            }
            let i = random_index(&mut self.rng, self.genomes.len());
            self.genomes.swap_remove(i);
        }
    }

    /// Update the shadow's activity tracker at the given tick.
    ///
    /// Call this in the same stats block as the real tracker, every `window_size` ticks.
    pub fn update_activity(&mut self, tick: u64) {
        let version = self.hash_version;
        self.tracker.update(
            tick,
            self.genomes
                .iter()
                .map(move |g| topology_hash_versioned(g, version)),
        );
    }

    /// Current activity statistics from the shadow population.
    pub fn stats(&self) -> &ActivityStats {
        &self.tracker.stats
    }

    /// Number of organisms in the shadow (must equal the real population size).
    pub fn len(&self) -> usize {
        self.genomes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.genomes.is_empty()
    }

    /// Mean active connections per shadow genome (confound monitor).
    /// If this drifts far from the real run's `mean_connections`, note it.
    pub fn mean_connections(&self) -> f64 {
        if self.genomes.is_empty() {
            return 0.0;
        }
        let total: usize = self
            .genomes
            .iter()
            .map(|g| g.connections.iter().filter(|c| c.enabled).count())
            .sum();
        total as f64 / self.genomes.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ActivityTrackingConfig, BrainConfig, FruitConfig, MutationConfig};

    fn default_mutation_cfg() -> MutationConfig {
        MutationConfig {
            weight_probability: 0.8,
            weight_step: 0.5,
            add_connection_probability: 0.02,
            add_node_probability: 0.01,
            toggle_probability: 0.01,
            prune_probability: 0.02,
            add_recurrent_probability: 0.0,
            trait_probability: 0.05,
        }
    }

    fn default_fruit_cfg() -> FruitConfig {
        FruitConfig::default()
    }

    fn default_brain_cfg() -> BrainConfig {
        BrainConfig {
            inputs: 20,
            hidden: 8,
            outputs: 7,
        }
    }

    fn default_activity_cfg() -> ActivityTrackingConfig {
        ActivityTrackingConfig {
            enabled: true,
            window_size: 1,
            shadow: true,
            shadow_stream: 2,
            lineage_window: 1000,
            hash_version: 2,
        }
    }

    fn make_shadow_from_config(size: usize) -> ShadowPopulation {
        let cfg = default_activity_cfg();
        let brain_cfg = default_brain_cfg();
        let mutation_cfg = default_mutation_cfg();
        let fruit_cfg = default_fruit_cfg();
        let seed = 42;

        // Create minimal genomes (no connections) for testing
        let genomes = (0..size).map(|_| crate::genome::Genome {
            connections: vec![],
            hidden_nodes: vec![],
            num_inputs: 20,
            num_outputs: 7,
            mutation_rate: 0.1,
            mutation_step: 0.5,
            color_hue: 0.0,
            aging_rate: 0.001,
            body_size: 1.0,
            preferred_temp: 0.5,
            diet_preference: 0.5,
        });
        ShadowPopulation::new(genomes, &cfg, &brain_cfg, &mutation_cfg, &fruit_cfg, seed)
    }

    #[test]
    fn population_tracks_births_and_deaths() {
        let mut shadow = make_shadow_from_config(10);
        // 2 asexual births, 0 sexual, 1 death → net +1
        shadow.mirror(2, 0, 1);
        assert_eq!(shadow.len(), 11, "10 + 2 births - 1 death = 11");
    }

    #[test]
    fn population_handles_extinction() {
        let mut shadow = make_shadow_from_config(5);
        // Kill all 5 — should not panic
        shadow.mirror(0, 0, 5);
        assert_eq!(shadow.len(), 0);
        // Mirror with empty population — should not panic
        shadow.mirror(1, 0, 0);
        assert_eq!(shadow.len(), 0); // no parents to reproduce
    }

    #[test]
    fn deterministic_given_same_seed() {
        let mut s1 = make_shadow_from_config(20);
        let mut s2 = make_shadow_from_config(20);
        for tick in 0..10u64 {
            s1.mirror(3, 1, 2);
            s2.mirror(3, 1, 2);
            s1.update_activity(tick);
            s2.update_activity(tick);
        }
        assert_eq!(
            s1.stats().novel_traits,
            s2.stats().novel_traits,
            "shadow must be deterministic"
        );
        assert_eq!(s1.len(), s2.len());
    }

    #[test]
    fn sexual_birth_does_not_panic() {
        let mut shadow = make_shadow_from_config(10);
        // 0 asexual, 3 sexual births, 2 deaths
        shadow.mirror(0, 3, 2);
        assert_eq!(shadow.len(), 11, "10 + 3 - 2 = 11");
    }
}
