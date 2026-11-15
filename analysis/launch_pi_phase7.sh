#!/usr/bin/env bash
# Four targeted ablation arms, NOT a factorial experiment. Run only after
# baseline calibration; uses committed source and preserves prior outputs.
set -euo pipefail
PI=${PI:-pi@dchavan}
TICKS=${TICKS:-500000}
SEEDS=${SEEDS:-5}
THREADS=${THREADS:-2}
[[ $TICKS =~ ^[1-9][0-9]*$ && $SEEDS =~ ^[1-9][0-9]*$ && $THREADS =~ ^[1-3]$ ]] || { echo 'Invalid numeric experiment settings' >&2; exit 1; }
ssh "$PI" bash -s -- "$TICKS" "$SEEDS" "$THREADS" <<'REMOTE'
set -euo pipefail
cd ~/Digital-Terrarium
[[ -f ~/.cargo/env ]] && source ~/.cargo/env
[[ -z $(git status --porcelain --untracked-files=no) ]] || { echo 'Pi tracked files have edits; refusing to replace them' >&2; exit 1; }
if pgrep -x sim-runner >/dev/null; then echo 'A simulation is already active; refusing to oversubscribe the Pi' >&2; exit 1; fi
git pull --ff-only origin main
cargo build -p sim-runner --release
ROOT="output/phase7/launch_$(date -u +%Y%m%dT%H%M%SZ)_$$"
mkdir -p "$ROOT"
cat > "$ROOT/run.sh" <<'JOB'
#!/usr/bin/env bash
set -euo pipefail
ROOT=$1
trap 'code=$?; printf "%s\n" "$code" > "$ROOT/exit_code"; if (( code == 0 )); then printf "completed\n" > "$ROOT/status"; else printf "failed\n" > "$ROOT/status"; fi' EXIT
printf 'running\n' > "$ROOT/status"
export RAYON_NUM_THREADS=$4
R=./target/release/sim-runner
"$R" sweep --config configs/default.toml --param reproduction.sexual --values 1,0 --seeds "$3" --ticks "$2" --log-interval 500 --output "$ROOT/sex"
"$R" sweep --config configs/default.toml --param niche_construction.enabled --values 0 --seeds "$3" --ticks "$2" --log-interval 500 --output "$ROOT/niche"
"$R" sweep --config configs/default.toml --param epigenetics.enabled --values 0 --seeds "$3" --ticks "$2" --log-interval 500 --output "$ROOT/epi"
JOB
nohup bash "$ROOT/run.sh" "$ROOT" "$1" "$2" "$3" > "$ROOT/launcher.log" 2>&1 < /dev/null &
echo "Launched Phase 7: PID $!, outputs $ROOT"
REMOTE
