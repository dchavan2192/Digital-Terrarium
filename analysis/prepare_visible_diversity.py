#!/usr/bin/env python3
"""Copy a clean pinned build and frozen inputs before a one-off guarded pilot.

Preparation only: never starts a simulation or replaces an attempted batch.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prepare(args):
    source, binary, batch = args.source.resolve(), args.binary.resolve(), args.batch.resolve()
    revision = subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
    dirty = subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain'], text=True).strip()
    if revision != args.revision or dirty:
        raise ValueError('source is not the requested clean commit')
    if subprocess.run(['pgrep', '-x', 'sim-runner'], capture_output=True).returncode != 1:
        raise ValueError('sim-runner already exists, or process check failed')
    disk_free = shutil.disk_usage(source).free
    available = int(next(line.split()[1] for line in Path('/proc/meminfo').read_text().splitlines()
                         if line.startswith('MemAvailable:'))) * 1024
    if disk_free < 20 * 1024**3 or available < 3 * 1024**3:
        raise ValueError('require20GiB disk and3GiB available RAM')
    if not binary.is_file():
        raise ValueError('missing pinned executable')
    batch.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(binary, batch / 'sim-runner')
    (batch / 'sim-runner').chmod(0o755)
    files = {'visible_diversity.toml': source / 'configs/visible_diversity.toml',
             'delayed_choice.toml': source / 'configs/delayed_choice.toml',
             'pi_thermal_guard.py': source / 'analysis/pi_thermal_guard.py',
             'run_guarded_acquisition.py': source / 'analysis/run_guarded_acquisition.py'}
    for name, path in files.items():
        shutil.copyfile(path, batch / name)
    hashes = {p.name: sha(p) for p in sorted(batch.iterdir())}
    hashes['sim-runner'] = sha(binary)
    if sha(batch / 'sim-runner') != hashes['sim-runner']:
        raise ValueError('binary copy mismatch')
    temperature = float(Path('/sys/class/thermal/thermal_zone0/temp').read_text()) / 1000
    throttled = subprocess.run(['vcgencmd', 'get_throttled'], capture_output=True, text=True, check=True)
    command = [sys.executable, str(batch / 'run_guarded_acquisition.py'), '--task',
               'acquire-visible-diversity', '--batch', str(batch), '--wall-seconds', '1800']
    record = dict(schema_version=1, status='prepared_not_launched',
                  prepared_utc=datetime.now(timezone.utc).isoformat(), source_revision=revision,
                  source_dirty=False, source_root=str(source), batch=str(batch),
                  artifacts_sha256=hashes, preparation_script_sha256=sha(Path(__file__)),
                  plan_sha256=sha(source / 'docs/results/VISIBLE_DIVERSITY_PLAN_2026-11-18.md'),
                  resources=dict(free_disk_bytes=disk_free, available_ram_bytes=available,
                                 temperature_c=temperature, throttled_output=throttled.stdout.strip()),
                  build=dict(workers=1, offline=True, wall_timeout_seconds=900),
                  command=command, schedules='remain disabled; one-off only')
    (batch / 'launch.json').write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps(record, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('source', 'binary', 'batch'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--revision', required=True)
    arguments = parser.parse_args()
    try:
        prepare(arguments)
    except (OSError, ValueError, StopIteration, subprocess.SubprocessError) as error:
        parser.exit(1, f'diversity preparation refused: {error}\n')
