#!/usr/bin/env bash
# Fixed exploratory demo-setting recurrence comparison. One worker, guarded.
set -euo pipefail
ROOT=$(cd "${1:?batch directory required}" && pwd)
cd /home/pi/Digital-Terrarium
exec 9>"$ROOT/.jobs.lock"
flock -n 9 || exit 3
trap 'code=$?; printf "%s\n" "$code" > "$ROOT/exit_code"; if (( code == 0 )); then echo completed > "$ROOT/status"; else echo failed > "$ROOT/status"; fi' EXIT
# Guard must be running and cool before the first runner is allowed to start.
for attempt in {1..30}; do
  if python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); sys.exit(d["state"]!="monitoring")' "$ROOT/thermal_guard/status.json" 2>/dev/null; then break; fi
  sleep 1
done
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); sys.exit(d["state"]!="monitoring")' "$ROOT/thermal_guard/status.json"
echo running > "$ROOT/status"
while read -r seed rate; do
  kill -0 "$(cat "$ROOT/guard.pid")"
  job="$ROOT/rate_${rate}/seed_${seed}"
  mkdir -p "$job"
  RAYON_NUM_THREADS=1 "$ROOT/sim-runner" --config "$ROOT/base.toml" --seed "$seed" \
    --set food.regrowth_rate=0.002 --set creature.hard_cap=500 \
    --set "mutation.add_recurrent_probability=$rate" \
    --ticks 200000 --log-interval 500 --checkpoint-every 50000 \
    --export-genomes-every 200000 --output "$job" > "$job/runner.log" 2>&1 &
  runner_pid=$!
  echo "$runner_pid" > "$ROOT/runner.pid"
  while kill -0 "$runner_pid" 2>/dev/null; do
    if ! kill -0 "$(cat "$ROOT/guard.pid")" 2>/dev/null; then
      kill -STOP "$runner_pid" 2>/dev/null || true
      echo 'Thermal guard exited; runner left paused. Restart guard before recovery.' >&2
      exit 1
    fi
    sleep 5
  done
  wait "$runner_pid"
  printf '%s %s\n' "$seed" "$rate" >> "$ROOT/completed_jobs.txt"
done < "$ROOT/jobs.txt"
