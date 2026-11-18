#!/usr/bin/env python3
"""Frozen long baseline recurrence-pressure study, one guarded Pi job at a time.

Prepare copies first; run only a fresh prepared campaign. Never restart this
worker on an attempted campaign or overwrite checkpoints. See the saved plan.
"""
import argparse
from datetime import datetime, timezone
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time
import tomllib

ENGINE = 'b40ae9c15ef5f90f17e7e2bbbc286c1c19c0b88c'
BINARY_SHA = '05e6fe23121818fb897332a53f314da11e2ea98fed6f61cfe52d607009833e63'
WALL_SECONDS = 21600
JOBS = [(4000, 0), (4000, .3), (4001, .3), (4001, 0),
        (4002, 0), (4002, .3), (4003, .3), (4003, 0)]


def utc():
    return datetime.now(timezone.utc).isoformat()


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def atomic_json(path, value):
    pending = path.with_name(path.name + '.pending')
    pending.write_text(json.dumps(value, indent=2) + '\n')
    pending.replace(path)


def require_resources(batch):
    if shutil.disk_usage(batch).free < 20 * 1024**3:
        raise ValueError('less than 20 GiB disk headroom; stop without starting another job')
    memory = Path('/proc/meminfo').read_text()
    available = int(next(line.split()[1] for line in memory.splitlines()
                         if line.startswith('MemAvailable:'))) * 1024
    if available < 3 * 1024**3:
        raise ValueError('less than 3 GiB available RAM; no runner started')


def active_runners():
    matches = []
    for path in Path('/proc').iterdir():
        if path.name.isdecimal():
            try:
                if Path(os.readlink(path / 'exe')).name == 'sim-runner':
                    matches.append(int(path.name))
            except (FileNotFoundError, ProcessLookupError):
                pass
            except PermissionError:
                # Other users' processes cannot safely be identified here.
                # The Pi run account owns the simulation jobs in this project.
                continue
    return matches


def prepare(args):
    root = Path(__file__).resolve().parents[1]
    batch = args.batch.resolve()
    binary, base = args.binary.resolve(), args.base.resolve()
    if binary.is_symlink() or sha(binary) != BINARY_SHA:
        raise ValueError('binary does not match the declared clean evaluator/engine pin')
    config = tomllib.loads(base.read_text())
    if (config['food']['regrowth_rate'] != .008 or config['creature']['hard_cap'] != 2000
            or config['mutation']['add_recurrent_probability'] != 0):
        raise ValueError('base is not the frozen baseline configuration')
    # Whole-tree identity, not merely the three values above.
    original = subprocess.check_output(['git', 'show', f'{ENGINE}:configs/default.toml'], cwd=root)
    if base.read_bytes() != original:
        raise ValueError('base differs from exact pinned engine defaults')
    batch.mkdir(parents=True, exist_ok=False)
    (batch / 'status').write_text('preparing\n')
    require_resources(batch)
    records = []
    for index, (seed, rate) in enumerate(JOBS):
        name = f'job_{index:02d}_seed_{seed}_rate_{rate}'
        child = batch / name
        child.mkdir()
        job = dict(seed=seed, recurrence_probability=rate, ticks=500000,
                   log_interval=1000, checkpoint_every=50000, export_genomes_every=500000)
        (child / 'job.json').write_text(json.dumps(job, indent=2) + '\n')
        shutil.copyfile(binary, child / 'sim-runner')
        (child / 'sim-runner').chmod(0o755)
        shutil.copyfile(base, child / 'base.toml')
        for file in ('pi_thermal_guard.py', 'run_guarded_acquisition.py'):
            shutil.copyfile(root / 'analysis' / file, child / file)
        records.append(dict(name=name, job=job, sha256={
            p.name: sha(p) for p in sorted(child.iterdir()) if p.is_file()}))
    # The worker itself travels with the campaign. Later repository changes
    # cannot silently alter the queue or safety policy.
    shutil.copyfile(Path(__file__), batch / 'run_long_recurrence.py')
    manifest = dict(schema_version=1, task='baseline_long_recurrence_pressure',
                    prepared_utc=utc(), engine_revision=ENGINE, source_dirty=False,
                    binary_sha256=BINARY_SHA, base_sha256=sha(base),
                    worker_sha256=sha(batch / 'run_long_recurrence.py'),
                    wall_seconds_per_job=WALL_SECONDS,
                    total_active_wall_seconds=8 * WALL_SECONDS,
                    bounded_cleanup_seconds_per_job=20,
                    minimum_free_disk_bytes=20 * 1024**3,
                    minimum_available_ram_bytes=3 * 1024**3,
                    runner_rss_limit_bytes=2 * 1024**3,
                    checkpoint_byte_limit=256 * 1024**2,
                    jobs=records,
                    interpretation='Structural/demographic diagnostic; not memory benefit or selection-versus-drift.')
    atomic_json(batch / 'campaign.json', manifest)
    (batch / 'status').write_text('prepared\n')
    print(batch)


