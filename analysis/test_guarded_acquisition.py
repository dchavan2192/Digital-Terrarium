"""Harmless direct-child supervisor fixtures; no Pi or simulation is used."""
import json
import hashlib
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import unittest
import uuid

SUPERVISOR = Path(__file__).with_name('run_guarded_acquisition.py').resolve()
ARTIFACTS = Path(__file__).resolve().parents[1] / 'output' / 'guarded-acquisition-tests'

GUARD = '''import argparse,json,os,time,signal
from pathlib import Path
p=argparse.ArgumentParser()
p.add_argument('--batch',type=Path);p.add_argument('--output',type=Path)
p.add_argument('--high',type=float);p.add_argument('--low',type=float)
p.add_argument('--poll',type=float);p.add_argument('--sensor')
a=p.parse_args()
mode=MODE
if mode=='startup_failure': raise SystemExit(19)
start=time.monotonic()
written=False
hot=False
while True:
 if (a.batch/'status').read_text().strip() in ('completed','failed'): break
 if mode=='guard_failure' and time.monotonic()-start>.20: raise SystemExit(19)
 if mode=='hot_after_start' and not hot and (a.batch/'runner.started').exists():
  os.kill(int((a.batch/'runner.started').read_text()),signal.SIGSTOP);hot=True
 if mode!='stale' or not written:
  data=dict(state='monitoring',updated_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),guard_pid=os.getpid(),batch=str(a.batch),executable=str(a.batch/'sim-runner'),high_c=a.high,low_c=a.low,poll_seconds=a.poll,temperature_c=55.0)
  if mode=='wrong_identity': data['executable']='/unrelated/sim-runner'
  if hot: data.update(state='paused_hot',temperature_c=80.0)
  tmp=a.output/'status.pending'
  tmp.write_text(json.dumps(data));tmp.replace(a.output/'status.json');written=True
 time.sleep(.02)
'''

RUNNER = '''#!/usr/bin/env python3
import os,signal,time
from pathlib import Path
# Publish the PID atomically: existence must never expose an empty file.
Path('runner.started.pending').write_text(str(os.getpid()))
Path('runner.started.pending').replace('runner.started')
if MODE=='success': time.sleep(.10)
else:
 signal.signal(signal.SIGTERM,signal.SIG_IGN)
 while True: time.sleep(.05)
'''


