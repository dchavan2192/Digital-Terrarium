#!/usr/bin/env python3
"""Run the three frozen sustained-recall donors on one matched local food schedule."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

from check_food_cue_payoff import audit, require
from retention_feedback_subsets import COHORTS, ROOT

ARCHIVE_SHA256 = {
    "retention_wiring_raw.tar.gz": "52355d4dc66ecd6be7abd8a6b490fc2cf55b2d93b6809eb2450a62becfaf1dc7",
    "delay_training_raw.tar.gz": "5ce48820f9fa7cd7da872fbd5e51fc5476d709625fb1f69ebedb525e03b415c1",
}


def digest(path):
    with path.open("rb") as stream:
        hasher = hashlib.sha256()
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(block)
        return hasher.hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def run(binary, output, schedule_seed, distance=0.25):
    binary = binary.resolve(strict=True)
    binary_hash = digest(binary)
    output.mkdir(parents=True, exist_ok=False)
    summaries = []
    common_schedule = None
    for archive, cohort in COHORTS.items():
        path = ROOT / archive
        archive_hash = digest(path)
        require(archive_hash == ARCHIVE_SHA256[archive], f"frozen archive changed: {archive}")
        with tarfile.open(path) as stream:
            rows = json.load(stream.extractfile("selection.json"))
        donors = [row for row in rows if (row[0], row[1]) in cohort]
        require(len(donors) == len(cohort) and {(s, a) for s, a, _, _ in donors} == cohort,
                f"selected donor records missing or duplicated in {archive}")
        for seed, arm, champion, genome in donors:
            folder = output / f"{seed}_{arm}"
            folder.mkdir()
            source = folder / "donor.json"
            save(source, genome)
            provenance = dict(archive=archive, archive_sha256=archive_hash, member="selection.json",
                              founder_seed=seed, training_arm=arm, champion_id=champion,
                              donor_sha256=digest(source), binary_sha256=binary_hash,
                              schedule_seed=schedule_seed, distance=distance)
            save(folder / "provenance.json", provenance)
            subprocess.run([str(binary), "food-cue-transfer", "--donor", str(source),
                            "--output", str(folder), "--seed", str(schedule_seed),
                            "--distance", str(distance)], check=True)
            sessions = list(folder.glob("session_*"))
            require(len(sessions) == 1, f"expected one completed session for {seed} {arm}")
            session = sessions[0]
            manifest = json.loads((session / "manifest.json").read_text())
            require(manifest["task"] == "food_cue_transfer" and manifest["donor"] == genome,
                    "native session did not use frozen genome")
            require(manifest["schedule_seed"] == schedule_seed, "incorrect schedule seed")
            require(manifest["start"] == [40.0, 40.0 - distance], "incorrect starting distance")
            schedule = json.loads((session / "schedule.json").read_text())
            if common_schedule is None:
                common_schedule = schedule
            require(schedule == common_schedule, "pulse schedule changed between donors")
            summaries.append(dict(**provenance, session=str(session.relative_to(output)),
                                  outcomes=audit(session)))
    save(output / "summary.json", summaries)
    return summaries


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT.parents[1] / "target/release/sim-runner")
    parser.add_argument("--output", type=Path, required=True, help="new local output directory")
    parser.add_argument("--seed", type=int, default=6400)
    parser.add_argument("--distance", type=float, default=0.25, help="starting distance below food row")
    args = parser.parse_args()
    try:
        run(args.binary, args.output, args.seed, args.distance)
        print(args.output / "summary.json")
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as exc:
        parser.exit(1, f"Transfer failed: {exc}\n")
