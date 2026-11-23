#!/usr/bin/env python3
"""Derive a browser replay from saved native results; never evaluate a policy."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = ROOT / "output/visible-validation/robustness_20261118T020500Z"
DEFAULT_DEST = ROOT / "web/demo/data/visible-task.json"
DEFAULT_DIVERSITY = ROOT / "output/visible-diversity/pilot_20261122T233000Z/runs/session_1795391295051362357_59243_0"
COHORTS = {"original": [2000, 2001, 2002], "terminal_only": [2100, 2101, 2102],
           "terminal_progress": [2100, 2101, 2102]}
CONTROL_NAMES = ("cue_independent", "manufactured_feed_forward", "manufactured_recurrent")


def pack_trial(trial):
    trace = trial["trajectory"]
    assert len(trace) == trial["movement_ticks"] and trace
    poses = [trial["initial_pose"]] + [step["pose_after"] for step in trace]
    return {key: trial[key] for key in ("cue", "target", "choice", "reward", "movement_ticks")} | {
        "poses": [[p["x"], p["y"], p["heading"]] for p in poses],
        "initial_action": {key: trace[0]["actions"][key] for key in ("turn", "speed")},
        "contact_fraction": trace[-1]["contact_fraction"],
    }


def pack_cases(cases):
    return [{"name": case["case"], "trials": [pack_trial(t) for t in
             case["evaluation"]["intact"]["trials"]]} for case in cases]


def identity(path, **metadata):
    raw = path.read_bytes()
    return {**metadata, "path": path.relative_to(ROOT).as_posix(),
            "sha256": hashlib.sha256(raw).hexdigest()}


def pack_arena(name, kind, arena, study):
    return {"name": name, "kind": kind, "study": study,
            "arena": {key: arena[key] for key in (
                "goal_left_x", "goal_left_y", "goal_right_x", "goal_right_y",
                "goal_radius", "horizon", "start_x", "start_y", "start_heading")}}


def export(source=DEFAULT_SOURCE, diversity=DEFAULT_DIVERSITY):
    policies, sources, resolved, controls = [], [], None, None
    for cohort, seeds in COHORTS.items():
        paths = sorted((source / cohort / "runs").glob("session_*/result.json"))
        if len(paths) != 1:
            raise ValueError(f"Expected exactly one saved result for {cohort}")
        path = paths[0]
        raw = path.read_bytes()
        result = json.loads(raw)
        assert [p["seed"] for p in result["policies"]] == seeds
        assert result["status"] == "completed"
        if resolved is None:
            resolved = result["resolved_cases"]
            controls = result["case_controls"]
        else:
            assert result["resolved_cases"] == resolved
            assert result["case_controls"] == controls
        sources.append(identity(path, cohort=cohort, study="robustness", role="evaluations", evaluator=result["evaluator"]))
        for policy in result["policies"]:
            policies.append({"cohort": cohort, "study": "robustness", "seed": policy["seed"],
                             "champion": policy["training"]["champion_id"],
                             "cases": pack_cases(policy["cases"])})
    for name in CONTROL_NAMES:
        policies.append({"cohort": "controls", "study": "robustness", "name": name,
                         "cases": pack_cases([{"case": c["case"], "evaluation": c[name]}
                                              for c in controls])})
    cases = [pack_arena(c["case"]["name"], c["case"]["kind"], c["arena"], "robustness") for c in resolved]
    older_revision = sources[0]["evaluator"]["source_revision"]
    manifest = json.loads((diversity / "manifest.json").read_bytes())
    summary = json.loads((diversity / "summary.json").read_bytes())
    lock = json.loads((diversity / "selection_lock.json").read_bytes())
    assert summary["status"] == "completed" and summary["technical_failures"] == 0
    assert lock["status"] == "locked" and lock["selected_champions"] == 8
    assert len(summary["runs"]) == 8 and manifest["protocol"]["seeds"] == [5000, 5001, 5002, 5003]
    evaluator = {key: manifest[key] for key in ("source_revision", "source_dirty", "core_build", "compiler", "target", "build_profile")}
    sources.append(identity(diversity / "summary.json", study="diversity", role="evaluations",
                            cohorts=["diversity_fixed", "diversity_varied"], evaluator=evaluator))
    sources.append(identity(diversity / "manifest.json", study="diversity", role="manifest"))
    sources.append(identity(diversity / "selection_lock.json", study="diversity", role="selection_lock"))
    kinds = {case["name"]: case["kind"] for case in manifest["protocol"]["holdouts"]}
    assert len(kinds) == 6
    cases.extend(pack_arena(c["case"], kinds[c["case"]], c["arena"], "diversity") for c in manifest["holdout_cases"])
    for arm in ("fixed", "varied"):
        for seed in manifest["protocol"]["seeds"]:
            matches = [run for run in summary["runs"] if run["arm"] == arm and run["seed"] == seed]
            assert len(matches) == 1
            run = matches[0]
            assert run["status"] == "completed" and all(c["status"] == "completed" for c in run["holdout_cases"])
            selected = [r for r in lock["runs"] if r["arm"] == arm and r["seed"] == seed]
            assert len(selected) == 1 and selected[0]["champion_id"] == run["champion_id"]
            policies.append({"cohort": f"diversity_{arm}", "study": "diversity", "seed": seed,
                             "champion": run["champion_id"], "cases": pack_cases(run["holdout_cases"])})
    assert len(cases) == 13 and len(policies) == 20
    for policy in policies:
        assert [c["name"] for c in policy["cases"]] == [c["name"] for c in cases if c["study"] == policy["study"]]
        assert all(sorted(t["cue"] for t in c["trials"]) == [-1, 1] for c in policy["cases"])
    studies = [
        {"id": "robustness", "label": "Nov 18 · earlier visible robustness study", "evaluator": older_revision,
         "note": "Nine training-selected champions, seven evaluation cases; earlier engine revision. Fixed controls belong to this study.",
         "report": "https://github.com/dchavan2192/Digital-Terrarium/blob/main/docs/results/VISIBLE_VALIDATION_2026-11-17.md"},
        {"id": "diversity", "label": "Nov 22 · fixed versus varied training pilot", "evaluator": evaluator["source_revision"],
         "note": "Eight training-selected champions, six fresh held-out cases. Four paired seeds; mixed outcomes do not establish an improvement. All champions locked before held-out evaluation.",
         "report": "https://github.com/dchavan2192/Digital-Terrarium/blob/main/docs/results/VISIBLE_DIVERSITY_PILOT_2026-11-22.md"},
    ]
    for study, plan in zip(studies, ("VISIBLE_VALIDATION_PLAN_2026-11-17.md", "VISIBLE_DIVERSITY_PLAN_2026-11-18.md")):
        study["plan"] = f"https://github.com/dchavan2192/Digital-Terrarium/blob/main/docs/results/{plan}"
    return {"version": 2, "description": "Prerecorded native visible-cue task evaluations from two separate studies; all 17 declared training champions retained.",
            "studies": studies, "sources": sources, "cases": cases, "policies": policies}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE)
    parser.add_argument("--diversity-session", type=Path, default=DEFAULT_DIVERSITY)
    parser.add_argument("--output", type=Path, default=DEFAULT_DEST)
    args = parser.parse_args()
    payload = json.dumps(export(args.source, args.diversity_session), separators=(",", ":"), allow_nan=False) + "\n"
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(payload)
    print(f"{args.output}: {len(payload.encode())} bytes; 17 champions / 111 champion cases / 222 cue trials + earlier fixed controls")
