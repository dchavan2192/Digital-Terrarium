#!/usr/bin/env python3
"""Replay the archived close-gap paths with full food uptake at each correct landing."""
import argparse
import json
import tarfile

from check_food_cue_payoff import ARMS, check_run, near, require, validate_schedule
from run_food_cue_population import SEEDS, counts, differences
from run_food_cue_transfer import ROOT, digest


ARCHIVE = ROOT / "food_cue_population_raw.tar.gz"
SHA256 = "9240a2a0d2fcbc8b27a7ff8ef7fceb3ac03c46e7efd0b13b7f03593fed326b05"
POLICIES = ("recurrent", "boundary_reset")


def replay(run, config, *, full_uptake):
    energy = config["energy"]["starting"]
    threshold = config["reproduction"]["threshold"]
    investment = config["reproduction"]["parent_cost"]
    pulse = config["food"]["energy_per_food"]
    require(config["energy"]["maximum"] >= threshold + pulse, "energy cap could truncate a pulse")
    births = 0
    for event in run["events"]:
        # All income arrives on the final tick of an event. The largest possible
        # reserve before the meal is the starting reserve, since costs are nonnegative.
        require(0 < energy < threshold, "possible birth or death before pulse")
        costs = sum(event[k] for k in ("basal_cost", "movement_cost", "neural_cost"))
        require(energy - costs > 0, "insufficient reserve to survive until the meal")
        if full_uptake:
            income = pulse * event["food_units"] if event["choice"] == event["food"] else 0.
        else:
            near(energy, event["energy_before"], "reconstructed native starting reserve")
            income = event["algae_energy"]
        require(energy + income <= config["energy"]["maximum"], "possible capped energy")
        energy += income - costs
        born = int(energy >= threshold)
        energy -= born * investment
        births += born
        if not full_uptake:
            require(born == len(event["newborns"]), "native event birth mismatch")
            near(energy, event["energy_after"], "reconstructed native final reserve")
    if not full_uptake:
        require(births == run["offspring"], "native lifetime birth mismatch")
    return dict(policy=run["policy"], arm=run["arm"], direct_births=births)


def evaluate():
    require(digest(ARCHIVE) == SHA256, "archived population checksum changed")
    # Read the pinned archive sequentially without extracting files to disk.
    files = {}
    with tarfile.open(ARCHIVE, "r|gz") as stream:
        for member in stream:
            if member.name == "summary.json" or (member.name.startswith("distance_0.0625/")
                    and member.name.endswith(("/manifest.json", "/schedule.json", "/result.json"))):
                files[member.name] = json.load(stream.extractfile(member))
    close, = [row for row in files["summary.json"]["distances"] if row["distance"] == 0.0625]
    records = []
    for donor in close["donors"]:
        session = donor["session"]
        manifest = files[f"{session}/manifest.json"]
        schedule = files[f"{session}/schedule.json"]
        validate_schedule(schedule)
        runs = [r for r in files[f"{session}/result.json"]["runs"] if r["policy"] in POLICIES]
        require(len(runs) == 4 and {(r["policy"], r["arm"]) for r in runs} ==
                {(p, a) for p in POLICIES for a in ARMS}, "incomplete paired lifetimes")
        for run in runs:
            check_run(run, schedule, manifest)
        native = [replay(r, manifest["config"], full_uptake=False) for r in runs]
        observed = differences(native)
        require(observed == donor["birth_differences"], "archived donor contrasts disagree")
        meals = [replay(r, manifest["config"], full_uptake=True) for r in runs]
        records.append(dict(founder_seed=donor["founder_seed"], champion_id=donor["champion_id"],
                            passed_five_blanks=donor["passed_five_blanks"], native=observed,
                            full_uptake=differences(meals), native_births=native, full_uptake_births=meals))
    require(len(records) == 31 and {r["founder_seed"] for r in records} == SEEDS, "wrong cohort")
    passers = [r for r in records if r["passed_five_blanks"]]
    require(len(passers) == 8, "wrong earlier recall subgroup")
    totals = {group: {k: counts([{"birth_differences": r[k]} for r in members])
                      for k in ("native", "full_uptake")}
              for group, members in (("all", records), ("five_blank_passers", passers))}
    return dict(interpretation="Standardised potential payoff on archived paths and costs. "
                "Full uptake credits every correctly landed pulse; these projected births are "
                "an accounting counterfactual, not native births or a training result.",
                archive_sha256=SHA256, distance=close["distance"], schedule_seed=6400,
                totals=totals, donors=records)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.parse_args()
    print(json.dumps(evaluate(), indent=2, allow_nan=False))
