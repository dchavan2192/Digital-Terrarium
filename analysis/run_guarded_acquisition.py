#!/usr/bin/env python3
"""Supervise one fresh pinned acquisition batch; never signal unrelated processes.

The active-command wall watchdog includes guard startup and thermal pauses.
Terminal guard cleanup is separately bounded (12 seconds plus termination grace).
Guard loss or an
interruption leaves the directly owned runner stopped; watchdog expiry instead
terminates it without SIGCONT. Batch contents and partial artifacts are retained.
"""
import argparse
from datetime import datetime, timezone
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def utc():
    return datetime.now(timezone.utc).isoformat()


def atomic_json(path, value):
    pending = path.with_name(path.name + '.pending')
    pending.write_text(json.dumps(value, indent=2) + '\n')
    pending.replace(path)


def signal_owned(child, sig):
    # A direct child that exits remains unreaped (and cannot have its PID reused)
    # until Popen.poll/send_signal observes it. No PID discovery or process groups.
    if child is None or child.poll() is not None:
        return False
    try:
        child.send_signal(sig)
        return True
    except ProcessLookupError:
        return False


def terminate_owned(child, grace):
    if child is None:
        return None
    signal_owned(child, signal.SIGTERM)
    try:
        return child.wait(timeout=grace)
    except subprocess.TimeoutExpired:
        # SIGTERM stays pending on a stopped task. SIGKILL works without resuming.
        signal_owned(child, signal.SIGKILL)
        return child.wait(timeout=grace)


def guard_status(path, guard, batch, launched_wall, stale_seconds):
    if guard.poll() is not None:
        return None, f'guard exited with code {guard.returncode}'
    try:
        stat = path.stat()
        data = json.loads(path.read_text())
        now = time.time()
        stamp = datetime.strptime(data['updated_utc'], '%Y-%m-%dT%H:%M:%SZ').replace(tzinfo=timezone.utc).timestamp()
        if (data['guard_pid'] != guard.pid or data['batch'] != str(batch)
                or data['executable'] != str(batch / 'sim-runner')
                or data['high_c'] != 75 or data['low_c'] != 65
                or data['poll_seconds'] != 5):
            return None, 'guard status identity or policy mismatch'
        # One second accounts for the guard's UTC string precision, not stale mtime.
        if (stat.st_mtime < launched_wall - 1 or now - stat.st_mtime > stale_seconds
                or stat.st_mtime > now + 2 or stamp < launched_wall - 1
                or now - stamp > stale_seconds + 1 or stamp > now + 2):
            return None, 'guard heartbeat is stale or has an invalid timestamp'
        if data['state'] not in ('monitoring', 'paused_hot', 'paused_sensor_missing'):
            return None, f"unexpected guard state {data['state']}"
        if data['state'] == 'monitoring':
            temp = data['temperature_c']
            if not isinstance(temp, (int, float)) or not math.isfinite(temp) or not 0 <= temp < 75:
                return None, 'monitoring status lacks a safe finite temperature'
        return data, None
    except FileNotFoundError:
        return None, 'guard has not written a status'
    except (OSError, ValueError, KeyError, TypeError) as error:
        return None, f'invalid guard status: {error}'


def thermal_snapshot(sensor):
    record = {'utc': utc()}
    try:
        temp = float(sensor.read_text().strip()) / 1000
        if not math.isfinite(temp) or not 0 <= temp <= 150:
            raise ValueError('non-finite/out-of-range temperature')
        record['temperature_c'] = temp
    except (OSError, ValueError) as error:
        record.update(temperature_c=None, temperature_error=str(error))
    try:
        result = subprocess.run(['vcgencmd', 'get_throttled'], capture_output=True,
                                text=True, timeout=1, check=False)
        record.update(throttled_output=result.stdout.strip(),
                      throttled_exit_code=result.returncode)
        if result.returncode:
            record['throttled_error'] = result.stderr.strip() or 'vcgencmd returned nonzero'
    except (OSError, subprocess.TimeoutExpired) as error:
        record.update(throttled_output=None, throttled_error=str(error))
    return record


