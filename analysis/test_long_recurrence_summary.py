"""Synthetic complete/partial/corrupt long-run datasets; never read Pi outcomes."""
import copy
import csv
import hashlib
import json
from pathlib import Path
import re
import tomllib
import unittest
from unittest.mock import patch
import uuid

import run_long_recurrence as worker
import summarize_long_recurrence as summary

ROOT = Path(__file__).resolve().parents[1]
ARTIFACTS = ROOT / 'output' / 'long-recurrence-summary-tests'


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, allow_nan=False) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class SummaryTests(unittest.TestCase):
    def setUp(self):
        self.batch = ARTIFACTS / uuid.uuid4().hex
        self.batch.mkdir(parents=True)
        self.base = (ROOT / 'configs/default.toml').read_text()
        self.config = tomllib.loads(self.base)
        self.binary = b'not executable; synthetic provenance fixture only\n'
        worker_bytes = b'# synthetic pinned worker\n'
        pinned = {'sim-runner': hashlib.sha256(self.binary).hexdigest(),
                  'base.toml': hashlib.sha256(self.base.encode()).hexdigest(),
                  **{name: hashlib.sha256(b'# synthetic pinned input\n').hexdigest()
                     for name in ('pi_thermal_guard.py', 'run_guarded_acquisition.py')}}
        self.patches = [patch.object(summary, 'BINARY_SHA', hashlib.sha256(self.binary).hexdigest()),
                        patch.object(summary, 'BASE_SHA', hashlib.sha256(self.base.encode()).hexdigest()),
                        patch.object(summary, 'WORKER_SHA', hashlib.sha256(worker_bytes).hexdigest()),
                        patch.object(summary, 'PINNED_INPUTS', pinned)]
        for item in self.patches:
            item.start()
            self.addCleanup(item.stop)
        (self.batch / 'run_long_recurrence.py').write_text('# synthetic pinned worker\n')
        self.records = []
        for index, (seed, rate) in enumerate(worker.JOBS):
            name = f'job_{index:02d}_seed_{seed}_rate_{rate}'
            child = self.batch / name
            child.mkdir()
            (child / 'sim-runner').write_bytes(self.binary)
            (child / 'base.toml').write_text(self.base)
            for filename in ('pi_thermal_guard.py', 'run_guarded_acquisition.py'):
                (child / filename).write_text('# synthetic pinned input\n')
            job = dict(seed=seed, recurrence_probability=rate, ticks=500000,
                       log_interval=1000, checkpoint_every=50000, export_genomes_every=500000)
            write_json(child / 'job.json', job)
            self.records.append(dict(name=name, job=job,
                                     sha256={p.name: digest(p) for p in child.iterdir()}))
            if index < getattr(self, 'run_count', 8):
                self.write_run(index)
        self.campaign = dict(schema_version=1, task='baseline_long_recurrence_pressure',
            engine_revision=summary.ENGINE, binary_sha256=summary.BINARY_SHA,
            base_sha256=summary.BASE_SHA, source_dirty=False,
            worker_sha256=digest(self.batch / 'run_long_recurrence.py'),
            wall_seconds_per_job=21600, total_active_wall_seconds=172800,
            bounded_cleanup_seconds_per_job=20, minimum_free_disk_bytes=20 * 1024**3,
            minimum_available_ram_bytes=3 * 1024**3, runner_rss_limit_bytes=2 * 1024**3,
            checkpoint_byte_limit=256 * 1024**2, jobs=self.records)
        self.save_campaign()
        (self.batch / 'status').write_text('completed\n')
        (self.batch / 'exit_code').write_text('0\n')
        write_json(self.batch / 'worker.json', dict(state='completed', exit_code=0,
            current_job=self.records[-1]['name'], completed_jobs=[r['name'] for r in self.records]))

    def save_campaign(self):
        write_json(self.batch / 'campaign.json', self.campaign)

    def child(self, index):
        return self.batch / self.records[index]['name']

    def session(self, index):
        return self.child(index) / 'runs' / 'session_fixture'

    def csv_path(self, index):
        return self.session(index) / f"stats_seed{self.records[index]['job']['seed']}.csv"

    def config_text(self, seed, rate):
        text = re.sub(r'^seed = .*$', f'seed = {seed}', self.base, count=1, flags=re.M)
        return re.sub(r'^add_recurrent_probability = .*$',
                      f'add_recurrent_probability = {rate}', text, flags=re.M)

    def write_run(self, index, native='completed', endpoint=500000):
        record = self.records[index]
        child, job = self.child(index), record['job']
        seed, rate = job['seed'], job['recurrence_probability']
        session = self.session(index)
        session.mkdir(parents=True, exist_ok=True)
        config_text = self.config_text(seed, rate)
        config = tomllib.loads(config_text)
        (session / f'config_seed{seed}.toml').write_text(config_text)
        prefix = dict(schema_version=2, source_revision=summary.ENGINE,
            source_dirty='false', target='aarch64-unknown-linux-gnu', build_profile='release',
            compiler='synthetic fixture', core_build_id=summary.CORE_BUILD,
            starting_tick=0, simulated_ticks=endpoint, checkpoint_every=50000,
            export_genomes_every=500000, seed=seed, requested_ticks=500000,
            completed_ticks=endpoint, log_interval=1000, status=native,
            survived=native != 'extinct', elapsed_seconds=1.0,
            error='synthetic technical failure' if native == 'failed' else '')
        lines = [f'{key} = {json.dumps(value)}' for key, value in prefix.items()]
        for line in config_text.splitlines():
            if line.startswith('[['):
                line = '[[config.' + line[2:]
            elif line.startswith('['):
                line = '[config.' + line[1:]
            lines.append(line)
        (session / f'manifest_seed{seed}.toml').write_text('\n'.join(lines) + '\n')
        ticks = list(range(1000, endpoint + 1, 1000))
        if not ticks or ticks[-1] != endpoint:
            ticks.append(endpoint)
        rows = []
        for tick in ticks:
            extinct = native == 'extinct' and tick == endpoint
            row = {key: 0 for key in summary.METRICS}
            row.update(tick=tick, population=0 if extinct else 2,
                mean_connections=0 if extinct else 2,
                mean_recurrent_connections=0 if extinct or rate == 0 else 1,
                recurrent_brain_fraction=0 if extinct or rate == 0 else 1,
                mean_brain_activity=3,
                births=0 if tick % 1000 else 1,
                deaths=0 if tick % 1000 else 1, cap_blocked_births=0,
                total_births=tick // 1000, total_deaths=tick // 1000,
                total_cap_blocked_births=0)
            rows.append(row)
        self.write_csv(index, rows)
        exported_config = copy.deepcopy(config)
        exported_config['food']['droughts'].setdefault('schedule_seed', None)
        exported_config['epigenetics'].setdefault('stress_threshold', .3)
        records = [] if native == 'extinct' else [dict(id=i, expressed_brain=dict(
            connections=[{}] * (2 if rate == 0 else 1), recurrent=[] if rate == 0 else [{}]))
            for i in (1, 2)]
        write_json(session / f'genomes_seed{seed}_tick{endpoint}.json',
            dict(format='terrarium-genomes', version=1, config=exported_config,
                 tick=endpoint, core_build=summary.CORE_BUILD,
                 target='aarch64-unknown-linux-gnu', genomes=records))
        (child / 'status').write_text('completed\n')
        (child / 'exit_code').write_text('0\n')
        write_json(child / 'supervisor.json', dict(state='completed', runner_exit_code=0,
            guard_exit_code=0, supervisor_exit_code=0, runner_left_paused=False))
        write_json(child / 'supervisor_manifest.json', dict(task='ecology-long',
            sha256={k: v for k, v in record['sha256'].items() if k != 'run_guarded_acquisition.py'},
            runner_rss_limit_bytes=2 * 1024**3,
            temperature_high_c=75, temperature_low_c=65, guard_poll_seconds=5,
            wall_seconds=21600, wall_includes_startup_and_cooling=True,
            environment_override={'RAYON_NUM_THREADS': '1'}))
        write_json(child / 'thermal_guard/status.json',
            dict(state='finished', running_matches=0, paused=[]))
        (child / 'guard.log').write_text('state=monitoring temp=55.0 matches=0\n')
        (child / 'thermal_supervisor.jsonl').write_text(json.dumps(dict(
            temperature_c=55.0, throttled_output='throttled=0xe0000')) + '\n')

    def read_csv(self, index):
        with self.csv_path(index).open() as stream:
            return list(csv.DictReader(stream))

    def write_csv(self, index, rows):
        with self.csv_path(index).open('w') as stream:
            writer = csv.DictWriter(stream, fieldnames=rows[0].keys())
            writer.writeheader()
            writer.writerows(rows)

    def test_complete_fixed_windows_and_four_pairs(self):
        result = summary.summarize(self.batch)
        self.assertTrue(result['complete'])
        self.assertEqual(len(result['runs']), 8)
        self.assertEqual(result['paired_summary']['available_pairs'], 4)
        for run in result['runs']:
            self.assertEqual(run['csv_rows'], 500)
            self.assertEqual(run['early']['samples'], 51)
            self.assertEqual(run['late']['samples'], 126)
            self.assertEqual(run['late']['mean']['population'], 2)
            self.assertEqual(run['early']['event_counts']['births'], 51)
            self.assertEqual(run['late']['event_counts']['births'], 126)
            self.assertEqual(run['late']['population_sd'], 0)
            self.assertEqual(run['late']['near_cap_fraction'], 0)
            self.assertEqual(run['late_minus_early']['mean_brain_activity'], 0)
        for pair in result['paired']:
            self.assertEqual(pair['late_difference_on_minus_off']['mean_recurrent_connections'], 1)

    def test_late_cap_events_include_start_tick_without_recounting_earlier_events(self):
        rows = self.read_csv(0)
        previous = 0
        for row in rows:
            tick = int(row['tick'])
            counter = 10 if tick == 500000 else 3 if tick >= 375000 else 1 if tick >= 374000 else 0
            row['total_cap_blocked_births'] = counter
            row['cap_blocked_births'] = counter - previous
            previous = counter
        self.write_csv(0, rows)
        result = summary.summarize(self.batch)
        self.assertEqual(result['runs'][0]['late']['event_counts']['cap_blocked_births'], 9)

    def test_bad_csv_finite_sampling_order_and_off_recurrence_rejected(self):
        original = self.read_csv(0)
        for fault in ('missing', 'duplicate', 'unsorted', 'nan', 'infinity', 'off_recurrence'):
            with self.subTest(fault=fault):
                rows = copy.deepcopy(original)
                if fault == 'missing':
                    rows.pop(20)
                elif fault == 'duplicate':
                    rows[20]['tick'] = rows[19]['tick']
                elif fault == 'unsorted':
                    rows[20], rows[21] = rows[21], rows[20]
                elif fault == 'nan':
                    rows[20]['population'] = 'nan'
                elif fault == 'infinity':
                    rows[20]['mean_brain_activity'] = 'inf'
                else:
                    rows[20]['mean_recurrent_connections'] = '1'
                self.write_csv(0, rows)
                with self.assertRaises(ValueError):
                    summary.summarize(self.batch)
        self.write_csv(0, original)

    def test_full_config_pin_and_manifest_source_corruption_rejected(self):
        child = self.child(0)
        for path, replacement in (
            (child / 'base.toml', self.base.replace('connection_cost = 0.001', 'connection_cost = 0.002')),
            (self.session(0) / 'config_seed4000.toml',
             self.config_text(4000, 0).replace('width = 1000.0', 'width = 1001.0')),
            (self.session(0) / 'manifest_seed4000.toml',
             (self.session(0) / 'manifest_seed4000.toml').read_text().replace(summary.ENGINE, 'wrong-source'))):
            with self.subTest(path=str(path)):
                original = path.read_text()
                self.assertNotEqual(original, replacement)
                path.write_text(replacement)
                with self.assertRaises(ValueError):
                    summary.summarize(self.batch)
                path.write_text(original)

    def test_export_count_duplicate_id_and_edge_crosscheck_rejected(self):
        path = self.session(0) / 'genomes_seed4000_tick500000.json'
        original = json.loads(path.read_text())
        for fault in ('population', 'duplicate', 'edges', 'config'):
            with self.subTest(fault=fault):
                value = copy.deepcopy(original)
                if fault == 'population':
                    value['genomes'].pop()
                elif fault == 'duplicate':
                    value['genomes'][1]['id'] = value['genomes'][0]['id']
                elif fault == 'edges':
                    value['genomes'][0]['expressed_brain']['connections'].append({})
                else:
                    value['config']['energy']['connection_cost'] = 123
                write_json(path, value)
                with self.assertRaises(ValueError):
                    summary.summarize(self.batch)
        write_json(path, original)

    def test_extinction_retained_zero_population_not_zero_living_neural_mean(self):
        self.write_run(0, native='extinct', endpoint=120123)
        result = summary.summarize(self.batch)
        run = result['runs'][0]
        self.assertEqual(run['status'], 'extinct')
        self.assertEqual(run['observed_tick'], 120123)
        self.assertIsNone(run['late'])
        self.assertEqual(run['early']['samples'], 51)
        self.assertEqual(run['final']['population'], 0)
        self.assertIsNone(run['final']['mean_recurrent_connections'])
        self.assertEqual(run['final']['mean_brain_activity'], 3)
        self.assertEqual(result['paired_summary']['available_pairs'], 3)
        self.assertIsNone(result['paired'][0]['late_difference_on_minus_off'])
        self.assertTrue(result['paired'][0]['missing_reason'])

    def test_partial_failed_and_unstarted_slots_retained_not_pooled(self):
        self.run_count = 2
        self.setUp()  # A second fresh synthetic directory; prior artifacts preserved.
        self.write_run(2, native='failed', endpoint=120000)
        child = self.child(2)
        (child / 'status').write_text('failed\n')
        (child / 'exit_code').write_text('124\n')
        write_json(child / 'supervisor.json', dict(state='technical_timeout',
            runner_exit_code=-9, guard_exit_code=0, supervisor_exit_code=124,
            runner_left_paused=False))
        (self.batch / 'status').write_text('failed\n')
        (self.batch / 'exit_code').write_text('1\n')
        write_json(self.batch / 'worker.json', dict(state='failed', exit_code=1,
            current_job=self.records[2]['name'], completed_jobs=[r['name'] for r in self.records[:2]]))
        with self.assertRaises(ValueError):
            summary.summarize(self.batch)
        result = summary.summarize(self.batch, allow_incomplete=True)
        self.assertFalse(result['complete'])
        self.assertEqual(len(result['runs']), 8)
        self.assertEqual(result['coverage']['technical_failed'], 1)
        self.assertEqual(result['coverage']['not_started'], 5)
        self.assertEqual(result['paired_summary']['available_pairs'], 1)
        self.assertEqual(result['runs'][2]['early']['samples'], 51)
        self.assertIsNone(result['runs'][2]['late'])
        self.assertTrue(all(r['late'] is None for r in result['runs'][2:]))

    def test_active_buffered_prefix_tail_is_explicitly_ignored_only_when_mutable(self):
        self.run_count = 2
        self.setUp()
        self.write_run(2, native='running', endpoint=120000)
        child = self.child(2)
        (child / 'status').write_text('running\n')
        write_json(child / 'supervisor.json', dict(state='running', runner_exit_code=None,
            guard_exit_code=None, supervisor_exit_code=None, runner_left_paused=False))
        manifest = self.session(2) / 'manifest_seed4001.toml'
        manifest.write_text(manifest.read_text().replace('completed_ticks = 120000', 'completed_ticks = 0')
                            .replace('simulated_ticks = 120000', 'simulated_ticks = 0'))
        with self.csv_path(2).open('a') as stream:
            stream.write('121000,2')
        (self.batch / 'status').write_text('running\n')
        write_json(self.batch / 'worker.json', dict(state='running',
            current_job=self.records[2]['name'], completed_jobs=[r['name'] for r in self.records[:2]]))
        result = summary.summarize(self.batch, allow_incomplete=True)
        self.assertEqual(result['coverage']['active'], 1)
        self.assertEqual(result['coverage']['not_started'], 5)
        self.assertEqual(result['runs'][2]['observed_tick'], 120000)
        self.assertTrue(result['runs'][2]['truncated_csv_tail_ignored'])
        self.assertIsNone(result['runs'][2]['late'])
        self.assertEqual(result['paired_summary']['available_pairs'], 1)

    def test_terminal_truncated_csv_and_nonfinite_json_rejected(self):
        path = self.csv_path(0)
        original = path.read_text()
        path.write_text(original + '501000,2')
        with self.assertRaisesRegex(ValueError, 'truncated'):
            summary.summarize(self.batch)
        path.write_text(original)
        export = self.session(0) / 'genomes_seed4000_tick500000.json'
        value = json.loads(export.read_text())
        value['unused_but_invalid_number'] = float('nan')
        export.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, 'nonfinite'):
            summary.summarize(self.batch)

    def test_job_order_or_guard_failure_cannot_claim_completed_campaign(self):
        original = self.campaign['jobs'][:]
        self.campaign['jobs'][0], self.campaign['jobs'][1] = self.campaign['jobs'][1], self.campaign['jobs'][0]
        self.save_campaign()
        with self.assertRaises(ValueError):
            summary.summarize(self.batch)
        self.campaign['jobs'] = original
        self.save_campaign()
        write_json(self.child(0) / 'thermal_guard/status.json',
                   dict(state='stopped_safely_paused', running_matches=1, paused=[{'pid': 999}]))
        with self.assertRaisesRegex(ValueError, 'guard'):
            summary.summarize(self.batch)

    def test_negative_activity_costs_and_impossible_counter_delta_rejected(self):
        original = self.read_csv(0)
        for key in ('mean_brain_activity', 'energy_brain_structure_cost', 'energy_brain_activity_cost'):
            with self.subTest(metric=key):
                rows = copy.deepcopy(original)
                rows[20][key] = '-0.01'
                self.write_csv(0, rows)
                with self.assertRaisesRegex(ValueError, 'negative'):
                    summary.summarize(self.batch)
        rows = copy.deepcopy(original)
        rows[20]['total_births'] = rows[19]['total_births']
        self.write_csv(0, rows)
        with self.assertRaisesRegex(ValueError, 'increment'):
            summary.summarize(self.batch)

    def test_late_minus_early_reports_actual_window_difference(self):
        rows = self.read_csv(0)
        for row in rows:
            if int(row['tick']) >= 375000:
                row['mean_brain_activity'] = '5'
        self.write_csv(0, rows)
        result = summary.summarize(self.batch)
        self.assertEqual(result['runs'][0]['late_minus_early']['mean_brain_activity'], 2)

    def test_parallel_active_or_skipped_queue_slots_rejected(self):
        for indices in ((0, 1), (1,)):
            with self.subTest(started=indices):
                self.run_count = 0
                self.setUp()
                for index in indices:
                    self.write_run(index, native='running', endpoint=120000)
                    (self.child(index) / 'status').write_text('running\n')
                    write_json(self.child(index) / 'supervisor.json', dict(state='running',
                        runner_exit_code=None, guard_exit_code=None, supervisor_exit_code=None,
                        runner_left_paused=False))
                (self.batch / 'status').write_text('running\n')
                write_json(self.batch / 'worker.json', dict(state='running',
                    current_job=self.records[indices[-1]]['name'], completed_jobs=[]))
                with self.assertRaisesRegex(ValueError, 'parallel|skipped'):
                    summary.summarize(self.batch, allow_incomplete=True)

    def test_report_hash_identifies_exact_csv_bytes_consumed_before_file_changes(self):
        path = self.csv_path(0)
        consumed = path.read_bytes()
        original_read = Path.read_bytes
        changed = False

        def changing_read(candidate):
            nonlocal changed
            data = original_read(candidate)
            if candidate == path and not changed:
                changed = True
                # Model a live file update immediately after one atomic read.
                # The archived digest must describe that read, not later bytes.
                candidate.write_bytes(data + b'\n')
            return data

        with patch.object(Path, 'read_bytes', changing_read):
            result = summary.summarize(self.batch)
        relative = str(path.relative_to(self.batch))
        self.assertTrue(changed)
        self.assertEqual(result['artifact_sha256'][relative], hashlib.sha256(consumed).hexdigest())
        self.assertNotEqual(result['artifact_sha256'][relative], digest(path))
        self.assertEqual(result['runs'][0]['csv_rows'], 500)


if __name__ == '__main__':
    unittest.main()
