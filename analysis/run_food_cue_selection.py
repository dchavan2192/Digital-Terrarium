#!/usr/bin/env python3
"""Run and audit the paired timing-by-objective training and frozen-champion evaluation."""
import argparse
import json
import math
import os
from pathlib import Path
import subprocess
import tarfile
from statistics import mean

from check_food_cue_payoff import audit, near, require
from check_wiring import probe
from run_food_cue_population import differences
from run_food_cue_transfer import ROOT, digest, save

ARMS = ("fixed_turn", "varied_turn", "fixed_births", "varied_births")
TIE_ARMS = ("fixed_births", "varied_births", "fixed_random", "varied_random")
ORDERS = (6801, 6802)
TIE_ORDERS = (6803, 6804)
PILOT = ROOT / "food_cue_selection_raw.tar.gz"
PILOT_HASH = "30d2d1d18aa72c87a218c9265d245c7e9a48aeeca729c295369759e91cd54843"
GAP = 0.0625


def load(folder, name):
    return json.loads((folder / name).read_bytes())


def native(binary, folder, command, *options):
    env = dict(os.environ, RAYON_NUM_THREADS="1")
    result = subprocess.run([str(binary), command, "--output", str(folder), *options], env=env)
    sessions = list(folder.glob("session_*"))
    require(len(sessions) == 1, f"expected one native session in {folder}")
    if result.returncode:
        gate = sessions[0] / "founder_gate.json"
        if command != "food-cue-selection" or not gate.exists() or load(sessions[0], gate.name)["passed"]:
            result.check_returncode()
    return sessions[0]


def verify_gate(gate):
    rows = gate["rows"]
    seeds = {r["seed"] for r in rows}
    require(seeds == set(range(6700, 6708)) and len(rows) == 8, "incomplete founder gate")
    qualification = {}
    for timing in ("fixed", "varied"):
        for r in rows:
            require(len(r[timing]) == 16 and all(type(v) is int and v >= 0 for v in r[timing]),
                    "invalid founder tallies")
        qualified = sum(len(set(r[timing])) >= 2 for r in rows)
        require(qualified == gate[f"{timing}_seeds_with_variation"], f"incorrect {timing} founder gate")
        qualification[timing] = qualified
    require(gate["required_seeds"] == 4 and
            gate["passed"] is all(n >= 4 for n in qualification.values()), "incorrect gate verdict")
    return {r["seed"]: r for r in rows}


def verify_training(folder, gate, *, tie_stats=None):
    tie_study = tie_stats is not None
    arms = TIE_ARMS if tie_study else ARMS
    rows = verify_gate(gate)
    locks = load(folder, "selection.json")
    require(len(locks) == 32 and {(s, a) for s, a, _, _ in locks} ==
            {(s, a) for s in rows for a in arms}, "incomplete champion lock")
    founders = {}
    for seed, arm, winner, genome in locks:
        record = load(folder, f"{seed}_{arm}.json")
        births, generations = record["births"], record["generations"]
        require(len(births) == 160 and len(generations) == 13, "incomplete training history")
        timing, objective = arm.split("_")
        delays = [5] * 4 if timing == "fixed" else [3, 4, 6, 7]
        origin = [b["genome"] for b in births[:16]]
        require(origin == founders.setdefault(seed, origin), "starting genomes changed across arms")
        require(all(b["parent"] is None for b in births[:16]), "founder has parent")
        require(rows[seed][timing] == [b["native_births"] for b in births[:16]], "founder gate scores changed")
        for b in births:
            outputs = {d: probe(b["genome"], d, False, False) for d in set(delays)}
            # Native scoring reads a normalized turn back from the changed heading.
            signed = [sign * ((math.pi / 2 + turn * 0.6 - math.pi / 2) / 0.6)
                      for d in delays for sign, turn in zip((-1, 1), outputs[d])]
            correct = 2 * sum(t > 0 for t in signed)
            margin = min(signed)
            require(b["turn_correct"] == correct, "wrong training cue score")
            near(b["min_signed_margin"], margin, "training turn margin")
            require(type(b["native_births"]) is int and b["native_births"] >= 0, "invalid birth score")

        def order(i):
            b = births[i]
            if objective == "turn":
                return -b["turn_correct"], -b["min_signed_margin"], i
            return -b["native_births"], i

        candidates = set(range(16))
        random_orders = record.get("tie_orders", [])
        require(len(random_orders) == (12 if objective == "random" else 0), "wrong number of tie draws")
        for generation, ranked in enumerate(generations):
            if objective == "random" and generation < 12:
                draw = random_orders[generation]
                require(len(draw) == 16 and set(draw) == candidates, "invalid tie permutation")
                expected = sorted(draw, key=lambda i: -births[i]["native_births"])
            else:
                expected = sorted(candidates, key=order)
            require(len(ranked) == 16 and set(ranked) == candidates and ranked == expected,
                    f"{seed}/{arm}: wrong candidates or order in generation {generation}")
            if generation < 12:
                elite = set(ranked[:4])
                offspring = range(16 + 12 * generation, 28 + 12 * generation)
                require(all(births[i]["parent"] in elite for i in offspring), "unselected parent")
                candidates = elite.union(offspring)
        require(winner == generations[-1][0] and genome == births[winner]["genome"], "champion lock mismatch")
        if tie_study:
            parental_elites = [ranked[:4] for ranked in generations[:-1]]
            recurrent = {i for i, b in enumerate(births) if any(
                e["recurrent"] and e["enabled"] for e in b["genome"]["connections"])}
            above_four = [i for i, b in enumerate(births) if b["native_births"] > 4]
            new_recurrent = [len((set(elite) - set(parental_elites[g - 1] if g else [])) & recurrent)
                             for g, elite in enumerate(parental_elites)]
            recurrent_slots = [len(set(elite) & recurrent) for elite in parental_elites]
            tie_stats[seed, arm] = dict(recurrent_elite_slots=sum(recurrent_slots),
                                        recurrent_elite_slots_by_generation=recurrent_slots,
                                        new_recurrent_elites=sum(new_recurrent),
                                        new_recurrent_elites_by_generation=new_recurrent,
                                        candidates_above_four=len(above_four),
                                        first_generation_above_four=next(
                                            (g for g, ranked in enumerate(generations)
                                             if births[ranked[0]]["native_births"] > 4), None))
    return locks


