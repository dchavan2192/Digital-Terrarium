//! Stable species tracking, done entirely inside sim-wasm.
//!
//! sim-core clusters genomes every tick (NEAT compatibility distance, Stanley &
//! Miikkulainen 2002) but only publishes per-species aggregates whose `id` is a
//! per-tick index that reshuffles when species vanish, and it discards the
//! per-organism assignment. A front end needs ids that persist so a species keeps
//! its colour. This tracker re-clusters with the *same distance function and the
//! same threshold sim-core used this tick* (`TickStats::speciation_threshold`),
//! but as a record of lineages splitting, not a fresh partition every update:
//!
//! 1. A creature's species is decided once, at birth, and kept for life. Genomes
//!    do not change during a lifetime, so there is nothing to re-test.
//! 2. A newborn joins its parent's species if its genome is within the threshold
//!    of that species' founding genome. Otherwise it joins the nearest other
//!    species within the threshold, or founds a new species descended from its
//!    parent's.
//! 3. A species' reference is the genome of its founder and never changes. A
//!    lineage that drifts far enough from it therefore founds a successor species.
//!
//! The first version re-tested every creature on every update against a threshold
//! that moves each tick, and replaced the reference whenever the reference
//! creature died. Creatures near a boundary kept flipping, which produced about
//! one "new species"/"extinct" announcement every 120 ticks (measured 2026-11-16).
//!
//! Counts can differ from `TickStats::species_count`, which is a fresh partition
//! of the current population. Observation only: no RNG, no mutation of the
//! simulation.

use serde::Serialize;
use sim_core::genome::Genome;
use sim_core::organism::{Organism, OrganismId};

/// Time constant (ticks) of the population EMA used for `trend`. Presentation only.
pub const TREND_TAU_TICKS: f64 = 500.0;
/// Smallest population at which a species can be "established" and announced.
pub const DEFAULT_MIN_ANNOUNCE: usize = 5;
/// A species is announced only once it also holds this share of all living
/// creatures. Five members is under 2% of a 300-creature world: announcing every
/// such twig filled the event feed with births and extinctions of lineages that
/// never mattered. Presentation only; sim-core's species statistics are unaffected.
pub const DEFAULT_ANNOUNCE_SHARE: f64 = 0.05;

/// One tracked species. Serialized into the species list JSON.
#[derive(Debug, Clone, Serialize)]
pub struct TrackedSpecies {
    pub stable_id: u32,
    pub parent_stable_id: Option<u32>,
    pub first_seen_tick: u64,
    /// Reached `min_announce` members at least once (a `species_born` event was emitted).
    pub established: bool,
    pub population: usize,
    pub peak: usize,
    pub mean_hue: f64,
    pub mean_energy: f64,
    pub mean_body_size: f64,
    pub mean_connections: f64,
    pub mean_generation: f64,
    pub mean_trophic_level: f64,
    pub herbivore_fraction: f64,
    pub predator_fraction: f64,
    /// (population - EMA) / EMA, EMA time constant [`TREND_TAU_TICKS`].
    pub trend: f64,
    /// Genome of the founder; fixed for the life of the species.
    #[serde(skip)]
    rep: Genome,
    #[serde(skip)]
    ema: f64,
    #[serde(skip)]
    hue_x: f64,
    #[serde(skip)]
    hue_y: f64,
}

/// Lifecycle notifications produced by [`SpeciesTracker::update`].
pub enum SpeciesChange {
    Born {
        species: u32,
        parent: Option<u32>,
        size: usize,
    },
    Extinct {
        species: u32,
        peak: usize,
        lifespan: u64,
        established: bool,
    },
}

pub struct SpeciesTracker {
    pub species: Vec<TrackedSpecies>,
    next_id: u32,
    /// (organism id, stable species id), sorted by organism id — last update.
    prev: Vec<(OrganismId, u32)>,
    /// Stable species id per organism index (aligned with `sim.organisms`).
    pub assignment: Vec<u32>,
    pub min_announce: usize,
    /// Share of the living population a species must reach to be announced.
    pub announce_share: f64,
    last_tick: Option<u64>,
}

impl Default for SpeciesTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeciesTracker {
    pub fn new() -> Self {
        SpeciesTracker {
            species: Vec::new(),
            next_id: 0,
            prev: Vec::new(),
            assignment: Vec::new(),
            min_announce: DEFAULT_MIN_ANNOUNCE,
            announce_share: DEFAULT_ANNOUNCE_SHARE,
            last_tick: None,
        }
    }

