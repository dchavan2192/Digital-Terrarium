"""Counterfactual readout of locked controllers and descendants from the tie study."""
import json
import tarfile

from check_food_cue_payoff import near, require
from food_cue_feeding_gate import replay
from run_food_cue_transfer import ROOT, digest

ARCHIVE = ROOT / "food_cue_tie_raw.tar.gz"
HASH = "8142edbfa813fa55516e886ffc5b43efa34fea244a9c1d7e60ed668d2d5bbf5a"
DELAYS = (2, 5, 8, 11)
MARGIN = 0.1  # Same signed turn margin used in the stationary recall studies.


def turns(genome, delay):
    return {cue: replay(genome, delay, cue)[0] for cue in (-1, 0, 1)}


def adjusted(outputs, cue, fraction):
    return max(-1., min(1., outputs[cue] - fraction * outputs[0]))


def main():
    require(digest(ARCHIVE) == HASH, "unexpected source archive")
    champions, descendants = [], []
    with tarfile.open(ARCHIVE) as archive:
        def read(path):
            return json.load(archive.extractfile(path))
        summary = read("summary.json")
        training = summary["training_session"]
        for record in summary["records"]:
            arm, seed = record["arm"], record["seed"]
            if not arm.endswith("_random"):
                continue
            history = read(f"{training}/{seed}_{arm}.json")["births"]
            genome = history[record["champion_id"]]["genome"]
            traces = {d: turns(genome, d) for d in DELAYS}
            for order in record["orders"]:
                runs = read(order["session"] + "/result.json")["runs"]
                for run in runs:
                    if run["policy"] != "recurrent":
                        continue
                    for fraction in (0., .5, 1.):
                        clear = robust = 0
                        for e in run["events"]:
                            outputs = traces[e["delay"]]
                            near(e["choice_turn"], outputs[e["cue"]], "native turn replay")
                            signed = e["food"] * adjusted(outputs, e["cue"], fraction)
                            clear += signed > 1e-8
                            robust += signed >= MARGIN
                        champions.append(dict(seed=seed, arm=arm, order=order["order_seed"],
                                              cue_arm=run["arm"], correction=fraction,
                                              signed_correct=clear, margin_correct=robust))
            delays = [5] if arm == "fixed_random" else [3, 4, 6, 7]
            for i, candidate in enumerate(history[16:], 16):
                g = candidate["genome"]
                if not any(e["enabled"] and e["recurrent"] for e in g["connections"]):
                    continue
                passes = [0, 0, 0]
                best_centered_margin = -1.
                for delay in delays:
                    outputs = turns(g, delay)
                    best_centered_margin = max(best_centered_margin, min(
                        c * adjusted(outputs, c, 1.) for c in (-1, 1)))
                    for j, f in enumerate((0., .5, 1.)):
                        passes[j] += all(c * adjusted(outputs, c, f) >= MARGIN for c in (-1, 1))
                descendants.append(dict(seed=seed, arm=arm, id=i, delays_passing=passes,
                                        best_centered_margin=best_centered_margin))
    print(json.dumps(dict(archive_sha256=HASH, margin=MARGIN, champions=champions,
                          recurrent_descendants=descendants), indent=1))


if __name__ == "__main__":
    main()
