"""Thermal policy and process identity tests; never signal a real process."""
import importlib.util
from pathlib import Path
import signal
import sys
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('pi_thermal_guard', Path(__file__).with_name('pi_thermal_guard.py'))
guard = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = guard
SPEC.loader.exec_module(guard)


class GuardTests(unittest.TestCase):
    def setUp(self):
        self.sent = []
        self.p = guard.Process(123, '1000')
        self.control = guard.Controller(75, 65, lambda p, s: self.sent.append((p, s)) or True)

    def test_hysteresis_and_new_arrivals(self):
        self.control.update({self.p: 'R'}, 75)
        self.assertEqual(self.sent, [(self.p, signal.SIGSTOP)])
        self.control.update({self.p: 'T'}, 70)
        self.assertEqual(len(self.sent), 1)
        other = guard.Process(124, '1001')
        self.control.update({self.p: 'T', other: 'R'}, 70)
        self.assertIn((other, signal.SIGSTOP), self.sent)
        self.control.update({self.p: 'T', other: 'T'}, 65)
        self.assertEqual(len([s for _, s in self.sent if s == signal.SIGCONT]), 2)

    def test_missing_sensor_and_hot_shutdown_remain_paused(self):
        self.control.update({self.p: 'R'}, None)
        self.assertEqual(self.control.shutdown({self.p: 'T'}, 72), 'stopped_safely_paused')
        self.assertEqual(len(self.sent), 1)
        self.control.update({self.p: 'T'}, 65)
        self.assertEqual(self.sent[-1], (self.p, signal.SIGCONT))

    def test_external_stop_and_pid_reuse_are_not_resumed(self):
        self.control.update({self.p: 'T'}, 80)
        self.control.update({self.p: 'T'}, 60)
        self.assertFalse(self.sent)
        self.control.update({self.p: 'R'}, 80)
        reused = guard.Process(self.p.pid, '2000')
        self.control.update({reused: 'R'}, 60)
        self.assertEqual(self.sent, [(self.p, signal.SIGSTOP)])

    def test_finished_process_clears_owned_pause(self):
        self.control.update({self.p: 'R'}, 80)
        self.control.update({}, 80)
        self.assertFalse(self.control.paused)

    def test_external_resume_is_stopped_again_when_hot_or_sensor_missing(self):
        self.control.update({self.p: 'R'}, 80)
        self.control.update({self.p: 'R'}, 80)
        self.control.update({self.p: 'R'}, None)
        self.assertEqual(self.sent, [(self.p, signal.SIGSTOP)] * 3)
        self.assertEqual(self.control.paused, {self.p})

    def test_shutdown_pauses_new_arrivals_if_now_unsafe(self):
        self.assertEqual(self.control.shutdown({self.p: 'R'}, None), 'stopped_safely_paused')
        self.assertEqual(self.sent, [(self.p, signal.SIGSTOP)])

    def test_exact_executable_and_birth_identity_checked_before_pidfd_signal(self):
        with patch.object(guard.os, 'pidfd_open', return_value=9, create=True), \
             patch.object(guard.os, 'close'), \
             patch.object(guard, 'identify', return_value=(guard.Process(123, '999'), 'R')), \
             patch.object(guard.signal, 'pidfd_send_signal', create=True) as send:
            self.assertFalse(guard.send_verified(self.p, signal.SIGSTOP, Path('/batch/sim-runner')))
            send.assert_not_called()
        with patch.object(guard.os, 'pidfd_open', return_value=9, create=True), \
             patch.object(guard.os, 'close'), \
             patch.object(guard, 'identify', return_value=None), \
             patch.object(guard.signal, 'pidfd_send_signal', create=True) as send:
            self.assertFalse(guard.send_verified(self.p, signal.SIGCONT, Path('/batch/sim-runner')))
            send.assert_not_called()

    def test_sensor_invalid_and_valid(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'temp'
            for value in ('nan', 'broken', '-1000', '151000'):
                path.write_text(value)
                self.assertIsNone(guard.temperature(path))
            path.write_text('74500')
            self.assertEqual(guard.temperature(path), 74.5)

    def test_identify_requires_exact_executable_and_parses_unusual_names(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            proc = root / '123'
            proc.mkdir()
            fields = ['R'] + ['0'] * 18 + ['1000'] + ['0'] * 5
            (proc / 'stat').write_text('123 (odd ) worker) ' + ' '.join(fields))
            (proc / 'exe').symlink_to('/batch/sim-runner')
            self.assertEqual(guard.identify(123, Path('/batch/sim-runner'), root), (self.p, 'R'))
            self.assertIsNone(guard.identify(123, Path('/other/sim-runner'), root))

    def test_verified_pidfd_signal_is_sent_and_handle_closed(self):
        with patch.object(guard.os, 'pidfd_open', return_value=9, create=True), \
             patch.object(guard.os, 'close') as close, \
             patch.object(guard, 'identify', return_value=(self.p, 'R')), \
             patch.object(guard.signal, 'pidfd_send_signal', create=True) as send:
            self.assertTrue(guard.send_verified(self.p, signal.SIGSTOP, Path('/batch/sim-runner')))
            send.assert_called_once_with(9, signal.SIGSTOP)
            close.assert_called_once_with(9)


if __name__ == '__main__':
    unittest.main()