    pub fn clear(&mut self) {
        let (min, share) = (self.min_announce, self.announce_share);
        *self = Self::new();
        self.min_announce = min;
        self.announce_share = share;
    }

    /// Previous-update species of an organism id (works for organisms that have
    /// since died, until the next update).
    pub fn species_of_id(&self, id: OrganismId) -> Option<u32> {
        self.prev
            .binary_search_by_key(&id, |p| p.0)
            .ok()
            .map(|i| self.prev[i].1)
    }

    fn slot_of(&self, stable_id: u32) -> Option<usize> {
        // `species` stays sorted by stable_id (append-only ids, order-preserving retain).
        self.species
            .binary_search_by_key(&stable_id, |s| s.stable_id)
            .ok()
    }

    /// Re-cluster. `order` lists organism indices in ascending id order.
    pub fn update(
        &mut self,
        orgs: &[Organism],
        order: &[u32],
        threshold: f64,
        tick: u64,
        mut emit: impl FnMut(SpeciesChange),
    ) {
        let dt = match self.last_tick {
            Some(t) => tick.saturating_sub(t) as f64,
            None => 0.0,
        };
        self.last_tick = Some(tick);

        for s in &mut self.species {
            s.population = 0;
            s.mean_energy = 0.0;
            s.mean_body_size = 0.0;
            s.mean_connections = 0.0;
            s.mean_generation = 0.0;
            s.mean_trophic_level = 0.0;
            s.herbivore_fraction = 0.0;
            s.predator_fraction = 0.0;
            s.hue_x = 0.0;
            s.hue_y = 0.0;
        }
        self.assignment.clear();
        self.assignment.resize(orgs.len(), u32::MAX);

        for &k in order {
            let k = k as usize;
            let o = &orgs[k];
            let g = &o.genome;
            // Known creature: its species was decided at birth.
            let mut slot = self.species_of_id(o.id).and_then(|sid| self.slot_of(sid));
            if slot.is_none() {
                // Newborn (or the very first update): the parent's species, if the
                // child is still within the threshold of its founding genome.
                slot = o
                    .parent_id
                    .and_then(|p| self.species_of_id(p))
                    .and_then(|sid| self.slot_of(sid))
                    .filter(|&si| g.genome_distance(&self.species[si].rep) < threshold);
            }
            if slot.is_none() {
                // Otherwise the nearest species within the threshold, if any.
                let mut best: Option<(usize, f64)> = None;
                for (si, sp) in self.species.iter().enumerate() {
                    let d = g.genome_distance(&sp.rep);
                    if d < threshold && best.is_none_or(|(_, bd)| d < bd) {
                        best = Some((si, d));
                    }
                }
                slot = best.map(|(si, _)| si);
            }
            let si = match slot {
                Some(si) => si,
                None => {
                    let parent = o.parent_id.and_then(|p| self.species_of_id(p));
                    let id = self.next_id;
                    self.next_id += 1;
                    self.species.push(TrackedSpecies {
                        stable_id: id,
                        parent_stable_id: parent,
                        first_seen_tick: tick,
                        established: false,
                        population: 0,
                        peak: 0,
                        mean_hue: 0.0,
                        mean_energy: 0.0,
                        mean_body_size: 0.0,
                        mean_connections: 0.0,
                        mean_generation: 0.0,
                        mean_trophic_level: 0.0,
                        herbivore_fraction: 0.0,
                        predator_fraction: 0.0,
                        trend: 0.0,
                        rep: g.clone(),
                        ema: 0.0,
                        hue_x: 0.0,
                        hue_y: 0.0,
                    });
                    self.species.len() - 1
                }
            };

            let s = &mut self.species[si];
            self.assignment[k] = s.stable_id;
            s.population += 1;
            let total = o.energy_from_food + o.energy_from_attacks;
            let pred = if total > 0.0 {
                o.energy_from_attacks / total
            } else {
                0.0
            };
            s.mean_energy += o.energy;
            s.mean_body_size += g.body_size;
            s.mean_connections += o.brain.connection_count() as f64;
            s.mean_generation += o.generation as f64;
            s.mean_trophic_level += 1.0 + pred;
            if pred < 0.25 {
                s.herbivore_fraction += 1.0;
            } else if pred > 0.5 {
                s.predator_fraction += 1.0;
            }
            let rad = g.color_hue.to_radians();
            s.hue_x += rad.cos();
            s.hue_y += rad.sin();
        }

        // Finalize aggregates, refresh representatives, announce births.
        let alpha = if dt > 0.0 {
            1.0 - (-dt / TREND_TAU_TICKS).exp()
        } else {
            1.0
        };
        let announce_at = self
            .min_announce
            .max((self.announce_share * orgs.len() as f64).ceil() as usize);
        for s in &mut self.species {
            if s.population == 0 {
                continue;
            }
            let n = s.population as f64;
            s.mean_energy /= n;
            s.mean_body_size /= n;
            s.mean_connections /= n;
            s.mean_generation /= n;
            s.mean_trophic_level /= n;
            s.herbivore_fraction /= n;
            s.predator_fraction /= n;
            s.mean_hue = s.hue_y.atan2(s.hue_x).to_degrees().rem_euclid(360.0);
            s.peak = s.peak.max(s.population);
            s.ema = if s.ema > 0.0 {
                s.ema + alpha * (n - s.ema)
            } else {
                n
            };
            s.trend = if s.ema > 0.0 {
                (n - s.ema) / s.ema
            } else {
                0.0
            };
            if !s.established && s.population >= announce_at {
                s.established = true;
                emit(SpeciesChange::Born {
                    species: s.stable_id,
                    parent: s.parent_stable_id,
                    size: s.population,
                });
            }
        }
        self.species.retain(|s| {
            if s.population == 0 {
                emit(SpeciesChange::Extinct {
                    species: s.stable_id,
                    peak: s.peak,
                    lifespan: tick.saturating_sub(s.first_seen_tick),
                    established: s.established,
                });
                false
            } else {
                true
            }
        });

        self.prev.clear();
        for &k in order {
            let k = k as usize;
            self.prev.push((orgs[k].id, self.assignment[k]));
        }
    }

