#!/usr/bin/env bash
# Run, or continue, every job of one validation batch.
#
#   bash pi_validation_jobs.sh <batch-directory>
#
# The batch directory is self-contained: a pinned `sim-runner` executable, the
# pinned `base.toml`, `settings.env` and `jobs.txt` (one "rate seed" per line).
# Each job is a single run with periodic checkpoints, written under
# rate_<rate>/seed_<seed>/session_*/.
#
# Safe to re-run after a crash or power cut: finished jobs are skipped, and an
# interrupted job resumes from its latest checkpoint in a NEW session directory.
# Nothing is overwritten or deleted. A resumed job therefore has several session
# CSVs; rows after the checkpoint tick in the interrupted session are repeated
# (with identical values) by the session that resumed it.
set -uo pipefail

[[ $# -eq 1 && -d $1 ]] || { echo "usage: $0 <batch-directory>" >&2; exit 2; }
ROOT=$(cd "$1" && pwd)
RUNNER="$ROOT/sim-runner"
[[ -x $RUNNER && -f $ROOT/base.toml && -f $ROOT/jobs.txt && -f $ROOT/settings.env ]] || {
  echo "incomplete batch directory: $ROOT" >&2
  exit 2
}
# shellcheck disable=SC1091
source "$ROOT/settings.env"
for name in TICKS CHECKPOINT JOBS LOG_INTERVAL HARD_CAP THREADS_PER_JOB; do
  [[ ${!name:-} =~ ^[1-9][0-9]*$ ]] || { echo "settings.env: $name must be a positive integer" >&2; exit 2; }
done

# One runner per batch. flock releases the lock when the process dies, so a
# power cut never leaves a stale lock behind.
if command -v flock >/dev/null; then
  exec 9>"$ROOT/.lock"
  flock -n 9 || { echo "another runner already owns $ROOT" >&2; exit 3; }
else
  echo "flock not available: not guarding against a second runner" >&2
fi

printf 'running\n' > "$ROOT/status"
trap 'code=$?; printf "%s\n" "$code" > "$ROOT/exit_code"
      if (( code == 0 )); then printf "completed\n" > "$ROOT/status"; else printf "failed\n" > "$ROOT/status"; fi' EXIT

# Thermal trace for the whole batch (Raspberry Pi only).
if command -v vcgencmd >/dev/null; then
  (
    while kill -0 $$ 2>/dev/null; do
      printf '%s %s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$(vcgencmd measure_temp)" "$(vcgencmd get_throttled)"
      sleep 60
    done >> "$ROOT/thermal.log"
  ) &
fi

# A job is finished when one of its sessions reached the target tick or went extinct.
job_done() {
  local dir=$1 seed=$2 manifest
  for manifest in "$dir"/session_*/manifest_seed"$seed".toml; do
    [[ -f $manifest ]] || continue
    if grep -q '^status = "extinct"' "$manifest"; then return 0; fi
    if grep -q '^status = "completed"' "$manifest" && grep -q "^completed_ticks = $TICKS\$" "$manifest"; then
      return 0
    fi
  done
  return 1
}

# Checkpoints are written atomically, so any file with this name is complete.
latest_checkpoint() {
  local dir=$1 seed=$2
  ls "$dir"/session_*/checkpoint_seed"$seed"_tick*.json 2>/dev/null |
    awk -F'_tick' '{ t = $NF; sub(/\.json$/, "", t); print t + 0 "\t" $0 }' |
    sort -n | tail -1 | cut -f2-
}

run_job() {
  local rate=$1 seed=$2
  local dir="$ROOT/rate_$rate/seed_$seed"
  local log checkpoint tick
  mkdir -p "$dir"
  if job_done "$dir" "$seed"; then
    echo "[skip]   rate=$rate seed=$seed already finished"
    return 0
  fi
  log="$dir/run_$(date -u +%Y%m%dT%H%M%SZ)_$$.log"
  checkpoint=$(latest_checkpoint "$dir" "$seed")
  if [[ -n $checkpoint ]]; then
    tick=${checkpoint##*_tick}
    tick=${tick%.json}
    if (( tick >= TICKS )); then
      echo "[check]  rate=$rate seed=$seed has a final checkpoint but no completed manifest; inspect $dir" >&2
      return 1
    fi
    echo "[resume] rate=$rate seed=$seed from tick $tick"
    "$RUNNER" --resume "$checkpoint" --ticks "$TICKS" --log-interval "$LOG_INTERVAL" \
      --checkpoint-every "$CHECKPOINT" --output "$dir" > "$log" 2>&1
  else
    echo "[start]  rate=$rate seed=$seed"
    "$RUNNER" --config "$ROOT/base.toml" --seed "$seed" \
      --set "creature.hard_cap=$HARD_CAP" --set "food.regrowth_rate=$rate" \
      --ticks "$TICKS" --log-interval "$LOG_INTERVAL" \
      --checkpoint-every "$CHECKPOINT" --output "$dir" > "$log" 2>&1
  fi
  local code=$?
  if (( code == 0 )); then
    echo "[done]   rate=$rate seed=$seed"
  else
    echo "[failed] rate=$rate seed=$seed exit=$code (see $log)" >&2
  fi
  return "$code"
}

export ROOT RUNNER TICKS CHECKPOINT LOG_INTERVAL HARD_CAP
export RAYON_NUM_THREADS=$THREADS_PER_JOB
export -f run_job job_done latest_checkpoint

# xargs exits non-zero if any job failed; the other jobs still run.
xargs -n 2 -P "$JOBS" bash -c 'run_job "$1" "$2"' _ < "$ROOT/jobs.txt"
