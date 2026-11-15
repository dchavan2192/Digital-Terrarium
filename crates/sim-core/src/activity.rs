//! Evolutionary Activity Statistics — measuring open-ended evolution.
//!
//! Tracks structural novelty and cumulative activity to determine whether
//! the simulation produces genuinely open-ended evolution (unbounded novelty)
//! or converges to a fixed set of strategies.
//!
//! Citations:
//! - Bedau, M.A. & Packard, N.H. (1991) "Measurement of Evolutionary Activity,
//!   Teleology, and Life" *ALife II* pp.431-461
//! - Bedau, M.A., Snyder, E. & Packard, N.H. (1998) "A Classification of
//!   Long-Term Evolutionary Dynamics" *ALife VI* pp.228-237
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
#[derive(serde::Serialize, serde::Deserialize)]
struct ActivityRecord {
    #[allow(dead_code)]
    first_seen: u64,
    cumulative_count: u64,
    #[allow(dead_code)]
    last_seen: u64,
}

/// Tracks evolutionary activity statistics over time.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ActivityTracker {
    traits: HashMap<u64, ActivityRecord>,
    window_size: u64,
    last_update_tick: u64,
    /// Stats from the most recent update window.
    pub stats: ActivityStats,
}

/// Snapshot of activity metrics computed each window.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ActivityStats {
    /// New structural topologies seen this window.
    pub novel_traits: u64,
    /// Distinct topologies alive right now.
    pub active_traits: u64,
    /// Mean cumulative activity Ā(t) = A_cum(t) / D(t), averaged over the traits
    /// that are *currently active* (extant), Bedau et al. (1998) style. Extinct
    /// traits are excluded, so this is not diluted by the growing archive of dead
    /// topologies. (Note: Bedau's D(t) is the diversity = `active_traits`.)
    pub mean_cumulative_activity: f64,
    /// A_cum(t) — total cumulative activity summed over currently active traits.
    pub total_cumulative_activity: u64,
    /// Total distinct traits ever observed.
    pub total_traits_ever: u64,
}

impl ActivityTracker {
    pub fn new(window_size: u64) -> Self {
        ActivityTracker {
            traits: HashMap::new(),
            window_size,
            last_update_tick: 0,
            stats: ActivityStats::default(),
        }
    }

    /// Update activity tracking. Call every tick; only does real work every window_size ticks.
    pub fn update(&mut self, tick: u64, genomes: impl Iterator<Item = u64>) {
        if tick < self.last_update_tick + self.window_size && tick > 0 {
            return;
        }
        self.last_update_tick = tick;

        let mut active_this_window: HashMap<u64, u64> = HashMap::new();
        for hash in genomes {
            *active_this_window.entry(hash).or_insert(0) += 1;
        }

        let mut novel = 0u64;
        for (&hash, &count) in &active_this_window {
            match self.traits.get_mut(&hash) {
                Some(record) => {
                    record.cumulative_count += count;
                    record.last_seen = tick;
                }
                None => {
                    self.traits.insert(
                        hash,
                        ActivityRecord {
                            first_seen: tick,
                            cumulative_count: count,
                            last_seen: tick,
                        },
                    );
                    novel += 1;
                }
            }
        }

        let total_traits_ever = self.traits.len() as u64;
        // Bedau, Snyder & Packard (1998): sum cumulative activity over extant
        // components only, then divide by diversity (number of extant components).
        let active = active_this_window.len() as u64;
        let total_cumulative: u64 = active_this_window
            .keys()
            .map(|h| self.traits.get(h).map_or(0, |r| r.cumulative_count))
            .sum();
        let mean_cumulative = if active > 0 {
            total_cumulative as f64 / active as f64
        } else {
            0.0
        };

        self.stats = ActivityStats {
            novel_traits: novel,
            active_traits: active,
            mean_cumulative_activity: mean_cumulative,
            total_cumulative_activity: total_cumulative,
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
    fn cumulative_activity_grows() {
        let mut tracker = ActivityTracker::new(1);
        tracker.update(0, vec![100, 100, 100].into_iter());
        let d0 = tracker.stats.mean_cumulative_activity;
        tracker.update(1, vec![100, 100, 100].into_iter());
        let d1 = tracker.stats.mean_cumulative_activity;
        assert!(d1 > d0, "D(t) should grow: {} > {}", d1, d0);
    }

    #[test]
    fn mean_activity_ignores_extinct_traits() {
        let mut tracker = ActivityTracker::new(1);
        // Trait 100 accumulates a lot, then goes extinct.
        tracker.update(0, vec![100; 50].into_iter());
        tracker.update(1, vec![200].into_iter());
        // Only trait 200 (cumulative 1) is extant.
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
}
