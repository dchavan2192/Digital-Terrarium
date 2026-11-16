#!/usr/bin/env python3
"""Pause only one pinned Pi batch's runners when hot; resume after cooling.

Linux pidfds bind each signal to the verified process, avoiding PID reuse.
Run beside an existing batch: python3 pi_thermal_guard.py --batch BATCH_DIR
Missing/invalid temperature pauses runners. Shutdown resumes owned pauses only
when the current sensor reading is at or below --low; otherwise they stay safely
paused and the status file records how to recover by restarting this guard.
"""
import argparse
from dataclasses import asdict, dataclass
import fcntl
import json
import math
import os
from pathlib import Path
import signal
import sys
import time


@dataclass(frozen=True)
class Process:
    pid: int
    start: str


def identify(pid, executable, proc_root=Path('/proc')):
    """Return immutable process identity and state only for the exact executable."""
    try:
        directory = proc_root / str(pid)
        before = (directory / 'stat').read_text().rsplit(')', 1)[1].split()
        if os.readlink(directory / 'exe') != str(executable):
            return None
        after = (directory / 'stat').read_text().rsplit(')', 1)[1].split()
        if before[19] != after[19]:
            return None
        return Process(int(pid), after[19]), after[0]
    except (OSError, ValueError, IndexError):
        return None


def discover(executable):
    result = {}
    for path in Path('/proc').iterdir():
        if path.name.isdecimal():
            found = identify(int(path.name), executable)
            if found:
                result[found[0]] = found[1]
    return result


def send_verified(process, sig, executable):
    """Recheck executable and birth identity after obtaining a PID-bound handle."""
    try:
        fd = os.pidfd_open(process.pid)
    except ProcessLookupError:
        return False
    try:
        found = identify(process.pid, executable)
        if found is None or found[0] != process:
            return False
        try:
            signal.pidfd_send_signal(fd, sig)
            return True
        except ProcessLookupError:
            return False
    finally:
        os.close(fd)


def temperature(sensor):
    try:
        value = float(sensor.read_text().strip()) / 1000.0
        return value if math.isfinite(value) and 0 <= value <= 150 else None
    except (OSError, ValueError):
        return None


def log(message):
    print(time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), message, flush=True)


class Controller:
    def __init__(self, high, low, send, paused=()):
        self.high, self.low, self.send = high, low, send
        self.paused = set(paused)
        self.hot = bool(self.paused)

    def update(self, processes, temp):
        # Old identities are forgotten; a reused PID cannot inherit a pause.
        self.paused.intersection_update(processes)
        if temp is None or temp >= self.high:
            self.hot = True
        elif temp <= self.low:
            self.hot = False
        if self.hot:
            for process, state in processes.items():
                # A manual SIGCONT must not bypass the thermal policy. Re-stop
                # any running match while hot, even if we already own its pause.
                if state not in ('T', 't'):
                    if self.send(process, signal.SIGSTOP):
                        self.paused.add(process)
                        log(f'paused pid={process.pid} start={process.start} temp={temp}')
        else:
            for process in list(self.paused):
                if self.send(process, signal.SIGCONT):
                    log(f'resumed pid={process.pid} start={process.start} temp={temp}')
                    self.paused.remove(process)

    def shutdown(self, processes, temp):
        self.paused.intersection_update(processes)
        # Do not resume hot/missing-sensor runners merely because monitoring ends.
        # A fresh unsafe reading also protects arrivals since the last poll.
        if temp is None or temp >= self.high or temp <= self.low:
            self.update(processes, temp)
        return 'stopped_safely_paused' if self.paused else 'stopped'


def saved_pauses(path, executable):
    try:
        data = json.loads(path.read_text())
    except (OSError, ValueError):
        return []
    if data.get('executable') != str(executable):
        return []
    result = []
    for item in data.get('paused', []):
        try:
            ref = Process(int(item['pid']), str(item['start']))
            found = identify(ref.pid, executable)
            if found and found[0] == ref and found[1] in ('T', 't'):
                result.append(ref)
        except (TypeError, ValueError, KeyError):
            continue
    return result