def verify_archived_controls(training):
    require(digest(PILOT) == PILOT_HASH, "original selection archive checksum changed")
    with tarfile.open(PILOT) as archive:
        original = json.load(archive.extractfile("summary.json"))["training_session"]
        for seed in range(6700, 6708):
            for arm in ("fixed_births", "varied_births"):
                filename = f"{seed}_{arm}.json"
                require((training / filename).read_bytes() == archive.extractfile(f"{original}/{filename}").read(),
                        f"oldest-first history differs from original pilot: {seed}/{arm}")


def summary(rows, tie_study=False):
    grouped = {arm: [r for r in rows if r["arm"] == arm] for arm in (TIE_ARMS if tie_study else ARMS)}
    metrics = tuple(rows[0]["mean_outcomes"])

    def aggregate(values):
        return dict(mean=mean(values), positive=sum(x > 0 for x in values),
                    zero=values.count(0), negative=sum(x < 0 for x in values))

    arms = {arm: {metric: aggregate([r["mean_outcomes"][metric] for r in group]) for metric in metrics}
            for arm, group in grouped.items()}
    by_seed = {(r["seed"], r["arm"]): r["mean_outcomes"] for r in rows}
    seeds = sorted({r["seed"] for r in rows})
    pairs = {}
    comparisons = ((("fixed_random", "fixed_births"), ("varied_random", "varied_births"),
                    ("varied_random", "fixed_random"), ("varied_births", "fixed_births")) if tie_study else
                   (("fixed_births", "fixed_turn"), ("varied_births", "varied_turn"),
                    ("varied_turn", "fixed_turn"), ("varied_births", "fixed_births")))
    for left, right in comparisons:
        effects = {s: {k: by_seed[s, left][k] - by_seed[s, right][k] for k in metrics} for s in seeds}
        pairs[f"{left}_minus_{right}"] = dict(by_seed=effects,
            summary={k: aggregate([v[k] for v in effects.values()]) for k in metrics})
    return dict(arms=arms, paired_effects=pairs)


