//! Read-only robustness diagnostics for training-selected task policies.
use crate::acquisition;
use crate::delayed_choice::{self, ArenaConfig};
use crate::run::{self, RunResult};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sim_core::brain::Brain;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    version: u32,
    cases: Vec<Case>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Case {
    Reference {
        name: String,
    },
    RigidRotation {
        name: String,
        radians: f64,
    },
    RelativeGoals {
        name: String,
        forward: f64,
        lateral: f64,
    },
}
impl Case {
    fn name(&self) -> &str {
        match self {
            Self::Reference { name }
            | Self::RigidRotation { name, .. }
            | Self::RelativeGoals { name, .. } => name,
        }
    }
    fn resolve(&self, base: &ArenaConfig) -> RunResult<ArenaConfig> {
        base.validate()?;
        let mut arena = base.clone();
        match self {
            Self::Reference { .. } => {}
            Self::RigidRotation { radians, .. } => {
                if !radians.is_finite() || radians.abs() > 2.0 * std::f64::consts::PI {
                    return Err(
                        "suite rotation must be finite and bounded by one revolution".into(),
                    );
                }
                let (sin, cos) = (sim_core::math::sin(*radians), sim_core::math::cos(*radians));
                let rotate = |x: f64, y: f64| {
                    (
                        base.start_x + (x - base.start_x) * cos - (y - base.start_y) * sin,
                        base.start_y + (x - base.start_x) * sin + (y - base.start_y) * cos,
                    )
                };
                (arena.goal_left_x, arena.goal_left_y) = rotate(base.goal_left_x, base.goal_left_y);
                (arena.goal_right_x, arena.goal_right_y) =
                    rotate(base.goal_right_x, base.goal_right_y);
                arena.start_heading =
                    (base.start_heading + radians).rem_euclid(2.0 * std::f64::consts::PI);
            }
            Self::RelativeGoals {
                forward, lateral, ..
            } => {
                if !forward.is_finite()
                    || !lateral.is_finite()
                    || *forward <= 0.0
                    || *lateral <= 0.0
                    || *forward > 1_000_000.0
                    || *lateral > 1_000_000.0
                {
                    return Err(
                        "relative goals require finite positive forward/lateral <=1000000".into(),
                    );
                }
                let (sin, cos) = (
                    sim_core::math::sin(base.start_heading),
                    sim_core::math::cos(base.start_heading),
                );
                arena.goal_left_x = base.start_x + forward * cos - lateral * sin;
                arena.goal_left_y = base.start_y + forward * sin + lateral * cos;
                arena.goal_right_x = base.start_x + forward * cos + lateral * sin;
                arena.goal_right_y = base.start_y + forward * sin - lateral * cos;
            }
        }
        arena.validate()?;
        Ok(arena)
    }
}

fn load_suite(path: &Path) -> RunResult<(Suite, String)> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(65_537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65_536 {
        return Err("validation suite exceeds64KiB".into());
    }
    let suite: Suite = toml::from_str(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)
        .map_err(|e| format!("invalid validation suite: {e}"))?;
    if suite.version != 1
        || suite.cases.is_empty()
        || suite.cases.len() > 16
        || !matches!(suite.cases[0], Case::Reference { .. })
        || suite
            .cases
            .iter()
            .filter(|c| matches!(c, Case::Reference { .. }))
            .count()
            != 1
    {
        return Err("suite v1 requires1..16 cases, exactly one reference placed first".into());
    }
    let mut names = BTreeSet::new();
    for case in &suite.cases {
        let name = case.name();
        if name.is_empty()
            || name.len() > 64
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || !names.insert(name)
        {
            return Err(
                "case names must be unique ASCII alphanumeric/underscore strings of length1..64"
                    .into(),
            );
        }
    }
    let fingerprint = bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    Ok((suite, format!("{fingerprint:016x}")))
}

fn trace_budget(arena: &ArenaConfig, cases: usize, policies: usize, genes: usize) -> RunResult<()> {
    let steps =
        u64::from(arena.cue_ticks) + u64::from(arena.blank_delay) + u64::from(arena.horizon);
    // Three fixed controls per case, each with intact+cleared balanced cues;
    // plus the original six manufactured control trials once per invocation.
    let trials = (policies as u64)
        .checked_add(3)
        .and_then(|n| n.checked_mul(cases as u64))
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| n.checked_add(6))
        .ok_or("validation trial budget overflow")?;
    let samples = (steps + 1)
        .checked_mul(trials)
        .ok_or("validation trace budget overflow")?;
    let passes = steps
        .checked_mul(trials)
        .ok_or("validation work overflow")?;
    if samples > 100_000
        || passes > 1_000_000
        || passes.saturating_mul(genes.max(4) as u64) > 100_000_000
    {
        return Err(
            "validation exceeds100000 trace samples,1000000 neural steps or100000000 edge-steps"
                .into(),
        );
    }
    Ok(())
}

