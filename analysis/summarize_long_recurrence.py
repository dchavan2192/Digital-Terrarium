#!/usr/bin/env python3
"""Audit the fixed eight-world study. Partial reports require explicit opt-in.

Read-only: never resume jobs, evaluate controllers or alter the frozen campaign.
Live CSVs are buffered; observed_tick is a saved prefix, not a current tick.
"""
import argparse
import copy
import csv
import hashlib
import io
import json
import math
from pathlib import Path
import statistics
import tomllib

ENGINE = 'b40ae9c15ef5f90f17e7e2bbbc286c1c19c0b88c'
CORE_BUILD = '297edc68fb18e05e'
BINARY_SHA = '05e6fe23121818fb897332a53f314da11e2ea98fed6f61cfe52d607009833e63'
BASE_SHA = 'bcfd543bc75ed4127389c17c24e6282a8e735c96a903d0a55abd6e0fd165079d'
WORKER_SHA = '0f0d0953b5feedb951b949729adc7d5504ce3506dca5445f9bc5eb99edc729ae'
PINNED_INPUTS = {
    'base.toml': BASE_SHA, 'sim-runner': BINARY_SHA,
    'pi_thermal_guard.py': 'e25611f799fac2537df86738e99cd29ad801a87bf4f4408df0c6d20e56a6176c',
    'run_guarded_acquisition.py': '79cc58bfdefb45c83c3005e4f4594787e830a3ea3eb550838095b3d2d7a5d425'}
JOBS = [(4000, 0), (4000, .3), (4001, .3), (4001, 0),
        (4002, 0), (4002, .3), (4003, .3), (4003, 0)]
METRICS = ['population', 'mean_connections', 'mean_recurrent_connections',
           'recurrent_brain_fraction', 'mean_brain_activity', 'births', 'deaths',
           'cap_blocked_births', 'energy_brain_structure_cost', 'energy_brain_activity_cost']
EVENTS = ['births', 'deaths', 'cap_blocked_births']
COUNTERS = ['total_' + key for key in EVENTS]
NEURAL = ['mean_connections', 'mean_recurrent_connections', 'recurrent_brain_fraction',
          'mean_feed_forward_connections']


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def finite_tree(value):
    if isinstance(value, float):
        require(math.isfinite(value), 'nonfinite JSON number')
    elif isinstance(value, dict):
        for item in value.values():
            finite_tree(item)
    elif isinstance(value, list):
        for item in value:
            finite_tree(item)


def read_json(path, reader=None):
    require(path.stat().st_size <= 256 * 1024**2, f'JSON audit input exceeds256MiB: {path}')
    raw = reader(path) if reader else path.read_bytes()
    require(len(raw) <= 256 * 1024**2, 'JSON audit input exceeds256MiB')
    result = json.loads(raw)
    finite_tree(result)
    return result


def read_rows(path, mutable, reader=None):
    text = (reader(path) if reader else path.read_bytes()).decode()
    truncated = bool(text and not text.endswith('\n'))
    if truncated:
        require(mutable, 'terminal CSV has a truncated final record')
        text = text[:text.rfind('\n') + 1]
    if not text:
        require(mutable, 'terminal CSV empty')
        return [], truncated
    reader = csv.DictReader(io.StringIO(text))
    fields = reader.fieldnames
    require(fields and len(fields) == len(set(fields)), 'missing/duplicate CSV columns')
    require(set(['tick'] + METRICS + COUNTERS) <= set(fields), 'CSV missing required metrics')
    rows = []
    for raw in reader:
        require(None not in raw and all(v not in (None, '') for v in raw.values()), 'malformed CSV row')
        row = {k: float(v) for k, v in raw.items()}
        require(all(math.isfinite(v) for v in row.values()), 'nonfinite CSV value')
        for key in ['tick', 'population'] + EVENTS + COUNTERS:
            require(row[key].is_integer() and row[key] >= 0, f'invalid count {key}')
        require(0 < row['tick'] <= 500000 and row['population'] <= 2000, 'tick/population out of range')
        require(0 <= row['mean_recurrent_connections'] <= row['mean_connections']
                and 0 <= row['recurrent_brain_fraction'] <= 1, 'invalid enabled-edge metrics')
        require(all(row[k] >= 0 for k in ('mean_brain_activity', 'energy_brain_structure_cost',
                                         'energy_brain_activity_cost')), 'negative activity/brain energy cost')
        if rows:
            require(row['tick'] > rows[-1]['tick'], 'duplicate/unsorted CSV ticks')
            require(all(row[k] >= rows[-1][k] for k in COUNTERS), 'cumulative counter decreased')
            require(all(row['total_' + k] - rows[-1]['total_' + k] >= row[k] for k in EVENTS),
                    'sampled event exceeds cumulative interval increment')
        require(all(row['total_' + key] >= row[key] for key in EVENTS), 'event exceeds cumulative total')
        rows.append(row)
    return rows, truncated


