//! Versioned, same-build simulation persistence.
//!
//! Checkpoints preserve live neural weights, the full PRNG state, activity archive,
//! species continuity and environmental state. They promise exact continuation
//! only for the same core source, toolchain, feature set, target and build profile.
//! Cross-platform floating-point equivalence has not been established.
use crate::brain::MAX_NODES;
use crate::genome::Genome;
use crate::sim::{Simulation, SpeciesInfo, TickStats};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const CHECKPOINT_VERSION: u32 = 1;
pub const CORE_BUILD_ID: &str = env!("TERRARIUM_CORE_BUILD");
pub const CORE_TARGET: &str = env!("TERRARIUM_CORE_TARGET");
const FORMAT: &str = "terrarium-checkpoint";
const MAX_CHECKPOINT_BYTES: u64 = 256 * 1024 * 1024;
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum CheckpointError {
    Io(std::io::Error),
    Json(serde_json::Error),
    UnsupportedVersion(u32),
    IncompatibleBuild { saved: String, current: String },
    InvalidState(String),
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "checkpoint I/O: {e}"),
            Self::Json(e) => write!(f, "invalid checkpoint JSON: {e}"),
            Self::UnsupportedVersion(v) => write!(
                f,
                "unsupported checkpoint version {v}; expected {CHECKPOINT_VERSION}"
            ),
            Self::IncompatibleBuild { saved, current } => write!(
                f,
                "checkpoint build {saved} differs from current {current}; exact resume requires the original build/target"
            ),
            Self::InvalidState(e) => write!(f, "invalid checkpoint state: {e}"),
        }
    }
}
impl std::error::Error for CheckpointError {}
impl From<std::io::Error> for CheckpointError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for CheckpointError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    core_build: String,
    target: String,
    /// Non-cryptographic integrity check for accidental corruption, not authenticity.
    state_checksum: String,
    state: serde_json::Value,
}

fn invalid(message: impl Into<String>) -> CheckpointError {
    CheckpointError::InvalidState(message.into())
}

fn checksum(bytes: &[u8]) -> String {
    let mut h = 0xcbf29ce484222325_u64;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

/// Canonical ordering also works if another package enables serde_json's
/// preserve_order feature. This makes archive HashMap iteration order irrelevant.
fn canonicalize(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for v in map.values_mut() {
                canonicalize(v);
            }
            let sorted: std::collections::BTreeMap<_, _> =
                std::mem::take(map).into_iter().collect();
            map.extend(sorted);
        }
        serde_json::Value::Array(values) => {
            for v in values {
                canonicalize(v);
            }
        }
        _ => {}
    }
}

impl Simulation {
    /// Canonical complete state without build/target metadata, for explicit
    /// cross-platform comparisons. This API makes no portability guarantee.
    pub fn state_json(&self) -> Result<Vec<u8>, CheckpointError> {
        validate(self)?;
        let mut state = serde_json::to_value(self)?;
        // Config omits this inactive deprecated field in ordinary output.
        state["config"]["epigenetics"]["stress_threshold"] =
            serde_json::json!(self.config.epigenetics.stress_threshold);
        canonicalize(&mut state);
        Ok(serde_json::to_vec(&state)?)
    }

    /// Serialize complete state without rebuilding brains or consuming randomness.
    pub fn to_checkpoint_bytes(&self) -> Result<Vec<u8>, CheckpointError> {
        let state: serde_json::Value = serde_json::from_slice(&self.state_json()?)?;
        let state_checksum = checksum(&serde_json::to_vec(&state)?);
        let envelope = Envelope {
            format: FORMAT.into(),
            version: CHECKPOINT_VERSION,
            core_build: CORE_BUILD_ID.into(),
            target: CORE_TARGET.into(),
            state_checksum,
            state,
        };
        let bytes = serde_json::to_vec(&envelope)?;
        if bytes.len() as u64 > MAX_CHECKPOINT_BYTES {
            return Err(invalid("checkpoint exceeds 256 MiB limit"));
        }
        Ok(bytes)
    }

    /// Restore only compatible, internally coherent state. Does not normalize it.
    pub fn from_checkpoint_bytes(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() as u64 > MAX_CHECKPOINT_BYTES {
            return Err(invalid("checkpoint exceeds 256 MiB limit"));
        }
        let mut envelope: Envelope = serde_json::from_slice(bytes)?;
        if envelope.format != FORMAT {
            return Err(invalid("unrecognized checkpoint format"));
        }
        if envelope.version != CHECKPOINT_VERSION {
            return Err(CheckpointError::UnsupportedVersion(envelope.version));
        }
        if envelope.core_build != CORE_BUILD_ID || envelope.target != CORE_TARGET {
            return Err(CheckpointError::IncompatibleBuild {
                saved: format!("{} ({})", envelope.core_build, envelope.target),
                current: format!("{CORE_BUILD_ID} ({CORE_TARGET})"),
            });
        }
        canonicalize(&mut envelope.state);
        if checksum(&serde_json::to_vec(&envelope.state)?) != envelope.state_checksum {
            return Err(invalid("state checksum mismatch"));
        }
        let sim: Self = serde_json::from_value(envelope.state)?;
        validate(&sim)?;
        Ok(sim)
    }