def write_status(path, args, executable, controller, temp, processes, state):
    data = dict(state=state, updated_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                guard_pid=os.getpid(), batch=str(args.batch), executable=str(executable),
                temperature_c=temp, sensor=str(args.sensor), high_c=args.high,
                low_c=args.low, poll_seconds=args.poll, running_matches=len(processes),
                paused=[asdict(p) for p in sorted(controller.paused, key=lambda p: p.pid)])
    temporary = path.with_name(path.name + '.pending')
    temporary.write_text(json.dumps(data, indent=2) + '\n')
    temporary.replace(path)


def parse_args(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--batch', type=Path, required=True)
    parser.add_argument('--high', type=float, default=75)
    parser.add_argument('--low', type=float, default=65)
    parser.add_argument('--poll', type=float, default=5)
    parser.add_argument('--sensor', type=Path, default=Path('/sys/class/thermal/thermal_zone0/temp'))
    parser.add_argument('--output', type=Path, help='Default: BATCH/thermal_guard')
    args = parser.parse_args(argv)
    if not (math.isfinite(args.high) and math.isfinite(args.low)
            and 0 < args.low < args.high <= 85):
        parser.error('require 0 < low < high <= 85 Celsius')
    if not math.isfinite(args.poll) or not 0 < args.poll <= 60:
        parser.error('poll must be > 0 and <= 60 seconds')
    args.batch = args.batch.resolve()
    if not (args.batch / 'sim-runner').is_file() or not (args.batch / 'status').is_file():
        parser.error('batch needs its pinned sim-runner and status file')
    args.output = (args.output or args.batch / 'thermal_guard').resolve()
    return args


def main(argv=None):
    args = parse_args(argv)
    if not sys.platform.startswith('linux') or not hasattr(os, 'pidfd_open') or not hasattr(signal, 'pidfd_send_signal'):
        raise SystemExit('Linux with Python pidfd support is required; no processes signalled')
    executable = args.batch / 'sim-runner'
    # Lock always belongs to the batch, even when --output differs.
    lock = (args.batch / '.thermal_guard.lock').open('a')
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        raise SystemExit('A thermal guard already owns this batch')
    args.output.mkdir(parents=True, exist_ok=True)
    status_path = args.output / 'status.json'
    controller = Controller(args.high, args.low,
                            lambda process, sig: send_verified(process, sig, executable),
                            saved_pauses(status_path, executable))
    stopping = False

    def stop(_sig, _frame):
        nonlocal stopping
        stopping = True

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    log(f'guard started batch={args.batch} high={args.high} low={args.low} poll={args.poll}')
    completed = False
    previous_state = None
    try:
        while not stopping:
            processes = discover(executable)
            temp = temperature(args.sensor)
            controller.update(processes, temp)
            batch_state = (args.batch / 'status').read_text().strip()
            if not processes and batch_state in ('completed', 'failed'):
                controller.paused.clear()
                write_status(status_path, args, executable, controller, temp, processes, 'finished')
                log(f'guard finished batch_status={batch_state}')
                completed = True
                break
            state = ('paused_sensor_missing' if temp is None else 'paused_hot') if controller.hot else 'monitoring'
            if state != previous_state:
                log(f'state={state} temp={temp} matches={len(processes)}')
                previous_state = state
            write_status(status_path, args, executable, controller, temp, processes, state)
            # A signal interrupts monitoring within at most one second.
            deadline = time.monotonic() + args.poll
            while not stopping and time.monotonic() < deadline:
                time.sleep(min(1, max(0, deadline - time.monotonic())))
    finally:
        if not completed:
            processes, temp = discover(executable), temperature(args.sensor)
            state = controller.shutdown(processes, temp)
            write_status(status_path, args, executable, controller, temp, processes, state)
            log(f'guard {state}; restart guard to recover owned pauses')
        lock.close()


if __name__ == '__main__':
    main()
