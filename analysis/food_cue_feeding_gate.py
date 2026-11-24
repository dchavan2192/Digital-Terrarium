#!/usr/bin/env python3
"""Replay the feeding tick of the eight prior recall passers in the archived close-gap assay."""
import argparse
from collections import defaultdict
import json
import math
from pathlib import Path
import tarfile

from check_food_cue_payoff import near, require
from run_food_cue_population import five_blank_status
from run_food_cue_transfer import ROOT, digest

ARCHIVE = ROOT / "food_cue_population_raw.tar.gz"
ARCHIVE_HASH = "9240a2a0d2fcbc8b27a7ff8ef7fceb3ac03c46e7efd0b13b7f03593fed326b05"


def replay(donor, delay, cue):
    # The one-input wiring-study genome uses turn node 2 and eating node 4.
    require(donor["num_inputs"] == 1 and donor["num_outputs"] == 7
            and donor["hidden_nodes"] == [{"id": 9, "depth": 0.5}, {"id": 10, "depth": 0.5}],
            "unsupported wiring-study controller")
    incoming = defaultdict(list)
    for e in sorted(donor["connections"], key=lambda e: (e["recurrent"], e["from"])):
        if e["enabled"]:
            incoming[e["to"]].append((e["from"], e["weight"] * (1 - e["methylation"]), e["recurrent"]))
    previous = [0.] * 11
    for tick in range(delay + 3):
        state = [1., cue if tick == 0 else 0.] + [0.] * 9
        for node in (9, 10, *range(2, 9)):
            x = 0.
            for src, weight, recurrent in incoming[node]:
                x += (previous if recurrent else state)[src] * weight
            squared = x * x
            state[node] = math.copysign(1., x) if abs(x) > 4 else x * (27 + squared) / (27 + 9 * squared)
        if tick == delay + 1:
            choice_turn = state[2]
        previous = state
    return choice_turn, previous[4]


def inspect(archive=ARCHIVE):
    require(digest(archive) == ARCHIVE_HASH, "population archive checksum mismatch")
    seeds = {s for s, passed in five_blank_status().items() if passed}
    events = []
    with tarfile.open(archive) as raw:
        def read(name):
            return json.load(raw.extractfile(name))
        close, = (d for d in read("summary.json")["distances"] if d["distance"] == 0.0625)
        cohort = [d for d in close["donors"] if d["founder_seed"] in seeds]
        require(len(cohort) == 8 and {d["founder_seed"] for d in cohort} == seeds, "incomplete cohort")
        for d in cohort:
            session = d["session"]
            donor = read(str(Path(session).parent / "donor.json"))
            manifest = read(f"{session}/manifest.json")
            config = manifest["config"]
            require(donor == manifest["donor"], "original donor mismatch")
            require(manifest["start"] == [40., 39.9375], "wrong close gap")
            require(config["food"]["fruit"]["enabled"] is False and config["food"]["regrowth_rate"] == 0,
                    "diet efficiency or regrowth could alter consumption")
            near(config["creature"]["max_speed"], 0., "fixed feeding pose")
            food_energy = config["food"]["energy_per_food"]
            near(food_energy, 15, "energy per algae unit")
            rows = read(f"{session}/result.json")["runs"]
            chosen, = (r for r in rows if r["policy"] == "recurrent" and r["arm"] == "predictive")
            require(len(chosen["events"]) == 16, "incomplete lifetime")
            for e in chosen["events"]:
                require(e["cue"] == e["food"], "unreliable cue")
                turn, eat = replay(donor, e["delay"], e["cue"])
                near(e["choice_turn"], turn, "replayed pre-feeding turn")
                food_cell = manifest["positive_patch"] if e["food"] == 1 else manifest["negative_patch"]
                require(e["pulse_cell"] == food_cell, "food cell mismatch")
                cell = [math.floor(e[axis] / config["food"]["cell_size"]) for axis in ("choice_x", "choice_y")]
                landed = cell == food_cell
                require(landed == (e["food"] == e["choice"]), "incorrect recorded landing")
                # In this fruit-disabled assay a positive eat action consumes up to one unit.
                near(e["food_units"], 1., "food delivered")
                meal = int(landed and eat > 0.)
                near(e["food_remaining"], 1. - meal, "remaining algae after pulse")
                near(e["algae_energy"], food_energy * meal, "native harvested energy")
                require(e["energy_before"] + food_energy < config["energy"]["maximum"],
                        "reserve could reach cap before costs")
                events.append(dict(seed=d["founder_seed"], index=e["index"], delay=e["delay"],
                                   cue=e["cue"], turn=turn, landed=landed, feeding_tick_eat=eat,
                                   consumed=e["algae_energy"] > 0))
    by_delay = {}
    for delay in (2, 5, 8, 11):
        rows = [r for r in events if r["delay"] == delay]
        require(len(rows) == 32, "incomplete delay stratum")
        by_delay[delay] = dict(opportunities=len(rows), correct_landings=sum(r["landed"] for r in rows),
                               meals=sum(r["consumed"] for r in rows),
                               landed_eat_nonpositive=sum(r["landed"] and r["feeding_tick_eat"] <= 0 for r in rows))
    return dict(archive_sha256=ARCHIVE_HASH, by_delay=by_delay, events=events)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, default=ARCHIVE)
    args = parser.parse_args()
    try:
        print(json.dumps(inspect(args.archive), indent=2, allow_nan=False))
    except (ValueError, OSError, KeyError, TypeError, tarfile.TarError) as exc:
        parser.exit(1, f"Feeding audit failed: {exc}\n")
