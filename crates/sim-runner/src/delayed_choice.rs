//! A controlled movement instrument, independent of the ecological simulation.
use crate::run::RunResult;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sim_core::brain::{Brain, BrainOutput, MAX_NODES};
use std::f64::consts::PI;
use std::io::Read;
use std::path::Path;

pub(crate) const SENSOR_CONVENTIONS: &str = "same type1 goal circles; ray proximity=1-hit_distance/range, nohit proximity0/type-1; heading=heading/pi-1; all scalar inputs except heading and signed cue fixed";
pub(crate) const MOTION_CONVENTIONS: &str = "turn clamped[-1,1], speed clamped[0,1]; turn then translate; continuous segment first-contact terminal, wrong contact and timeout reward0; held-pose motor outputs discarded";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArenaConfig {
    pub ray_count: usize,
    pub ray_fov: f64,
    pub ray_range: f64,
    pub start_x: f64,
    pub start_y: f64,
    pub start_heading: f64,
    pub goal_left_x: f64,
    pub goal_left_y: f64,
    pub goal_right_x: f64,
    pub goal_right_y: f64,
    pub goal_radius: f64,
    pub max_turn: f64,
    pub max_speed: f64,
    pub cue_ticks: u32,
    pub blank_delay: u32,
    pub horizon: u32,
    pub energy: f64,
    pub body: f64,
    pub kin: f64,
    pub temperature: f64,
    pub pheromone: f64,
    pub light: f64,
    pub niche: f64,
    pub food: f64,
    pub cue_weight: f64,
    pub turn_bias: f64,
    pub recurrent_weight: f64,
    pub speed_bias: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct Pose {
    x: f64,
    y: f64,
    heading: f64,
}

impl ArenaConfig {
    pub(crate) fn input_count(&self) -> usize {
        2 * self.ray_count + 10
    }

    fn start(&self) -> Pose {
        Pose {
            x: self.start_x,
            y: self.start_y,
            heading: self.start_heading,
        }
    }

    fn goals(&self) -> [(f64, f64); 2] {
        [
            (self.goal_left_x, self.goal_left_y),
            (self.goal_right_x, self.goal_right_y),
        ]
    }

    fn validate(&self) -> RunResult<()> {
        if self.ray_count == 0 || self.ray_count > 16 {
            return Err("arena ray_count must be in 1..=16".into());
        }
        if self.cue_ticks == 0
            || self.cue_ticks > 10_000
            || self.blank_delay > 10_000
            || self.horizon == 0
            || self.horizon > 10_000
        {
            return Err("require cue_ticks/horizon in 1..=10000 and blank_delay <=10000".into());
        }
        for (name, value) in [
            ("ray_fov", self.ray_fov),
            ("ray_range", self.ray_range),
            ("start_x", self.start_x),
            ("start_y", self.start_y),
            ("start_heading", self.start_heading),
            ("goal_left_x", self.goal_left_x),
            ("goal_left_y", self.goal_left_y),
            ("goal_right_x", self.goal_right_x),
            ("goal_right_y", self.goal_right_y),
            ("goal_radius", self.goal_radius),
            ("max_turn", self.max_turn),
            ("max_speed", self.max_speed),
            ("energy", self.energy),
            ("body", self.body),
            ("kin", self.kin),
            ("temperature", self.temperature),
            ("pheromone", self.pheromone),
            ("light", self.light),
            ("niche", self.niche),
            ("food", self.food),
            ("cue_weight", self.cue_weight),
            ("turn_bias", self.turn_bias),
            ("recurrent_weight", self.recurrent_weight),
            ("speed_bias", self.speed_bias),
        ] {
            if !value.is_finite() || value.abs() > 1_000_000.0 {
                return Err(format!(
                    "arena {name} must be finite and bounded by 1000000"
                ));
            }
        }
        if !(0.0..2.0 * PI).contains(&self.start_heading)
            || self.ray_fov <= 0.0
            || self.ray_fov > 2.0 * PI
            || self.ray_range <= 0.0
            || self.goal_radius <= 0.0
            || self.max_turn <= 0.0
            || self.max_turn > PI
            || self.max_speed <= 0.0
        {
            return Err("invalid arena angle, range, radius or movement bound".into());
        }
        for value in [
            self.energy,
            self.temperature,
            self.pheromone,
            self.light,
            self.food,
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err("unsigned constant sensors must be in [0,1]".into());
            }
        }
        for value in [self.body, self.kin, self.niche] {
            if !(-1.0..=1.0).contains(&value) {
                return Err("signed constant sensors must be in [-1,1]".into());
            }
        }
        let goals = self.goals();
        let separation_x = goals[0].0 - goals[1].0;
        let separation_y = goals[0].1 - goals[1].1;
        if (separation_x * separation_x + separation_y * separation_y).sqrt()
            <= 2.0 * self.goal_radius
        {
            return Err("goal discs must be strictly nonoverlapping".into());
        }
        let (dx, dy) = (
            sim_core::math::cos(self.start_heading),
            sim_core::math::sin(self.start_heading),
        );
        let mut projected = Vec::new();
        for (x, y) in goals {
            let (x, y) = (x - self.start_x, y - self.start_y);
            if (x * x + y * y).sqrt() <= self.goal_radius {
                return Err("release pose must be outside both goals".into());
            }
            projected.push((x * dx + y * dy, -x * dy + y * dx));
        }
        let tolerance = 1e-9 * (1.0 + projected[0].0.abs() + projected[0].1.abs());
        if projected[0].0 <= 0.0
            || projected[0].1 <= 0.0
            || projected[1].1 >= 0.0
            || (projected[0].0 - projected[1].0).abs() > tolerance
            || (projected[0].1 + projected[1].1).abs() > tolerance
        {
            return Err("goals must be forward and mirror-symmetric about release heading".into());
        }
        Ok(())
    }
}

