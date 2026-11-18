#!/usr/bin/env python3
"""Derive a browser replay from saved native results; never evaluate a policy."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = ROOT / "output/visible-validation/robustness_20261118T020500Z"
DEFAULT_DEST = ROOT / "web/demo/data/visible-task.json"
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


def export(source=DEFAULT_SOURCE):
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
        sources.append({"cohort": cohort, "path": path.relative_to(ROOT).as_posix(),
                        "sha256": hashlib.sha256(raw).hexdigest(), "evaluator": result["evaluator"]})
        for policy in result["policies"]:
            policies.append({"cohort": cohort, "seed": policy["seed"],
                             "champion": policy["training"]["champion_id"],
                             "cases": pack_cases(policy["cases"])})
    for name in CONTROL_NAMES:
        policies.append({"cohort": "controls", "name": name,
                         "cases": pack_cases([{"case": c["case"], "evaluation": c[name]}
                                              for c in controls])})
    cases = [{"name": c["case"]["name"], "kind": c["case"]["kind"],
              "arena": {key: c["arena"][key] for key in (
                  "goal_left_x", "goal_left_y", "goal_right_x", "goal_right_y",
                  "goal_radius", "horizon", "start_x", "start_y", "start_heading")}}
             for c in resolved]
    assert len(cases) == 7 and len(policies) == 12
    for policy in policies:
        assert [c["name"] for c in policy["cases"]] == [c["name"] for c in cases]
        assert all(sorted(t["cue"] for t in c["trials"]) == [-1, 1] for c in policy["cases"])
    return {"version": 1, "description": "Prerecorded native visible-cue task evaluation; all declared training champions retained.",
            "sources": sources, "cases": cases, "policies": policies}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE)
    parser.add_argument("--output", type=Path, default=DEFAULT_DEST)
    args = parser.parse_args()
    payload = json.dumps(export(args.source), separators=(",", ":"), allow_nan=False) + "\n"
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(payload)
    print(f"{args.output}: {len(payload.encode())} bytes; 63 champion cases / 126 cue trials + fixed controls")
