"""Exploratory run-level ecology; these endpoints do not change selection criteria.

Usage: python3 analysis/analyze_validation_ecology.py BATCH_DIR OUTPUT_JSON
"""
import csv
import json
from pathlib import Path
import statistics
import sys

from summarize_validation import summarize


def analyze(batch):
    batch = Path(batch)
    validated = summarize(batch)  # First enforce completed jobs and matching resumed rows.
    runs = []
    for run in validated['runs']:
        rate, seed = run['regrowth_rate'], run['seed']
        rows = {}
        for path in sorted((batch / f'rate_{rate:.3f}' / f'seed_{seed}').glob('session_*/stats_*.csv')):
            with path.open() as stream:
                for row in csv.DictReader(stream):
                    rows[int(row['tick'])] = row
        early = [r for tick, r in sorted(rows.items()) if 20000 <= tick <= 100000]
        late = [r for tick, r in sorted(rows.items()) if 100000 < tick <= 200000]
        if validated['target_ticks'] != 200000 or len(early) != 161 or len(late) != 200:
            raise ValueError('This exploratory analysis requires the original 200k/500-tick schedule')
        mean = lambda window, key: statistics.mean(float(r[key]) for r in window)
        runs.append(dict(
            rate=rate, seed=seed,
            early_population=mean(early, 'population'), late_population=mean(late, 'population'),
            early_connections=mean(early, 'mean_connections'), late_connections=mean(late, 'mean_connections'),
            late_energy=mean(late, 'mean_energy'), late_species=mean(late, 'species_count'),
            late_population_cv=statistics.stdev(float(r['population']) for r in late) / mean(late, 'population'),
            maximum_logged_population=max(int(r['population']) for r in rows.values()),
        ))
    groups = []
    for rate in sorted({r['rate'] for r in runs}):
        group = [r for r in runs if r['rate'] == rate]
        result = dict(rate=rate, seeds=len(group))
        for key in ('early_population', 'late_population', 'early_connections',
                    'late_connections', 'late_energy', 'late_species', 'late_population_cv'):
            result[key] = statistics.mean(r[key] for r in group)
        result['maximum_logged_population'] = max(r['maximum_logged_population'] for r in group)
        result['runs_with_connection_decline'] = sum(r['late_connections'] < r['early_connections'] for r in group)
        groups.append(result)
    return dict(batch=batch.name, analysis='exploratory; each run is the independent unit',
                early_window='20000 <= tick <= 100000', late_window='100000 < tick <= 200000',
                by_rate=groups, runs=runs)


if __name__ == '__main__':
    result = analyze(sys.argv[1])
    Path(sys.argv[2]).write_text(json.dumps(result, indent=2) + '\n')
    for group in result['by_rate']:
        print(json.dumps(group))
