//! Isolated instrument validation: synthetic cues, no ecological fitness claim.
use sim_core::brain::{Brain, MAX_NODES};

const MAX_BLANK_DELAY: u32 = 10_000;

#[derive(Debug)]
struct Trial {
    cue: i8,
    cue_response: f64,
    probe_response: f64,
    predicted_cue: i8,
}

impl Trial {
    fn correct(&self) -> bool {
        self.predicted_cue == self.cue
    }
}

#[derive(Debug)]
struct Control {
    name: &'static str,
    recurrent_edges: usize,
    clear_state_each_tick: bool,
    trials: Vec<Trial>,
}

impl Control {
    fn accuracy(&self) -> f64 {
        self.trials.iter().filter(|t| t.correct()).count() as f64 / self.trials.len() as f64
    }

    fn json(&self) -> String {
        let trials = self.trials.iter().map(|trial| {
            format!(
                "{{\"cue\":{},\"cue_response\":{},\"probe_response\":{},\"predicted_cue\":{},\"correct\":{}}}",
                trial.cue, trial.cue_response, trial.probe_response, trial.predicted_cue, trial.correct()
            )
        }).collect::<Vec<_>>().join(",");
        format!(
            "{{\"name\":\"{}\",\"recurrent_edges\":{},\"clear_state_each_tick\":{},\"accuracy\":{},\"trials\":[{}]}}",
            self.name,
            self.recurrent_edges,
            self.clear_state_each_tick,
            self.accuracy(),
            trials
        )
    }
}

fn reference(recurrent: bool) -> Brain {
    // One synthetic cue input, seven action outputs. Turn is node 2.
    // The output self-loop retains either sign through arbitrarily many blanks.
    Brain::from_neat_with_memory(
        1,
        7,
        &[],
        &[(1, 2, 2.0, true)],
        if recurrent { &[(2, 2, 2.0, 1)] } else { &[] },
    )
}

fn trial(brain: &Brain, cue: i8, blank_delay: u32, clear_state_each_tick: bool) -> Trial {
    // Fresh buffers per trial prevent the preceding answer from becoming a cue.
    let mut prev = [0.0; MAX_NODES];
    let mut act = [0.0; MAX_NODES];
    let mut step = |input: f64| {
        if clear_state_each_tick {
            prev.fill(0.0);
        }
        let (output, _) = brain.forward_with_memory(&[input], &prev, &mut act);
        prev.copy_from_slice(&act);
        output.turn
    };
    let cue_response = step(f64::from(cue));
    for _ in 0..blank_delay {
        step(0.0);
    }
    // There is no timing or probe sensor. This input is exactly identical in
    // both cue conditions, and identical to every preceding blank tick.
    let probe_response = step(0.0);
    Trial {
        cue,
        cue_response,
        probe_response,
        // A fixed, cue-independent tie break makes constant-zero controls 50%.
        predicted_cue: if probe_response >= 0.0 { 1 } else { -1 },
    }
}

fn controls(blank_delay: u32) -> Result<Vec<Control>, &'static str> {
    if blank_delay > MAX_BLANK_DELAY {
        return Err("--blank-delay must be between 0 and 10000");
    }
    Ok([
        ("recurrent_reference", true, false),
        ("feed_forward_control", false, false),
        ("same_network_state_cleared", true, true),
    ]
    .into_iter()
    .map(|(name, recurrent, clear_state_each_tick)| {
        let brain = reference(recurrent);
        Control {
            name,
            recurrent_edges: brain.recurrent_count(),
            clear_state_each_tick,
            trials: [-1, 1]
                .into_iter()
                .map(|cue| trial(&brain, cue, blank_delay, clear_state_each_tick))
                .collect(),
        }
    })
    .collect())
}

pub fn delayed_cue(blank_delay: u32) -> crate::run::RunResult<()> {
    let controls = controls(blank_delay)?;
    let valid = controls[0].accuracy() == 1.0
        && controls[1..].iter().all(|c| c.accuracy() == 0.5)
        && controls
            .iter()
            .flat_map(|c| &c.trials)
            .all(|t| t.cue_response.is_finite() && t.probe_response.is_finite());
    println!(
        "{{\"schema_version\":1,\"assay\":\"delayed_cue\",\"interpretation\":\"synthetic instrument validation; not evolved cognition, learning or ecological fitness\",\"blank_delay_ticks\":{blank_delay},\"cue_ticks\":1,\"probe_ticks\":1,\"probe_inputs\":[0.0],\"input_count\":1,\"output_count\":7,\"readout\":\"turn\",\"zero_tie_prediction\":1,\"state_reset_each_trial\":true,\"statistical_replicates\":false,\"instrument_validated\":{valid},\"controls\":[{}]}}",
        controls
            .iter()
            .map(Control::json)
            .collect::<Vec<_>>()
            .join(",")
    );
    if valid {
        Ok(())
    } else {
        Err("delayed-cue instrument controls failed".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistence_solves_balanced_cues_and_both_controls_remain_at_chance() {
        for delay in [0, 1, 5, 100, MAX_BLANK_DELAY] {
            let c = controls(delay).unwrap();
            assert_eq!(c[0].accuracy(), 1.0);
            assert_eq!(c[1].accuracy(), 0.5);
            assert_eq!(c[2].accuracy(), 0.5);
            assert_eq!(c[0].recurrent_edges, c[2].recurrent_edges);
            for control in &c {
                assert_eq!(control.trials.iter().map(|t| t.cue).sum::<i8>(), 0);
                for t in &control.trials {
                    assert_eq!(t.cue_response.signum(), f64::from(t.cue));
                }
            }
        }
    }

    #[test]
    fn fresh_trial_state_makes_probe_response_independent_of_trial_order() {
        let brain = reference(true);
        let negative_first = trial(&brain, -1, 5, false);
        let positive = trial(&brain, 1, 5, false);
        let negative_after_positive = trial(&brain, -1, 5, false);
        assert_eq!(
            negative_first.probe_response,
            negative_after_positive.probe_response
        );
        assert_eq!(negative_first.probe_response, -positive.probe_response);
        for cleared in [true, false] {
            let plain = reference(false);
            assert_eq!(trial(&plain, -1, 5, cleared).probe_response, 0.0);
            assert_eq!(trial(&plain, 1, 5, cleared).probe_response, 0.0);
        }
    }

    #[test]
    fn excessive_delays_are_rejected_before_work() {
        assert!(controls(MAX_BLANK_DELAY + 1).is_err());
    }
}
