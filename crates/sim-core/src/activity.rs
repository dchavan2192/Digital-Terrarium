//! Evolutionary Activity Statistics — measuring open-ended evolution.
//!
//! Tracks structural novelty and cumulative activity to determine whether
//! the simulation produces genuinely open-ended evolution (unbounded novelty)
//! or converges to a fixed set of strategies.
//!
//! ## Metric definitions
//!
//! Component i is a topology hash. The simulation samples every `window_size` ticks.
//!
//! * `p_i(t)` = presence = Σ_{k: t_k ≤ t} 1[n_i(t_k) > 0]
//!   Each sampling window where topology i is alive contributes exactly 1.
//!   (Bedau, Snyder & Packard 1998, eq. 1 — **presence**, not count-weighted.)
//! * `D(t)` = diversity = #{i : n_i(t) > 0} = `active_traits`
//! * `A_cum(t)` = Σ_{i extant} p_i(t) = `total_cumulative_activity`
//! * `Ā_cum(t)` = A_cum / D = `mean_cumulative_activity` (Bedau mean)
//! * `M(t)` = median_{i extant} p_i(t) = `median_extant_activity` (Channon 2003)
//!
//! Count-weighted increments (`+= n`) are scientifically incorrect: a dominant
//! topology held by N organisms gains N× more "activity" than a singleton,
//! inflating the metric without novelty (Dolson et al. 2019 §2.1).
//!
//! Citations:
//! - Bedau, M.A. & Packard, N.H. (1991) "Measurement of Evolutionary Activity,
//!   Teleology, and Life" *ALife II* pp.431-461
//! - Bedau, M.A., Snyder, E. & Packard, N.H. (1998) "A Classification of
//!   Long-Term Evolutionary Dynamics" *ALife VI* pp.228-237
//! - Channon, A. (2003) "Improving and still passing the ALife test:
//!   Component-normalised activity statistics" *Artificial Life VIII*
//! - Dolson, E. et al. (2019) "The MODES Toolbox" *Artificial Life* 25(1):50-73

use crate::genome::Genome;
use std::collections::HashMap;

/// Topology hash selected by `[activity_tracking] hash_version`
/// (2 = order-independent, the default; 1 = legacy order-dependent).
#[inline]
pub fn topology_hash_versioned(genome: &Genome, version: u32) -> u64 {
    if version == 1 {
        topology_hash_legacy(genome)
    } else {
        topology_hash(genome)
    }
}

/// SplitMix64 finalizer — strong 64-bit avalanche so a commutative sum of
/// per-gene hashes does not collide trivially (Steele, Lea & Flood 2014).
#[inline(always)]
fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// Order-independent hash of a genome's connection topology (ignoring weights).
///
/// Each gene (from, to, enabled) is hashed independently and the results are
/// combined with a commutative wrapping sum, so two genomes with identical wiring
/// hash identically regardless of gene order. The hidden-node count is mixed in
/// last. Zero allocation. Two organisms with the same wiring but different weights
/// are the same "trait."
pub fn topology_hash(genome: &Genome) -> u64 {
    let mut acc: u64 = 0;
    for c in &genome.connections {
        let key = ((c.from as u64) << 32) | ((c.to as u64) << 1) | (c.enabled as u64);
        acc = acc.wrapping_add(mix64(key));
    }
    mix64(acc ^ mix64(0xcbf29ce484222325 ^ genome.hidden_nodes.len() as u64))
}

/// Legacy (pre-Phase-7.5) FNV-1a chain over genes in storage order.
/// Order-dependent: identical wiring stored in a different order hashes differently.
pub fn topology_hash_legacy(genome: &Genome) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for c in &genome.connections {
        // Hash (from, to, enabled) — structural identity
        hash ^= c.from as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
        hash ^= c.to as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        hash ^= c.enabled as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    // Also hash hidden node count for topological identity
    hash ^= genome.hidden_nodes.len() as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    hash
}

/// Record for a single structural trait (topology).
/// `presence` counts the number of sampling windows where this topology
/// was alive in at least one organism. This is Bedau et al. (1998) eq.(1).
#[derive(serde::Serialize, serde::Deserialize)]
struct ActivityRecord {
    first_seen: u64,
    /// Presence: number of sampling windows where this topology was alive.
    /// Bedau eq.(1): p_i(t) = Σ_{k} 1[n_i(t_k) > 0].
    presence: u64,
    last_seen: u64,
}

/// Tracks evolutionary activity statistics over time.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ActivityTracker {
    traits: HashMap<u64, ActivityRecord>,
    window_size: u64,
    last_update_tick: u64,
    /// Scratch buffer for median computation; reused across windows (no per-window alloc).
    #[serde(skip)]
    scratch: Vec<u64>,
    /// Stats from the most recent update window.
    pub stats: ActivityStats,
}

