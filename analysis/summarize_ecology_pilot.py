"""Summarize one completed ecology pilot using standard-library CSV/TOML readers.

Usage: python3 analysis/summarize_ecology_pilot.py SESSION_DIR SUMMARY_JSON
Late window is 10000 < tick <= 20000, sampled every 500 ticks; minima and
ledger residual maxima are sampled observations, not whole-run extrema.
"""
import csv
import hashlib
import json
from pathlib import Path
import statistics
import sys
import tomllib


def summarize(session):
    session = Path(session)
    runs = []
    hashes = {}
    for manifest_path in sorted((session / 'sweep').glob('manifest_*.toml')):
        manifest = tomllib.loads(manifest_path.read_text())
        if manifest['status'] != 'completed' or manifest['completed_ticks'] != 20000:
            raise ValueError(f'Incomplete pilot run: {manifest_path}')
        suffix = manifest_path.name.removeprefix('manifest_').removesuffix('.toml')
        csv_path = manifest_path.with_name(f'stats_{suffix}.csv')
        config_path = manifest_path.with_name(f'config_{suffix}.toml')
        with csv_path.open() as stream:
            rows = list(csv.DictReader(stream))
        late = [r for r in rows if 10000 < int(r['tick']) <= 20000]
        if len(late) != 20 or int(rows[-1]['tick']) != 20000:
            raise ValueError(f'Unexpected sampling horizon: {csv_path}')
        if tomllib.loads(config_path.read_text()) != manifest['config']:
            raise ValueError(f'Manifest/config mismatch: {config_path}')
        run = {key: manifest[key] for key in (
            'source_revision', 'source_dirty', 'compiler', 'target', 'build_profile',
            'stats_schema_version', 'topology_hash_version', 'status',
            'requested_ticks', 'completed_ticks', 'elapsed_seconds', 'log_interval')}
        run.update(
            regrowth_rate=manifest['config']['food']['regrowth_rate'],
            seed=manifest['seed'], hard_cap=manifest['config']['creature']['hard_cap'],
            final_population=int(rows[-1]['population']),
            minimum_sampled_population=min(int(r['population']) for r in rows),
            late_mean_population=statistics.mean(float(r['population']) for r in late),
            late_mean_energy=statistics.mean(float(r['mean_energy']) for r in late),
            late_mean_attacks=statistics.mean(float(r['attacks']) for r in late),
            late_mean_shares=statistics.mean(float(r['shares']) for r in late),
            total_cap_blocked_births=int(rows[-1]['total_cap_blocked_births']),
            max_sampled_abs_energy_residual=max(abs(float(r['energy_balance_residual'])) for r in rows),
        )
        runs.append(run)
        for artifact in (manifest_path, csv_path, config_path):
            hashes[str(artifact.relative_to(session))] = hashlib.sha256(artifact.read_bytes()).hexdigest()
    if {(r['regrowth_rate'], r['seed']) for r in runs} != {(v, s) for v in (0.004, 0.006, 0.008) for s in (0, 1)} or len(runs) != 6:
        raise ValueError('Expected exactly three rates by two seeds')
    return dict(schema_version=1, session=session.name, late_window='10000 < tick <= 20000',
                measurement='20 equally spaced late-window snapshots per run; run is experimental unit',
                runs=runs, sha256=hashes)


if __name__ == '__main__':
    result = summarize(sys.argv[1])
    Path(sys.argv[2]).write_text(json.dumps(result, indent=2) + '\n')
    for rate in (0.004, 0.006, 0.008):
        group = [r for r in result['runs'] if r['regrowth_rate'] == rate]
        print(rate, 'late population', statistics.mean(r['late_mean_population'] for r in group))
