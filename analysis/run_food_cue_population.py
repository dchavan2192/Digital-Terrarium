#!/usr/bin/env python3
"""Measure paired direct-offspring differences in the held-out wiring-study founders."""
import argparse
import json
from pathlib import Path
import subprocess

from check_food_cue_payoff import require
from run_food_cue_distance import compare
from run_food_cue_transfer import ROOT, digest, run, save

SEEDS = set(range(6200, 6232)) - {6226}
COHORT = {"retention_wiring_raw.tar.gz": {(seed, "selected") for seed in SEEDS}}
PAIRS = ROOT / "retention_wiring_pairs.json"
PAIRS_HASH = "b47cb4a9ca98d14d26c4e52ac087e2e8fd4c1b69f6459dd91f610f0e350aa291"


def five_blank_status():
    require(digest(PAIRS) == PAIRS_HASH, "previous wiring status checksum changed")
    rows = [r for r in json.loads(PAIRS.read_bytes()) if r["seed"] in SEEDS
            and r["arm"] == "selected" and r["delay"] == 5]
    require(len(rows) == len(SEEDS) and {r["seed"] for r in rows} == SEEDS,
            "expected one archived five-blank result per founder")
    require(all(type(r["passed"]) is bool for r in rows), "invalid archived recall status")
    return {r["seed"]: r["passed"] for r in rows}


def differences(outcomes):
    births = {(r["policy"], r["arm"]): r["direct_births"] for r in outcomes}
    diffs = {arm: births["recurrent", arm] - births["boundary_reset", arm]
             for arm in ("predictive", "shuffled")}
    return dict(**diffs, interaction=diffs["predictive"] - diffs["shuffled"])


def counts(donors):
    results = {}
    for metric in ("predictive", "shuffled", "interaction"):
        values = [d["birth_differences"][metric] for d in donors]
        results[metric] = dict(positive=sum(v > 0 for v in values), zero=values.count(0),
                               negative=sum(v < 0 for v in values))
    return dict(founders=len(donors), **results)


def population(binary, output):
    previous = five_blank_status()
    output.mkdir(parents=True, exist_ok=False)
    baseline = {}
    distances = []
    for gap in (0.25, 0.0625):
        folder = output / f"distance_{gap}"
        donors = run(binary, folder, 6400, gap, cohorts=COHORT)
        require(len(donors) == len(SEEDS) and {d["founder_seed"] for d in donors} == SEEDS,
                "incomplete transfer cohort")
        records = []
        for donor in donors:
            seed = donor["founder_seed"]
            session = folder / donor["session"]
            if gap == 0.25:
                baseline[seed] = {name: json.loads((session / f"{name}.json").read_bytes())
                                  for name in ("manifest", "schedule", "result")}
            else:
                compare(session, baseline[seed], gap)
            records.append(dict(donor, session=str(session.relative_to(output)),
                                passed_five_blanks=previous[seed],
                                birth_differences=differences(donor["outcomes"])))
        distances.append(dict(distance=gap, totals=counts(records), donors=records,
                              by_five_blank_status={
                                  label: counts([d for d in records if d["passed_five_blanks"] == status])
                                  for status, label in [(True, "passed"), (False, "failed")]
                              }))
    save(output / "summary.json",
         dict(schedule_seed=6400, primary_distance=0.0625, earlier_status_file=PAIRS.name,
              earlier_status_sha256=PAIRS_HASH, distances=distances))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT.parents[1] / "target/release/sim-runner")
    parser.add_argument("--output", type=Path, required=True, help="new local output directory")
    args = parser.parse_args()
    try:
        population(args.binary, args.output)
        print(args.output / "summary.json")
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as exc:
        parser.exit(1, f"Population study failed: {exc}\n")