/// Snapshot of activity metrics computed each window.
///
/// All activity quantities use **presence**-based increments (each extant
/// topology contributes +1 per window, regardless of population size).
/// This is Bedau, Snyder & Packard (1998) eq.(1).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ActivityStats {
    /// New structural topologies seen this window.
    pub novel_traits: u64,
    /// Distinct topologies alive right now — Bedau's D(t).
    pub active_traits: u64,
    /// A_cum(t) — total presence summed over currently extant topologies.
    /// Bedau, Snyder & Packard (1998) eq.(4).
    pub total_cumulative_activity: u64,
    /// Ā_cum(t) = A_cum / D — mean presence over extant topologies.
    /// Bedau, Snyder & Packard (1998) eq.(6). Grows when long-lived topologies
    /// persist, but only above what the neutral shadow shows (see shadow.rs).
    pub mean_cumulative_activity: f64,
    /// M(t) — median presence across extant topologies.
    /// Channon (2003): the median is more robust than the mean because a few
    /// extremely persistent topologies cannot inflate it.
    pub median_extant_activity: f64,
    /// Total distinct traits ever observed.
    pub total_traits_ever: u64,
}

impl ActivityTracker {
    pub fn new(window_size: u64) -> Self {
        ActivityTracker {
            traits: HashMap::new(),
            window_size,
            last_update_tick: 0,
            scratch: Vec::new(),
            stats: ActivityStats::default(),
        }
    }

    /// Update activity tracking. Call every tick; only does real work every window_size ticks.
    pub fn update(&mut self, tick: u64, genomes: impl Iterator<Item = u64>) {
        if tick < self.last_update_tick + self.window_size && tick > 0 {
            return;
        }
        self.last_update_tick = tick;

        // Collect extant topology hashes and deduplicate.
        // Using a temporary HashMap so we only visit extant traits.
        let mut active_this_window: HashMap<u64, ()> = HashMap::new();
        for hash in genomes {
            active_this_window.entry(hash).or_insert(());
        }

        let mut novel = 0u64;
        for &hash in active_this_window.keys() {
            match self.traits.get_mut(&hash) {
                Some(record) => {
                    // Presence-based: +1 per window where topology is alive,
                    // regardless of how many organisms carry it.
                    record.presence += 1;
                    record.last_seen = tick;
                }
                None => {
                    self.traits.insert(
                        hash,
                        ActivityRecord {
                            first_seen: tick,
                            presence: 1,
                            last_seen: tick,
                        },
                    );
                    novel += 1;
                }
            }
        }

        let total_traits_ever = self.traits.len() as u64;
        let active = active_this_window.len() as u64;

        // A_cum(t) and Ā_cum(t) — sum and mean presence over extant traits only.
        // Bedau, Snyder & Packard (1998) eqs.(4) and (6).
        let total_presence: u64 = active_this_window
            .keys()
            .map(|h| self.traits.get(h).map_or(0, |r| r.presence))
            .sum();
        let mean_presence = if active > 0 {
            total_presence as f64 / active as f64
        } else {
            0.0
        };

        // M(t) — median presence across extant traits (Channon 2003).
        let median_presence = if active > 0 {
            self.scratch.clear();
            self.scratch.extend(
                active_this_window
                    .keys()
                    .map(|h| self.traits.get(h).map_or(0, |r| r.presence)),
            );
            let mid = self.scratch.len() / 2;
            // select_nth_unstable is O(n) average, no allocation.
            let (_, &mut median_val, _) = self.scratch.select_nth_unstable(mid);
            median_val as f64
        } else {
            0.0
        };

        self.stats = ActivityStats {
            novel_traits: novel,
            active_traits: active,
            total_cumulative_activity: total_presence,
            mean_cumulative_activity: mean_presence,
            median_extant_activity: median_presence,
            total_traits_ever,
        };
    }
}

