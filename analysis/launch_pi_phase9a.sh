#!/usr/bin/env bash
# Committed-source ecology pilot on the Pi. Unique output, two worker threads,
# explicit completion/failure markers; never overwrites an old experiment.
# TICKS=200000 SEEDS=10 VALUES=0.002,0.004,0.006 bash analysis/launch_pi_phase9a.sh
set -euo pipefail
PI=${PI:-pi@dchavan}
TICKS=${TICKS:-100000}
SEEDS=${SEEDS:-3}
VALUES=${VALUES:-0.002,0.004,0.006,0.008,0.010,0.012}
THREADS=${THREADS:-2}
[[ $TICKS =~ ^[1-9][0-9]*$ && $SEEDS =~ ^[1-9][0-9]*$ && $THREADS =~ ^[1-3]$ && $VALUES =~ ^[0-9.eE,+-]+$ ]] || { echo 'Invalid numeric experiment settings' >&2; exit 1; }
ssh "$PI" bash -s -- "$TICKS" "$SEEDS" "$VALUES" "$THREADS" <<'REMOTE'
set -euo pipefail
cd ~/Digital-Terrarium
[[ -f ~/.cargo/env ]] && source ~/.cargo/env
[[ -z $(git status --porcelain --untracked-files=no) ]] || { echo 'Pi tracked files have edits; refusing to replace them' >&2; exit 1; }
if pgrep -x sim-runner >/dev/null; then echo 'A simulation is already active; refusing to oversubscribe the Pi' >&2; exit 1; fi
git pull --ff-only origin main
cargo build -p sim-runner --release
ROOT="output/phase9a/launch_$(date -u +%Y%m%dT%H%M%SZ)_$$"
mkdir -p "$ROOT"
cat > "$ROOT/run.sh" <<'JOB'
#!/usr/bin/env bash
set -euo pipefail
ROOT=$1
trap 'code=$?; printf "%s\n" "$code" > "$ROOT/exit_code"; if (( code == 0 )); then printf "completed\n" > "$ROOT/status"; else printf "failed\n" > "$ROOT/status"; fi' EXIT
printf 'running\n' > "$ROOT/status"
RAYON_NUM_THREADS=$5 ./target/release/sim-runner sweep --config configs/default.toml \
  --set creature.hard_cap=2000 --param food.regrowth_rate --values "$4" \
  --seeds "$3" --ticks "$2" --log-interval 500 --output "$ROOT"
JOB
nohup bash "$ROOT/run.sh" "$ROOT" "$1" "$2" "$3" "$4" > "$ROOT/launcher.log" 2>&1 < /dev/null &
echo "Launched Phase 9a: PID $!, outputs $ROOT"
REMOTE
