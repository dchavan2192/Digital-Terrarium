//! Stable species tracking, done entirely inside sim-wasm.
//!
//! sim-core clusters genomes every tick (NEAT compatibility distance, Stanley &
//! Miikkulainen 2002) but only publishes per-species aggregates whose `id` is a
//! per-tick index that reshuffles when species vanish, and it discards the
//! per-organism assignment. A front end needs ids that persist so a species keeps
//! its colour. This tracker re-clusters with the *same distance function and the
//! same threshold sim-core used this tick* (`TickStats::speciation_threshold`),
//! but with persistent representatives and membership hysteresis:
//!
//! 1. Each organism first tries the species it (or, for newborns, its parent)
//!    belonged to last update. Most organisms need exactly one distance call.
//! 2. Otherwise it joins the oldest species whose representative is within the
//!    threshold, or founds a new species whose parent is its parent's species.
//! 3. A representative is kept until the organism it was copied from dies; it is
//!    then replaced by the oldest (lowest-id) member, so it drifts with the clade.
//!
//! Counts can differ slightly from `TickStats::species_count` (different
//! representatives). Observation only: no RNG, no mutation of the simulation.

use serde::Serialize;
use sim_core::genome::Genome;
use sim_core::organism::{Organism, OrganismId};

/// Time constant (ticks) of the population EMA used for `trend`. Presentation only.
pub const TREND_TAU_TICKS: f64 = 500.0;
/// Default population at which a species is "established" and announced.
pub const DEFAULT_MIN_ANNOUNCE: usize = 5;

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
    #[serde(skip)]
    rep: Genome,
    #[serde(skip)]
    rep_org: OrganismId,
    #[serde(skip)]
    rep_alive: bool,
    #[serde(skip)]
    oldest_member: Option<usize>,
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
            last_tick: None,
        }
    }

    pub fn clear(&mut self) {
        let min = self.min_announce;
        *self = Self::new();
        self.min_announce = min;
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
            s.rep_alive = false;
            s.oldest_member = None;
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
            let hint = self
                .species_of_id(o.id)
                .or_else(|| o.parent_id.and_then(|p| self.species_of_id(p)));

            let mut slot = hint
                .and_then(|sid| self.slot_of(sid))
                .filter(|&si| g.genome_distance(&self.species[si].rep) < threshold);
            if slot.is_none() {
                slot = self
                    .species
                    .iter()
                    .position(|s| g.genome_distance(&s.rep) < threshold);
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
                        rep_org: o.id,
                        rep_alive: false,
                        oldest_member: None,
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
            if s.oldest_member.is_none() {
                s.oldest_member = Some(k);
            }
            if s.rep_org == o.id {
                s.rep_alive = true;
            }
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
            if !s.rep_alive
                && let Some(k) = s.oldest_member
            {
                s.rep = orgs[k].genome.clone();
                s.rep_org = orgs[k].id;
            }
            if !s.established && s.population >= self.min_announce {
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