impl ActivityTracker {
    pub(crate) fn validate_checkpoint(&self, tick: u64, window_size: u64) -> Result<(), String> {
        if self.window_size == 0
            || self.window_size != window_size
            || self.last_update_tick > tick
            || !self.stats.mean_cumulative_activity.is_finite()
            || self.stats.active_traits > self.stats.total_traits_ever
            || self.stats.total_traits_ever != self.traits.len() as u64
            || self
                .traits
                .values()
                .any(|r| r.first_seen > r.last_seen || r.last_seen > tick)
        {
            return Err("invalid checkpoint activity tracker".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_novel_traits() {
        let mut tracker = ActivityTracker::new(1);
        // First tick: 3 distinct traits
        tracker.update(0, vec![100, 200, 300, 100].into_iter());
        assert_eq!(tracker.stats.novel_traits, 3);
        assert_eq!(tracker.stats.active_traits, 3);
        assert_eq!(tracker.stats.total_traits_ever, 3);

        // Second tick: 1 new trait, 2 old
        tracker.update(1, vec![100, 200, 400].into_iter());
        assert_eq!(tracker.stats.novel_traits, 1);
        assert_eq!(tracker.stats.active_traits, 3);
        assert_eq!(tracker.stats.total_traits_ever, 4);
    }

    #[test]
    fn presence_not_count_weighted() {
        let mut tracker = ActivityTracker::new(1);
        // 50 organisms all with hash 100 — should count as presence=1, not 50
        tracker.update(0, vec![100; 50].into_iter());
        assert_eq!(tracker.stats.total_cumulative_activity, 1);
        assert_eq!(tracker.stats.active_traits, 1);
        assert!((tracker.stats.mean_cumulative_activity - 1.0).abs() < 1e-12);
    }

    #[test]
    fn presence_grows_per_window() {
        let mut tracker = ActivityTracker::new(1);
        tracker.update(0, vec![100, 100, 100].into_iter());
        let p0 = tracker.stats.mean_cumulative_activity; // presence=1 → mean=1.0
        tracker.update(1, vec![100, 100, 100].into_iter());
        let p1 = tracker.stats.mean_cumulative_activity; // presence=2 → mean=2.0
        assert!(p1 > p0, "presence should grow: {} > {}", p1, p0);
        assert!((p0 - 1.0).abs() < 1e-12);
        assert!((p1 - 2.0).abs() < 1e-12);
    }

    #[test]
    fn cumulative_activity_ignores_extinct_traits() {
        let mut tracker = ActivityTracker::new(1);
        // Trait 100 alive (with 50 organisms), then extinct.
        tracker.update(0, vec![100; 50].into_iter());
        tracker.update(1, vec![200].into_iter());
        // Only trait 200 (presence=1) is extant.
        assert_eq!(tracker.stats.active_traits, 1);
        assert_eq!(tracker.stats.total_cumulative_activity, 1);
        assert!((tracker.stats.mean_cumulative_activity - 1.0).abs() < 1e-12);
    }

    #[test]
    fn window_skips_intermediate_ticks() {
        let mut tracker = ActivityTracker::new(10);
        tracker.update(0, vec![100].into_iter());
        assert_eq!(tracker.stats.novel_traits, 1);
        // Tick 5: should be skipped (window_size=10)
        tracker.update(5, vec![200].into_iter());
        assert_eq!(tracker.stats.novel_traits, 1); // unchanged
        // Tick 10: should update
        tracker.update(10, vec![200].into_iter());
        assert_eq!(tracker.stats.novel_traits, 1); // 1 new trait this window
        assert_eq!(tracker.stats.total_traits_ever, 2);
    }

    #[test]
    fn median_correct() {
        let mut tracker = ActivityTracker::new(1);
        // 3 traits, each alive for increasing windows: 1, 2, 3
        // After tick 0: all three alive, presence = [1, 1, 1], median = 1
        tracker.update(0, vec![1, 2, 3].into_iter());
        assert!((tracker.stats.median_extant_activity - 1.0).abs() < 1e-12);
        // After tick 1: only 2 and 3 alive — 1 goes extinct, presence = [2, 2]
        tracker.update(1, vec![2, 3].into_iter());
        assert!((tracker.stats.median_extant_activity - 2.0).abs() < 1e-12);
        // After tick 2: only 3 alive — presence = [3], median = 3
        tracker.update(2, vec![3].into_iter());
        assert!((tracker.stats.median_extant_activity - 3.0).abs() < 1e-12);
    }

    #[test]
    fn hash_order_independent() {
        // Two genomes with the same topology in different connection orders must
        // produce the same hash (topology_hash is commutative).
        use crate::genome::{ConnectionGene, HiddenNode};
        let make_genome = |connections: Vec<ConnectionGene>| crate::genome::Genome {
            connections,
            hidden_nodes: vec![HiddenNode { id: 3, depth: 0.5 }],
            num_inputs: 2,
            num_outputs: 1,
            mutation_rate: 0.1,
            mutation_step: 0.1,
            color_hue: 0.0,
            aging_rate: 0.001,
            body_size: 1.0,
            preferred_temp: 0.5,
            diet_preference: 0.5,
        };
        let c1 = ConnectionGene {
            from: 0,
            to: 3,
            weight: 1.0,
            enabled: true,
            methylation: 0.0,
            recurrent: false,
        };
        let c2 = ConnectionGene {
            from: 1,
            to: 3,
            weight: -0.5,
            enabled: false,
            methylation: 0.0,
            recurrent: false,
        };
        let g_ab = make_genome(vec![c1, c2]);
        let g_ba = make_genome(vec![c2, c1]);
        assert_eq!(
            topology_hash(&g_ab),
            topology_hash(&g_ba),
            "hash must be order-independent"
        );
        // Different topology should hash differently (high probability)
        let c3 = ConnectionGene {
            from: 1,
            to: 3,
            weight: -0.5,
            enabled: true, // enabled differs from c2
            methylation: 0.0,
            recurrent: false,
        };
        let g_diff = make_genome(vec![c1, c3]);
        assert_ne!(
            topology_hash(&g_ab),
            topology_hash(&g_diff),
            "different topology must (almost certainly) hash differently"
        );
    }
}