def parse_args(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--batch', type=Path, required=True)
    p.add_argument('--wall-seconds', type=float, default=1800)
    p.add_argument('--guard-stale-seconds', type=float, default=20)
    p.add_argument('--startup-seconds', type=float, default=15,
                   help='Time allowed for first valid guard heartbeat; cooling uses the wall budget')
    p.add_argument('--poll-seconds', type=float, default=0.5)
    p.add_argument('--terminate-grace-seconds', type=float, default=2)
    p.add_argument('--sensor', type=Path, default=Path('/sys/class/thermal/thermal_zone0/temp'))
    args = p.parse_args(argv)
    for key in ('wall_seconds', 'guard_stale_seconds', 'startup_seconds',
                'poll_seconds', 'terminate_grace_seconds'):
        if not math.isfinite(getattr(args, key)) or getattr(args, key) <= 0:
            p.error(f'{key} must be finite and positive')
    if args.poll_seconds > 1 or args.wall_seconds > 1800:
        p.error('poll interval must be <=1 and wall budget <=1800 seconds')
    args.batch = args.batch.resolve()
    required = ['sim-runner', 'visible_acquisition.toml', 'delayed_choice.toml', 'pi_thermal_guard.py']
    if not args.batch.is_dir() or any(not (args.batch / name).is_file() or (args.batch / name).is_symlink() for name in required):
        p.error('batch must contain the pinned runner, protocol, arena and thermal guard')
    if not os.access(args.batch / 'sim-runner', os.X_OK):
        p.error('pinned sim-runner must be executable')
    return args


def supervise(args):
    batch = args.batch
    # Exclusive creation makes reruns explicit; even completed batches are not reused.
    with (batch / '.acquisition_supervisor.lock').open('x') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        return run_fresh(args)


def run_fresh(args):
    batch = args.batch
    names = ['status', 'supervisor.json', 'supervisor_manifest.json', 'exit_code',
             'guard.log', 'runner.log', 'thermal_supervisor.jsonl', 'thermal_guard',
             '.thermal_guard.lock', 'runs']
    if any((batch / name).exists() for name in names):
        raise ValueError('batch already has operational artifacts; use a fresh batch')
    # Validate freshness before any status/log mutation.
    guard_dir = batch / 'thermal_guard'
    guard_dir.mkdir()
    (batch / 'status').write_text('running\n')
    runner_cmd = [str(batch / 'sim-runner'), 'acquire-visible',
                  '--protocol', str(batch / 'visible_acquisition.toml'),
                  '--arena', str(batch / 'delayed_choice.toml'), '--output', str(batch / 'runs')]
    guard_cmd = [sys.executable, str(batch / 'pi_thermal_guard.py'), '--batch', str(batch),
                 '--high', '75', '--low', '65', '--poll', '5', '--sensor', str(args.sensor),
                 '--output', str(guard_dir)]
    start_wall, start_mono = time.time(), time.monotonic()
    manifest = dict(started_utc=utc(), supervisor_pid=os.getpid(), batch=str(batch),
                    runner_command=runner_cmd, guard_command=guard_cmd,
                    environment_override={'RAYON_NUM_THREADS': '1'},
                    wall_seconds=args.wall_seconds, wall_includes_startup_and_cooling=True,
                    wall_watchdog_scope='guard startup, cooling and active acquisition; terminal cleanup separately bounded',
                    post_runner_guard_wait_seconds=12,
                    guard_stale_seconds=args.guard_stale_seconds,
                    startup_seconds=args.startup_seconds, poll_seconds=args.poll_seconds,
                    terminate_grace_seconds=args.terminate_grace_seconds,
                    temperature_high_c=75, temperature_low_c=65, guard_poll_seconds=5,
                    telemetry_interval_seconds=5,
                    completion_scope='process supervision only; scientific artifact audit remains separate',
                    sha256={name: hashlib.sha256((batch / name).read_bytes()).hexdigest()
                            for name in ['sim-runner', 'visible_acquisition.toml',
                                         'delayed_choice.toml', 'pi_thermal_guard.py']})
    (batch / 'supervisor_manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    stopped = False

    def interrupt(_sig, _frame):
        nonlocal stopped
        stopped = True

    handlers = {sig: signal.signal(sig, interrupt) for sig in (signal.SIGTERM, signal.SIGINT)}
    runner = guard = None
    state = dict(state='starting_guard', started_utc=manifest['started_utc'],
                 guard_pid=None, runner_pid=None, runner_exit_code=None,
                 runner_left_paused=False, reason=None)
    exit_code, last_snapshot, first_heartbeat = 1, -math.inf, False

    def save():
        state.update(updated_utc=utc(), elapsed_wall_seconds=time.monotonic() - start_mono)
        atomic_json(batch / 'supervisor.json', state)

    def fail_paused(reason):
        # Stop immediately, then stop again after the guard can no longer resume
        # an owned pause in its cool-shutdown handler.
        signal_owned(runner, signal.SIGSTOP)
        terminate_owned(guard, args.terminate_grace_seconds)
        paused = signal_owned(runner, signal.SIGSTOP)
        state.update(state='failed', reason=reason, runner_left_paused=paused,
                     runner_exit_code=None if runner is None else runner.poll())
        (batch / 'status').write_text('failed\n')
        save()

    try:
        with (batch / 'guard.log').open('x') as guard_log, \
             (batch / 'runner.log').open('x') as runner_log, \
             (batch / 'thermal_supervisor.jsonl').open('x') as thermal_log:
            guard = subprocess.Popen(guard_cmd, cwd=batch, stdout=guard_log, stderr=subprocess.STDOUT)
            guard_launched = time.time()
            state['guard_pid'] = guard.pid
            save()
            while True:
                elapsed = time.monotonic() - start_mono
                if stopped:
                    fail_paused('interrupted; restart/recovery requires explicit inspection')
                    exit_code = 130
                    break
                if elapsed >= args.wall_seconds:
                    # No SIGCONT even if the thermal guard previously stopped it.
                    state['runner_exit_code'] = terminate_owned(runner, args.terminate_grace_seconds)
                    state.update(state='technical_timeout', reason='wall watchdog expired; startup/cooling included')
                    (batch / 'status').write_text('failed\n')
                    terminate_owned(guard, args.terminate_grace_seconds)
                    save()
                    exit_code = 124
                    break
                if elapsed - last_snapshot >= 5:
                    thermal_log.write(json.dumps(thermal_snapshot(args.sensor)) + '\n')
                    thermal_log.flush()
                    last_snapshot = elapsed
                # A finished runner no longer needs active thermal protection.
                if runner is not None and runner.poll() is not None:
                    code = runner.returncode
                    state.update(state='completed' if code == 0 else 'failed',
                                 runner_exit_code=code, reason=None if code == 0 else 'runner returned nonzero')
                    (batch / 'status').write_text(('completed' if code == 0 else 'failed') + '\n')
                    # Normal guard completion: after the runner is gone and batch
                    # status final, the existing guard exits on its next poll.
                    try:
                        guard_code = guard.wait(timeout=12)
                    except subprocess.TimeoutExpired:
                        guard_code = terminate_owned(guard, args.terminate_grace_seconds)
                        state.update(state='failed', reason='guard did not finish normally after runner exit')
                    state['guard_exit_code'] = guard_code
                    if guard_code != 0:
                        state.update(state='failed', reason='guard failed during finalization')
                    if state['state'] == 'failed':
                        (batch / 'status').write_text('failed\n')
                    save()
                    exit_code = 0 if state['state'] == 'completed' else 1
                    break
                data, error = guard_status(guard_dir / 'status.json', guard, batch,
                                           guard_launched, args.guard_stale_seconds)
                if data is None:
                    waiting_first = (not first_heartbeat and runner is None
                                     and error == 'guard has not written a status'
                                     and elapsed < args.startup_seconds)
                    if not waiting_first:
                        fail_paused(error)
                        break
                else:
                    first_heartbeat = True
                    state['guard_state'] = data['state']
                    if runner is None and data['state'] == 'monitoring':
                        environment = os.environ.copy()
                        environment['RAYON_NUM_THREADS'] = '1'
                        runner = subprocess.Popen(runner_cmd, cwd=batch, env=environment,
                                                  stdout=runner_log, stderr=subprocess.STDOUT)
                        state.update(state='running', runner_pid=runner.pid)
                    elif runner is None:
                        state['state'] = 'waiting_for_cooling'
                save()
                time.sleep(min(args.poll_seconds, max(0, args.wall_seconds - (time.monotonic() - start_mono))))
    except Exception as error:
        # Safety precedes persistence: even a log/status I/O error stops the runner.
        signal_owned(runner, signal.SIGSTOP)
        terminate_owned(guard, args.terminate_grace_seconds)
        paused = signal_owned(runner, signal.SIGSTOP)
        state.update(state='failed', reason=f'supervisor exception: {error}', runner_left_paused=paused)
        try:
            (batch / 'status').write_text('failed\n')
            save()
        except OSError:
            pass
        exit_code = 1
    finally:
        if guard is not None and guard.poll() is None:
            terminate_owned(guard, args.terminate_grace_seconds)
        for sig, handler in handlers.items():
            signal.signal(sig, handler)
        state['guard_exit_code'] = None if guard is None else guard.returncode
        state['ended_utc'] = utc()
        state['supervisor_exit_code'] = exit_code
        state['runner_exit_code'] = None if runner is None else runner.poll()
        # A stopped runner is intentionally alive after guard loss/interruption.
        try:
            save()
            (batch / 'exit_code').write_text(str(exit_code) + '\n')
        except OSError:
            pass
    return exit_code


def main(argv=None):
    args = parse_args(argv)
    try:
        return supervise(args)
    except (OSError, ValueError) as error:
        print(f'acquisition supervisor refused: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
