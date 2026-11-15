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
│   │   │   ├── world.rs   # FoodGrid (biomes, seasons, droughts), Pheromone/Elevation/Construction grids, SpatialGrid, wraparound math
│   │   │   ├── organism.rs # Organism struct, lineage ids (root/clade), entropy tracking
│   │   │   ├── genome.rs  # NEAT genome: connections, hidden nodes, crossover, methylation, traits, distance
│   │   │   ├── brain.rs   # NEAT feed-forward net (variable topology), stack-allocated forward pass, in-place weight refresh
│   │   │   ├── activity.rs # OEE activity stats (Bedau & Packard): topology hash, novelty, cumulative activity
│   │   │   ├── view.rs    # Read-only view helpers for the WASM front end
│   │   │   └── sim.rs     # Simulation state + deterministic tick loop + species clustering
│   │   └── tests/
│   │       └── integration.rs  # 43 tests: config, determinism, energy, bounds, biomes, seasons, Phase 7/7.5 features
│   ├── sim-wasm/          # C-ABI WASM bindings for the Three.js web demo (Phase 8)
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
│   ├── default.toml       # All tunable parameters — THE source of truth for numbers
│   └── intense.toml       # Harsher variant
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
4. **Energy is the universal currency.** Moving, thinking, reproducing — everything costs energy. Food is the only source; no transfer (sharing, combat, reproduction) may create energy.
5. **Intelligence has a price.** Brain cost per neuron per tick is the key evolutionary tension.

## Code Conventions

- **Rust 2024 edition.** The keyword `gen` is reserved — use `r#gen` when calling `rand::Rng::gen`.
- **`cargo fmt --all`** before every commit. CI enforces this.
- **`cargo clippy -- -D warnings`** must pass. CI treats warnings as errors.
- **`cargo test --workspace`** must pass. Currently 47 integration tests + 4 activity.rs unit tests + 1 doc test.
- **Brain I/O is fixed by the sensor layout**: `brain.inputs = 2 × ray_count + 9` (19 with 5 rays) and `brain.outputs = 7`. `Config::validate` rejects anything else.
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
5. Add `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` to the commit message
6. Do NOT push unless the user explicitly asks

## Simulation Tick Pipeline

1. Rebuild spatial index (counting-sort)
2. For each organism (rayon-parallel, read-only, no RNG): sense (vision rays, kin similarity, signal, temperature, pheromone, light, niche level) → think (NEAT brain forward) → produce Action
3. Apply actions: turn, move (elevation slope penalty), eat algae/fruit, metabolize; epigenetic methylation under stress (energy < `stress_repro_fraction × reproduction.threshold`) and in-place refresh of the living brain's weights `w·(1−m)`
4. Pheromone deposit + decay; niche construction deposit/degrade + decay
5. Combat: attack nearest neighbor, size-scaled damage, 75% energy transfer
6. Sharing: donate energy to nearest neighbor (recipient gets `transfer_efficiency` ≤ 1 of it — energy is conserved; kin benefit must come from energy being worth more to the recipient)
7. Reproduce: energy > threshold → asexual clone or sexual NEAT crossover. In sexual births the mate pays `mate_cost` too (twofold cost of sex), and each organism mates at most once per tick. Child inherits methylation + Gaussian epimutation noise
8. Die: energy <= 0, max age, or Gompertz-Makeham probabilistic mortality
9. Spawn/expire drought zones (stochastic localized food crashes)
10. Regrow food (per-cell rate × biome × season × drought suppression × niche-construction multiplier)
11. Stats: species clustering, trophic levels, entropy, OEE activity (every `window_size` ticks), windowed lineage diversity (every `lineage_window` ticks)

## Neural Network (NEAT)

- **Topology**: Variable — starts fully connected (19→8→7), evolves via add-connection (2%), add-node (1%), toggle (1%), prune-connection (4%); all in `[mutation]`
- **19 Inputs**: 5 vision rays × 2 (distance + type) + energy + heading + signal + body_size + kin_similarity + temperature + pheromone + light + niche_level
- **7 Outputs**: turn [-1,1], speed [0,1], eat, attack, signal [-1,1], share, construct [-1,1]
- **Epigenetics**: effective weight = `weight × (1 − methylation)`, applied both at birth and to the living brain
- **Activation**: fast_tanh (Pade approximant, ~3-5x faster than std)
- **Forward pass**: Stack-allocated `[f64; MAX_NODES]`, single linear pass through pre-sorted connections