    /// Number of species that have reached `min_announce` members and are alive.
    pub fn established_count(&self) -> usize {
        self.species.iter().filter(|s| s.established).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::{Config, Simulation};

    fn population() -> Simulation {
        let config = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
        Simulation::new(config)
    }

    fn order(sim: &Simulation) -> Vec<u32> {
        // Founders are created in ascending id order.
        (0..sim.organisms.len() as u32).collect()
    }

    fn update(
        t: &mut SpeciesTracker,
        sim: &Simulation,
        threshold: f64,
        tick: u64,
    ) -> (usize, usize) {
        let (mut born, mut extinct) = (0, 0);
        t.update(&sim.organisms, &order(sim), threshold, tick, |c| match c {
            SpeciesChange::Born { .. } => born += 1,
            SpeciesChange::Extinct { .. } => extinct += 1,
        });
        (born, extinct)
    }

    #[test]
    fn a_creature_keeps_its_species_for_life() {
        let sim = population();
        let mut t = SpeciesTracker::new();
        // A huge threshold puts every founder in one species.
        assert_eq!(update(&mut t, &sim, 1e9, 0), (1, 0));
        assert_eq!(t.species.len(), 1);
        let first = t.assignment.clone();
        // A threshold of zero would split everyone apart if creatures were
        // re-tested. They are not: nothing is born, nothing goes extinct.
        assert_eq!(update(&mut t, &sim, 0.0, 1), (0, 0));
        assert_eq!(t.assignment, first);
        assert_eq!(t.species.len(), 1);
    }

    #[test]
    fn small_lineages_are_tracked_but_not_announced() {
        let sim = population();
        let n = sim.organisms.len();
        let mut t = SpeciesTracker::new();
        // Threshold zero: every founder is its own one-member species.
        let (born, extinct) = update(&mut t, &sim, 0.0, 0);
        assert_eq!(
            (born, extinct),
            (0, 0),
            "single-member species are not news"
        );
        assert_eq!(t.species.len(), n);
        assert_eq!(t.established_count(), 0);
        assert!(t.assignment.iter().all(|&s| s != u32::MAX));
    }

    #[test]
    fn announcement_needs_a_share_of_the_population() {
        let sim = population();
        let n = sim.organisms.len();
        let mut t = SpeciesTracker::new();
        t.announce_share = 2.0; // more than everyone: can never be reached
        assert_eq!(update(&mut t, &sim, 1e9, 0), (0, 0));
        assert_eq!(t.established_count(), 0);
        // Lower the bar to 5% and the one big species is announced exactly once.
        t.announce_share = DEFAULT_ANNOUNCE_SHARE;
        assert!(n >= 20, "5% of the population must exceed one creature");
        assert_eq!(update(&mut t, &sim, 1e9, 1), (1, 0));
        assert_eq!(update(&mut t, &sim, 1e9, 2), (0, 0));
        assert_eq!(t.established_count(), 1);
    }
}
