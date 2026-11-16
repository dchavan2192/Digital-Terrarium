"""Summarize a held-out validation batch against its pre-written criteria.

Usage:
    python3 analysis/summarize_validation.py BATCH_DIR [SUMMARY_JSON] [--allow-incomplete]

BATCH_DIR is a (mirror of a) directory made by analysis/launch_pi_validation.sh:
rate_<rate>/seed_<seed>/session_*/{stats,manifest,config}_seed<seed>.{csv,toml},
plus settings.env and jobs.txt. Measurements, criteria and the decision rule are
fixed in docs/results/PHASE9A_VALIDATION_PLAN.md; this script only applies them.

A job interrupted and resumed has several sessions. Their rows are joined on
tick; a tick logged by two sessions must carry identical values, because a
resumed run continues the exact state. A mismatch is an error, not a warning.

Without --allow-incomplete the script refuses a batch with unfinished jobs, so a
partial batch can never produce a decision. Standard library only.
"""
import csv
import hashlib
import json
from pathlib import Path
import statistics
import sys
import tomllib

LATE_START = 100_000        # late window: LATE_START < tick <= target
BURN_IN = 20_000            # minimum-after-burn-in window: BURN_IN <= tick <= target
POPULATION_RANGE = (800.0, 1300.0)
MINIMUM_FLOOR = 200
HARD_CAP = 2000


def read_settings(batch):
    settings = {}
    for line in (batch / 'settings.env').read_text().splitlines():
        key, _, value = line.partition('=')
        if key and value:
            settings[key] = int(value)
    return settings


def read_jobs(batch):
    jobs = []
    for line in (batch / 'jobs.txt').read_text().splitlines():
        rate, seed = line.split()
        jobs.append((rate, int(seed)))
    if len(set(jobs)) != len(jobs):
        raise ValueError('jobs.txt lists a job twice')
    return jobs


