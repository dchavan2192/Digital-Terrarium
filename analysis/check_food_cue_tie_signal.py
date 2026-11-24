"""Describe cue sensitivity among recurrent descendants in the random-tie histories."""
import argparse
import json
from pathlib import Path

from check_wiring import probe
from run_food_cue_selection import verify_gate, verify_training


def signal_counts(training):
    gate = json.loads((training / "founder_gate.json").read_text())
    verify_gate(gate)
    locks = verify_training(training, gate, tie_stats={})
    rows = []
    for seed, arm, _, _ in locks:
        if not arm.endswith("_random"):
            continue
        delays = (5,) if arm.startswith("fixed") else (3, 4, 6, 7)
        record = json.loads((training / f"{seed}_{arm}.json").read_text())
        count = sensitive = opposite = 0
        for child in record["births"][16:]:
            genome = child["genome"]
            if not any(e["enabled"] and e["recurrent"] for e in genome["connections"]):
                continue
            count += 1
            pairs = [probe(genome, delay, False, False) for delay in delays]
            sensitive += any(abs(a - b) > 1e-8 for a, b in pairs)
            opposite += any(a * b < 0 for a, b in pairs)
        rows.append(dict(seed=seed, arm=arm, recurrent_descendants=count,
                         different_turn_magnitude_or_sign=sensitive, opposite_turns=opposite))
    return rows


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("training_session", type=Path)
    args = parser.parse_args()
    print(json.dumps(signal_counts(args.training_session), indent=2))
