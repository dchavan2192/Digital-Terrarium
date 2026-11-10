# Terrarium — Project Guide for Autonomous Agents

You are working on **Terrarium**, an evolutionary simulation in Rust where creatures with evolvable neural-net brains live, compete, cooperate, and evolve in a sealed 2D world.

## Critical Safety Rules

**READ THESE FIRST. Violations are not recoverable.**

1. **NEVER delete files or directories.** No `rm`, `git rm`, `fs::remove`, or any destructive file operation. If something is unused, leave it — the owner will clean up.
2. **NEVER operate outside this project directory.** All file reads, writes, and shell commands must target paths under this repo root. No touching `~/`, `/tmp`, other projects, or system files.
3. **NEVER force-push, reset --hard, or rewrite git history.** Only create new commits. Never amend published commits.
4. **NEVER commit secrets, credentials, or .env files.**
5. **NEVER push to remote unless explicitly asked.** Commits are fine, pushes require explicit user instruction.
6. **NEVER skip pre-commit hooks** (no `--no-verify`).

## Architecture

```
terrarium/
├── crates/
│   ├── sim-core/          # Headless simulation engine (THE core — no graphics)
│   │   ├── src/
│   │   │   ├── lib.rs     # Public API: re-exports Config, Simulation, SpeciesInfo
│   │   │   ├── config.rs  # Typed TOML config with validation
│   │   │   ├── world.rs   # FoodGrid (biomes, seasons, droughts), SpatialGrid, wraparound math
│   │   │   ├── organism.rs # Organism struct, entropy tracking
│   │   │   ├── genome.rs  # NEAT genome: connections, hidden nodes, pruning, traits, distance
│   │   │   ├── brain.rs   # NEAT feed-forward net (variable topology), stack-allocated forward pass
│   │   │   └── sim.rs     # Simulation state + deterministic tick loop + species clustering
│   │   └── tests/
│   │       └── integration.rs  # 22 tests: config, determinism, energy, bounds, biomes, seasons
│   ├── sim-viewer/        # macroquad + egui visual app (native + WASM)
│   │   └── src/
│   │       ├── main.rs    # Entry point, main loop, click-to-select, WASM seed randomization
│   │       ├── camera.rs  # Pan/zoom camera
│   │       ├── renderer.rs # Food grid + creature triangles + drought zone rendering
│   │       └── ui.rs      # egui side panel: stats, species panel, controls, inspector
│   └── sim-runner/        # Headless CLI for experiments
│       └── src/
│           ├── main.rs    # CLI with clap: single run or sweep subcommand
│           ├── run.rs     # Single-run logic, CSV stats output
│           └── sweep.rs   # Parallel parameter sweep via rayon
├── configs/
│   └── default.toml       # All tunable parameters — THE source of truth for numbers
├── analysis/              # Python scripts for experiment analysis
│   ├── analyze_experiments.py  # Sweep analysis with per-experiment figures
│   └── plot_readme.py     # README dashboard plots
├── docs/
│   ├── EXPERIMENT_FINDINGS.md  # v1/v2 experiment results and analysis
│   └── PROGRESS.md        # Development progress checklist
├── web/                   # WASM host page for GitHub Pages live demo
│   └── index.html
├── output/                # Generated CSV/TOML from runs (gitignored)
├── .github/workflows/
│   ├── ci.yml             # Tests, clippy, fmt, WASM build
│   └── deploy-pages.yml   # WASM build + GitHub Pages deployment
└── README.md
```

## Key Design Principles

1. **sim-core has ZERO rendering code.** It runs identically headless or with the viewer.
2. **Same seed = same run.** Deterministic via `ChaCha8Rng`. Never break this invariant.
3. **Every number in config, not code.** All tunable parameters live in `configs/default.toml`. Never hard-code simulation constants.
4. **Energy is the universal currency.** Moving, thinking, reproducing — everything costs energy. Food is the only source.
5. **Intelligence has a price.** Brain cost per neuron per tick is the key evolutionary tension.

## Code Conventions

- **Rust 2024 edition.** The keyword `gen` is reserved — use `r#gen` when calling `rand::Rng::gen`.
- **`cargo fmt --all`** before every commit. CI enforces this.
- **`cargo clippy -- -D warnings`** must pass. CI treats warnings as errors.
- **`cargo test --workspace`** must pass. Currently 22 integration tests + 1 doc test.
- Keep `#[inline]` on hot-path functions in world.rs and brain.rs.
- Stack-allocate in the tick loop. No `Vec` allocations per organism per tick — use fixed-size arrays with `MAX_*` constants.
- Prefer `.min()` / `.max()` / `.clamp()` over if-else for bounds.

## How to Run