def load_job(batch, rate, seed, target):
    """Return (state, run_dict). state is 'completed', 'extinct' or 'incomplete'."""
    job_dir = batch / f'rate_{rate}' / f'seed_{seed}'
    rows_by_tick = {}
    sessions, hashes, manifests = [], {}, []
    for manifest_path in sorted(job_dir.glob(f'session_*/manifest_seed{seed}.toml')):
        manifest = tomllib.loads(manifest_path.read_text())
        session = manifest_path.parent
        csv_path = session / f'stats_seed{seed}.csv'
        config_path = session / f'config_seed{seed}.toml'
        config = manifest['config']
        if config['world']['seed'] != seed or f"{config['food']['regrowth_rate']:.3f}" != rate:
            raise ValueError(f'{manifest_path}: configuration does not match its job directory')
        if config['creature']['hard_cap'] != HARD_CAP:
            raise ValueError(f'{manifest_path}: hard_cap is not {HARD_CAP}')
        if config_path.exists() and tomllib.loads(config_path.read_text()) != config:
            raise ValueError(f'{config_path}: stored configuration differs from the manifest')
        if csv_path.exists():
            with csv_path.open() as stream:
                for row in csv.DictReader(stream):
                    tick = int(row['tick'])
                    earlier = rows_by_tick.get(tick)
                    if earlier is not None and earlier != row:
                        raise ValueError(f'{csv_path}: tick {tick} differs from another session of the same job')
                    rows_by_tick[tick] = row
        manifests.append(manifest)
        sessions.append(session.name)
        for artifact in (manifest_path, csv_path, config_path):
            if artifact.exists():
                hashes[str(artifact.relative_to(batch))] = hashlib.sha256(artifact.read_bytes()).hexdigest()

    extinct = any(m['status'] == 'extinct' for m in manifests)
    completed = any(m['status'] == 'completed' and m['completed_ticks'] == target for m in manifests)
    if not (completed or extinct):
        return 'incomplete', None, hashes

    rows = [rows_by_tick[t] for t in sorted(rows_by_tick)]
    last_tick = int(rows[-1]['tick'])
    run = dict(
        regrowth_rate=float(rate), seed=seed, sessions=sessions, resumed=len(sessions) > 1,
        survived=completed and not extinct and int(rows[-1]['population']) > 0,
        last_tick=last_tick, final_population=int(rows[-1]['population']),
        total_cap_blocked_births=int(rows[-1]['total_cap_blocked_births']),
        minimum_logged_population=min(int(r['population']) for r in rows),
        max_abs_energy_residual=max(abs(float(r['energy_balance_residual'])) for r in rows),
        elapsed_seconds=sum(m['elapsed_seconds'] for m in manifests),
        source_revision=sorted({m['source_revision'] for m in manifests}),
        source_dirty=sorted({m['source_dirty'] for m in manifests}),
        core_build_id=sorted({m['core_build_id'] for m in manifests}),
        target=sorted({m['target'] for m in manifests}),
        compiler=sorted({m['compiler'] for m in manifests}),
    )
    if completed and not extinct:
        late = [r for r in rows if LATE_START < int(r['tick']) <= target]
        after_burn_in = [r for r in rows if BURN_IN <= int(r['tick']) <= target]
        interval = manifests[0]['log_interval']
        expected_late = (target - LATE_START) // interval
        if last_tick != target or len(late) != expected_late:
            raise ValueError(f'rate {rate} seed {seed}: expected {expected_late} late snapshots ending at {target}, '
                             f'found {len(late)} ending at {last_tick}')
        run.update(
            late_mean_population=statistics.mean(float(r['population']) for r in late),
            late_mean_energy=statistics.mean(float(r['mean_energy']) for r in late),
            late_mean_attacks=statistics.mean(float(r['attacks']) for r in late),
            late_mean_shares=statistics.mean(float(r['shares']) for r in late),
            late_mean_species=statistics.mean(float(r['species_count']) for r in late),
            late_mean_connections=statistics.mean(float(r['mean_connections']) for r in late),
            minimum_after_burn_in=min(int(r['population']) for r in after_burn_in),
            max_generation=int(rows[-1]['max_generation']),
        )
    return ('extinct' if extinct else 'completed'), run, hashes


def judge(rate, runs, expected_seeds):
    """Apply the four pre-written criteria to one rate. Needs every seed finished."""
    survived = [r for r in runs if r['survived']]
    criteria = {
        'all_seeds_survived': len(survived) == expected_seeds,
        'no_cap_blocked_births': all(r['total_cap_blocked_births'] == 0 for r in runs),
    }
    result = dict(regrowth_rate=float(rate), seeds=len(runs), survived=len(survived))
    if survived:
        late = [r['late_mean_population'] for r in survived]
        result.update(
            mean_late_population=statistics.mean(late),
            sd_late_population=statistics.stdev(late) if len(late) > 1 else 0.0,
            min_late_population=min(late), max_late_population=max(late),
            lowest_minimum_after_burn_in=min(r['minimum_after_burn_in'] for r in survived),
            cap_blocked_births=sum(r['total_cap_blocked_births'] for r in runs),
            max_abs_energy_residual=max(r['max_abs_energy_residual'] for r in runs),
        )
        criteria['mean_population_in_range'] = POPULATION_RANGE[0] <= result['mean_late_population'] <= POPULATION_RANGE[1]
        criteria['minimum_above_floor'] = (len(survived) == expected_seeds
                                           and result['lowest_minimum_after_burn_in'] >= MINIMUM_FLOOR)
    else:
        criteria['mean_population_in_range'] = False
        criteria['minimum_above_floor'] = False
    result['criteria'] = criteria
    result['acceptable'] = all(criteria.values())
    return result


