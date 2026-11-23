"""Disable individual recurrent genes in saved champions and replay the signed cue."""
import argparse
import csv
import gzip
import json
import tarfile
from pathlib import Path
from check_wiring import probe


def evaluate(genome):
    visible = probe(genome, 0, True, True)
    erased = probe(genome, 0, False, True)
    assert erased[0] == erased[1]
    # A boundary reset makes the probe independent of all earlier ticks.
    for delay in range(33):
        x, y = probe(genome, delay, False, False)
        yield (delay, x, y, *visible, int(min(-x, y) >= .1 and visible[0] < 0 <= visible[1]))


def analyze(path, writer):
    with tarfile.open(path) as source:
        winners = json.load(source.extractfile('selection.json'))
        recorded = json.load(source.extractfile('diagnostics.json'))['rows']
    native = {(r['seed'], r['arm'], r['delay']): r for r in recorded}
    checked = 0
    for seed, arm, champion, genome in winners:
        edges = genome['connections']
        ids = [i for i, e in enumerate(edges) if e['enabled'] and e['recurrent']]
        for label, disabled in [('intact', []), ('all', ids), *(('single', [i]) for i in ids)]:
            # Make a new connection list so no intervention can affect the next one.
            altered = dict(genome, connections=[dict(e, enabled=False) if i in disabled else e
                                                for i, e in enumerate(edges)])
            edge = edges[disabled[0]] if label == 'single' else {}
            for delay, x, y, vx, vy, passed in evaluate(altered):
                if label == 'intact' and (seed, arm, delay) in native:
                    row = native[seed, arm, delay]
                    assert row['id'] == champion and row['hidden'] == [x, y]
                    assert row['visible_reset'] == [vx, vy] and row['edges'] == len(ids)
                    checked += 1
                if label == 'all':
                    assert x == y and not passed
                writer.writerow([seed, arm, champion, len(ids), label, *[edge.get(k,'') for k in ('from','to','weight')],
                                 delay, x, y, vx, vy, passed])
    assert checked == len(recorded)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('archives', type=Path, nargs='+')
    args = parser.parse_args()
    with gzip.open(args.output, 'wt', newline='') as file:
        writer = csv.writer(file, lineterminator='\n')
        writer.writerow('seed arm champion recurrent_edges condition source target weight delay negative positive visible_negative visible_positive passed'.split())
        for archive in args.archives:
            analyze(archive, writer)
