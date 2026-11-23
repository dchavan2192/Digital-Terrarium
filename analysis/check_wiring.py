"""Verify the native wiring study and emit per-seed diagnostic outcomes."""
import json
import math
import sys
from pathlib import Path


def probe(g, delay, visible, reset):
    nodes = [9, 10, *range(2, 9)]
    incoming = {n: [] for n in nodes}
    for edge in sorted(g['connections'], key=lambda e: (e['recurrent'], e['from'])):
        if edge['enabled']:
            incoming[edge['to']].append((edge['from'], edge['weight']*(1-edge['methylation']), edge['recurrent']))
    responses = []
    for cue in [-1, 1]:
        prev = [0.] * 11
        for tick in range(delay+2):
            if reset and tick == delay+1:
                prev = [0.] * 11
            act = [1., cue if tick == 0 or (visible and tick == delay+1) else 0., *([0.]*9)]
            for n in nodes:
                x = 0.
                for source, weight, recur in incoming[n]:
                    x += (prev if recur else act)[source]*weight
                square = x*x
                act[n] = math.copysign(1., x) if abs(x)>4 else x*(27.+square)/(27.+9.*square)
            prev = act
        responses.append(prev[2])
    return responses


def check(path):
    load = lambda name: json.loads((path/name).read_bytes())
    locks = load('selection.json')
    arms = ['selected','neutral','feed_forward']
    assert [(s,a) for s,a,_,_ in locks] == [(s,a) for s in range(6200,6232) for a in arms]
    rows = load('diagnostics.json')['rows']
    lookup = {(r['seed'],r['arm'],r['delay']):r for r in rows}
    assert len(lookup)==len(rows)==576
    results = []
    for seed,arm,winner,g in locks:
        data = load(f'{seed}_{arm}.json')
        births, gens = data['births'], data['generations']
        assert len(births)==304 and len(gens)==25
        if arm=='selected':
            founders=births[:16]
        assert founders==births[:16]
        for i,b in enumerate(births):
            edges=b['genome']['connections']
            if i<16 or arm=='feed_forward':
                assert all(not e['recurrent'] for e in edges)
            assert b['responses']==probe(b['genome'],5,False,False)
        for t,pop in enumerate(gens):
            def key(i):
                a,b=births[i]['responses']
                return (-int(a<0)-int(b>=0),-min(-a,b),i)
            assert len(set(pop))==len(pop)==16 and pop==sorted(pop,key=key)
            if t==0:
                assert set(pop)==set(range(16))
            else:
                kept=set(pop)&set(gens[t-1])
                kids=set(range(16+12*(t-1),16+12*t))
                assert len(kept)==4 and set(pop)==kept|kids
                assert arm=='neutral' or kept==set(gens[t-1][:4])
                assert all(births[i]['parent'] in kept for i in kids)
        assert winner==gens[-1][0] and births[winner]['genome']==g
        for delay in [4,5,6,19,20,21]:
            row=lookup[seed,arm,delay]
            assert row['id']==winner and row['edges']==sum(e['enabled'] and e['recurrent'] for e in g['connections'])
            for field,v,r in [('visible',True,False),('hidden',False,False),('visible_reset',True,True),('hidden_reset',False,True)]:
                assert row[field]==probe(g,delay,v,r)
            x,y=row['hidden']; a,b=row['visible_reset']; z,w=row['hidden_reset']
            sign=bool(row['edges'] and x<0<=y and z==w and a<0<=b)
            results.append(dict(seed=seed,arm=arm,delay=delay,sign=sign,passed=sign and min(-x,y)>=.1))
    return results


if __name__=='__main__':
    print(json.dumps(check(Path(sys.argv[1])),indent=1))
