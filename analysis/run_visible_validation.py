#!/usr/bin/env python3
"""Prepare immutable copies and serially supervise the three frozen cohorts.

Use only after reviewing the committed prospective visible validation plan.
Every attempted parent is retained; reruns require a new parent directory.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def run(args):
    root = Path(__file__).resolve().parents[1]
    batch = args.batch.resolve()
    suite = root / 'configs/visible_validation.toml'
    if sha(suite) != '595d8892befd21e25368fa9a0ce13ecdb62e4958e01e18ef8c3fedee9b4eff8a':
        raise ValueError('frozen suite hash mismatch')
    sources = [('original', args.original), ('terminal_only', args.control),
               ('terminal_progress', args.progress)]
    batch.mkdir(parents=True, exist_ok=False)
    (batch / 'status').write_text('preparing\n')
    records = []
    for label, source in sources:
        source = source.resolve()
        manifest = json.loads((source / 'manifest.json').read_text())
        # Seeds are fixed by cohort, never selected from validation outcomes.
        seeds = [2000, 2001, 2002] if label == 'original' else [2100, 2101, 2102]
        child = batch / label
        child.mkdir()
        inputs = child / 'input'
        inputs.mkdir()
        files = ['manifest.json', 'summary.json']
        for seed in seeds:
            files += [f'seed_{seed}/summary.json', f'seed_{seed}/final_population.json']
        inventory = {}
        for name in files:
            src = source / name
            if src.is_symlink() or not src.is_file() or not src.resolve().is_relative_to(source):
                raise ValueError(f'unsafe/missing source {src}')
            dest = inputs / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, dest)
            inventory[name] = sha(src)
            if sha(dest) != inventory[name]:
                raise ValueError(f'copy mismatch {name}')
        shutil.copyfile(args.binary, child / 'sim-runner')
        (child / 'sim-runner').chmod(0o755)
        for src in (suite, root / 'analysis/pi_thermal_guard.py',
                    root / 'analysis/run_guarded_acquisition.py'):
            shutil.copyfile(src, child / src.name)
        records.append(dict(cohort=label, source=str(source), source_manifest=manifest,
                            source_sha256=inventory, seeds=seeds))
    script_names = ['analysis/run_visible_validation.py', 'analysis/run_guarded_acquisition.py',
                    'analysis/summarize_visible_validation.py', 'analysis/pi_thermal_guard.py',
                    'configs/visible_validation.toml']
    (batch / 'campaign.json').write_text(json.dumps(dict(
        schema_version=1, evaluator_revision=args.evaluator_revision,
        binary_sha256=sha(args.binary), cohorts=records,
        execution_order=[r['cohort'] for r in records], wall_seconds_per_cohort=1800,
        code_sha256={name: sha(root / name) for name in script_names}), indent=2) + '\n')
    (batch / 'status').write_text('running\n')
    for record in records:
        child = batch / record['cohort']
        command = [sys.executable, str(child / 'run_guarded_acquisition.py'),
                   '--task', 'validate-visible', '--batch', str(child), '--wall-seconds', '1800']
        with (child / 'launch.log').open('x') as log:
            result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=False)
        if result.returncode:
            (batch / 'status').write_text('failed\n')
            raise ValueError(f"cohort {record['cohort']} failed; preserve and inspect")
    (batch / 'status').write_text('completed\n')
    print(batch)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('batch', 'binary', 'original', 'control', 'progress'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--evaluator-revision', required=True)
    arguments = parser.parse_args()
    try:
        run(arguments)
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f'validation campaign refused: {error}\n')
