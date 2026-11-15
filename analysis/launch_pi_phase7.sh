#!/usr/bin/env bash
# Launch Phase 7 ON/OFF ablation experiments on the Pi 5 (never locally).
# Syncs local source to the Pi (so uncommitted sweep toggles are included),
# builds there, and starts all sweeps under nohup so they survive SSH drops.
#
# Usage (from repo root):  bash analysis/launch_pi_phase7.sh
# Fetch results later:     rsync -av pi@100.127.61.93:~/Digital-Terrarium/output/phase7/ output/phase7/
set -euo pipefail

PI=pi@100.127.61.93
REMOTE=~/Digital-Terrarium
TICKS=500000
SEEDS=5

rsync -av --exclude target --exclude output --exclude .git --exclude 'analysis/.venv' \
  ./crates ./configs ./Cargo.toml ./Cargo.lock "$PI:$REMOTE/"

ssh "$PI" bash -s <<EOF
set -e
cd $REMOTE
source ~/.cargo/env 2>/dev/null || true
cargo build -p sim-runner --release
mkdir -p output/phase7
nohup bash -c '
  R=./target/release/sim-runner
  \$R sweep --config configs/default.toml --param reproduction.sexual --values 1,0 --seeds $SEEDS --ticks $TICKS --log-interval 500 --output output/phase7/sex   > output/phase7/sex.log 2>&1
  \$R sweep --config configs/default.toml --param niche_construction.enabled --values 0 --seeds $SEEDS --ticks $TICKS --log-interval 500 --output output/phase7/niche > output/phase7/niche.log 2>&1
  \$R sweep --config configs/default.toml --param epigenetics.enabled --values 0 --seeds $SEEDS --ticks $TICKS --log-interval 500 --output output/phase7/epi   > output/phase7/epi.log 2>&1
  echo DONE > output/phase7/DONE
' > output/phase7/launcher.log 2>&1 &
echo "Launched on Pi (PID \$!)"
EOF
