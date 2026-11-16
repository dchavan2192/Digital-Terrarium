#!/usr/bin/env bash
# Held-out validation of candidate food-regrowth rates on the Pi.
#
# Unlike the sweep launchers, every job is a single run with periodic
# checkpoints, driven by a pinned executable and pinned base config that are
# copied into the batch directory. An interrupted batch can therefore continue
# even after the repository on the Pi has been updated.
#
#   bash analysis/launch_pi_validation.sh                       # new batch
#   RESUME=output/phase9a/validation_... bash analysis/launch_pi_validation.sh   # continue one
#
# Plan and acceptance criteria: docs/results/PHASE9A_VALIDATION_PLAN.md.
# Simulations of this length run on the Pi only.
set -euo pipefail
PI=${PI:-pi@dchavan}
TICKS=${TICKS:-200000}
SEEDS=${SEEDS:-100,101,102,103,104,105,106,107,108,109}
VALUES=${VALUES:-0.008,0.010,0.012}
JOBS=${JOBS:-2}                    # concurrent runs
THREADS_PER_JOB=${THREADS_PER_JOB:-1}
CHECKPOINT=${CHECKPOINT:-50000}
HARD_CAP=${HARD_CAP:-2000}
LOG_INTERVAL=${LOG_INTERVAL:-500}
RESUME=${RESUME:-}

[[ $TICKS =~ ^[1-9][0-9]*$ && $CHECKPOINT =~ ^[1-9][0-9]*$ && $HARD_CAP =~ ^[1-9][0-9]*$ &&
   $LOG_INTERVAL =~ ^[1-9][0-9]*$ && $JOBS =~ ^[1-3]$ && $THREADS_PER_JOB =~ ^[1-3]$ &&
   $SEEDS =~ ^[0-9]+(,[0-9]+)*$ && $VALUES =~ ^[0-9.]+(,[0-9.]+)*$ ]] || {
  echo 'Invalid experiment settings' >&2
  exit 1
}
(( JOBS * THREADS_PER_JOB <= 3 )) || { echo 'JOBS x THREADS_PER_JOB must leave one Pi core free (max 3)' >&2; exit 1; }
[[ -z $RESUME || $RESUME =~ ^output/phase9a/validation_[0-9TZ_]+$ ]] || { echo 'RESUME must be output/phase9a/validation_<id>' >&2; exit 1; }

JOB_SCRIPT=$(dirname "$0")/pi_validation_jobs.sh
[[ -f $JOB_SCRIPT ]] || { echo "missing $JOB_SCRIPT" >&2; exit 1; }

# The job script travels with the batch so a later repository update cannot change it.
ssh "$PI" 'mkdir -p ~/Digital-Terrarium/output/phase9a/.incoming'
scp -q "$JOB_SCRIPT" "$PI:Digital-Terrarium/output/phase9a/.incoming/pi_validation_jobs.sh"

ssh "$PI" bash -s -- "$TICKS" "$SEEDS" "$VALUES" "$JOBS" "$THREADS_PER_JOB" "$CHECKPOINT" "$HARD_CAP" "$LOG_INTERVAL" "$RESUME" <<'REMOTE'
set -euo pipefail
TICKS=$1 SEEDS=$2 VALUES=$3 JOBS=$4 THREADS_PER_JOB=$5 CHECKPOINT=$6 HARD_CAP=$7 LOG_INTERVAL=$8 RESUME=${9:-}
cd ~/Digital-Terrarium
[[ -f ~/.cargo/env ]] && source ~/.cargo/env
if pgrep -x sim-runner >/dev/null; then
  echo 'A simulation is already active; refusing to oversubscribe the Pi' >&2
  exit 1
fi

if [[ -n $RESUME ]]; then
  ROOT=$RESUME
  [[ -x $ROOT/sim-runner && -f $ROOT/run_jobs.sh ]] || { echo "not a validation batch: $ROOT" >&2; exit 1; }
  echo "Continuing $ROOT with its own pinned executable and settings"
else
  [[ -z $(git status --porcelain --untracked-files=no) ]] || {
    echo 'Pi tracked files have edits; refusing to replace them' >&2
    exit 1
  }
  git pull --ff-only origin main
  cargo build -p sim-runner --release
  ROOT="output/phase9a/validation_$(date -u +%Y%m%dT%H%M%SZ)_$$"
  mkdir -p "$ROOT"
  cp target/release/sim-runner "$ROOT/sim-runner"
  cp configs/default.toml "$ROOT/base.toml"
  cp output/phase9a/.incoming/pi_validation_jobs.sh "$ROOT/run_jobs.sh"
  git rev-parse HEAD > "$ROOT/source_revision"
  git status --porcelain > "$ROOT/source_status"
  {
    printf 'TICKS=%s\nCHECKPOINT=%s\nJOBS=%s\nLOG_INTERVAL=%s\nHARD_CAP=%s\nTHREADS_PER_JOB=%s\n' \
      "$TICKS" "$CHECKPOINT" "$JOBS" "$LOG_INTERVAL" "$HARD_CAP" "$THREADS_PER_JOB"
  } > "$ROOT/settings.env"
  # Seeds outermost, so a partly finished batch covers every rate evenly.
  for seed in ${SEEDS//,/ }; do
    for rate in ${VALUES//,/ }; do
      printf '%s %s\n' "$rate" "$seed"
    done
  done > "$ROOT/jobs.txt"
fi

nohup bash "$ROOT/run_jobs.sh" "$ROOT" >> "$ROOT/launcher.log" 2>&1 < /dev/null &
echo "Launched validation: PID $!, batch $ROOT, $(wc -l < "$ROOT/jobs.txt") jobs"
REMOTE
