"""Replay each frozen champion at every integer blank delay from zero through 32."""
import csv
import json
import sys
import tarfile
from pathlib import Path
from check_wiring import probe


def replay(archive, output):
    with tarfile.open(archive) as source:
        winners = json.load(source.extractfile("selection.json"))
        native = json.load(source.extractfile("diagnostics.json"))["rows"]
    recorded = {(r["seed"], r["arm"], r["delay"]): r for r in native}
    assert len(winners) == len({(s, a) for s, a, _, _ in winners}) == 96
    matched = 0
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["seed", "arm", "champion", "blank_ticks", "negative_turn", "positive_turn",
                     "choice", "worst_signed_margin", "retention_pass"])
    for seed, arm, champion, genome in winners:
        for delay in range(33):
            hidden = probe(genome, delay, False, False)
            visible = probe(genome, delay, True, True)
            reset = probe(genome, delay, False, True)
            assert reset[0] == reset[1]
            if (seed, arm, delay) in recorded:
                r = recorded[seed, arm, delay]
                assert (champion, hidden, visible, reset) == (r["id"], r["hidden"], r["visible_reset"], r["hidden_reset"])
                matched += 1
            x, y = hidden
            choice = ("correct" if x < 0 <= y else "reversed" if y < 0 <= x else
                      "both_negative" if x < 0 else "both_positive")
            margin = min(-x, y)
            edges = any(e["enabled"] and e["recurrent"] for e in genome["connections"])
            passed = edges and margin >= .1 and visible[0] < 0 <= visible[1]
            writer.writerow([seed, arm, champion, delay, x, y, choice, margin, int(passed)])
    assert matched == len(native) == 576


if __name__ == "__main__":
    replay(Path(sys.argv[1]), sys.stdout)
