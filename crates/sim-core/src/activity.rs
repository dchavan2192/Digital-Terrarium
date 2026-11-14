//! Evolutionary Activity Statistics — measuring open-ended evolution.
//!
//! Tracks structural novelty and cumulative activity to determine whether
//! the simulation produces genuinely open-ended evolution (unbounded novelty)
//! or converges to a fixed set of strategies.
//!
//! Citations:
//! - Bedau, M.A. & Packard, N.H. (1991) "Measurement of Evolutionary Activity,
//!   Teleology, and Life" *ALife II* pp.431-461
//! - Dolson, E. et al. (2019) "The MODES Toolbox" *Artificial Life* 25(1):50-73

use crate::genome::Genome;
use std::collections::HashMap;

/// FNV-1a hash of a genome's connection topology (ignoring weights).
/// Two organisms with the same wiring but different weights are the same "trait."
pub fn topology_hash(genome: &Genome) -> u64 {
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
struct ActivityRecord {
    #[allow(dead_code)]
    first_seen: u64,
    cumulative_count: u64,
    #[allow(dead_code)]
    last_seen: u64,
}

/// Tracks evolutionary activity statistics over time.
pub struct ActivityTracker {
    traits: HashMap<u64, ActivityRecord>,
    window_size: u64,
    last_update_tick: u64,
    /// Stats from the most recent update window.
    pub stats: ActivityStats,
}

/// Snapshot of activity metrics computed each window.
#[derive(Debug, Clone, Default)]
pub struct ActivityStats {
    /// New structural topologies seen this window.
    pub novel_traits: u64,
    /// Distinct topologies alive right now.
    pub active_traits: u64,
    /// D(t) — mean cumulative activity across all traits ever seen.
    /// If this grows unboundedly, evolution is open-ended.
    pub mean_cumulative_activity: f64,
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
        let mean_cumulative = if total_traits_ever > 0 {
            self.traits
                .values()
                .map(|r| r.cumulative_count)
                .sum::<u64>() as f64
                / total_traits_ever as f64
        } else {
            0.0
        };

        self.stats = ActivityStats {
            novel_traits: novel,
            active_traits: active_this_window.len() as u64,
            mean_cumulative_activity: mean_cumulative,
            total_traits_ever,
        };
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