    /// Atomically replace a checkpoint after syncing an adjacent uniquely named
    /// temporary file. A failed write leaves its temporary file for inspection;
    /// it never truncates the last successful checkpoint.
    pub fn save_checkpoint(&self, path: impl AsRef<Path>) -> Result<(), CheckpointError> {
        atomic_write(path.as_ref(), &self.to_checkpoint_bytes()?)
    }

    /// Export inherited genomes and explicitly separate expressed living brains.
    /// This is an analysis artifact, not a resumable world checkpoint.
    pub fn export_genomes(&self, path: impl AsRef<Path>) -> Result<(), CheckpointError> {
        validate(self)?;
        let records: Vec<_> = self
            .organisms
            .iter()
            .map(|o| {
                serde_json::json!({
                    "id": o.id, "parent_id": o.parent_id, "generation": o.generation,
                    "root_ancestor_id": o.root_ancestor_id, "birth_tick": o.birth_tick,
                    "clade_id": o.clade_id, "age": o.age, "energy": o.energy,
                    "genome": o.genome, "expressed_brain": o.brain
                })
            })
            .collect();
        let mut value = serde_json::json!({
            "format": "terrarium-genomes", "version": 1,
            "core_build": CORE_BUILD_ID, "target": CORE_TARGET,
            "tick": self.tick, "config": self.config, "genomes": records
        });
        value["config"]["epigenetics"]["stress_threshold"] =
            serde_json::json!(self.config.epigenetics.stress_threshold);
        canonicalize(&mut value);
        atomic_write(path.as_ref(), &serde_json::to_vec(&value)?)
    }

    pub fn load_checkpoint(path: impl AsRef<Path>) -> Result<Self, CheckpointError> {
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_CHECKPOINT_BYTES + 1)
            .read_to_end(&mut bytes)?;
        Self::from_checkpoint_bytes(&bytes)
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), CheckpointError> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| invalid("checkpoint requires a filename"))?;
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| invalid("system clock before epoch"))?
        .as_nanos();
    let temp = parent.join(format!(
        ".{}.{}.{nonce}.{}.tmp",
        name.to_string_lossy(),
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temp, path)?;
    // Persist the directory entry as well as file contents on Unix hosts.
    #[cfg(unix)]
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}

