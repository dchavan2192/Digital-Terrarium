#!/usr/bin/env python3
"""Audit the saved two-patch food cue instrument without running the simulation."""
import argparse
from collections import Counter
import json
import math
from pathlib import Path


DELAYS = (2, 5, 8, 11)
POLICIES = ("recurrent", "boundary_reset", "visible_reset", "feed_forward")
ARMS = ("predictive", "shuffled")
FOOD_ENERGY = 15.0
PARENT_COST = 30.0


def require(condition, message):
    if not condition:
        raise ValueError(message)


def number(value, label):
    require(type(value) in (float, int) and math.isfinite(value), f"invalid {label}")
    return value


def integer(value, label):
    require(type(value) is int, f"invalid {label}")
    return value


def near(actual, expected, label):
    actual = number(actual, label)
    if not math.isclose(actual, expected, rel_tol=1e-10, abs_tol=1e-8):
        raise ValueError(f"{label}: got {actual}, expected {expected}")


def validate_schedule(schedule):
    require(len(schedule) == 16, "expected 16 scheduled exposures")
    strata = Counter()
    for event in schedule:
        delay = integer(event["delay"], "delay")
        food = integer(event["food"], "food patch")
        shuffled = integer(event["shuffled"], "shuffled cue")
        strata[delay, food, shuffled] += 1
    expected = Counter((d, food, cue) for d in DELAYS for food in (-1, 1) for cue in (-1, 1))
    require(strata == expected, "expected every (delay, food patch, shuffled cue) exactly once")


def check_run(run, schedule, manifest):
    arm, policy = run["arm"], run["policy"]
    config = manifest["config"]
    cell_size = number(config["food"]["cell_size"], "food cell size")
    require(cell_size > 0, "food cell size must be positive")
    patches = {1: manifest["positive_patch"], -1: manifest["negative_patch"]}
    require(patches == {1: [3, 4], -1: [4, 4]}, "incorrect left/right patch geometry")
    events = run["events"]
    require(len(events) == len(schedule), "incomplete event sequence")
    founder = integer(events[0]["founder_id"], "founder id")
    reproduction = config["reproduction"]
    threshold = number(reproduction["threshold"], "asexual reproduction threshold")
    starting_energy = number(config["energy"]["starting"], "starting reserve")
    near(starting_energy, 45, "pinned starting reserve")
    near(threshold, 60, "pinned reproduction threshold")
    near(reproduction["parent_cost"], PARENT_COST, "pinned parent investment")
    child_energy = number(reproduction["child_energy"], "newborn endowment")
    near(child_energy, 30, "pinned newborn endowment")
    require(reproduction["sexual"] is False, "expected asexual reproduction")
    near(config["food"]["energy_per_food"], FOOD_ENERGY, "pinned algae energy")
    child_ids = set()
    previous_energy = starting_energy
    tick = hits = 0
    choices = []
    for i, (event, planned) in enumerate(zip(events, schedule)):
        label = f"{arm}/{policy} event {i}"
        delay, food = planned["delay"], planned["food"]
        cue = food if arm == "predictive" else planned["shuffled"]
        require(integer(event["start_tick"], label) == tick, f"{label}: incorrect start tick")
        tick += delay + 3  # one cue, blank ticks, one movement decision, one feeding tick
        require(integer(event["index"], label) == i, f"{label}: event index")
        require(integer(event["pulse_tick"], label) == tick, f"{label}: food pulse timing")
        for field, value in [("food", food), ("delay", delay), ("cue", cue), ("founder_id", founder)]:
            require(integer(event[field], field) == value, f"{label}: {field} mismatch")
        near(event["food_units"], 1.0, f"{label}: food supplied")
        choice = integer(event["choice"], "selected patch")
        require(choice in (-1, 1), f"{label}: invalid choice")
        cell = [math.floor(number(event[key], key) / cell_size) for key in ("choice_x", "choice_y")]
        require(cell == patches[choice], f"{label}: chosen patch disagrees with committed position")
        require(event["pulse_cell"] == patches[food], f"{label}: pulse delivered to wrong cell")
        choices.append(choice)
        hit = int(choice == food)
        hits += hit
        near(event["food_remaining"], 1 - hit, f"{label}: remaining food in pulse cell")
        near(event["algae_energy"], FOOD_ENERGY * hit, f"{label}: energy harvested")
        before = number(event["energy_before"], label)
        near(before, previous_energy, f"{label}: founder reserve carry-forward")
        costs = [number(event[key], key) for key in ("basal_cost", "movement_cost", "neural_cost")]
        require(all(c >= 0 for c in costs), f"{label}: negative expense")
        children = event["newborns"]
        # No positive energy input before the single meal. Starting a trial below
        # threshold rules out earlier births, and only the focal founder reproduces.
        pre_birth = before + FOOD_ENERGY * hit - sum(costs)
        require(before < threshold, f"{label}: threshold already met before cue")
        require(len(children) == int(pre_birth >= threshold), f"{label}: births disagree with threshold")
        for child in children:
            identifier = integer(child["id"], "newborn id")
            require(identifier != founder and identifier not in child_ids, f"{label}: repeated ID")
            require(integer(child["parent_id"], "parent id") == founder, f"{label}: wrong parent")
            require(integer(child["tick"], "birth tick") == tick, f"{label}: birth outside feeding tick")
            require(integer(child["native_birth_tick"], "native birth tick") == tick - 1,
                    f"{label}: zero-based core birth tick disagrees with completed feeding step")
            near(child["energy"], child_energy, f"{label}: newborn endowment")
            child_ids.add(identifier)
        previous_energy = number(event["energy_after"], label)
        near(previous_energy, before + FOOD_ENERGY * hit - sum(costs) - PARENT_COST * len(children),
             f"{label}: founder energy ledger")
        require(previous_energy > 0, f"{label}: founder died")
        if policy in ("recurrent", "visible_reset"):
            require(choice == cue, f"{label}: reference did not follow the available cue")
    if policy in ("feed_forward", "boundary_reset"):
        require(len(set(choices)) == 1, f"{arm}/{policy}: absent cue unexpectedly changed choice")
    if policy in POLICIES:
        expected_hits = 16 if arm == "predictive" and policy in ("recurrent", "visible_reset") else 8
        require(hits == expected_hits, f"{arm}/{policy}: hits {hits} != {expected_hits}")
    for key, expected in (("hits", hits), ("offspring", len(child_ids)), ("final_tick", tick)):
        require(integer(run[key], key) == expected, f"{arm}/{policy}: incorrect run total {key}")
    return {"choices_matching_food": hits, "direct_births": len(child_ids), "last_pulse_tick": tick}