def run_campaign(batch):
    batch = batch.resolve()
    with (batch / '.campaign.lock').open('x') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if (batch / 'status').read_text().strip() != 'prepared':
            raise ValueError('campaign already attempted/not prepared; preserve it')
        campaign = json.loads((batch / 'campaign.json').read_text())
        if (campaign['engine_revision'] != ENGINE or campaign['binary_sha256'] != BINARY_SHA
                or campaign['wall_seconds_per_job'] != WALL_SECONDS
                or [(r['job']['seed'], r['job']['recurrence_probability'])
                    for r in campaign['jobs']] != JOBS
                or sha(batch / 'run_long_recurrence.py') != campaign['worker_sha256']):
            raise ValueError('campaign differs from fixed source/order/worker')
        for record in campaign['jobs']:
            for name, digest in record['sha256'].items():
                if sha(batch / record['name'] / name) != digest:
                    raise ValueError('pinned campaign input changed')
        stopped = False

        def interrupt(_sig, _frame):
            nonlocal stopped
            stopped = True

        handlers = {sig: signal.signal(sig, interrupt) for sig in (signal.SIGTERM, signal.SIGINT)}
        state = dict(state='running', started_utc=utc(), worker_pid=os.getpid(),
                     current_job=None, supervisor_pid=None, completed_jobs=[])
        (batch / 'status').write_text('running\n')
        atomic_json(batch / 'worker.json', state)
        exit_code = 1
        supervisor = None
        try:
            for record in campaign['jobs']:
                if stopped:
                    raise ValueError('interrupted before next job; nothing else started')
                require_resources(batch)
                if active_runners():
                    raise ValueError('another sim-runner exists; no concurrent job allowed')
                child = batch / record['name']
                command = [sys.executable, str(child / 'run_guarded_acquisition.py'),
                           '--task', 'ecology-long', '--batch', str(child),
                           '--wall-seconds', str(WALL_SECONDS)]
                with (child / 'launcher.log').open('x') as log:
                    supervisor = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT,
                                                  start_new_session=True)
                    state.update(current_job=record['name'], supervisor_pid=supervisor.pid)
                    atomic_json(batch / 'worker.json', state)
                    started = time.monotonic()
                    interrupted = False
                    while supervisor.poll() is None:
                        if stopped or time.monotonic() - started > WALL_SECONDS + 20:
                            supervisor.terminate()
                            interrupted = True
                            try:
                                supervisor.wait(timeout=20)
                            except subprocess.TimeoutExpired:
                                # Preserve its still-guarded descendants for inspection;
                                # do not kill a supervisor and orphan its protection.
                                raise ValueError('supervisor unresponsive after TERM; inspect owned processes')
                            break
                        time.sleep(.5)
                if interrupted or supervisor.returncode != 0:
                    raise ValueError(f"job {record['name']} interrupted/failed; stop queue and preserve partials")
                status = json.loads((child / 'supervisor.json').read_text())
                guard = json.loads((child / 'thermal_guard/status.json').read_text())
                if (status['state'] != 'completed' or status['runner_exit_code'] != 0
                        or status['guard_exit_code'] != 0 or guard['state'] != 'finished'
                        or guard['paused'] or guard['running_matches']):
                    raise ValueError('job did not finish cleanly; stop queue')
                state['completed_jobs'].append(record['name'])
                atomic_json(batch / 'worker.json', state)
            state['state'] = 'completed'
            exit_code = 0
        except (OSError, ValueError, KeyError) as error:
            state.update(state='failed', reason=str(error))
        finally:
            # Persistence/resource exceptions after Popen must also stop the
            # directly owned supervisor, whose handler pauses its own runner.
            if supervisor is not None and supervisor.poll() is None:
                supervisor.terminate()
                try:
                    supervisor.wait(timeout=20)
                except subprocess.TimeoutExpired:
                    state['supervisor_cleanup_unresponsive'] = True
                    state['reason'] = 'supervisor unresponsive; inspect its guarded descendants before recovery'
                    state['state'] = 'failed'
                    exit_code = 1
            state.update(ended_utc=utc(), exit_code=exit_code)
            atomic_json(batch / 'worker.json', state)
            (batch / 'status').write_text(state['state'] + '\n')
            (batch / 'exit_code').write_text(str(exit_code) + '\n')
            for sig, handler in handlers.items():
                signal.signal(sig, handler)
        return exit_code


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    setup = commands.add_parser('prepare')
    for name in ('batch', 'binary', 'base'):
        setup.add_argument('--' + name, type=Path, required=True)
    execute = commands.add_parser('run')
    execute.add_argument('batch', type=Path)
    args = parser.parse_args()
    try:
        result = prepare(args) if args.command == 'prepare' else run_campaign(args.batch)
        raise SystemExit(result or 0)
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f'long recurrence campaign refused: {error}\n')
