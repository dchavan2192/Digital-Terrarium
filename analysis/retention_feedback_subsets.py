"""Exhaustively replay recurrent-edge subsets of the three sustained-recall champions."""
import csv
import json
import sys
import tarfile
from pathlib import Path
from lesion_retention_edges import evaluate

ROOT = Path(__file__).resolve().parents[1] / "docs/results"
COHORTS = {
    "retention_wiring_raw.tar.gz": {(6226, "selected")},
    "delay_training_raw.tar.gz": {(6320, "fixed"), (6320, "variable")},
}


def replay():
    for archive, cohort in COHORTS.items():
        with tarfile.open(ROOT / archive) as f:
            winners = json.load(f.extractfile("selection.json"))
        targets = [(s,a,i,g) for s,a,i,g in winners if (s,a) in cohort]
        assert len(targets) == len(cohort)
        for seed, arm, champion, genome in targets:
            edges = genome["connections"]
            ids = [i for i,e in enumerate(edges) if e["enabled"] and e["recurrent"]]
            for mask in range(1 << len(ids)):
                keep = [i for j,i in enumerate(ids) if mask & (1 << j)]
                network = dict(genome, connections=[
                    dict(e, enabled=False) if i in ids and i not in keep else e
                    for i,e in enumerate(edges)])
                trials = list(evaluate(network))
                passed = [d for d,*_,ok in trials if ok]
                if not keep:
                    assert all(x == y for _,x,y,*_ in trials) and not passed
                if keep == ids:
                    assert len(passed) == 33
                labels = [f"{edges[i]['from']}>{edges[i]['to']}" for i in keep]
                yield seed,arm,champion,mask," ".join(labels),len(passed)," ".join(map(str,passed))


if __name__ == "__main__":
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow("seed arm champion mask retained_feedback pass_count passing_delays".split())
    writer.writerows(replay())