## Key Experiment Results (November 2026)

**COOPERATION RESULT WITHDRAWN (2026-11-15)** — the v2 "87% cooperation" finding came from an energy-creation bug: sharing paid recipients 1.5× the donor's cost, so neighbours sharing back and forth gained free energy. Sharing is now energy-conserving (`transfer_efficiency` ≤ 1, enforced by `Config::validate` and the `energy_never_increases_without_food` test). Cooperation experiments must be re-run.

**Predation partially emerged** — trophic level shifted from 1.0 to 1.1-1.3 after combat rebalance (energy_per_food 30→15, transfer_ratio 0.5→0.75).

**Brain cost unsolved** — connections converge to ~180 regardless of cost. Need connection-proportional cost model.

## Config Parameters Reference

All in `configs/default.toml`. Key ones:

| Path | Default | Purpose |
|------|---------|---------|
| `food.regrowth_rate` | 0.0005 | Food scarcity — the main experimental lever |
| `food.energy_per_food` | 15.0 | Energy per food cell (halved from 30 to enable predation) |
| `food.droughts.spawn_rate` | 0.0002 | ~1 drought per 5000 ticks |
| `food.droughts.severity` | 0.05 | Regrowth drops to 5% in drought zones |
| `energy.brain_cost` | 0.001 | Intelligence tax per neuron per tick |
| `energy.connection_cost` | 0.001 | Per-synapse cost per tick (Attwell & Laughlin 2001) |
| `energy.activity_weight_cost` | 0.0005 | Cost per unit of |weight × activation| |
| `energy.base_metabolism` | 0.3 | Flat survival cost (× body_size^0.75) |
| `combat.energy_transfer_ratio` | 0.75 | Predator absorbs 75% of damage as energy |
| `sharing.transfer_efficiency` | 0.9 | Recipient gets 90% of what donor pays; validated ≤ 1 (energy conservation). Legacy key `benefit_multiplier` must also be ≤ 1 |
| `reproduction.threshold` | 60 | Energy needed to reproduce |
| `reproduction.sexual` | true | NEAT crossover with a nearby mate |
| `reproduction.mate_cost` | 30 (= parent_cost) | Mate's cost per sexual birth — twofold cost of sex |
| `creature.hard_cap` | 500 | Max living creatures |
| `speciation.dynamic` | true | Auto-compute species threshold from genome variance |
| `epigenetics.stress_repro_fraction` | 0.25 | Stressed when energy < 0.25 × reproduction.threshold |
| `niche_construction.regrowth_bonus` | 0.5 | Cell regrowth × (1 + level × bonus) |
| `activity_tracking.lineage_window` | 1000 | Window for lineage diversity |

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
| Kin selection | Hamilton (1964) — to be re-tested with energy-conserving sharing |
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

1. **Phase 7 experiments not yet run** — Phase 7.5 correctness fixes (docs/PROGRESS.md) must land first; the Pi runs then measure sexual reproduction, niche construction and epigenetics ON/OFF
2. **Crossover only mixes weights** (audit item G) — the child's edge set equals the fitter parent's; hidden-node ids are not global innovation numbers
3. **No neutral shadow model for OEE** — activity stats still lack a neutral baseline (design: docs/research/OEE_SHADOW_DESIGN.md)
4. **Brain cost vs complexity** — connection and activity costs exist now; whether they constrain topology is untested
5. **Droughts not yet tested in sweeps** — need experiment data
6. **Elevation is the only barrier** — the world is still toroidal; no hard barriers for allopatric speciation
7. **Duplicate directory** — `Digital_Terrarium\` (with backslash) is a stale copy; the owner will clean it up

## Adding New Features — Checklist

1. Does it need a new config parameter? → Add to `configs/default.toml` AND `config.rs`
2. Does it affect the tick loop? → Keep it allocation-free, maintain determinism
3. Does it add a new stat? → Add field to `TickStats` (with `serde::Serialize`)
4. Does it change organism behavior? → Update sensor inputs if needed (brain input count must match `config.brain.inputs`)
5. Write or update tests in `integration.rs`
6. Run the full CI check before committing
