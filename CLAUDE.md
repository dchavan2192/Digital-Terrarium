# Terrarium — Project Guide for Claude Code

## Critical Safety Rules

1. **NEVER delete files or directories** — no `rm`, `rmdir`, `git clean`, or any destructive filesystem operations
2. **NEVER operate outside the project directory** — all work stays within `Digital_Terrarium/`
3. **NEVER force-push or rewrite history** — no `git push --force`, `git reset --hard`, `git rebase`
4. **NEVER push without explicit user permission** — `git push` is denied by default
5. **Web search and fetch are allowed** for research and documentation

## What Is This?

An evolutionary simulation in Rust where creatures with tiny neural-net brains evolve in a 2D toroidal world. Creatures sense food and neighbors via vision rays, make decisions through a feed-forward neural network, and reproduce when they accumulate enough energy. Brains evolve through weight mutation across generations.

**Core research question**: Can meaningful behavioral complexity emerge from simple evolutionary pressure alone?

## Architecture

```
Digital_Terrarium/
├── crates/
│   ├── sim-core/          # Pure simulation logic (no I/O, no rendering)
│   │   ├── src/
│   │   │   ├── lib.rs     # Public API: Config, Simulation
│   │   │   ├── config.rs  # TOML config structs with validation
│   │   │   ├── brain.rs   # Feed-forward neural net (stack-allocated forward pass)
│   │   │   ├── genome.rs  # Genome: brain weights + evolvable mutation params + color
│   │   │   ├── organism.rs # Organism struct
│   │   │   ├── world.rs   # FoodGrid + SpatialGrid (counting-sort, zero-alloc rebuild)
│   │   │   └── sim.rs     # Tick loop: spatial → sense → think → act → reproduce → die → regrow
│   │   └── tests/
│   │       └── integration.rs  # 10 integration tests
│   ├── sim-viewer/        # macroquad + egui visual renderer
│   │   └── src/
│   │       ├── main.rs    # Window setup + main loop
│   │       ├── camera.rs  # Pan/zoom camera
│   │       ├── renderer.rs # Food grid, creatures, selection
│   │       └── ui.rs      # egui side panel (stats, controls, inspector)
│   └── sim-runner/        # Headless CLI for experiments
│       └── src/
│           ├── main.rs    # clap CLI: single run + sweep subcommand
│           ├── run.rs     # Single simulation run with CSV logging
│           └── sweep.rs   # Parallel parameter sweep via rayon
├── configs/
│   └── default.toml       # All tunable parameters — source of truth
├── .github/workflows/
│   └── ci.yml             # CI: check, clippy, test, fmt, WASM build
├── CLAUDE.md              # This file
└── README.md
```

## Key Design Principles

- **Deterministic**: Same config + seed = same run (ChaCha8Rng)
- **Zero-alloc hot path**: Reusable buffers, stack arrays, no heap allocation per tick
- **Config-driven**: Every constant lives in `configs/default.toml`, not in code
- **Separation**: sim-core has no I/O — rendering and CLI are separate crates

## How to Run

```bash
# Run all tests
cargo test --workspace

# Run viewer (visual)
cargo run --release -p sim-viewer

# Run headless simulation
cargo run --release -p sim-runner -- --ticks 10000

# Parameter sweep
cargo run --release -p sim-runner -- sweep --param food.regrowth_rate --values 0.001,0.002,0.005 --seeds 3 --ticks 5000

# Format + check
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

## Commit Checklist

Before every commit:
1. `cargo test --workspace` — all 10 tests pass
2. `cargo clippy --workspace -- -D warnings` — no warnings
3. `cargo fmt --all --check` — formatted
4. Commit message: imperative mood, concise, explains *why*

## Performance Notes

- Current: ~195 ticks/s at 4000 population
- SpatialGrid uses counting-sort rebuild (two-pass, flat index buffer)
- Vision uses dot-product cone test (no atan2)
- Brain forward pass uses `[f64; MAX_HIDDEN]` stack arrays
- Config values are hoisted out of inner loops

## Tech Stack

- **Rust 2024 edition** — `gen` is reserved, use `rng.r#gen::<f64>()`
- **rand 0.8 + rand_chacha** — deterministic RNG
- **serde + toml** — config serialization
- **macroquad 0.4 + egui 0.31 + egui-macroquad 0.17** — viewer
- **clap 4** — CLI parsing
- **csv 1** — stats output
- **rayon 1** — parallel sweeps

## Simulation Tick Pipeline

1. Rebuild spatial index (counting sort)
2. For each organism: sense (vision rays) → think (brain forward) → decide action
3. Apply actions: turn, move, eat
4. Metabolize: deduct base + movement + brain costs
5. Reproduce: energy > threshold → spawn mutated offspring
6. Die: energy ≤ 0 or age ≥ max_age
7. Regrow food

## Neural Network

- **Topology**: 13 inputs → 8 hidden (tanh) → 3 outputs (tanh)
- **Inputs**: 5 vision rays × 2 (distance + type) + energy + heading + signal
- **Outputs**: turn [-1,1], speed [0,1], eat (>0 = try to eat)
- **Evolution**: Weight mutation only (no topology changes yet)

## Config Reference

All parameters in `configs/default.toml`. Key ones:
- `food.regrowth_rate` — primary resource pressure knob
- `energy.brain_cost` — the "intelligence tax"
- `creature.hard_cap` — population ceiling
- `mutation.weight_probability` / `weight_step` — evolution speed

## Current State

- All 3 crates built and working
- 10 integration tests passing
- CI pipeline running (GitHub Actions)
- Performance optimized (~195 ticks/s at 4000 pop)
