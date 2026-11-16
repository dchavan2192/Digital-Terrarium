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

fn condition(brain: &Brain, config: &ArenaConfig, clear: bool) -> RunResult<Value> {
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
            hold_trace.push(json!({"phase":if presenting {"cue"}else{"blank"},"tick":tick+1,"pose":pose,"inputs":inputs,"actions":actions}));
            // Position AND heading are frozen; no motor or action state carries over.
        }
        let mut trajectory = Vec::new();
        let mut choice = "timeout";
        for tick in 1..=config.horizon {
            let inputs = sensors(config, pose, 0.0);
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
            trajectory.push(json!({"tick":tick,"pose_before":before,"inputs":inputs,"actions":actions,"pose_after":after,"contact_fraction":hit.map(|(_,t)|t)}));
            pose = after;
            if hit.is_some() {
                break;
            }
        }
        // The hidden answer is used ONLY here, after movement terminates.
        let target = if cue == 1 { "left" } else { "right" };
        let reward = u8::from(choice == target);
        trials.push(json!({"cue":cue,"target":target,"choice":choice,"reward":reward,"movement_ticks":trajectory.len(),"initial_pose":initial_pose,"hold_trace":hold_trace,"trajectory":trajectory}));
    }
    let mean_reward = trials
        .iter()
        .map(|trial| trial["reward"].as_u64().unwrap())
        .sum::<u64>() as f64
        / 2.0;
    Ok(json!({"clear_state_each_tick":clear,"mean_reward":mean_reward,"trials":trials}))
}

pub(crate) fn evaluate_brain(brain: &Brain, config: &ArenaConfig) -> RunResult<Value> {
    config.validate()?;
    if brain.inputs != config.input_count() || brain.outputs != 7 || brain.slot_count() > MAX_NODES
    {
        return Err(
            "brain layout differs from arena ray/scalar input layout and seven outputs".into(),
        );
    }
    Ok(
        json!({"intact":condition(brain,config,false)?,"same_network_state_cleared":condition(brain,config,true)?}),
    )
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

pub(crate) fn validate_controls(config: &ArenaConfig) -> RunResult<Value> {
    config.validate()?;
    let recurrent = reference(config, true);
    let plain = reference(config, false);
    let mut controls = vec![
        condition(&recurrent, config, false)?,
        condition(&plain, config, false)?,
        condition(&recurrent, config, true)?,
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
        condition["trials"][0]["initial_pose"] == condition["trials"][1]["initial_pose"]
            && condition["trials"][0]["trajectory"][0]["inputs"]
                == condition["trials"][1]["trajectory"][0]["inputs"]
    });
    let no_timeout = controls.iter().all(|condition| {
        condition["trials"]
            .as_array()
            .unwrap()
            .iter()
            .all(|trial| trial["choice"] != "timeout")
    });
    let checks = json!({"reference_reward_one":controls[0]["mean_reward"]==1.0,
        "feed_forward_reward_half":controls[1]["mean_reward"]==0.5,
        "cleared_reward_half":controls[2]["mean_reward"]==0.5,
        "feed_forward_trajectories_identical_across_cues":same_trajectories(&controls[1]),
        "cleared_trajectories_identical_across_cues":same_trajectories(&controls[2]),
        "hold_poses_fixed":held_fixed,"release_pose_and_inputs_identical":common_release,"all_controls_make_a_choice":no_timeout});
    if !checks.as_object().unwrap().values().all(|v| v == true) {
        return Err(format!("delayed-choice controls failed: {checks}"));
    }
    Ok(
        json!({"schema_version":1,"assay":"delayed_choice_controls","arena":config,"instrument_validated":true,"checks":checks,"controls":controls,
        "interpretation":"controlled movement and delayed cue retention; not learning, ecological fitness, evolved cognition or UAL",
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
    fn controls_choose_and_external_state_cannot_carry_the_cue() {
        let config = config();
        let result = validate_controls(&config).unwrap();
        assert_eq!(result["instrument_validated"], true);
        for check in result["checks"].as_object().unwrap().values() {
            assert_eq!(check, true);
        }
        assert_eq!(result["controls"][0]["trials"][0]["choice"], "right");
        assert_eq!(result["controls"][0]["trials"][1]["choice"], "left");
        assert_eq!(result["controls"][1]["trials"][0]["choice"], "left");
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
            evaluate_brain(&brain, &config).unwrap(),
            evaluate_brain(&brain, &config).unwrap()
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
        let result = evaluate_brain(&reference(&config, true), &config).unwrap();
        for trial in result["intact"]["trials"].as_array().unwrap() {
            assert_eq!(trial["choice"], "timeout");
            assert_eq!(trial["reward"], 0);
        }
        config.horizon = 20;
        let result = evaluate_brain(&reference(&config, false), &config).unwrap();
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
        assert!(evaluate_brain(&brain, &config).is_err());
    }
}
