#!/usr/bin/env python3
"""Replay the frozen food-cue donors across four prespecified starting distances."""
import argparse
import json
from pathlib import Path
import subprocess
import tarfile

from check_food_cue_payoff import near, require
from run_food_cue_transfer import ROOT, digest, run, save

PILOT = ROOT / "food_cue_transfer_raw.tar.gz"
PILOT_SHA256 = "b1303de9464c0d691960b9ee28b287f3d2c98c01d20e22e86e5b5a09786d955f"
DISTANCES = (0.25, 0.125, 0.0625, 0.5)


def pilot_artifacts():
    require(digest(PILOT) == PILOT_SHA256, "prior transfer pilot archive hash mismatch")
    pilots = {}
    with tarfile.open(PILOT) as archive:
        for row in json.load(archive.extractfile("summary.json")):
            donor = (row["founder_seed"], row["training_arm"], row["champion_id"])
            artifacts = {}
            for name in ("manifest", "schedule", "result"):
                raw = archive.extractfile(f"{row['session']}/{name}.json").read()
                artifacts[name] = json.loads(raw)
                if name == "result":
                    artifacts["raw_result"] = raw
            require(donor not in pilots, "duplicated donor in prior pilot")
            pilots[donor] = artifacts
    require(len(pilots) == 3, "incomplete prior pilot")
    return pilots


def compare(session, prior, distance):
    manifest = json.loads((session / "manifest.json").read_bytes())
    original = prior["manifest"]
    for key in ("donor", "transferred", "config", "positive_patch", "negative_patch", "phases",
                "start_heading", "choice_speed_limit", "choice_turn_limit", "sign_convention"):
        require(manifest[key] == original[key], f"{session}: changed {key}")
    require(json.loads((session / "schedule.json").read_bytes()) == prior["schedule"],
            f"{session}: different pulse schedule")
    payload = (session / "result.json").read_bytes()
    if distance == 0.25:
        require(payload == prior["raw_result"], f"{session}: baseline result differs from archive")
    baseline = {(r["policy"], r["arm"]): r["events"] for r in prior["result"]["runs"]}
    landings = {}
    for r in json.loads(payload)["runs"]:
        key = (r["policy"], r["arm"])
        old = baseline[key]
        events = r["events"]
        require(len(old) == len(events), f"{session}: incomplete lifetime")
        sides = {}
        for food, name in [(1, "left"), (-1, "right")]:
            opportunities = [e for e in events if e["food"] == food]
            sides[name] = dict(opportunities=len(opportunities),
                               hits=sum(e["choice"] == food for e in opportunities),
                               outside_patches=sum(e["choice"] == 0 for e in opportunities),
                               meals_consumed=sum(e["algae_energy"] > 0 for e in opportunities),
                               direct_births=sum(len(e["newborns"]) for e in opportunities))
        for e, initial in zip(events, old):
            label = f"{session.name} {key} event {e['index']}"
            for field in ("choice_x", "choice_turn"):
                near(e[field], initial[field], f"{label}: changed {field}")
            near(e["choice_y"] - manifest["start"][1], initial["choice_y"] - original["start"][1],
                 f"{label}: changed forward displacement")
        landings[key] = sides
    return landings


def sweep(binary, output):
    pilots = pilot_artifacts()
    output.mkdir(parents=True, exist_ok=False)
    rows = []
    for distance in DISTANCES:
        folder = output / f"distance_{distance}"
        donors = run(binary, folder, 6400, distance)
        require({(d["founder_seed"], d["training_arm"], d["champion_id"]) for d in donors} == set(pilots),
                "distance cohort differs from prior archive")
        for donor in donors:
            key = (donor["founder_seed"], donor["training_arm"], donor["champion_id"])
            session = folder / donor["session"]
            sides = compare(session, pilots[key], distance)
            outcomes = [dict(**r, by_food_side=sides[r["policy"], r["arm"]]) for r in donor["outcomes"]]
            rows.append(dict(donor, session=str(session.relative_to(output)), outcomes=outcomes))
    save(output / "summary.json", dict(prior_archive=PILOT.name, prior_archive_sha256=PILOT_SHA256,
                                      schedule_seed=6400, distances=DISTANCES, donors=rows))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT.parents[1] / "target/release/sim-runner")
    parser.add_argument("--output", type=Path, required=True, help="new local output directory")
    args = parser.parse_args()
    try:
        sweep(args.binary, args.output)
        print(args.output / "summary.json")
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as exc:
        parser.exit(1, f"Distance study failed: {exc}\n")
