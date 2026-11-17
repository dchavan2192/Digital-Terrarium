"""Strict audit of the frozen eight-run demo recurrence pilot (standard library).

python3 analysis/summarize_memory_growth.py BATCH OUTPUT_JSON
Rejects incomplete, unexpected or mismatched data instead of reporting partial success.
Checkpoints remain on Pi; mirrored CSV/config/manifest/final export and launch files
are hashed. Late window is inclusive 150000..200000 as fixed before launch.
"""
import copy
import csv
import hashlib
import json
import math
from pathlib import Path
import re
import statistics
import sys
import tomllib

JOBS = [(3000, '0'), (3000, '0.3'), (3001, '0.3'), (3001, '0'),
        (3002, '0'), (3002, '0.3'), (3003, '0.3'), (3003, '0')]
METRICS = ['population', 'mean_connections', 'mean_recurrent_connections',
           'recurrent_brain_fraction', 'mean_brain_activity', 'births', 'deaths',
           'cap_blocked_births', 'energy_brain_structure_cost', 'energy_brain_activity_cost']


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def summarize(batch):
    batch = Path(batch)
    require((batch / 'status').read_text().strip() == 'completed', 'batch not completed')
    require((batch / 'exit_code').read_text().strip() == '0', 'batch exit code not zero')
    expected_jobs = [f'{seed} {rate}' for seed, rate in JOBS]
    for name in ['jobs.txt', 'completed_jobs.txt']:
        require((batch / name).read_text().splitlines() == expected_jobs, f'{name}: job order differs')
    hashes = {}
    for line in (batch / 'pinned.sha256').read_text().splitlines():
        digest, name = line.split(maxsplit=1)
        path = batch / Path(name).name
        require(sha(path) == digest, f'pinned artifact changed: {path}')
    base = tomllib.loads((batch / 'base.toml').read_text())
    original = tomllib.loads((batch / 'original_base.toml').read_text())
    expected = copy.deepcopy(original)
    for keys, value in [(('food', 'regrowth_rate'), .002), (('creature', 'hard_cap'), 500),
                        (('food', 'seasons', 'period'), 3600), (('food', 'droughts', 'spawn_rate'), .0006),
                        (('food', 'droughts', 'duration'), 1500), (('daynight', 'period'), 600)]:
        node = expected
        for key in keys[:-1]:
            node = node[key]
        node[keys[-1]] = value
    require(base == expected, 'base differs beyond fixed demo overrides')
    runs = []
    trajectory = []
    for seed, rate in JOBS:
        root = batch / f'rate_{rate}' / f'seed_{seed}'
        paths = list(root.glob(f'session_*/manifest_seed{seed}.toml'))
        require(len(paths) == 1, f'{root}: expected exactly one session; recovery needs explicit audit')
        manifest = tomllib.loads(paths[0].read_text())
        session = paths[0].parent
        config = copy.deepcopy(base)
        config['world']['seed'] = seed
        config['mutation']['add_recurrent_probability'] = float(rate)
        require(manifest['config'] == config, f'{session}: effective config differs')
        require(tomllib.loads((session / f'config_seed{seed}.toml').read_text()) == config, 'config artifact differs')
        require(manifest['source_revision'] == '0da954fa2911e5a0f075375b41196f8dafa6e4c8'
                and manifest['core_build_id'] == 'abad322241615ca2', 'unexpected pinned engine')
        require(manifest['requested_ticks'] == 200000 and manifest['log_interval'] == 500
                and manifest['starting_tick'] == 0, 'unexpected time protocol')
        status = manifest['status']
        require(status in ('completed', 'extinct'), f'{session}: technical failure/incomplete')
        with (session / f'stats_seed{seed}.csv').open() as stream:
            raw = list(csv.DictReader(stream))
        require(bool(raw), 'empty CSV')
        rows = [{key: float(value) for key, value in row.items()} for row in raw]
        require(all(math.isfinite(x) for row in rows for x in row.values()), 'non-finite CSV')
        ticks = [r['tick'] for r in rows]
        require(ticks == sorted(set(ticks)), 'duplicate or unsorted ticks')
        if status == 'completed':
            require(manifest['completed_ticks'] == 200000 and manifest['survived']
                    and ticks == list(range(500, 200001, 500)), 'incomplete completed run')
        else:
            require(not manifest['survived'] and manifest['completed_ticks'] <= 200000, 'bad extinct status')
        late = [r for r in rows if 150000 <= r['tick'] <= 200000] if status == 'completed' else []
        final = rows[-1]
        require(final['tick'] == manifest['completed_ticks'], 'CSV does not reach manifest endpoint')
        require(all(0 <= r['mean_recurrent_connections'] <= r['mean_connections']
                    and 0 <= r['recurrent_brain_fraction'] <= 1 for r in rows), 'invalid recurrence metrics')
        require(rate != '0' or all(r['mean_recurrent_connections'] == 0
                                  and r['recurrent_brain_fraction'] == 0 for r in rows), 'recurrence in off control')
        export_path = session / f'genomes_seed{seed}_tick{manifest["completed_ticks"]}.json'
        export = json.loads(export_path.read_text())
        # TOML omits None; checkpoint/export deliberately restores deprecated
        # stress_threshold for exact state roundtrips (checkpoint.rs). It is
        # ignored by the simulation and defaults to 0.3 in this pinned engine.
        export_config = copy.deepcopy(config)
        export_config['food']['droughts'].setdefault('schedule_seed', None)
        export_config['epigenetics'].setdefault('stress_threshold', 0.3)
        require(export['format'] == 'terrarium-genomes' and export['version'] == 1
                and export['config'] == export_config and export['tick'] == final['tick']
                and export['core_build'] == manifest['core_build_id'], 'export provenance differs')
        records = export['genomes']
        require(len(records) == final['population'], 'export population differs')
        require(len({g['id'] for g in records}) == len(records), 'duplicate exported ID')
        totals = [len(g['expressed_brain']['connections']) + len(g['expressed_brain']['recurrent']) for g in records]
        rec = [len(g['expressed_brain']['recurrent']) for g in records]
        for key, value in [('mean_connections', statistics.mean(totals) if totals else 0),
                           ('mean_recurrent_connections', statistics.mean(rec) if rec else 0),
                           ('recurrent_brain_fraction', sum(x > 0 for x in rec) / len(rec) if rec else 0)]:
            require(math.isclose(final[key], value, rel_tol=1e-12, abs_tol=1e-12), f'export {key} differs')
        metrics = METRICS + ['total_births', 'total_deaths', 'total_cap_blocked_births']
        run = dict(seed=seed, recurrence_probability=float(rate), status=status,
                   completed_ticks=manifest['completed_ticks'], csv_rows=len(rows), late_rows=len(late),
                   final={key: final[key] for key in metrics},
                   late_mean={key: statistics.mean(r[key] for r in late) for key in METRICS} if late else None,
                   elapsed_seconds=manifest['elapsed_seconds'], source_dirty=manifest['source_dirty'],
                   export_crosscheck='population and expressed enabled-edge averages match endpoint CSV')
        if late:
            run['late_mean']['mean_feed_forward_connections'] = run['late_mean']['mean_connections'] - run['late_mean']['mean_recurrent_connections']
        runs.append(run)
        trajectory.append(dict(seed=seed, recurrence_probability=float(rate), rows=[{key: r[key] for key in ['tick', 'total_births', 'total_deaths', 'total_cap_blocked_births'] + METRICS} for r in rows]))
    for path in sorted(batch.rglob('*')):
        if path.is_file() and not path.name.startswith('checkpoint'):
            hashes[str(path.relative_to(batch))] = sha(path)
    paired = []
    for seed in range(3000, 3004):
        off, on = [next(r for r in runs if r['seed'] == seed and r['recurrence_probability'] == p) for p in (0, .3)]
        paired.append(dict(seed=seed, both_completed=all(r['status'] == 'completed' for r in (off, on)),
                           late_difference_on_minus_off={key: on['late_mean'][key] - off['late_mean'][key] for key in METRICS}
                           if all(r['late_mean'] is not None for r in (off, on)) else None))
    groups = []
    for p in (0, .3):
        group = [r for r in runs if r['recurrence_probability'] == p]
        complete = [r for r in group if r['late_mean'] is not None]
        groups.append(dict(recurrence_probability=p, runs=len(group), completed=len(complete),
                           late_across_run_mean={key: statistics.mean(r['late_mean'][key] for r in complete) for key in METRICS} if complete else None,
                           late_across_run_sd={key: statistics.stdev(r['late_mean'][key] for r in complete) for key in METRICS} if len(complete) > 1 else None))
    guard = json.loads((batch / 'thermal_guard/status.json').read_text())
    require(guard['state'] == 'finished' and guard['running_matches'] == 0 and not guard['paused'], 'guard not finished')
    log = (batch / 'guard.log').read_text()
    temperatures = [float(t) for t in re.findall(r'temp=([0-9.]+)', log)]
    return dict(schema_version=1, complete=True, engine_source=runs and '0da954fa2911e5a0f075375b41196f8dafa6e4c8',
                protocol_source=(batch / 'protocol_source_revision').read_text().strip(),
                late_window='150000 <= tick <= 200000, inclusive saved ticks only',
                interpretation='exploratory structural diagnostic; not memory benefit, selection-versus-drift, equilibrium or generalization',
                activity_units='raw weighted-signal activity from pre-action think phase; distinct from realized energy ledger costs',
                independent_unit='one evolutionary world; four paired seed labels, diverging random histories',
                runs=runs, by_probability=groups, paired=paired,
                guard=dict(state='finished', pauses=len(re.findall(r' paused pid=', log)),
                           resumes=len(re.findall(r' resumed pid=', log)),
                           max_logged_transition_temperature_c=max(temperatures),
                           temperature_limit_note='transition samples only, not a full thermal time series; no continuous throttling log'),
                artifact_sha256=hashes, trajectories=trajectory)


if __name__ == '__main__':
    result = summarize(sys.argv[1])
    destination = Path(sys.argv[2])
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')
    print(json.dumps({key: result[key] for key in ['complete', 'by_probability', 'guard']}, indent=2))
