"""Harmless process fixtures for the fixed long-recurrence queue; no Pi runs."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import unittest
from unittest.mock import patch
import uuid

import test_guarded_acquisition as fixtures
import run_long_recurrence as queue


FAKE_SUPERVISOR = '''import argparse,json,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--task');p.add_argument('--batch',type=Path);p.add_argument('--wall-seconds')
a=p.parse_args();b=a.batch
(b/'fixture.started').write_text('started')
time.sleep(.02)
code=FAILURE
(b/'thermal_guard').mkdir()
(b/'thermal_guard/status.json').write_text(json.dumps(dict(state='finished',paused=[],running_matches=0)))
(b/'supervisor.json').write_text(json.dumps(dict(state='completed' if code==0 else 'failed',runner_exit_code=code,guard_exit_code=0)))
raise SystemExit(code)
'''


class QueueTests(unittest.TestCase):
    def setUp(self):
        self.batch = fixtures.ARTIFACTS / ('queue_' + uuid.uuid4().hex)
        self.batch.mkdir(parents=True)
        (self.batch / 'status').write_text('prepared\n')
        (self.batch / 'run_long_recurrence.py').write_text('# inert pinned fixture\n')
        records = []
        for index, (seed, rate) in enumerate(queue.JOBS):
            name = f'job_{index}'
            child = self.batch / name
            child.mkdir()
            (child / 'run_guarded_acquisition.py').write_text(
                FAKE_SUPERVISOR.replace('FAILURE', '0'))
            records.append(dict(name=name, job=dict(seed=seed, recurrence_probability=rate),
                                sha256={'run_guarded_acquisition.py': queue.sha(child / 'run_guarded_acquisition.py')}))
        self.manifest = dict(engine_revision=queue.ENGINE, binary_sha256=queue.BINARY_SHA,
                             wall_seconds_per_job=queue.WALL_SECONDS, jobs=records,
                             worker_sha256=queue.sha(self.batch / 'run_long_recurrence.py'))
        self.save_manifest()

    def save_manifest(self):
        (self.batch / 'campaign.json').write_text(json.dumps(self.manifest))

    def run_queue(self, resources=None, runners=None):
        with patch.object(queue, 'require_resources', side_effect=resources), \
                patch.object(queue, 'active_runners', return_value=runners or []):
            return queue.run_campaign(self.batch)

    def test_all_eight_jobs_serial_and_freshness_lock(self):
        self.assertEqual(self.run_queue(), 0)
        state = json.loads((self.batch / 'worker.json').read_text())
        self.assertEqual(state['completed_jobs'], [r['name'] for r in self.manifest['jobs']])
        self.assertEqual(state['state'], 'completed')
        before = (self.batch / 'worker.json').read_bytes()
        with self.assertRaises(FileExistsError):
            self.run_queue()
        self.assertEqual((self.batch / 'worker.json').read_bytes(), before)

    def test_failed_job_stops_queue_and_retains_partial(self):
        first = self.manifest['jobs'][0]
        path = self.batch / first['name'] / 'run_guarded_acquisition.py'
        path.write_text(FAKE_SUPERVISOR.replace('FAILURE', '17'))
        first['sha256'][path.name] = queue.sha(path)
        self.save_manifest()
        self.assertEqual(self.run_queue(), 1)
        self.assertTrue((path.parent / 'fixture.started').exists())
        self.assertFalse((self.batch / self.manifest['jobs'][1]['name'] / 'fixture.started').exists())
        self.assertEqual(json.loads((self.batch / 'worker.json').read_text())['completed_jobs'], [])

    def test_changed_input_rejected_before_any_child(self):
        path = self.batch / self.manifest['jobs'][-1]['name'] / 'run_guarded_acquisition.py'
        path.write_text('# changed\n')
        with self.assertRaisesRegex(ValueError, 'input changed'):
            self.run_queue()
        self.assertFalse(list(self.batch.glob('*/fixture.started')))

    def test_post_spawn_persistence_failure_terminates_and_waits_owned_supervisor(self):
        owned = []
        original_popen, original_write = subprocess.Popen, queue.atomic_json
        failed = False

        def spawn(*args, **kwargs):
            process = original_popen(*args, **kwargs)
            owned.append(process)
            return process

        def write(path, value):
            nonlocal failed
            if value.get('supervisor_pid') is not None and not failed:
                failed = True
                raise OSError('fixture persistence failure after Popen')
            return original_write(path, value)

        try:
            with patch.object(queue.subprocess, 'Popen', side_effect=spawn), \
                    patch.object(queue, 'atomic_json', side_effect=write):
                self.assertEqual(self.run_queue(), 1)
            self.assertTrue(failed)
            self.assertEqual(len(owned), 1)
            self.assertEqual(owned[0].returncode, -signal.SIGTERM)
            self.assertFalse((self.batch / self.manifest['jobs'][1]['name'] / 'launcher.log').exists())
            state = json.loads((self.batch / 'worker.json').read_text())
            self.assertEqual(state['state'], 'failed')
            self.assertIn('fixture persistence failure', state['reason'])
        finally:
            # Test-failure cleanup is restricted to our own direct child handle.
            for process in owned:
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=2)

    def test_resource_failure_and_existing_runner_start_nothing(self):
        self.assertEqual(self.run_queue(resources=ValueError('low disk')), 1)
        self.assertFalse(list(self.batch.glob('*/fixture.started')))

    def test_unrelated_runner_blocks_queue_without_signaling_it(self):
        sentinel = subprocess.Popen([sys.executable, '-c', 'import time;time.sleep(10)'])
        try:
            self.assertEqual(self.run_queue(runners=[sentinel.pid]), 1)
            self.assertIsNone(sentinel.poll())
            self.assertFalse(list(self.batch.glob('*/fixture.started')))
        finally:
            sentinel.terminate()
            sentinel.wait(timeout=2)


@unittest.skipUnless(os.name == 'posix', 'requires POSIX process signals')
class LongSupervisorTests(unittest.TestCase):
    setUp = fixtures.SupervisorTests.setUp
    tearDown = fixtures.SupervisorTests.tearDown
    state = fixtures.SupervisorTests.state
    sentinel = fixtures.SupervisorTests.sentinel

    def prepare(self, guard='success', runner='success'):
        fixtures.SupervisorTests.prepare(self, guard, runner)
        executable = self.batch / 'sim-runner'
        executable.write_text(executable.read_text().replace(
            'while True: time.sleep(.05)',
            "while True: Path('runner.heartbeat').write_text(str(time.monotonic())); time.sleep(.02)"))
        (self.batch / 'base.toml').write_text('# inert fixture, never simulated\n')
        self.job = dict(seed=4000, recurrence_probability=.3, ticks=500000,
                        log_interval=1000, checkpoint_every=50000,
                        export_genomes_every=500000)
        self.save_job()

    def save_job(self):
        (self.batch / 'job.json').write_text(json.dumps(self.job))

    def command(self, wall=4, stale=.5):
        command = fixtures.SupervisorTests.command(self, wall=wall, stale=stale)
        if not sys.platform.startswith('linux') or hasattr(self, 'fixture_rss_kib'):
            # Emulate only Linux's VmRSS input on other POSIX hosts. All child
            # processes, heartbeats, STOP/TERM/KILL and watchdogs remain real.
            wrapper = '''import sys
from pathlib import Path
sys.path.insert(0, sys.argv.pop(1))
import run_guarded_acquisition as worker
original=Path.read_text
def read(path,*args,**kwargs):
 if str(path).startswith('/proc/') and str(path).endswith('/status'):
  return 'VmRSS: RSS_VALUE kB\\n'
 return original(path,*args,**kwargs)
Path.read_text=read
raise SystemExit(worker.main())
'''.replace('RSS_VALUE', str(getattr(self, 'fixture_rss_kib', 1024)))
            command = [sys.executable, '-c', wrapper, str(fixtures.SUPERVISOR.parent)] + command[2:]
        return command + ['--task', 'ecology-long']

    def run_supervisor(self, **kwargs):
        return subprocess.run(self.command(**kwargs), capture_output=True, text=True, timeout=8)

    def test_exact_job_command_success_and_resource_metadata(self):
        self.prepare()
        result = self.run_supervisor()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.state()['state'], 'completed')
        manifest = json.loads((self.batch / 'supervisor_manifest.json').read_text())
        self.assertEqual(manifest['runner_command'], [str(self.batch / 'sim-runner'),
            '--config', str(self.batch / 'base.toml'), '--seed', '4000', '--set',
            'mutation.add_recurrent_probability=0.3', '--ticks', '500000',
            '--log-interval', '1000', '--checkpoint-every', '50000',
            '--export-genomes-every', '500000', '--output', str(self.batch / 'runs')])
        self.assertEqual(manifest['runner_rss_limit_bytes'], 2 * 1024**3)
        self.assertTrue(manifest['wall_includes_startup_and_cooling'])
        self.assertEqual(manifest['environment_override'], {'RAYON_NUM_THREADS': '1'})
        self.assertEqual(set(manifest['sha256']),
                         {'sim-runner', 'pi_thermal_guard.py', 'base.toml', 'job.json'})

    def test_out_of_protocol_job_rejected_before_artifacts(self):
        self.prepare()
        for key, value in [('seed', 4004), ('recurrence_probability', .1),
                           ('ticks', 500001), ('extra', True)]:
            original = self.job.copy()
            self.job[key] = value
            self.save_job()
            result = self.run_supervisor()
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((self.batch / 'runner.started').exists())
            self.assertFalse((self.batch / 'supervisor_manifest.json').exists())
            self.job = original

    def test_guard_loss_stops_only_owned_runner(self):
        self.prepare('guard_failure', 'long')
        sentinel = self.sentinel()
        result = self.run_supervisor()
        self.assertNotEqual(result.returncode, 0)
        state = self.state()
        self.assertTrue(state['runner_left_paused'])
        self.assertIn('guard exited', state['reason'])
        self.assertIsNone(sentinel.poll())
        if sys.platform.startswith('linux'):
            stat = Path(f"/proc/{state['runner_pid']}/stat").read_text()
            self.assertEqual(stat.rsplit(')', 1)[1].split()[0], 'T')
        else:
            os.kill(state['runner_pid'], 0)
            heartbeat = self.batch / 'runner.heartbeat'
            before = heartbeat.read_bytes()
            time.sleep(.08)
            self.assertEqual(heartbeat.read_bytes(), before)

    def test_hot_stopped_runner_wall_timeout_without_resume(self):
        self.prepare('hot_after_start', 'long')
        result = self.run_supervisor(wall=.6)
        self.assertEqual(result.returncode, 124, result.stderr)
        state = self.state()
        self.assertEqual(state['state'], 'technical_timeout')
        self.assertEqual(state['runner_exit_code'], -signal.SIGKILL)
        self.assertFalse(state['runner_left_paused'])

    def test_guard_startup_failure_starts_no_runner(self):
        self.prepare('startup_failure', 'long')
        self.assertNotEqual(self.run_supervisor().returncode, 0)
        self.assertFalse((self.batch / 'runner.started').exists())
        self.assertIsNone(self.state()['runner_pid'])

    def test_stalled_guard_heartbeat_pauses_runner(self):
        self.prepare('stale', 'long')
        self.assertNotEqual(self.run_supervisor(stale=.15).returncode, 0)
        self.assertTrue(self.state()['runner_left_paused'])
        self.assertIn('stale', self.state()['reason'])

    def test_sampled_rss_limit_is_technical_failure(self):
        self.prepare('success', 'long')
        self.fixture_rss_kib = 3 * 1024**2
        result = self.run_supervisor()
        self.assertEqual(result.returncode, 125, result.stderr)
        self.assertEqual(self.state()['state'], 'technical_resource_limit')
        self.assertIn('RSS exceeded', self.state()['reason'])


if __name__ == '__main__':
    unittest.main()