@unittest.skipUnless(os.name == 'posix', 'requires POSIX process stop/kill semantics')
class SupervisorTests(unittest.TestCase):
    def setUp(self):
        self.batch = ARTIFACTS / uuid.uuid4().hex
        self.batch.mkdir(parents=True)
        self.sensor = self.batch / 'temperature'
        self.sensor.write_text('55000')
        self.owned_children = []

    def tearDown(self):
        # Only fixture-owned subprocess handles and the fixture's self-written
        # runner PID are eligible. Never discover/kill processes by name.
        for child in self.owned_children:
            if child.poll() is None:
                child.kill()
            child.wait(timeout=3)
        path = self.batch / 'runner.started'
        if path.exists():
            pid = int(path.read_text())
            # Failure fixtures deliberately leave this known runner stopped.
            # The known fixture process remains alive and stopped until this kill.
            state = self.state() if (self.batch / 'supervisor.json').exists() else {}
            if state.get('runner_left_paused'):
                try:
                    os.kill(pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass

    def prepare(self, guard='success', runner='success'):
        (self.batch / 'pi_thermal_guard.py').write_text(GUARD.replace('MODE', repr(guard)))
        executable = self.batch / 'sim-runner'
        executable.write_text(RUNNER.replace('MODE', repr(runner)))
        executable.chmod(0o755)
        for name in ('visible_acquisition.toml', 'delayed_choice.toml'):
            (self.batch / name).write_text('# inert fixture, not a science config\n')

    def command(self, wall=4, stale=.5):
        return [sys.executable, str(SUPERVISOR), '--batch', str(self.batch),
                '--sensor', str(self.sensor), '--wall-seconds', str(wall),
                '--guard-stale-seconds', str(stale), '--startup-seconds', '1',
                '--poll-seconds', '.02', '--terminate-grace-seconds', '.15']

    def run_supervisor(self, **kwargs):
        return subprocess.run(self.command(**kwargs), capture_output=True, text=True, timeout=6)

    def state(self):
        return json.loads((self.batch / 'supervisor.json').read_text())

    def sentinel(self):
        child = subprocess.Popen([sys.executable, '-c', 'import time;time.sleep(30)'])
        self.owned_children.append(child)
        return child

    def test_success_exact_command_metadata_telemetry_and_fresh_batch(self):
        self.prepare()
        result = self.run_supervisor()
        self.assertEqual(result.returncode, 0, result.stderr)
        state = self.state()
        self.assertEqual(state['state'], 'completed')
        self.assertEqual(state['runner_exit_code'], 0)
        self.assertEqual(state['guard_exit_code'], 0)
        self.assertFalse(state['runner_left_paused'])
        manifest = json.loads((self.batch / 'supervisor_manifest.json').read_text())
        self.assertEqual(manifest['environment_override'], {'RAYON_NUM_THREADS': '1'})
        self.assertEqual(manifest['runner_command'][1], 'acquire-visible')
        self.assertEqual(len(manifest['sha256']), 4)
        thermal = json.loads((self.batch / 'thermal_supervisor.jsonl').read_text().splitlines()[0])
        self.assertEqual(thermal['temperature_c'], 55.0)
        self.assertIn('throttled_output', thermal)
        old = (self.batch / 'supervisor_manifest.json').read_bytes()
        retry = self.run_supervisor()
        self.assertNotEqual(retry.returncode, 0)
        self.assertEqual((self.batch / 'supervisor_manifest.json').read_bytes(), old)

    def test_validation_command_hashes_all_inputs_and_preserves_sources(self):
        self.prepare()
        source = self.batch / 'input'
        source.mkdir()
        for name in ('manifest.json', 'summary.json', 'extra.json'):
            (source / name).write_text('{"fixture":true}\n')
        suite = self.batch / 'visible_validation.toml'
        suite.write_text('# inert fixture suite\n')
        before = {p.name: p.read_bytes() for p in source.iterdir()}
        result = subprocess.run(self.command() + ['--task', 'validate-visible'],
                                capture_output=True, text=True, timeout=6)
        self.assertEqual(result.returncode, 0, result.stderr)
        manifest = json.loads((self.batch / 'supervisor_manifest.json').read_text())
        self.assertEqual(manifest['task'], 'validate-visible')
        self.assertEqual(manifest['runner_command'], [str(self.batch / 'sim-runner'),
                         'validate-visible', '--session', str(source), '--suite', str(suite),
                         '--output', str(self.batch / 'runs')])
        self.assertEqual(len(manifest['sha256']), 6)
        for name, data in before.items():
            self.assertEqual((source / name).read_bytes(), data)
            self.assertEqual(manifest['sha256']['input/' + name], hashlib.sha256(data).hexdigest())

    def test_validation_missing_or_linked_input_is_rejected_before_artifacts(self):
        self.prepare()
        (self.batch / 'visible_validation.toml').write_text('# inert\n')
        command = self.command() + ['--task', 'validate-visible']
        missing = subprocess.run(command, capture_output=True, timeout=3)
        self.assertNotEqual(missing.returncode, 0)
        source = self.batch / 'input'
        source.mkdir()
        (source / 'manifest.json').write_text('{}')
        (source / 'summary.json').write_text('{}')
        (source / 'linked').symlink_to(self.sensor)
        linked = subprocess.run(command, capture_output=True, timeout=3)
        self.assertNotEqual(linked.returncode, 0)
        self.assertFalse((self.batch / 'supervisor_manifest.json').exists())
        self.assertFalse((self.batch / 'runner.started').exists())

    def test_guard_failure_stops_only_owned_runner_and_preserves_it_paused(self):
        self.prepare(guard='guard_failure', runner='long')
        sentinel = self.sentinel()
        result = self.run_supervisor()
        self.assertNotEqual(result.returncode, 0)
        state = self.state()
        self.assertEqual(state['state'], 'failed')
        self.assertTrue(state['runner_left_paused'])
        self.assertIn('guard exited', state['reason'])
        self.assertIsNone(sentinel.poll())
        if sys.platform.startswith('linux'):
            text = Path(f"/proc/{state['runner_pid']}/stat").read_text()
            self.assertEqual(text.rsplit(')', 1)[1].split()[0], 'T')

    def test_stale_guard_also_stops_owned_runner(self):
        self.prepare(guard='stale', runner='long')
        result = self.run_supervisor(stale=.15)
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(self.state()['runner_left_paused'])
        self.assertIn('stale', self.state()['reason'])

    def test_watchdog_terminates_without_resuming_even_if_term_is_ignored(self):
        self.prepare(guard='hot_after_start', runner='long')
        sentinel = self.sentinel()
        result = self.run_supervisor(wall=.35)
        self.assertEqual(result.returncode, 124, result.stderr)
        self.assertEqual(self.state()['state'], 'technical_timeout')
        self.assertEqual(self.state()['runner_exit_code'], -signal.SIGKILL)
        self.assertFalse(self.state()['runner_left_paused'])
        self.assertIsNone(sentinel.poll())

    def test_startup_failure_or_wrong_identity_never_starts_runner(self):
        self.prepare(guard='startup_failure', runner='long')
        result = self.run_supervisor()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.batch / 'runner.started').exists())
        self.assertIsNone(self.state()['runner_pid'])

    def test_wrong_identity_is_rejected_before_runner(self):
        self.prepare(guard='wrong_identity', runner='long')
        result = self.run_supervisor()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.batch / 'runner.started').exists())
        self.assertIn('identity', self.state()['reason'])

    def test_interruption_leaves_owned_runner_paused(self):
        self.prepare(runner='long')
        supervisor = subprocess.Popen(self.command())
        self.owned_children.append(supervisor)
        deadline = time.monotonic() + 3
        while not (self.batch / 'runner.started').exists() and time.monotonic() < deadline:
            time.sleep(.02)
        self.assertTrue((self.batch / 'runner.started').exists())
        supervisor.terminate()
        self.assertEqual(supervisor.wait(timeout=3), 130)
        self.assertTrue(self.state()['runner_left_paused'])


if __name__ == '__main__':
    unittest.main()