fn validate(sim: &Simulation) -> Result<(), CheckpointError> {
    sim.config.validate().map_err(|e| invalid(e.to_string()))?;
    sim.validate_checkpoint_internal().map_err(invalid)?;
    sim.food_grid
        .validate_checkpoint(&sim.config)
        .map_err(invalid)?;
    sim.pheromone_grid
        .validate_checkpoint(&sim.config)
        .map_err(invalid)?;
    sim.construction_grid
        .validate_checkpoint(&sim.config)
        .map_err(invalid)?;
    sim.elevation_map.validate_checkpoint().map_err(invalid)?;
    sim.spatial_grid.validate_checkpoint().map_err(invalid)?;
    for o in &sim.organisms {
        validate_genome(&o.genome)?;
        if o.genome.num_inputs != sim.config.brain.inputs
            || o.genome.num_outputs != sim.config.brain.outputs
        {
            return Err(invalid(
                "organism genome I/O differs from configured sensors/actions",
            ));
        }
        if !o.alive
            || !o.energy.is_finite()
            || o.energy <= 0.0
            || o.energy > sim.config.energy.maximum
        {
            return Err(invalid(
                "checkpoint organisms must be alive with finite positive bounded reserves",
            ));
        }
        if !o.pos_x.is_finite()
            || o.pos_x < 0.0
            || o.pos_x >= sim.config.world.width
            || !o.pos_y.is_finite()
            || o.pos_y < 0.0
            || o.pos_y >= sim.config.world.height
            || !o.heading.is_finite()
            || !o.signal.is_finite()
            || !o.energy_from_food.is_finite()
            || !o.energy_from_attacks.is_finite()
            || o.output_bins
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(invalid("invalid organism position/history"));
        }
        let b = &o.brain;
        if b.inputs != o.genome.num_inputs
            || b.outputs != o.genome.num_outputs
            || b.hidden != o.genome.hidden_nodes.len()
            || b.connections.len() != b.gene_index.len()
            || b.eval_order.len() != b.hidden + b.outputs
        {
            return Err(invalid("brain and genome dimensions differ"));
        }
        if b.connections.len() != o.genome.connections.iter().filter(|c| c.enabled).count() {
            return Err(invalid("brain enabled-edge count differs from genome"));
        }
        let mut allowed = [false; MAX_NODES];
        for node in allowed.iter_mut().skip(b.inputs + 1).take(b.outputs) {
            *node = true;
        }
        for node in &o.genome.hidden_nodes {
            allowed[node.id as usize] = true;
        }
        let mut seen = [false; MAX_NODES];
        let mut order = [usize::MAX; MAX_NODES];
        for (idx, &node) in b.eval_order.iter().enumerate() {
            let node = node as usize;
            if node >= MAX_NODES || node <= b.inputs || !allowed[node] || seen[node] {
                return Err(invalid("invalid brain evaluation node"));
            }
            seen[node] = true;
            order[node] = idx;
        }
        let mut last_target = 0;
        let mut used_genes = vec![false; o.genome.connections.len()];
        for (idx, &(from, to, weight)) in b.connections.iter().enumerate() {
            let (from, to) = (from as usize, to as usize);
            if from >= MAX_NODES
                || to >= MAX_NODES
                || !seen[to]
                || !weight.is_finite()
                || (from > b.inputs && (!seen[from] || order[from] >= order[to]))
                || (idx > 0 && order[to] < last_target)
            {
                return Err(invalid("invalid feed-forward brain edge/order"));
            }
            last_target = order[to];
            let gene = o
                .genome
                .connections
                .get(b.gene_index[idx] as usize)
                .ok_or_else(|| invalid("brain gene index out of bounds"))?;
            if used_genes[b.gene_index[idx] as usize] {
                return Err(invalid("duplicate brain gene mapping"));
            }
            used_genes[b.gene_index[idx] as usize] = true;
            if !gene.enabled || gene.from as usize != from || gene.to as usize != to {
                return Err(invalid("brain gene mapping mismatch"));
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_genome(g: &Genome) -> Result<(), CheckpointError> {
    if g.num_inputs == 0
        || g.num_inputs > MAX_NODES
        || g.num_outputs != crate::config::NUM_BRAIN_OUTPUTS
        || g.num_inputs + g.num_outputs + 1 > MAX_NODES
        || g.next_node_id as usize > MAX_NODES
        || g.connections.len() > MAX_NODES * MAX_NODES
        || (g.next_node_id as usize) < g.num_inputs + g.num_outputs + 1
    {
        return Err(invalid("genome dimensions exceed capacity"));
    }
    let mut known = [false; MAX_NODES];
    for present in known.iter_mut().take(g.num_inputs + g.num_outputs + 1) {
        *present = true;
    }
    for n in &g.hidden_nodes {
        let id = n.id as usize;
        if id >= MAX_NODES
            || known[id]
            || !n.depth.is_finite()
            || n.depth <= 0.0
            || n.depth >= 1.0
            || n.id >= g.next_node_id
        {
            return Err(invalid("invalid hidden node"));
        }
        known[id] = true;
    }
    let mut previous = None;
    for c in &g.connections {
        if c.from as usize >= MAX_NODES
            || c.to as usize >= MAX_NODES
            || !known[c.from as usize]
            || !known[c.to as usize]
            || !c.weight.is_finite()
            || !c.methylation.is_finite()
            || !(0.0..=1.0).contains(&c.methylation)
            || previous.is_some_and(|key| key >= (c.from, c.to))
        {
            return Err(invalid("invalid genome edge or ordering"));
        }
        previous = Some((c.from, c.to));
    }
    for v in [
        g.mutation_rate,
        g.mutation_step,
        g.color_hue,
        g.aging_rate,
        g.body_size,
        g.preferred_temp,
        g.diet_preference,
    ] {
        if !v.is_finite() {
            return Err(invalid("nonfinite genomic trait"));
        }
    }
    if !(0.5..=3.0).contains(&g.body_size)
        || !(0.0..=1.0).contains(&g.mutation_rate)
        || g.mutation_step <= 0.0
        || g.aging_rate < 0.0
        || !(0.0..360.0).contains(&g.color_hue)
        || !(0.0..=1.0).contains(&g.preferred_temp)
        || !(0.0..=1.0).contains(&g.diet_preference)
    {
        return Err(invalid("invalid genomic trait bounds"));
    }
    Ok(())
}

/// TickStats omits nested species when serialized as CSV. Checkpoints need both
/// the flat stats and the complete species list for exact immediate inspection.
pub(crate) mod tick_stats {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        stats: &TickStats,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        (stats, &stats.species).serialize(serializer)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<TickStats, D::Error> {
        let (mut stats, species): (TickStats, Vec<SpeciesInfo>) =
            Deserialize::deserialize(deserializer)?;
        stats.species = species;
        Ok(stats)
    }
}
