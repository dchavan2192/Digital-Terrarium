"""Reconstruct the four frozen populations' readouts from the saved circuit weights."""
import json
import math
import sys
from pathlib import Path


def response(weights, cue, visible=False, reset="intact"):
    bias, direct, recurrent = weights
    previous = 0.0
    for tick in range(7):
        if reset == "every_tick" or (reset == "boundary" and tick == 6):
            previous = 0.0
        stimulus = cue if tick == 0 or (visible and tick == 6) else 0.0
        x = bias + direct * stimulus + recurrent * previous
        square = x * x
        previous = (math.copysign(1, x) if abs(x) > 4 else
                    x * (27 + square) / (27 + 9 * square))
    return previous


def rank(candidate):
    negative, positive = candidate["responses"]
    return (-int(negative < 0) - int(positive >= 0), -min(-negative, positive), candidate["id"])


def check(directory):
    def load(name):
        return json.loads((directory / name).read_text())
    locked = load("selection.json")
    assert [seed for seed, _ in locked] == list(range(6000, 6004))
    diagnostics = load("diagnostics.json")["rows"]
    assert len(diagnostics) == 24
    for seed, winner in locked:
        history = load(f"seed_{seed}.json")
        assert len(history) == 17
        for i, generation in enumerate(history):
            assert len(generation) == 16 and generation == sorted(generation, key=rank)
            if i:
                parents = history[i-1][:4]
                assert all(p in generation for p in parents)
                assert all(c["parent"] in {p["id"] for p in parents} for c in generation if c not in parents)
            for c in generation:
                assert c["responses"] == [response(c["weights"], cue) for cue in (-1, 1)]
        assert winner == history[-1][0]
        for visible in (False, True):
            for reset in ("intact", "boundary", "every_tick"):
                row, = [r for r in diagnostics if (r["seed"], r["visible"], r["reset"]) == (seed, visible, reset)]
                pair = [response(winner["weights"], cue, visible, reset) for cue in (-1, 1)]
                assert row["id"] == winner["id"] and row["responses"] == pair
                assert row["correct"] == int(pair[0] < 0) + int(pair[1] >= 0)


if __name__ == "__main__":
    check(Path(sys.argv[1]))
    print("All four histories, final selections and 24 diagnostic pairs verified.")
