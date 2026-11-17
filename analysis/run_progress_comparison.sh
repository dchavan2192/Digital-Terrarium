#!/usr/bin/env bash
# Execute the predeclared control then treatment, once, with owned supervisors.
set -euo pipefail
ROOT=$(cd "${1:?comparison directory required}" && pwd)
[[ ! -e $ROOT/status ]] || { echo 'Comparison already attempted; preserve it and inspect before recovery' >&2; exit 2; }
exec 9>"$ROOT/.comparison.lock"
flock -n 9 || exit 3
for mode in terminal_only terminal_progress; do
  [[ -x $ROOT/$mode/sim-runner && -f $ROOT/$mode/visible_acquisition.toml &&
     -f $ROOT/$mode/delayed_choice.toml && -f $ROOT/$mode/pi_thermal_guard.py &&
     ! -e $ROOT/$mode/status ]] || { echo "Incomplete or reused sub-batch: $mode" >&2; exit 2; }
done
trap 'code=$?; printf "%s\n" "$code" > "$ROOT/exit_code"; if (( code == 0 )); then echo completed > "$ROOT/status"; else echo failed > "$ROOT/status"; fi' EXIT
echo running > "$ROOT/status"
for mode in terminal_only terminal_progress; do
  echo "Starting $mode at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  python3 "$ROOT/run_guarded_acquisition.py" --batch "$ROOT/$mode" --wall-seconds 1800
  echo "Finished $mode at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
done