def window(rows, start, end):
    selected = [r for r in rows if start <= r['tick'] <= end]
    if [r['tick'] for r in selected] != list(range(start, end + 1, 1000)):
        return None
    # Neural means over living organisms are undefined at extinction. Do not
    # turn the engine's zero sentinel into a measured zero-sized living brain.
    if any(r['population'] == 0 for r in selected):
        return None
    means = {key: statistics.mean(r[key] for r in selected) for key in METRICS}
    means['mean_feed_forward_connections'] = means['mean_connections'] - means['mean_recurrent_connections']
    counts = {key: selected[-1]['total_' + key] - selected[0]['total_' + key]
              + selected[0][key] for key in EVENTS}
    return dict(start_tick=start, end_tick=end, samples=len(selected), mean=means,
                population_sd=statistics.pstdev(r['population'] for r in selected),
                population_min=min(r['population'] for r in selected),
                population_max=max(r['population'] for r in selected),
                near_cap_fraction=sum(r['population'] >= 1800 for r in selected) / len(selected),
                event_counts=counts, any_cap_blocked_births=counts['cap_blocked_births'] > 0)


def check_export(path, config, manifest, final, reader=None):
    data = read_json(path, reader)
    expected = copy.deepcopy(config)
    expected['food']['droughts'].setdefault('schedule_seed', None)
    expected['epigenetics'].setdefault('stress_threshold', .3)
    require(data['format'] == 'terrarium-genomes' and data['version'] == 1
            and data['config'] == expected and data['tick'] == final['tick']
            and data['core_build'] == CORE_BUILD and data['target'] == manifest['target'],
            'endpoint export provenance/config mismatch')
    genomes = data['genomes']
    require(len(genomes) == final['population'], 'export/CSV population mismatch')
    ids = [g['id'] for g in genomes]
    require(len(set(ids)) == len(ids) and all(type(i) is int and i >= 0 for i in ids), 'invalid exported IDs')
    forward = [len(g['expressed_brain']['connections']) for g in genomes]
    recurrent = [len(g['expressed_brain']['recurrent']) for g in genomes]
    for key, value in [('mean_connections', statistics.mean(a + b for a, b in zip(forward, recurrent)) if genomes else 0),
                       ('mean_recurrent_connections', statistics.mean(recurrent) if genomes else 0),
                       ('recurrent_brain_fraction', sum(n > 0 for n in recurrent) / len(genomes) if genomes else 0)]:
        require(math.isclose(final[key], value, rel_tol=1e-12, abs_tol=1e-12), f'export {key} mismatch')


