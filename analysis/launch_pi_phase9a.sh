#!/usr/bin/env bash
# Phase 9a: find a food-limited carrying capacity (simulations run on the Pi 5 only).
# Raises hard_cap to 2000 so it acts only as a backstop, then sweeps food.regrowth_rate.
# Goal: mean population ~60-70% of the cap, with total_cap_blocked_births ~0.
#
# Prerequisite: the commit containing `sweep --set` and `cap_blocked_births` is on origin/main.
# Usage (from repo root):  bash analysis/launch_pi_phase9a.sh
# Fetch results later:     rsync -av pi@100.127.61.93:~/Digital-Terrarium/output/phase9a/ output/phase9a/
set -euo pipefail

PI=pi@100.127.61.93
TICKS=${TICKS:-100000}
SEEDS=${SEEDS:-3}
VALUES=${VALUES:-0.0002,0.0003,0.0004,0.0005}

ssh "$PI" bash -s <<EOF
set -e
cd ~/Digital-Terrarium
source ~/.cargo/env 2>/dev/null || true
git pull --ff-only origin main
cargo build -p sim-runner --release
mkdir -p output/phase9a
nohup ./target/release/sim-runner sweep --config configs/default.toml \
  --set creature.hard_cap=2000 \
  --param food.regrowth_rate --values $VALUES \
  --seeds $SEEDS --ticks $TICKS --log-interval 500 \
  --output output/phase9a > output/phase9a/sweep.log 2>&1 &
echo "Launched Phase 9a sweep on Pi (PID \$!)"
EOF
