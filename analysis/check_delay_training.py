"""Reconstruct training ranks, survival and held-out responses for the delay comparison."""
import json
import sys
from pathlib import Path
from check_wiring import probe


def audit(path):
    load = lambda name: json.loads((path/name).read_bytes())
    winners = load("selection.json")
    rows = load("diagnostics.json")["rows"]
    by_key = {(r["seed"], r["arm"], r["delay"]): r for r in rows}
    assert len(rows) == len(by_key) == 256
    assert [(s,a) for s,a,_,_ in winners] == [(s,a) for s in range(6300,6332) for a in ("fixed","variable")]
    summary = []
    for seed, arm, winner, genome in winners:
        train = [5]*4 if arm=="fixed" else [3,4,6,7]
        h = load(f"{seed}_{arm}.json")
        births, generations = h["births"], h["generations"]
        assert len(births)==304 and len(generations)==25
        start = [b["genome"] for b in births[:16]]
        if arm=="fixed":
            founders = start
        assert founders == start
        assert all(not e["recurrent"] for g in start for e in g["connections"])
        keys = []
        for i,b in enumerate(births):
            results = [b["responses"], *b["additional_responses"]]
            probes = {d:probe(b["genome"], d, False, False) for d in set(train)}
            assert results == [probes[d] for d in train]
            keys.append((-sum(int(x<0)+int(y>=0) for x,y in results), -min(min(-x,y) for x,y in results), i))
        for t,pop in enumerate(generations):
            members = set(range(16)) if t==0 else set(generations[t-1][:4]) | set(range(16+12*(t-1),16+12*t))
            assert pop == sorted(members,key=keys.__getitem__)
            if t:
                assert all(births[i]["parent"] in generations[t-1][:4] for i in range(16+12*(t-1),16+12*t))
        assert winner==generations[-1][0] and genome==births[winner]["genome"]
        passed = []
        for d in [8,9,18,19]:
            r = by_key[seed,arm,d]
            assert r["id"]==winner and r["edges"]==sum(e["enabled"] and e["recurrent"] for e in genome["connections"])
            for key,v,z in [("hidden",False,False),("hidden_reset",False,True),("visible",True,False),("visible_reset",True,True)]:
                assert r[key]==probe(genome,d,v,z)
            x,y = r["hidden"]
            a,b = r["visible_reset"]
            u,w = r["hidden_reset"]
            if r["edges"] and min(-x,y)>=0.1 and a<0<=b and u==w:
                passed.append(d)
        summary.append(dict(seed=seed,arm=arm,winner=winner,passed=passed))
    return summary


if __name__=="__main__":
    print(json.dumps(audit(Path(sys.argv[1])),indent=1))
