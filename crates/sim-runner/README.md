# Experiment artifacts

Every single run or sweep creates a new `session_<unix-nanoseconds>_<pid>_<counter>` directory beneath `--output`. It never appends to or truncates an earlier run. Sweeps validate **every generated configuration** after overrides before creating that session or starting jobs. Integer parameters require integers; toggles require 0 or 1. Values that collide in the six-decimal output labels are rejected.

Single-run artifacts live directly in the session; sweep artifacts live in `session_.../sweep/`. Existing `stats_seed...csv` and `config_seed...toml` basenames are preserved. Each run also writes an adjacent `manifest_seed...toml`: full configuration, compiled source revision and dirty flag, compiler, target, profile, schema/hash version, seed, requested/completed ticks, log interval, elapsed time, and completion status.

- `completed`: requested duration reached and artifacts flushed.
- `extinct`: finished early because the population died out; this is an experimental outcome, not execution failure.
- `failed`: caught simulation panic or artifact I/O error; the CLI returns a nonzero exit status.
- `running`, absent, or malformed manifest: incomplete/unverified; never treat this as a finished result. Abrupt process termination or power loss may leave this state. Manifest updates are not yet atomic checkpoints.

The stats CSV begins after the first simulation step, avoiding the old synthetic tick-zero all-zero row, and always includes the actual final tick even when it is not a log interval boundary. Source provenance is captured at build time, so running the binary from another checkout does not relabel it. A dirty build is explicitly marked and should not be used for a published result without preserving its source.

The Phase 9a Pi script uses `pi@dchavan`, defaults to two worker threads, refuses to overwrite tracked remote edits or launch over an active simulation, and creates `output/phase9a/launch_.../session_.../`. Its `status`, `exit_code`, and `launcher.log` describe the detached job. The Phase 7 script similarly creates `output/phase7/launch_.../{sex,niche,epi}/session_.../sweep/`; this is four targeted arms, not a factorial study. Point analysis at the specific launch/session, rather than mixing repeated experiments.

**Environment stream:** droughts draw from their own random stream, so their timing and placement depend only on the drought settings and the drought seed (`food.droughts.schedule_seed`, or `world.seed` when omitted). Switching a feature off, or anything else the organisms do, no longer shifts the schedule; `crates/sim-core/tests/environment_stream.rs` checks this. Set `--set food.droughts.schedule_seed=N` to replay one schedule across world seeds. Shared random inputs do not automatically reduce variance in nonlinear ecological systems; see [Glasserman and Yao (1992), Some Guidelines and Guarantees for Common Random Numbers](https://business.columbia.edu/sites/default/files-efs/pubfiles/4261/glasserman_yao_guidelines.pdf). Runs made before 2026-11-16 (manifests with `environmental_rng = "shared_with_evolution..."`) did not have this property.

## Checkpoints, resume, and genome export

Single runs support `--checkpoint-every N` and `--export-genomes-every N`. Each writes at absolute tick multiples of N **and at the final tick**, including extinction. Artifacts are named `checkpoint_seed42_tick1000.json` / `genomes_seed42_tick1000.json` inside the unique session. Specifying an interval of zero is an error. Sweeps currently do not accept these options.

```bash
# Run to tick 10000, retaining periodic complete state and living genomes.
cargo run -p sim-runner --release -- --ticks 10000 \
  --checkpoint-every 1000 --export-genomes-every 2000 --output output/pilot

# Continue the exact state in a NEW session to absolute tick 20000.
cargo run -p sim-runner --release -- \
  --resume output/pilot/session_ID/checkpoint_seed42_tick10000.json \
  --ticks 20000 --checkpoint-every 1000 --output output/continued
```

`--ticks` means the **absolute target tick**, not additional steps. A target at or behind the checkpoint tick is rejected before output creation. Resume uses the checkpoint's embedded configuration; explicit `--config` and `--resume` conflict. Extinct populations cannot be resumed. Genome export is an analysis artifact, not a substitute for a complete checkpoint.

Checkpoints are compatible only with the same core source/build fingerprint, target, compiler, features, and build settings. Keep the release binary with the experiment; updating/rebuilding the engine can intentionally invalidate an older checkpoint. Checkpoint files stay same-build only, even though trajectories themselves are now identical across platforms (see below). Manifests record the core build ID, checkpoint version, resume path, starting tick, absolute completed tick, and number of steps executed during this invocation.

## Single-run overrides

A fresh single run accepts `--seed N` and repeatable `--set KEY=NUMBER` (the same keys as sweeps), so one job never needs a hand-edited copy of the config. The effective configuration is validated before any artifact is created and is stored in the session as usual. Both options are rejected with `--resume`: a resumed run continues the checkpoint's own configuration.

```bash
cargo run -p sim-runner --release -- --seed 101 --set food.regrowth_rate=0.008 \
  --set creature.hard_cap=2000 --ticks 200000 --checkpoint-every 50000 --output output/validation
```

## Cross-platform fingerprints

`sim-runner fingerprint` prints a 64-bit fingerprint of the complete canonical state at chosen ticks and writes no experiment artifacts. Equal fingerprints mean bit-identical state. `--dump-dir DIR` also writes the full state as JSON, to find which values first differ.

```bash
cargo run -p sim-runner --release -- fingerprint --seed 42 --ticks 0,1,100,2000
node web/demo/tests/conformance.mjs target/wasm32-unknown-unknown/release/sim_wasm.wasm configs/default.toml 42 0,1,100,2000
```

The two commands print the same lines when the native and WASM builds follow the same trajectory; CI fails if they differ. Measured identical on 2026-11-15 through tick 20,000 for macOS (aarch64, parallel), Raspberry Pi 5 Linux (aarch64) and WASM (wasm32, single-threaded). Before that date they differed after one tick, because of platform maths functions and pointer-width random draws; see `crates/sim-core/src/math.rs`.

## Isolated delayed-cue instrument

```bash
cargo run -p sim-runner --release -- delayed-cue --blank-delay 5
```

This seconds-scale check prints one JSON result to stdout and returns nonzero
if its fixed controls fail. It runs no ecosystem and does not load a config or
exported genomes. It validates a synthetic cue-persistence instrument; it is
not evidence of evolved cognition, learning, UAL, or a fitness advantage.

The brain receives exactly one synthetic input. Each of two balanced trials
presents a cue (-1 or +1) for one tick, then `--blank-delay` all-zero blank ticks
(0–10,000), then one all-zero probe tick. The probe is identical in both
conditions, including when the blank delay is zero; it carries no cue, clock or
probe indicator. All state is reset before every trial. Turn is read only at
the probe; a nonnegative response predicts +1, with this fixed tie rule making
a constant-zero response score 50% on the balanced pair.

The hand-built positive control has seven outputs, no hidden nodes, a weight-2
edge from cue input node 1 to turn output node 2, and a weight-2 recurrent
self-loop on turn. It should score 100%. Removing the self-loop gives the
feed-forward control. Clearing state before every tick of the intact network
gives the second control, preserving its wiring; both should score 50%.
The JSON records individual cue/probe responses, predictions, scores and
protocol fields. These deterministic trials are not statistical replicates.
There is no time-dependent decision or learned association: the reference
simply retains either cue's sign through the blank interval.