def evaluate(binary, output, *, tie_study=False):
    binary = binary.resolve(strict=True)
    output.mkdir(parents=True, exist_ok=False)
    orders = TIE_ORDERS if tie_study else ORDERS
    training = native(binary, output / "training", "food-cue-selection", *(["--tie-study"] if tie_study else []))
    manifest = load(training, "manifest.json")
    expected_task = "food_cue_tie_selection" if tie_study else "food_cue_selection"
    require(manifest["task"] == expected_task and manifest["schema"] == 1
            and manifest["training_schedule_seed"] == 6800 and manifest["generations"] == 12
            and manifest["population"] == 16 and manifest["elites"] == 4 and manifest["children"] == 12
            and set(manifest["arms"]) == set(TIE_ARMS if tie_study else ARMS),
            "unexpected training protocol")
    gate = load(training, "founder_gate.json")
    verify_gate(gate)
    common = dict(binary_sha256=digest(binary), training_session=str(training.relative_to(output)),
                  gate=gate, held_out_order_seeds=orders, distance=GAP)
    if not gate["passed"]:
        save(output / "summary.json", dict(**common, status="gate_failed"))
        return
    tie_stats = {} if tie_study else None
    locks = verify_training(training, gate, tie_stats=tie_stats)
    if tie_study:
        verify_archived_controls(training)
        common["oldest_first_controls_archive_sha256"] = PILOT_HASH
    options = ["--distance", str(GAP)]
    controls, schedules = {}, {}
    for seed in orders:
        folder = output / f"order_{seed}" / "references"
        session = native(binary, folder, "food-cue-payoff", *options, "--seed", str(seed))
        manifest = load(session, "manifest.json")
        require(manifest["task"] == "food_cue_payoff" and manifest["schedule_seed"] == seed
                and manifest["start"] == [40., 40. - GAP], "unexpected reference configuration")
        controls[seed] = dict(session=str(session.relative_to(output)), outcomes=audit(session))
        schedules[seed] = load(session, "schedule.json")
    require(schedules[orders[0]] != schedules[orders[1]], "held-out orders identical")
    records = []
    for seed, arm, winner, genome in locks:
        folder = output / "champions"
        folder.mkdir(exist_ok=True)
        donor = folder / f"{seed}_{arm}.json"
        save(donor, genome)
        runs = []
        for order in orders:
            session = native(binary, output / f"order_{order}" / f"{seed}_{arm}", "food-cue-transfer",
                             *options, "--seed", str(order), "--donor", str(donor))
            manifest = load(session, "manifest.json")
            require(manifest["donor"] == genome and manifest["task"] == "food_cue_transfer"
                    and manifest["schedule_seed"] == order and manifest["start"] == [40., 40. - GAP],
                    "unexpected transfer configuration")
            require(load(session, "schedule.json") == schedules[order], "unmatched held-out schedule")
            outcomes = audit(session)
            metrics = differences(outcomes)
            baseline = next(r["direct_births"] for r in controls[order]["outcomes"]
                            if r["policy"] == "feed_forward" and r["arm"] == "predictive")
            for r in outcomes:
                metrics[f"{r['policy']}_{r['arm']}_births"] = r["direct_births"]
            metrics["predictive_intact_minus_one_side"] = metrics["recurrent_predictive_births"] - baseline
            runs.append(dict(order_seed=order, session=str(session.relative_to(output)),
                             outcomes=outcomes, metrics=metrics))
        records.append(dict(seed=seed, arm=arm, champion_id=winner, donor_sha256=digest(donor), orders=runs,
                            mean_outcomes={k: mean(r["metrics"][k] for r in runs) for k in runs[0]["metrics"]}))
        if tie_study:
            records[-1]["training"] = tie_stats[seed, arm]
    if tie_study:
        common["training_by_arm"] = {
            arm: dict(populations_above_four=sum(tie_stats[s, arm]["candidates_above_four"] > 0 for s in range(6700, 6708)),
                      recurrent_elite_slots=sum(tie_stats[s, arm]["recurrent_elite_slots"] for s in range(6700, 6708)),
                      new_recurrent_elites=sum(tie_stats[s, arm]["new_recurrent_elites"] for s in range(6700, 6708)),
                      candidates_above_four=sum(tie_stats[s, arm]["candidates_above_four"] for s in range(6700, 6708)))
            for arm in TIE_ARMS}
    save(output / "summary.json", dict(**common, status="complete", references=controls, records=records,
                                      **summary(records, tie_study)))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT.parents[1] / "target/release/sim-runner")
    parser.add_argument("--output", type=Path, required=True, help="new local output directory")
    parser.add_argument("--tie-study", action="store_true", help="compare randomized and oldest-first birth-score ties")
    args = parser.parse_args()
    try:
        evaluate(args.binary, args.output, tie_study=args.tie_study)
        print(args.output / "summary.json")
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as exc:
        parser.exit(1, f"Selection study failed: {exc}\n")