pub(crate) fn parse_config(path: &Path) -> RunResult<ArenaConfig> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| format!("cannot read arena {}: {e}", path.display()))?
        .take(65_537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65_536 {
        return Err("arena config exceeds 64KiB".into());
    }
    let config: ArenaConfig =
        toml::from_str(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)
            .map_err(|e| format!("invalid arena config: {e}"))?;
    config.validate()?;
    Ok(config)
}

/// Earliest intersection fraction along a continuous segment, preventing tunneling.
fn contact(from: Pose, to: Pose, center: (f64, f64), radius: f64) -> Option<f64> {
    let (x, y) = (from.x - center.0, from.y - center.1);
    let c = x * x + y * y - radius * radius;
    if c <= 0.0 {
        return Some(0.0);
    }
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let a = dx * dx + dy * dy;
    if a == 0.0 {
        return None;
    }
    let b = x * dx + y * dy;
    let discriminant = b * b - a * c;
    if discriminant < 0.0 {
        return None;
    }
    let t = (-b - discriminant.sqrt()) / a;
    (0.0..=1.0).contains(&t).then_some(t)
}

/// Both circles have exactly the same sensor type; the reward answer is absent.
fn sensors(config: &ArenaConfig, pose: Pose, cue: f64) -> Vec<f64> {
    let mut inputs = vec![0.0; config.input_count()];
    for ray in 0..config.ray_count {
        let offset = if config.ray_count == 1 {
            0.0
        } else {
            -config.ray_fov * 0.5 + config.ray_fov * ray as f64 / (config.ray_count - 1) as f64
        };
        let angle = pose.heading + offset;
        let end = Pose {
            x: pose.x + config.ray_range * sim_core::math::cos(angle),
            y: pose.y + config.ray_range * sim_core::math::sin(angle),
            heading: pose.heading,
        };
        let nearest = config
            .goals()
            .into_iter()
            .filter_map(|goal| contact(pose, end, goal, config.goal_radius))
            .min_by(f64::total_cmp);
        inputs[2 * ray] = nearest.map_or(0.0, |fraction| 1.0 - fraction);
        inputs[2 * ray + 1] = if nearest.is_some() { 1.0 } else { -1.0 };
    }
    let i = 2 * config.ray_count;
    inputs[i..].copy_from_slice(&[
        config.energy,
        pose.heading / PI - 1.0,
        cue,
        config.body,
        config.kin,
        config.temperature,
        config.pheromone,
        config.light,
        config.niche,
        config.food,
    ]);
    inputs
}