```bash
# Watch the simulation
cargo run -p sim-viewer --release

# Headless single run (100k ticks)
cargo run -p sim-runner --release -- --config configs/default.toml --ticks 100000

# Parameter sweep
cargo run -p sim-runner --release -- sweep \
  --config configs/default.toml \
  --param food.regrowth_rate \
  --values 0.001,0.002,0.005 \
  --seeds 5 --ticks 200000

# Full CI check (what the pipeline runs)
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

## Before Committing

1. `cargo fmt --all`
2. `cargo clippy --workspace -- -D warnings` — zero warnings
3. `cargo test --workspace` — all tests pass
4. Write a clear commit message describing *why*, not just *what*
5. Add `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>` to the commit message
6. Do NOT push unless the user explicitly asks

## Simulation Tick Pipeline

1. Rebuild spatial index (counting-sort)
2. For each organism: sense (vision rays + kin similarity + signal) → think (NEAT brain forward) → produce Action
3. Apply actions: turn, move, eat food
4. Combat: attack nearest neighbor, size-scaled damage, 75% energy transfer
5. Sharing: donate energy to nearest neighbor (recipient gets 1.5x — Hamilton's rule)
6. Reproduce: energy > threshold → spawn mutated NEAT offspring nearby
7. Die: energy <= 0, max age, or Gompertz-Makeham probabilistic mortality
8. Spawn/expire drought zones (stochastic localized food crashes)
9. Regrow food (per-cell rates × biome multiplier × seasonal oscillation × drought suppression)

## Neural Network (NEAT)

- **Topology**: Variable — starts fully connected (15→8→6), evolves via add-node (1%), add-connection (3%), prune-connection (2%)
- **15 Inputs**: 5 vision rays × 2 (distance + type) + energy + heading + signal + body_size + kin_similarity
- **6 Outputs**: turn [-1,1], speed [0,1], eat, attack, signal [-1,1], share
- **Activation**: fast_tanh (Pade approximant, ~3-5x faster than std)
- **Forward pass**: Stack-allocated `[f64; MAX_NODES]`, single linear pass through pre-sorted connections

## Key Experiment Results (November 2026)

**COOPERATION EVOLVED** — v2 experiments confirmed Hamilton's kin selection:
- `sharing.benefit_multiplier = 1.5` → 87% cooperation rate at `share_amount=5`
- Kin share ratio ~1.0 (virtually all sharing between relatives)
- Sharing populations 3x larger than non-sharing (1500 vs 450)
- See `docs/EXPERIMENT_FINDINGS.md` for full analysis

**Predation partially emerged** — trophic level shifted from 1.0 to 1.1-1.3 after combat rebalance (energy_per_food 30→15, transfer_ratio 0.5→0.75).

**Brain cost unsolved** — connections converge to ~180 regardless of cost. Need connection-proportional cost model.

## Config Parameters Reference

All in `configs/default.toml`. Key ones:

| Path | Default | Purpose |
|------|---------|---------|
| `food.regrowth_rate` | 0.0015 | Food scarcity — the main experimental lever |
| `food.energy_per_food` | 15.0 | Energy per food cell (halved from 30 to enable predation) |
| `food.droughts.spawn_rate` | 0.0002 | ~1 drought per 5000 ticks |
| `food.droughts.severity` | 0.05 | Regrowth drops to 5% in drought zones |
| `energy.brain_cost` | 0.002 | Intelligence tax per neuron per tick |
| `energy.base_metabolism` | 0.2 | Flat survival cost |
| `combat.energy_transfer_ratio` | 0.75 | Predator absorbs 75% of damage as energy |
| `sharing.benefit_multiplier` | 1.5 | Recipient gets 1.5x what donor pays (Hamilton's rule) |
| `reproduction.threshold` | 60 | Energy needed to reproduce |
| `creature.hard_cap` | 1500 | Max living creatures |
| `speciation.dynamic` | true | Auto-compute species threshold from genome variance |

## Biology Citations

Every mechanic is research-backed:

| Mechanic | Citation |
|----------|----------|
| Mortality | Gompertz (1825), Williams (1957), Kirkwood (1977) |
| Metabolism | Kleiber (1932) — body_size^0.75 scaling |
| Combat | Lotka (1925), Volterra (1926) |
| Trophic levels | Lindeman (1942) |
| Speciation | Stanley & Miikkulainen (2002) — NEAT compatibility distance |
| Intelligence cost | Aiello & Wheeler (1995), Laughlin et al. (1998) |
| Kin selection | Hamilton (1964) — rB > C confirmed in experiments |
| Cooperation | Trivers (1971) — reciprocal altruism |
| Density pressure | Allee (1931) |
| Seasonal cycles | Botkin (1990) |
| Droughts | Holling (1973), Connell (1978) — intermediate disturbance hypothesis |
| Spatial niches | Schluter (2000), Rosenzweig (1995) |
| Brain entropy | Shannon (1948), Beer & Williams (2015) |

## Remote Experiment Infrastructure

- **Pi 5** at `pi@100.127.61.93` (Tailscale SSH), project at `~/Digital-Terrarium`
- Experiments run via `nohup` — survive SSH disconnects
- Use `sim-runner sweep` for parameter sweeps, outputs to `output/`
- Analysis: `analysis/.venv/bin/python analysis/analyze_experiments.py --remote pi@100.127.61.93`

## Unsolved Problems / Next Steps

1. **Brain cost doesn't constrain complexity** — needs connection-proportional cost, not neuron-proportional
2. **Environment too easy** — all runs hit hard cap, need harsher conditions
3. **Droughts not yet tested in sweeps** — added post-v2, need experiment data
4. **No sexual reproduction** — NEAT crossover would accelerate adaptation
5. **No physical barriers** — toroidal world prevents population isolation for speciation
6. **Duplicate directory** — `Digital_Terrarium\` (with backslash) is a stale copy, safe to delete

## Adding New Features — Checklist

1. Does it need a new config parameter? → Add to `configs/default.toml` AND `config.rs`
2. Does it affect the tick loop? → Keep it allocation-free, maintain determinism
3. Does it add a new stat? → Add field to `TickStats` (with `serde::Serialize`)
4. Does it change organism behavior? → Update sensor inputs if needed (brain input count must match `config.brain.inputs`)
5. Write or update tests in `integration.rs`
6. Run the full CI check before committing