def audit(session):
    def read(name):
        return json.loads((session / name).read_text())
    manifest = read("manifest.json")
    require(isinstance(manifest, dict) and manifest, "missing run manifest")
    require(manifest["task"] == "food_cue_payoff" and manifest["schema"] == 1,
            "unsupported task or artifact version")
    schedule = read("schedule.json")
    validate_schedule(schedule)
    runs = read("result.json")["runs"]
    policies = {r["policy"] for r in runs}
    require(policies == set(POLICIES), "expected the four reference policies")
    expected = {(a, p) for a in ARMS for p in policies}
    require(len(runs) == len(expected) and {(r["arm"], r["policy"]) for r in runs} == expected,
            "expected each policy and cue pairing exactly once")
    outcomes = {(r["arm"], r["policy"]): check_run(r, schedule, manifest) for r in runs}
    by_arm = {(r["arm"], r["policy"]): r["events"] for r in runs}
    require(len({r["events"][0]["founder_id"] for r in runs}) == 1, "founder ID changed across runs")
    for policy in POLICIES:
        contrast = outcomes["predictive", policy]["direct_births"] - outcomes["shuffled", policy]["direct_births"]
        if policy in ("recurrent", "visible_reset"):
            require(contrast > 0, f"{policy}: birth instrument did not resolve the known payoff")
        else:
            require(contrast == 0, f"{policy}: birth contrast despite cue-independent behavior")
            for one, other in zip(by_arm["predictive", policy], by_arm["shuffled", policy]):
                require(one["choice"] == other["choice"], f"{policy}: cue-independent choice changed arm")
                for axis in ("choice_x", "choice_y"):
                    near(one[axis], other[axis], f"{policy}: committed {axis} changed arm")
    require(outcomes["predictive", "recurrent"]["direct_births"] >
            outcomes["predictive", "boundary_reset"]["direct_births"],
            "retaining the cue did not increase direct births over clearing state")
    return [{"policy": p, "arm": a, **outcomes[a, p]} for p in sorted(policies) for a in ARMS]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("session", type=Path)
    args = parser.parse_args()
    try:
        print(json.dumps(audit(args.session), indent=2))
    except (ValueError, OSError, KeyError, TypeError) as exc:
        parser.exit(1, f"Audit failed: {exc}\n")