def summarize(batch, allow_incomplete=False):
    batch = Path(batch)
    settings = read_settings(batch)
    target = settings['TICKS']
    if settings['HARD_CAP'] != HARD_CAP:
        raise ValueError(f"settings.env: HARD_CAP is not {HARD_CAP}")
    jobs = read_jobs(batch)
    rates = sorted({rate for rate, _ in jobs}, key=float)
    seeds_per_rate = {rate: sum(1 for r, _ in jobs if r == rate) for rate in rates}
    runs, incomplete, hashes = [], [], {}
    for rate, seed in jobs:
        state, run, job_hashes = load_job(batch, rate, seed, target)
        hashes.update(job_hashes)
        if state == 'incomplete':
            incomplete.append([rate, seed])
        else:
            runs.append(run)
    if incomplete and not allow_incomplete:
        raise SystemExit(f'{len(incomplete)} of {len(jobs)} jobs are unfinished; refusing to summarize a partial '
                         f'batch (use --allow-incomplete for a progress view only)')

    by_rate = []
    for rate in rates:
        group = [r for r in runs if f"{r['regrowth_rate']:.3f}" == rate]
        if len(group) == seeds_per_rate[rate]:
            by_rate.append(judge(rate, group, seeds_per_rate[rate]))
        else:
            by_rate.append(dict(regrowth_rate=float(rate), seeds=len(group), expected_seeds=seeds_per_rate[rate],
                                acceptable=None, note='not all seeds finished; criteria not applied'))
    complete = not incomplete
    acceptable = [r['regrowth_rate'] for r in by_rate if r['acceptable']]
    decision = None
    if complete:
        decision = (dict(proposed_default=min(acceptable), rule='smallest acceptable rate')
                    if acceptable else dict(proposed_default=None, rule='no rate met all four criteria'))
    source_file = batch / 'source_revision'
    return dict(
        schema_version=1, batch=batch.name, complete=complete, jobs=len(jobs), finished=len(runs),
        unfinished=incomplete, target_ticks=target,
        launched_source_revision=source_file.read_text().strip() if source_file.exists() else None,
        windows=dict(late=f'{LATE_START} < tick <= {target}', after_burn_in=f'{BURN_IN} <= tick <= {target}'),
        criteria=dict(population_range=POPULATION_RANGE, minimum_floor=MINIMUM_FLOOR, hard_cap=HARD_CAP),
        measurement='one run is the independent unit; snapshots within a run are not replicates',
        by_rate=by_rate, decision=decision, runs=runs, sha256=hashes,
    )


def main(argv):
    args = [a for a in argv if not a.startswith('--')]
    allow_incomplete = '--allow-incomplete' in argv
    if not args or any(a for a in argv if a.startswith('--') and a != '--allow-incomplete'):
        raise SystemExit(__doc__)
    result = summarize(args[0], allow_incomplete)
    if len(args) > 1:
        Path(args[1]).write_text(json.dumps(result, indent=2) + '\n')

    print(f"Batch {result['batch']}: {result['finished']} of {result['jobs']} jobs finished"
          + ('' if result['complete'] else '  (PARTIAL: progress view only, no decision)'))
    print('| Rate | Seeds | Survived | Late population (mean ± sd) | Range across seeds | Lowest minimum | Cap-blocked | Acceptable |')
    print('|---|---|---|---|---|---|---|---|')
    for r in result['by_rate']:
        if r['acceptable'] is None:
            print(f"| {r['regrowth_rate']:.3f} | {r['seeds']}/{r['expected_seeds']} | – | – | – | – | – | pending |")
            continue
        print(f"| {r['regrowth_rate']:.3f} | {r['seeds']} | {r['survived']} | "
              f"{r.get('mean_late_population', float('nan')):.1f} ± {r.get('sd_late_population', float('nan')):.1f} | "
              f"{r.get('min_late_population', float('nan')):.0f}–{r.get('max_late_population', float('nan')):.0f} | "
              f"{r.get('lowest_minimum_after_burn_in', '–')} | {r.get('cap_blocked_births', '–')} | "
              f"{'yes' if r['acceptable'] else 'no: ' + ', '.join(k for k, ok in r['criteria'].items() if not ok)} |")
    if result['decision']:
        print('Decision:', result['decision'])


if __name__ == '__main__':
    main(sys.argv[1:])