fn without_progress(mut evaluation: Value) -> Value {
    for name in ["intact", "same_network_state_cleared"] {
        if let Some(condition) = evaluation.get_mut(name).and_then(Value::as_object_mut) {
            condition.remove("mean_terminal_progress");
            if let Some(trials) = condition.get_mut("trials").and_then(Value::as_array_mut) {
                for trial in trials {
                    if let Some(t) = trial.as_object_mut() {
                        t.remove("terminal_progress");
                    }
                }
            }
        }
    }
    evaluation
}
fn pose_action_paths(condition: &Value) -> Vec<Value> {
    condition["trials"].as_array().unwrap().iter().map(|t| json!({"hold":t["hold_trace"].as_array().unwrap().iter()
        .map(|s|json!([s["pose"],s["actions"]])).collect::<Vec<_>>(),
        "movement":t["trajectory"].as_array().unwrap().iter().map(|s|json!([s["pose_before"],s["pose_after"],s["actions"]])).collect::<Vec<_>>()})).collect()
}

pub fn run(source: &Path, suite_path: &Path, output: &Path) -> RunResult<()> {
    let selected = acquisition::load_saved_selection(source)?;
    let (suite, fingerprint) = load_suite(suite_path)?;
    let arenas = suite
        .cases
        .iter()
        .map(|c| c.resolve(&selected.arena))
        .collect::<RunResult<Vec<_>>>()?;
    let genes = selected
        .policies
        .iter()
        .filter_map(|p| p.genome.as_ref())
        .map(|g| g.connections.len())
        .max()
        .unwrap_or(4);
    trace_budget(
        &selected.arena,
        arenas.len(),
        selected.policies.len(),
        genes,
    )?;
    let mut resolved = Vec::new();
    for (case, arena) in suite.cases.iter().zip(&arenas) {
        let preflight = delayed_choice::geometry_preflight(arena)?;
        if !preflight["goals"].as_array().unwrap().iter().all(|g| {
            g["witness_within_horizon"] == true && g["initial_ray_hits"].as_u64().unwrap() > 0
        }) {
            return Err(format!(
                "case {} lacks a visible-goal turn-in-place/straight witness under this preflight; not a proof of physical impossibility",
                case.name()
            ));
        }
        resolved.push(json!({"case":case,"arena":arena,"geometry_preflight":preflight}));
    }
    let base_controls = delayed_choice::validate_controls(&selected.arena, true)?;
    // Original-case regression for EVERY completed policy precedes all new cases.
    let mut reference = Vec::new();
    for policy in &selected.policies {
        if let Some(genome) = &policy.genome {
            let evaluation =
                delayed_choice::evaluate_brain(&genome.build_brain(), &selected.arena, true)?;
            if evaluation != without_progress(policy.training["champion_training_trials"].clone()) {
                return Err(format!(
                    "seed {} original-case replay differs; no changed-case evaluation performed",
                    policy.seed
                ));
            }
            reference.push(Some(evaluation));
        } else {
            reference.push(None);
        }
    }
    // Account for regression replay: reused as the reference case, not run twice.
    let mut controls = Vec::new();
    for (case, arena) in suite.cases.iter().zip(&arenas) {
        let plain = delayed_choice::reference(arena, false);
        let recurrent = delayed_choice::reference(arena, true);
        let cue_independent = Brain::from_neat(
            arena.input_count(),
            7,
            &[],
            &[(0, (arena.input_count() + 2) as u16, 2.0, true)],
        );
        let independent = delayed_choice::evaluate_brain(&cue_independent, arena, true)?;
        let paths = pose_action_paths(&independent["intact"]);
        if paths[0] != paths[1] {
            return Err("cue-independent control has cue-dependent actions or poses".into());
        }
        controls.push(json!({"case":case.name(),"manufactured_feed_forward":delayed_choice::evaluate_brain(&plain,arena,true)?,
            "manufactured_recurrent":delayed_choice::evaluate_brain(&recurrent,arena,true)?,
            "cue_independent":independent,"cue_independent_paths_equal":true,
            "interpretation":"fixed controls are reported on every case, never retuned or used to filter policy outcomes"}));
    }
    let mut policies = Vec::new();
    for (policy, baseline) in selected.policies.iter().zip(reference) {
        if let Some(genome) = &policy.genome {
            let brain = genome.build_brain();
            let mut cases = vec![
                json!({"case":suite.cases[0].name(),"evaluation":baseline.unwrap(),"matches_training_replay":true}),
            ];
            for (case, arena) in suite.cases.iter().zip(&arenas).skip(1) {
                cases.push(json!({"case":case.name(),"evaluation":delayed_choice::evaluate_brain(&brain,arena,true)?}));
            }
            policies.push(json!({"seed":policy.seed,"status":"evaluated","training":policy.training,"cases":cases}));
        } else {
            policies.push(json!({"seed":policy.seed,"status":"source_training_failed","training":policy.training,"cases":null}));
        }
    }
    let result = json!({"schema_version":1,"task":"saved_policy_visible_validation","status":if selected.metadata["technical_failures"].as_u64().unwrap_or(0)>0 {"completed_with_source_failures"}else{"completed"},
        "suite":suite,"resolved_cases":resolved,"suite_path":suite_path,"suite_fnv1a64_noncryptographic":fingerprint,
        "source":selected.metadata,"baseline_instrument_controls":base_controls,"case_controls":controls,"policies":policies,
        "selection":"one declared final training champion for every source seed; failed source searches retained; no validation-score selection",
        "interpretation":"prospective saved-policy robustness diagnostic; rotations mainly probe heading; not fresh-run confirmation, memory, lifetime learning or ecology",
        "evaluator":{"source_revision":env!("TERRARIUM_SOURCE_REVISION"),"source_dirty":env!("TERRARIUM_SOURCE_DIRTY"),
            "core_build":sim_core::checkpoint::CORE_BUILD_ID,"compiler":env!("TERRARIUM_COMPILER"),"target":env!("TERRARIUM_TARGET"),"profile":env!("TERRARIUM_BUILD_PROFILE")}});
    let destination = run::create_session(output)?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination.join("result.json"))
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(file, &result).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn arena() -> ArenaConfig {
        toml::from_str(include_str!("../../../configs/delayed_choice.toml")).unwrap()
    }
    #[test]
    fn frozen_suite_has_visible_reachable_cases_before_policy_evaluation() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/visible_validation.toml");
        let (suite, _) = load_suite(&path).unwrap();
        assert_eq!(suite.cases.len(), 7);
        for case in suite.cases {
            let resolved = case.resolve(&arena()).unwrap();
            let preflight = delayed_choice::geometry_preflight(&resolved).unwrap();
            assert!(
                preflight["goals"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|g| g["witness_within_horizon"] == true
                        && g["initial_ray_hits"].as_u64().unwrap() > 0),
                "{}",
                case.name()
            );
        }
    }

    #[test]
    fn rotation_preserves_relative_labels_and_changes_heading_only() {
        let base = arena();
        let rotated = Case::RigidRotation {
            name: "r".into(),
            radians: std::f64::consts::FRAC_PI_4,
        }
        .resolve(&base)
        .unwrap();
        assert!((rotated.start_heading - 3.0 * std::f64::consts::FRAC_PI_4).abs() < 1e-12);
        assert!((rotated.goal_left_x + 32.0_f64.sqrt()).abs() < 1e-12);
        assert!(rotated.goal_left_y.abs() < 1e-12);
        assert!(rotated.goal_right_x.abs() < 1e-12);
        assert!((rotated.goal_right_y - 32.0_f64.sqrt()).abs() < 1e-12);
        assert_eq!(rotated.horizon, base.horizon);
        assert_eq!(rotated.goal_radius, base.goal_radius);
    }
    #[test]
    fn relative_goals_use_release_frame_and_preflight_is_policy_free() {
        let mut base = arena();
        base.start_heading = 0.0;
        // Replace the base world goals consistently with the changed release frame.
        base.goal_left_x = 4.0;
        base.goal_left_y = 4.0;
        base.goal_right_x = 4.0;
        base.goal_right_y = -4.0;
        let changed = Case::RelativeGoals {
            name: "x".into(),
            forward: 3.0,
            lateral: 2.5,
        }
        .resolve(&base)
        .unwrap();
        assert_eq!((changed.goal_left_x, changed.goal_left_y), (3.0, 2.5));
        assert_eq!((changed.goal_right_x, changed.goal_right_y), (3.0, -2.5));
        let p = delayed_choice::geometry_preflight(&changed).unwrap();
        assert!(
            p["goals"]
                .as_array()
                .unwrap()
                .iter()
                .all(|g| g["witness_within_horizon"] == true)
        );
        assert!(trace_budget(&arena(), 7, 3, 231).is_ok());
        assert!(trace_budget(&arena(), usize::MAX, usize::MAX, usize::MAX).is_err());
    }
}