def summarize(batch, allow_incomplete=False):
    batch = Path(batch).resolve()
    hashes = {}

    def checked_path(path):
        require(path.is_file() and not path.is_symlink(), f'not a regular audit input: {path}')
        require(path.resolve().is_relative_to(batch), 'audit input escapes batch')
        return str(path.relative_to(batch))

    def consumed_bytes(path):
        relative = checked_path(path)
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        require(relative not in hashes or hashes[relative] == digest,
                'input changed during audit; refresh a stable mirror and retry')
        hashes[relative] = digest
        return data

    def json_at(path):
        return read_json(path, consumed_bytes)

    def text_at(path):
        return consumed_bytes(path).decode()

    def record_hash(path):
        relative = checked_path(path)
        if relative not in hashes:
            hashes[relative] = sha(path)
        return hashes[relative]

    campaign = json_at(batch / 'campaign.json')
    parent_state = text_at(batch / 'status').strip()
    require(parent_state in ('prepared', 'running', 'completed', 'failed'), 'invalid campaign status')
    require(allow_incomplete or parent_state == 'completed', 'campaign incomplete; use --allow-incomplete for a labeled snapshot')
    require(campaign['schema_version'] == 1 and campaign['task'] == 'baseline_long_recurrence_pressure'
            and campaign['engine_revision'] == ENGINE and campaign['source_dirty'] is False
            and campaign['binary_sha256'] == BINARY_SHA and campaign['base_sha256'] == BASE_SHA
            and campaign['worker_sha256'] == WORKER_SHA and campaign['wall_seconds_per_job'] == 21600
            and campaign['total_active_wall_seconds'] == 172800, 'campaign protocol/source mismatch')
    require(record_hash(batch / 'run_long_recurrence.py') == WORKER_SHA, 'copied worker changed')
    records = campaign['jobs']
    require(len(records) == 8, 'planned job count differs')
    worker = json_at(batch / 'worker.json') if (batch / 'worker.json').exists() else None
    names = [f'job_{i:02d}_seed_{s}_rate_{p}' for i, (s, p) in enumerate(JOBS)]
    if worker:
        done = worker['completed_jobs']
        require(done == names[:len(done)], 'worker completed jobs not an ordered prefix')
    else:
        require(parent_state == 'prepared', 'missing worker state')
    if parent_state == 'completed':
        require(worker['state'] == 'completed' and worker['exit_code'] == 0
                and worker['completed_jobs'] == names and text_at(batch / 'exit_code').strip() == '0',
                'campaign completion inconsistent')
    for name in ('campaign.json', 'run_long_recurrence.py', 'status'):
        record_hash(batch / name)
    if worker:
        record_hash(batch / 'worker.json')
    runs = []
    provenance = None
    for index, ((seed, rate), record) in enumerate(zip(JOBS, records)):
        name = names[index]
        require(record['name'] == name, 'job name/order mismatch')
        expected_job = dict(seed=seed, recurrence_probability=rate, ticks=500000,
                            log_interval=1000, checkpoint_every=50000, export_genomes_every=500000)
        require(record['job'] == expected_job, 'job protocol changed')
        child = batch / name
        pins = dict(PINNED_INPUTS, **{'job.json': record_hash(child / 'job.json')})
        require(set(record['sha256']) == set(pins), 'pinned input coverage mismatch')
        for file, digest in pins.items():
            require(record['sha256'][file] == digest and record_hash(child / file) == digest,
                    f'pinned artifact changed: {name}/{file}')
        require(json_at(child / 'job.json') == expected_job, 'copied job changed')
        base = tomllib.loads(text_at(child / 'base.toml'))
        config = copy.deepcopy(base)
        config['world']['seed'] = seed
        config['mutation']['add_recurrent_probability'] = float(rate)
        sessions = list((child / 'runs').glob('session_*'))
        require(len(sessions) <= 1, 'multiple sessions; recovery requires separate audit')
        supervisor = json_at(child / 'supervisor.json') if (child / 'supervisor.json').exists() else None
        child_state = text_at(child / 'status').strip() if (child / 'status').exists() else None
        status = ('technical_failed' if child_state == 'failed' else
                  'active' if child_state == 'running' else 'not_started')
        if supervisor:
            for file in ('supervisor.json', 'supervisor_manifest.json', 'status'):
                record_hash(child / file)
            sm = json_at(child / 'supervisor_manifest.json')
            require(sm['task'] == 'ecology-long' and sm['wall_seconds'] == 21600
                    and sm['runner_rss_limit_bytes'] == 2 * 1024**3
                    and sm['environment_override'] == {'RAYON_NUM_THREADS': '1'}
                    and sm['temperature_high_c'] == 75 and sm['temperature_low_c'] == 65
                    and sm['guard_poll_seconds'] == 5
                    and sm['sha256'] == {k: v for k, v in pins.items() if k != 'run_guarded_acquisition.py'},
                    'supervisor policy/input mismatch')
        if child_state == 'completed':
            require(supervisor and supervisor['state'] == 'completed'
                    and supervisor['runner_exit_code'] == 0 and supervisor['guard_exit_code'] == 0
                    and supervisor['supervisor_exit_code'] == 0
                    and text_at(child / 'exit_code').strip() == '0', 'unclean child completion')
            guard_path = child / 'thermal_guard/status.json'
            guard = json_at(guard_path)
            require(guard['state'] == 'finished' and guard['running_matches'] == 0
                    and guard['paused'] == [], 'guard not cleanly finished')
            record_hash(guard_path)
            record_hash(child / 'exit_code')
            require(len(sessions) == 1, 'completed child lacks native session')
        elif child_state is None:
            require(not sessions and not supervisor, 'session exists without operational state')
        else:
            require(child_state in ('running', 'failed') and supervisor, 'invalid child state')
        run = dict(job_name=name, seed=seed, recurrence_probability=rate, status=status,
                   native_status=None, observed_tick=None, completed_ticks=None, csv_rows=0,
                   early=None, late=None, late_minus_early=None, final=None, observed_last_snapshot=None,
                   export_crosscheck=None, trajectory=[], truncated_csv_tail_ignored=False,
                   observed_population=None, any_observed_cap_blocked_births=None,
                   supervision_status=supervisor['state'] if supervisor else None)
        if sessions:
            session = sessions[0]
            mp = session / f'manifest_seed{seed}.toml'
            manifest = tomllib.loads(text_at(mp))
            record_hash(mp)
            require(manifest['schema_version'] == 2 and manifest['seed'] == seed
                    and manifest['source_revision'] == ENGINE and manifest['source_dirty'] == 'false'
                    and manifest['core_build_id'] == CORE_BUILD and manifest['build_profile'] == 'release'
                    and manifest['target'] == 'aarch64-unknown-linux-gnu'
                    and manifest['starting_tick'] == 0 and manifest['requested_ticks'] == 500000
                    and manifest['log_interval'] == 1000 and manifest['checkpoint_every'] == 50000
                    and manifest['export_genomes_every'] == 500000 and manifest['config'] == config,
                    'native source/config/protocol mismatch')
            identity = {key: manifest[key] for key in ('source_revision', 'source_dirty', 'core_build_id',
                                                       'target', 'build_profile', 'compiler')}
            if provenance is None:
                provenance = identity
            require(identity == provenance, 'native build identities differ')
            native_status = manifest['status']
            require(native_status in ('running', 'completed', 'extinct', 'failed'), 'unknown native status')
            if child_state == 'completed':
                require(native_status in ('completed', 'extinct'), 'native not complete under completed supervisor')
                status = native_status
            run.update(status=status, native_status=native_status, source_provenance=identity)
            cp = session / f'config_seed{seed}.toml'
            if cp.exists():
                require(tomllib.loads(text_at(cp)) == config, 'effective config artifact changed')
                record_hash(cp)
            else:
                require(status == 'active', 'missing effective config artifact')
            sp = session / f'stats_seed{seed}.csv'
            rows, truncated = read_rows(sp, status not in ('completed', 'extinct'), consumed_bytes) if sp.exists() else ([], False)
            if sp.exists():
                record_hash(sp)
            ticks = [r['tick'] for r in rows]
            if rows:
                endpoint = int(ticks[-1])
                grid = list(range(1000, endpoint + 1, 1000))
                if endpoint % 1000:
                    require(native_status in ('extinct', 'failed') and rows[-1]['population'] == 0,
                            'unexpected off-grid CSV row')
                    grid.append(endpoint)
                require(ticks == grid, 'missing/irregular saved prefix')
                require(all(r['population'] > 0 for r in rows[:-1]), 'population resurrected after extinction')
                if rate == 0:
                    require(all(r['mean_recurrent_connections'] == 0 and r['recurrent_brain_fraction'] == 0
                                for r in rows), 'recurrence in off control')
                run.update(observed_tick=endpoint, csv_rows=len(rows),
                           observed_last_snapshot={k: rows[-1][k] for k in ['tick'] + METRICS + COUNTERS},
                           early=window(rows, 50000, 100000),
                           observed_population=dict(samples=len(rows),
                                                    minimum=min(r['population'] for r in rows),
                                                    maximum=max(r['population'] for r in rows),
                                                    sd=statistics.pstdev(r['population'] for r in rows)),
                           any_observed_cap_blocked_births=rows[-1]['total_cap_blocked_births'] > 0,
                           trajectory=[{k: r[k] for k in ['tick'] + METRICS + COUNTERS} for r in rows])
                final = dict(run['observed_last_snapshot'])
                final['mean_feed_forward_connections'] = final['mean_connections'] - final['mean_recurrent_connections']
                if not final['population']:
                    for key in NEURAL:
                        final[key] = None
                run['final'] = final
            if status in ('completed', 'extinct'):
                require(rows and rows[-1]['tick'] == manifest['completed_ticks'], 'terminal CSV/manifest endpoint mismatch')
                if status == 'completed':
                    require(manifest['completed_ticks'] == 500000 and manifest['survived'] is True
                            and ticks == list(range(1000, 500001, 1000)) and rows[-1]['population'] > 0,
                            'incomplete claimed completion')
                    run['late'] = window(rows, 375000, 500000)
                    require(run['early'] and run['late'], 'completed world lacks full living windows')
                    run['late_minus_early'] = {k: run['late']['mean'][k] - run['early']['mean'][k]
                                               for k in run['late']['mean']}
                else:
                    require(manifest['survived'] is False and rows[-1]['population'] == 0,
                            'extinction status/population mismatch')
                export = session / f'genomes_seed{seed}_tick{manifest["completed_ticks"]}.json'
                check_export(export, config, manifest, rows[-1], consumed_bytes)
                record_hash(export)
                run.update(completed_ticks=manifest['completed_ticks'], export_crosscheck=True)
            run['truncated_csv_tail_ignored'] = truncated
        else:
            require(child_state != 'completed', 'completed child has no native session')
        if worker and name in worker['completed_jobs']:
            require(run['status'] in ('completed', 'extinct'), 'worker claims an unaudited completed job')
        runs.append(run)
    started = [i for i, r in enumerate(runs) if r['status'] != 'not_started']
    require(started == list(range(len(started))), 'queue skipped an earlier unstarted job')
    unfinished = [i for i in started if runs[i]['status'] not in ('completed', 'extinct')]
    require(len(unfinished) <= 1 and (not unfinished or unfinished[0] == started[-1]),
            'parallel active jobs or queue continued after an unfinished job')
    if worker and started:
        require(worker['current_job'] == names[started[-1]], 'worker/job snapshot mismatch; refresh mirror')
    coverage = {key: sum(r['status'] == key for r in runs)
                for key in ('completed', 'extinct', 'active', 'technical_failed', 'not_started')}
    coverage['planned'] = 8
    complete = parent_state == 'completed' and coverage['completed'] + coverage['extinct'] == 8
    require(allow_incomplete or complete, 'not all eight worlds audited')
    paired = []
    for seed in range(4000, 4004):
        off, on = [next(r for r in runs if r['seed'] == seed and r['recurrence_probability'] == p) for p in (0, .3)]
        available = bool(off['late'] and on['late'])
        paired.append(dict(seed=seed, off_status=off['status'], on_status=on['status'],
                           both_completed=all(r['status'] == 'completed' for r in (off, on)),
                           late_difference_on_minus_off={k: on['late']['mean'][k] - off['late']['mean'][k]
                                                         for k in off['late']['mean']} if available else None,
                           missing_reason=None if available else 'full audited living late windows unavailable in one or both planned worlds'))
    groups = []
    for p in (0, .3):
        group = [r for r in runs if r['recurrence_probability'] == p]
        available = [r for r in group if r['late']]
        groups.append(dict(recurrence_probability=p, planned=4, available_late=len(available),
                           late_across_run_mean={k: statistics.mean(r['late']['mean'][k] for r in available)
                                                for k in available[0]['late']['mean']} if available else None))
    valid_pairs = [p['late_difference_on_minus_off'] for p in paired if p['late_difference_on_minus_off']]
    pair_summary = dict(planned_pairs=4, available_pairs=len(valid_pairs),
                        mean_on_minus_off={k: statistics.mean(p[k] for p in valid_pairs) for k in valid_pairs[0]} if valid_pairs else None,
                        range_on_minus_off={k: [min(p[k] for p in valid_pairs), max(p[k] for p in valid_pairs)]
                                           for k in valid_pairs[0]} if valid_pairs else None)
    return dict(schema_version=1, complete=complete, status=parent_state,
                interpretation='Structural/demographic diagnostic only; partial reports are not final results. No memory-benefit, equilibrium or selection-versus-drift claim.',
                snapshot_note='Non-atomic multi-file observation. Hashes identify the exact bytes consumed, including buffered CSV tails. Observed saved prefixes can lag the runner; refresh a stable mirror before final analysis.',
                independent_unit='world; four planned paired seed labels, not organisms or saved ticks',
                counter_window_rule='C(end)-C(start)+events_at_start; inclusive exact simulation-tick interval',
                activity_units='raw pre-action weighted-signal activity, distinct from energy-ledger costs',
                engine=provenance, campaign=campaign, coverage=coverage, runs=runs,
                paired=paired, paired_summary=pair_summary, by_probability=groups, artifact_sha256=hashes)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('batch', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--allow-incomplete', action='store_true')
    args = parser.parse_args()
    try:
        result = summarize(args.batch, args.allow_incomplete)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with args.output.open('x') as stream:
            json.dump(result, stream, indent=2, allow_nan=False)
            stream.write('\n')
        print(json.dumps({'complete': result['complete'], 'coverage': result['coverage']}, indent=2))
    except (OSError, ValueError, KeyError, TypeError, csv.Error) as error:
        parser.exit(1, f'long recurrence audit refused: {error}\n')
