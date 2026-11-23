"""Audit the frozen paired retention histories and summarize every seed."""
import json
import sys
from pathlib import Path
from check_stationary_retention import rank, response


def check(path):
    load = lambda name: json.loads((path / name).read_bytes())
    winners = load("selection.json")
    diagnostics = load("diagnostics.json")
    arms = ("selected", "neutral")
    assert [(s, a) for s, a, _ in winners] == [(s, a) for s in range(6100, 6132) for a in arms]
    lookup = {(r["seed"], r["arm"], r["delay"], r["visible"], r["reset"]): r for r in diagnostics["rows"]}
    assert len(lookup) == len(diagnostics["rows"]) == 1536
    records = []
    for seed, arm, winner in winners:
        history = load(f"{seed}_{arm}.json")
        assert len(history) == 17
        if arm == "selected":
            founders = history[0]
        else:
            assert history[0] == founders
        counts = []
        inherited = offspring = 0
        for i, pop in enumerate(history):
            assert len(pop) == len({c["id"] for c in pop}) == 16 and pop == sorted(pop, key=rank)
            counts.append(sum(rank(c)[0] == -2 for c in pop))
            if i:
                previous = {c["id"]: c for c in history[i - 1]}
                kept = {c["id"] for c in pop if c["id"] in previous}
                assert len(kept) == 4
                if arm == "selected":
                    assert kept == {c["id"] for c in history[i - 1][:4]}
                assert {c["id"] for c in pop} - kept == set(range(16 + (i-1)*12, 16+i*12))
            for c in pop:
                assert c["responses"] == [response(c["weights"], cue) for cue in (-1, 1)]
                if i and c["id"] in kept:
                    assert c == previous[c["id"]]
                elif i:
                    assert c["parent"] in kept
                    parent = previous[c["parent"]]
                    assert all(abs(w-p) <= .300000000000001 and abs(w)<=4 for w,p in zip(c["weights"],parent["weights"]))
                    if rank(parent)[0] == -2:
                        offspring += 1
                        inherited += rank(c)[0] == -2
        assert winner == history[-1][0]
        transfer = {}
        for delay in [1, 5, 10, 20]:
            scores = {}
            for visible in [False, True]:
                for reset in ["intact", "boundary", "every_tick"]:
                    row = lookup[seed, arm, delay, visible, reset]
                    r = [response(winner["weights"], cue, visible, reset, delay) for cue in (-1,1)]
                    assert row["id"] == winner["id"] and row["responses"] == r
                    assert row["correct"] == int(r[0]<0) + int(r[1]>=0)
                    scores[visible,reset] = row["correct"]
            transfer[delay] = (scores[False,"intact"],scores[False,"boundary"],scores[True,"boundary"]) == (2,1,2)
        records.append(dict(seed=seed,arm=arm,counts=counts,first=next((i for i,n in enumerate(counts) if n),None),
                            offspring_of_success=offspring,successful_offspring=inherited,transfer=transfer))
    return records


if __name__ == "__main__":
    print(json.dumps(check(Path(sys.argv[1])),indent=1))
