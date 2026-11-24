//! Embed a one-input donor in the native sensor layout while preserving its actions.
use crate::run::RunResult;
use sim_core::{
    Config,
    brain::{BrainOutput, MAX_NODES},
    config::NUM_BRAIN_OUTPUTS,
    genome::{ConnectionGene, Genome},
};

// Native input nodes are one-based: paired vision rays, reserve, heading, heard signal.
fn cue_node(config: &Config) -> usize {
    2 * config.creature.vision.ray_count + 3
}

pub(super) fn transfer(donor: &Genome, config: &Config) -> RunResult<Genome> {
    config.validate().map_err(|err| err.to_string())?;
    donor.validate().map_err(|err| err.to_string())?;
    if donor.num_inputs != 1 || donor.num_outputs != NUM_BRAIN_OUTPUTS {
        return Err("food cue donor requires one cue input and seven native action outputs".into());
    }
    let inputs = config.brain.inputs as u32;
    let first_hidden = inputs + NUM_BRAIN_OUTPUTS as u32 + 1;
    let mut hidden_ids: Vec<_> = donor.hidden_nodes.iter().map(|node| node.id).collect();
    hidden_ids.sort_unstable();
    let remap = |id| -> RunResult<u32> {
        match id {
            0 => Ok(0),
            1 => Ok(cue_node(config) as u32),
            id if id <= 1 + NUM_BRAIN_OUTPUTS as u32 => Ok(inputs + id - 1),
            id => hidden_ids
                .binary_search(&id)
                .map(|index| first_hidden + index as u32)
                .map_err(|_| format!("unknown hidden donor node {id}")),
        }
    };
    let mut native = donor.clone();
    native.num_inputs = config.brain.inputs;
    native.body_size = 1.0;
    // Keep vector order: build_brain uses vector positions as the runtime hidden slots.
    for node in &mut native.hidden_nodes {
        node.id = remap(node.id)?;
    }
    for gene in &mut native.connections {
        gene.from = remap(gene.from)?;
        gene.to = remap(gene.to)?;
    }
    native.connections.sort_by_key(ConnectionGene::key);
    native.validate().map_err(|err| err.to_string())?;
    Ok(native)
}

fn actions(output: BrainOutput) -> [f64; NUM_BRAIN_OUTPUTS] {
    [
        output.turn,
        output.speed,
        output.eat,
        output.attack,
        output.signal,
        output.share,
        output.construct,
    ]
}

pub(super) fn verify(donor: &Genome, transferred: &Genome, config: &Config) -> RunResult<()> {
    config.validate().map_err(|err| err.to_string())?;
    donor.validate().map_err(|err| err.to_string())?;
    transferred.validate().map_err(|err| err.to_string())?;
    if donor.num_inputs != 1 || transferred.num_inputs != config.brain.inputs {
        return Err("unexpected donor or native input dimensions for trace verification".into());
    }
    let original = donor.build_brain();
    let native = transferred.build_brain();
    let mut inputs = vec![0.; native.inputs];
    for cue in [-1., 1.] {
        for delay in [2, 5, 8, 11] {
            for (reset, visible) in [(false, false), (true, false), (true, true)] {
                let mut prev = [0.; MAX_NODES];
                let mut native_prev = prev;
                let mut next = prev;
                let mut native_next = prev;
                // Include the feeding tick after choice: eat may depend on recurrent state.
                let choice_tick = delay + 1;
                for tick in 0..=choice_tick + 1 {
                    if tick == choice_tick && reset {
                        prev.fill(0.);
                        native_prev.fill(0.);
                    }
                    let signal = if tick == 0 || tick == choice_tick && visible {
                        cue
                    } else {
                        0.
                    };
                    for (index, value) in inputs.iter_mut().enumerate() {
                        *value = ((index + tick) % 9) as f64 / 4. - 1.;
                    }
                    inputs[cue_node(config) - 1] = signal;
                    let (expected, activity) =
                        original.forward_with_memory(&[signal], &prev, &mut next);
                    let (actual, native_activity) =
                        native.forward_with_memory(&inputs, &native_prev, &mut native_next);
                    if actions(expected) != actions(actual) || activity != native_activity {
                        return Err(format!(
                            "donor transfer changed actions or activity at tick {tick}, cue {cue}, delay {delay}, reset {reset}, visible {visible}"
                        ));
                    }
                    std::mem::swap(&mut prev, &mut next);
                    std::mem::swap(&mut native_prev, &mut native_next);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::Simulation;
    use sim_core::genome::HiddenNode;

    #[test]
    fn transfers_sparse_hidden_ids_and_feedback_without_rewiring_actions() {
        let config = Config::parse(include_str!("../../../configs/default.toml")).unwrap();
        let sim = Simulation::new(config.clone());
        let mut donor = sim.organisms[0].genome.clone();
        donor.num_inputs = 1;
        donor.body_size = 2.;
        donor.hidden_nodes = vec![
            HiddenNode {
                id: u32::MAX,
                depth: 0.25,
            },
            HiddenNode {
                id: 1234,
                depth: 0.75,
            },
        ];
        donor.connections.clear();
        let mut edge = |from, to, recurrent| {
            donor.connections.push(ConnectionGene {
                from,
                to,
                recurrent,
                weight: 0.7,
                methylation: 0.2,
                enabled: true,
            })
        };
        edge(1, u32::MAX, false);
        edge(2, u32::MAX, true);
        edge(u32::MAX, 1234, false);
        for output in 2..=8 {
            edge(1234, output, false);
        }
        donor.connections.sort_by_key(ConnectionGene::key);
        let mut native = transfer(&donor, &config).unwrap();
        let first_hidden = (config.brain.inputs + NUM_BRAIN_OUTPUTS + 1) as u32;
        assert_eq!(native.hidden_nodes[0].id, first_hidden + 1);
        assert_eq!(native.hidden_nodes[1].id, first_hidden);
        assert_eq!(native.body_size, 1.);
        verify(&donor, &native, &config).unwrap();
        // A rewired cue reaching an otherwise unrelated native sensor must be detected.
        native
            .connections
            .iter_mut()
            .find(|gene| gene.from == cue_node(&config) as u32)
            .unwrap()
            .from = 1;
        native.connections.sort_by_key(ConnectionGene::key);
        assert!(verify(&donor, &native, &config).is_err());
    }
}