struct State {
    prev: [f64; MAX_NODES],
    act: [f64; MAX_NODES],
}
impl State {
    fn new() -> Self {
        Self {
            prev: [0.0; MAX_NODES],
            act: [0.0; MAX_NODES],
        }
    }
    fn think(&mut self, brain: &Brain, inputs: &[f64], clear: bool) -> RunResult<BrainOutput> {
        if clear {
            self.prev.fill(0.0);
        }
        let (output, activity) = brain.forward_with_memory(inputs, &self.prev, &mut self.act);
        if !activity.is_finite() || self.act.iter().any(|x| !x.is_finite()) {
            return Err("non-finite arena neural response".into());
        }
        self.prev.copy_from_slice(&self.act);
        Ok(output)
    }
}

fn condition(
    brain: &Brain,
    config: &ArenaConfig,
    clear: bool,
    cue_visible: bool,
    trace: bool,
    score_progress: bool,
) -> RunResult<Value> {
    let mut trials = Vec::new();
    for cue in [-1, 1] {
        let mut state = State::new();
        let mut pose = config.start();
        let initial_pose = pose;
        let mut hold_trace = Vec::new();
        for tick in 0..config.cue_ticks + config.blank_delay {
            let presenting = tick < config.cue_ticks;
            let inputs = sensors(config, pose, if presenting { f64::from(cue) } else { 0.0 });
            let actions = state.think(brain, &inputs, clear)?;
            if trace {
                hold_trace.push(json!({"phase":if presenting {"cue"}else{"blank"},"tick":tick+1,"pose":pose,"inputs":inputs,"actions":actions}));
            }
            // Position AND heading are frozen; no motor or action state carries over.
        }
        let mut trajectory = Vec::new();
        let mut choice = "timeout";
        let mut movement_ticks = 0;
        for tick in 1..=config.horizon {
            let inputs = sensors(config, pose, if cue_visible { f64::from(cue) } else { 0.0 });
            let actions = state.think(brain, &inputs, clear)?;
            let before = pose;
            let heading = (pose.heading + config.max_turn * actions.turn.clamp(-1.0, 1.0))
                .rem_euclid(2.0 * PI);
            let distance = config.max_speed * actions.speed.clamp(0.0, 1.0);
            let mut after = Pose {
                x: pose.x + distance * sim_core::math::cos(heading),
                y: pose.y + distance * sim_core::math::sin(heading),
                heading,
            };
            let hit = config
                .goals()
                .into_iter()
                .enumerate()
                .filter_map(|(goal, center)| {
                    contact(before, after, center, config.goal_radius).map(|t| (goal, t))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((goal, fraction)) = hit {
                after.x = before.x + (after.x - before.x) * fraction;
                after.y = before.y + (after.y - before.y) * fraction;
                choice = if goal == 0 { "left" } else { "right" };
            }
            if !after.x.is_finite() || !after.y.is_finite() || !after.heading.is_finite() {
                return Err("non-finite arena pose".into());
            }
            movement_ticks = tick;
            if trace {
                trajectory.push(json!({"tick":tick,"pose_before":before,"inputs":inputs,"actions":actions,"pose_after":after,"contact_fraction":hit.map(|(_,t)|t)}));
            }
            pose = after;
            if hit.is_some() {
                break;
            }
        }
        // The hidden answer is used ONLY here, after movement terminates.
        let target = if cue == 1 { "left" } else { "right" };
        let reward = u8::from(choice == target);
        let mut trial = json!({"cue":cue,"target":target,"choice":choice,"reward":reward,"movement_ticks":movement_ticks,"initial_pose":initial_pose,"hold_trace":hold_trace,"trajectory":trajectory});
        if score_progress {
            trial["terminal_progress"] = json!(terminal_progress(
                config,
                pose,
                if cue == 1 { 0 } else { 1 },
                reward == 1
            )?);
        }
        trials.push(trial);
    }
    let mean_reward = trials
        .iter()
        .map(|trial| trial["reward"].as_u64().unwrap())
        .sum::<u64>() as f64
        / 2.0;
    let mean_progress = score_progress.then(|| {
        trials
            .iter()
            .map(|t| t["terminal_progress"].as_f64().unwrap())
            .sum::<f64>()
            / 2.0
    });
    let mut result =
        json!({"clear_state_each_tick":clear,"mean_reward":mean_reward,"trials":trials});
    if let Some(progress) = mean_progress {
        result["mean_terminal_progress"] = json!(progress);
    }
    Ok(result)
}

fn evaluate_brain_mode(
    brain: &Brain,
    config: &ArenaConfig,
    cue_visible: bool,
    score_progress: bool,
) -> RunResult<Value> {
    config.validate()?;
    if brain.inputs != config.input_count() || brain.outputs != 7 || brain.slot_count() > MAX_NODES
    {
        return Err(
            "brain layout differs from arena ray/scalar input layout and seven outputs".into(),
        );
    }
    Ok(
        json!({"cue_visible":cue_visible,"intact":condition(brain,config,false,cue_visible,true,score_progress)?,"same_network_state_cleared":condition(brain,config,true,cue_visible,true,score_progress)?}),
    )
}

pub(crate) fn evaluate_brain(
    brain: &Brain,
    config: &ArenaConfig,
    cue_visible: bool,
) -> RunResult<Value> {
    evaluate_brain_mode(brain, config, cue_visible, false)
}

pub(crate) fn evaluate_brain_with_progress(
    brain: &Brain,
    config: &ArenaConfig,
) -> RunResult<Value> {
    evaluate_brain_mode(brain, config, true, true)
}

/// Terminal-only diagnostic consulted after motion ends, never by the controller.
fn terminal_progress(
    config: &ArenaConfig,
    terminal: Pose,
    target: usize,
    correct_contact: bool,
) -> RunResult<f64> {
    let (x, y) = config.goals()[target];
    let surface =
        |pose: Pose| ((pose.x - x).powi(2) + (pose.y - y).powi(2)).sqrt() - config.goal_radius;
    let start = surface(config.start());
    if !start.is_finite() || start <= 0.0 {
        return Err(
            "terminal progress requires a finite positive starting surface distance".into(),
        );
    }
    let raw_end = surface(terminal);
    if !raw_end.is_finite() {
        return Err("non-finite terminal surface distance".into());
    }
    // Confirmed swept correct contact is exactly full progress; rounding of the
    // intersection pose must not break ties between successful controllers.
    if correct_contact {
        return Ok(1.0);
    }
    let end = raw_end.max(0.0);
    Ok(((start - end) / start).clamp(0.0, 1.0))
}

pub(crate) fn visible_score_with_progress(
    brain: &Brain,
    config: &ArenaConfig,
) -> RunResult<(u8, f64)> {
    if brain.inputs != config.input_count() || brain.outputs != 7 || brain.slot_count() > MAX_NODES
    {
        return Err("brain layout differs from visible acquisition task".into());
    }
    let result = condition(brain, config, false, true, false, true)?;
    let correct = result["trials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["reward"].as_u64().unwrap() as u8)
        .sum();
    Ok((correct, result["mean_terminal_progress"].as_f64().unwrap()))
}

/// Training uses exactly the intact visible trials, without allocating step traces.
/// Only terminal correctness is selected; pose/target distances never enter fitness.
pub(crate) fn visible_score(brain: &Brain, config: &ArenaConfig) -> RunResult<u8> {
    if brain.inputs != config.input_count() || brain.outputs != 7 || brain.slot_count() > MAX_NODES
    {
        return Err("brain layout differs from visible acquisition task".into());
    }
    let result = condition(brain, config, false, true, false, false)?;
    Ok(result["trials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|trial| trial["reward"].as_u64().unwrap() as u8)
        .sum())
}

fn reference(config: &ArenaConfig, recurrent: bool) -> Brain {
    let turn = (config.input_count() + 1) as u16;
    let cue = (2 * config.ray_count + 3) as u16;
    let memory_edges = [(turn, turn, config.recurrent_weight, 3)];
    Brain::from_neat_with_memory(
        config.input_count(),
        7,
        &[],
        &[
            (0, turn, config.turn_bias, true),
            (cue, turn, config.cue_weight, true),
            (0, turn + 1, config.speed_bias, true),
        ],
        if recurrent { &memory_edges } else { &[] },
    )
}

pub(crate) fn validate_controls(config: &ArenaConfig, cue_visible: bool) -> RunResult<Value> {
    config.validate()?;
    let recurrent = reference(config, true);
    let plain = reference(config, false);
    let mut controls = vec![
        condition(&recurrent, config, false, cue_visible, true, false)?,
        condition(&plain, config, false, cue_visible, true, false)?,
        condition(&recurrent, config, true, cue_visible, true, false)?,
    ];
    for (control, name) in controls.iter_mut().zip([
        "recurrent_reference",
        "feed_forward_control",
        "same_network_state_cleared",
    ]) {
        control["name"] = json!(name);
    }
    let same_trajectories = |condition: &Value| {
        condition["trials"][0]["trajectory"] == condition["trials"][1]["trajectory"]
    };
    let held_fixed = controls.iter().all(|condition| {
        condition["trials"].as_array().unwrap().iter().all(|trial| {
            trial["hold_trace"]
                .as_array()
                .unwrap()
                .iter()
                .all(|tick| tick["pose"] == trial["initial_pose"])
        })
    });
    let common_release = controls.iter().all(|condition| {
        let cue_channel = 2 * config.ray_count + 2;
        let mut negative_inputs = condition["trials"][0]["trajectory"][0]["inputs"].clone();
        let mut positive_inputs = condition["trials"][1]["trajectory"][0]["inputs"].clone();
        if cue_visible {
            negative_inputs[cue_channel] = json!(0.0);
            positive_inputs[cue_channel] = json!(0.0);
        }
        condition["trials"][0]["initial_pose"] == condition["trials"][1]["initial_pose"]
            && negative_inputs == positive_inputs
    });
    let no_timeout = controls.iter().all(|condition| {
        condition["trials"]
            .as_array()
            .unwrap()
            .iter()
            .all(|trial| trial["choice"] != "timeout")
    });
    let mut checks = json!({"reference_reward_one":controls[0]["mean_reward"]==1.0,
        "hold_poses_fixed":held_fixed,"all_controls_make_a_choice":no_timeout});
    if cue_visible {
        let paths_differ = |condition: &Value| {
            let poses = |trial: &Value| {
                trial["trajectory"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|tick| tick["pose_after"].clone())
                    .collect::<Vec<_>>()
            };
            poses(&condition["trials"][0]) != poses(&condition["trials"][1])
        };
        let cue_persists = controls.iter().all(|condition| {
            condition["trials"].as_array().unwrap().iter().all(|trial| {
                trial["trajectory"].as_array().unwrap().iter().all(|tick| {
                    tick["inputs"][2 * config.ray_count + 2].as_f64() == trial["cue"].as_f64()
                })
            })
        });
        checks["feed_forward_reward_one"] = json!(controls[1]["mean_reward"] == 1.0);
        checks["cleared_reward_one"] = json!(controls[2]["mean_reward"] == 1.0);
        checks["feed_forward_choice_paths_differ_across_cues"] = json!(paths_differ(&controls[1]));
        checks["cleared_choice_paths_differ_across_cues"] = json!(paths_differ(&controls[2]));
        checks["release_pose_and_noncued_inputs_identical"] = json!(common_release);
        checks["cue_present_throughout_movement"] = json!(cue_persists);
    } else {
        checks["feed_forward_reward_half"] = json!(controls[1]["mean_reward"] == 0.5);
        checks["cleared_reward_half"] = json!(controls[2]["mean_reward"] == 0.5);
        checks["feed_forward_trajectories_identical_across_cues"] =
            json!(same_trajectories(&controls[1]));
        checks["cleared_trajectories_identical_across_cues"] =
            json!(same_trajectories(&controls[2]));
        checks["release_pose_and_inputs_identical"] = json!(common_release);
    }
    if !checks.as_object().unwrap().values().all(|v| v == true) {
        return Err(format!("delayed-choice controls failed: {checks}"));
    }
    Ok(
        json!({"schema_version":1,"assay":"delayed_choice_controls","cue_visible":cue_visible,"arena":config,"instrument_validated":true,"checks":checks,"controls":controls,
        "interpretation":if cue_visible {"visible-cue stimulus-response competence control; not memory use, learning, ecological fitness, evolved cognition or UAL"} else {"controlled movement and delayed cue retention; not learning, ecological fitness, evolved cognition or UAL"},
        "sensors":SENSOR_CONVENTIONS,"motion":MOTION_CONVENTIONS}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> ArenaConfig {
        toml::from_str(include_str!("../../../configs/delayed_choice.toml")).unwrap()
    }

    #[test]
    fn terminal_progress_is_bounded_terminal_only_and_preserves_rewards() {
        let config = config();
        assert_eq!(
            terminal_progress(&config, config.start(), 0, false).unwrap(),
            0.0
        );
        let nearer = Pose {
            x: -2.0,
            y: 2.0,
            heading: 0.0,
        };
        let closer = terminal_progress(&config, nearer, 0, false).unwrap();
        assert!(closer > 0.0 && closer < 1.0);
        assert_eq!(
            terminal_progress(
                &config,
                Pose {
                    x: 4.0,
                    y: -4.0,
                    heading: 0.0
                },
                0,
                false
            )
            .unwrap(),
            0.0
        );
        assert_eq!(terminal_progress(&config, nearer, 0, true).unwrap(), 1.0);
        assert!(
            terminal_progress(
                &config,
                Pose {
                    x: f64::NAN,
                    ..nearer
                },
                0,
                false
            )
            .is_err()
        );
        let mut invalid = config.clone();
        invalid.goal_radius = 32.0_f64.sqrt();
        assert!(terminal_progress(&invalid, config.start(), 0, false).is_err());
        let brain = reference(&config, false);
        let mut reported = evaluate_brain_with_progress(&brain, &config).unwrap();
        assert_eq!(
            visible_score_with_progress(&brain, &config).unwrap(),
            (2, 1.0)
        );
        for name in ["intact", "same_network_state_cleared"] {
            assert_eq!(reported[name]["mean_terminal_progress"], 1.0);
            reported[name]
                .as_object_mut()
                .unwrap()
                .remove("mean_terminal_progress");
            for trial in reported[name]["trials"].as_array_mut().unwrap() {
                trial.as_object_mut().unwrap().remove("terminal_progress");
            }
        }
        assert_eq!(
            reported,
            evaluate_brain(&brain, &config, true).unwrap(),
            "diagnostics must not change any inputs, actions, rewards or trajectories"
        );
    }

    #[test]
    fn controls_choose_and_external_state_cannot_carry_the_cue() {
        let config = config();
        let result = validate_controls(&config, false).unwrap();
        assert_eq!(result["instrument_validated"], true);
        for check in result["checks"].as_object().unwrap().values() {
            assert_eq!(check, true);
        }
        assert_eq!(result["controls"][0]["trials"][0]["choice"], "right");
        assert_eq!(result["controls"][0]["trials"][1]["choice"], "left");
        assert_eq!(result["controls"][1]["trials"][0]["choice"], "left");
    }

    #[test]
    fn visible_cue_controls_succeed_without_memory_and_hold_phases_are_unchanged() {
        let config = config();
        let hidden = validate_controls(&config, false).unwrap();
        let visible = validate_controls(&config, true).unwrap();
        assert_eq!(visible["cue_visible"], true);
        assert_eq!(visible["instrument_validated"], true);
        for (index, control) in visible["controls"].as_array().unwrap().iter().enumerate() {
            assert_eq!(control["mean_reward"], 1.0);
            for (trial_index, trial) in control["trials"].as_array().unwrap().iter().enumerate() {
                assert_eq!(trial["choice"], trial["target"]);
                assert_eq!(trial["movement_ticks"], 5);
                assert_eq!(
                    trial["hold_trace"],
                    hidden["controls"][index]["trials"][trial_index]["hold_trace"]
                );
                for step in trial["trajectory"].as_array().unwrap() {
                    assert_eq!(step["inputs"][12].as_f64(), trial["cue"].as_f64());
                }
            }
        }
    }

    #[test]
    fn trace_free_training_matches_the_instrument() {
        let config = config();
        for recurrent in [false, true] {
            let brain = reference(&config, recurrent);
            let detailed = condition(&brain, &config, false, true, true, false).unwrap();
            let compact = condition(&brain, &config, false, true, false, false).unwrap();
            for i in 0..2 {
                for key in [
                    "cue",
                    "target",
                    "choice",
                    "reward",
                    "movement_ticks",
                    "initial_pose",
                ] {
                    assert_eq!(detailed["trials"][i][key], compact["trials"][i][key]);
                }
            }
            assert_eq!(visible_score(&brain, &config).unwrap(), 2);
        }
    }

    #[test]
    fn identical_sensors_do_not_know_the_hidden_answer_and_trials_reset() {
        let config = config();
        let pose = config.start();
        let negative = sensors(&config, pose, -1.0);
        let positive = sensors(&config, pose, 1.0);
        let differing: Vec<_> = negative
            .iter()
            .zip(&positive)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(differing, vec![2 * config.ray_count + 2]);
        let brain = reference(&config, true);
        assert_eq!(
            evaluate_brain(&brain, &config, false).unwrap(),
            evaluate_brain(&brain, &config, false).unwrap()
        );
        let release = sensors(&config, pose, 0.0);
        assert_eq!(release[2 * config.ray_count + 2], 0.0);
    }

    #[test]
    fn swept_collision_finds_first_goal_even_when_endpoint_is_beyond_it() {
        let from = Pose {
            x: 0.0,
            y: 0.0,
            heading: 0.0,
        };
        let to = Pose {
            x: 10.0,
            y: 0.0,
            heading: 0.0,
        };
        assert_eq!(contact(from, to, (4.0, 0.0), 1.0), Some(0.3));
        assert_eq!(contact(from, to, (4.0, 2.0), 1.0), None);
    }

    #[test]
    fn timeout_and_wrong_first_goal_are_explicit_zero_rewards() {
        let mut config = config();
        config.horizon = 1;
        let result = evaluate_brain(&reference(&config, true), &config, false).unwrap();
        for trial in result["intact"]["trials"].as_array().unwrap() {
            assert_eq!(trial["choice"], "timeout");
            assert_eq!(trial["reward"], 0);
        }
        config.horizon = 20;
        let result = evaluate_brain(&reference(&config, false), &config, false).unwrap();
        assert_eq!(result["intact"]["trials"][0]["choice"], "left");
        assert_eq!(result["intact"]["trials"][0]["reward"], 0);
    }

    #[test]
    fn configuration_and_finite_neural_responses_are_checked() {
        let mut invalid = config();
        invalid.goal_left_x = invalid.goal_right_x;
        assert!(invalid.validate().is_err());
        let mut invalid = config();
        invalid.max_speed = f64::NAN;
        assert!(invalid.validate().is_err());
        let mut invalid = config();
        invalid.horizon = 0;
        assert!(invalid.validate().is_err());
        let mut invalid = config();
        invalid.goal_radius = 10.0;
        assert!(invalid.validate().is_err());
        let config = config();
        let turn = (config.input_count() + 1) as u16;
        let brain = Brain::from_neat(
            config.input_count(),
            7,
            &[],
            &[(0, turn, f64::MAX, true), (1, turn, f64::MAX, true)],
        );
        assert!(evaluate_brain(&brain, &config, false).is_err());
    }
}
