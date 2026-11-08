# Terrarium

**A digital terrarium where tiny creatures evolve brains.**

<!-- TODO: Replace with a recorded GIF of the viewer once the sim is tuned -->
<!-- ![Terrarium in action](docs/hero.gif) -->

Terrarium is a sealed digital world where creatures with small evolvable neural networks live, compete, cooperate, and evolve — while you set only the conditions. Run one jar and watch evolution unfold, or run thousands headless to answer a research question with data.

> **Core question:** What environmental conditions — scarcity, seasonality, predation, group living — make larger brains and cooperation evolve?

## How it works

Each creature has a **body** (position, energy, age), **senses** (vision rays, own energy), and a **brain** (feed-forward neural net whose weights are inherited and mutated). Every tick:

1. **Sense** — cast vision rays to detect food and other creatures
2. **Think** — run the neural network to decide: turn, speed, eat
3. **Act** — move, consume food from the grid, pay energy costs
4. **Reproduce** — if energy exceeds a threshold, spawn a mutated offspring
5. **Die** — from starvation or old age

Intelligence has a price: every neuron costs energy per tick. Bigger brains must earn their keep.

### Architecture

```
sim-core       Headless simulation engine — world, organisms, genome, brain, rules
sim-viewer     macroquad + egui visual viewer (native + future WASM)
sim-runner     Headless CLI for experiments — single runs and parallel parameter sweeps
```

The viewer and the runner call the same `sim-core`, so a run watched live and a run in an experiment follow identical rules for the same seed.

## Quickstart

### Requirements

- [Rust](https://rustup.rs/) (stable)

### Watch the simulation

```bash
cargo run -p sim-viewer --release
```

Controls: **scroll** to zoom, **right-drag** to pan, **space** to pause, **click** a creature to inspect it.

### Run a headless experiment

```bash
# Single run — 100k ticks, log stats every 100 ticks
cargo run -p sim-runner --release -- \
  --config configs/default.toml \
  --ticks 100000

# Parameter sweep — vary food scarcity across 30 seeds per setting
cargo run -p sim-runner --release -- sweep \
  --config configs/default.toml \
  --param food.regrowth_rate \
  --values 0.0005,0.001,0.002,0.005,0.01 \
  --seeds 30 \
  --ticks 100000
```

Results land in `output/` as CSV files with a copy of the config for full reproducibility.

## Configuration

Every tunable number lives in one TOML file — no magic constants in the code. Copy and tweak for experiments:

```bash
cp configs/default.toml configs/my_experiment.toml
# Edit, then run:
cargo run -p sim-viewer --release -- configs/my_experiment.toml
```

Key parameters:

| Group | Parameter | Default | What it does |
|-------|-----------|---------|-------------|
| Food | `regrowth_rate` | 0.002 | How fast food grows back — controls scarcity |
| Energy | `brain_cost` | 0.002/neuron/tick | The intelligence tax |
| Energy | `base_metabolism` | 0.1/tick | Cost of staying alive |
| Reproduction | `threshold` | 150 | Energy needed to reproduce |
| Mutation | `weight_probability` | 0.1 | Chance each brain weight mutates |
| Creature | `max_age` | 5000 ticks | Lifespan cap |

See [`configs/default.toml`](configs/default.toml) for the full list.

## Project structure

```
terrarium/
├── crates/
│   ├── sim-core/          # World, organisms, genome, brain, tick loop
│   │   ├── src/
│   │   │   ├── config.rs  # Typed config from TOML with validation
│   │   │   ├── world.rs   # Food grid, spatial hash, wraparound math
│   │   │   ├── organism.rs
│   │   │   ├── genome.rs  # Heritable traits, evolvable mutation rate
│   │   │   ├── brain.rs   # Feed-forward neural net (13→8→3)
│   │   │   └── sim.rs     # Deterministic tick loop
│   │   └── tests/
│   ├── sim-viewer/        # macroquad + egui visual app
│   └── sim-runner/        # Headless CLI with parallel sweeps (rayon)
├── configs/               # TOML experiment configs
├── analysis/              # Python notebooks and plot scripts (coming soon)
└── docs/                  # Design doc
```

## Experiments

The first experiment set (from the [design doc](docs/)):

| Experiment | Swept parameter | Hypothesis |
|-----------|----------------|-----------|
| Scarcity | `food.regrowth_rate` | Moderate scarcity favors bigger brains; extreme scarcity favors small, cheap ones |
| Seasonality | Food cycle amplitude | Changing environments favor memory and larger brains |
| Patchiness | Food clustering | Patchy food favors signaling about food location |
| Predation | Predator species on/off | Predators favor grouping and alarm signals |
| Kin structure | Offspring spawn distance | Spawning near kin favors energy sharing |

Each experiment runs 30+ seeds per setting and reports means with confidence intervals.

## Roadmap

- [x] Core simulation engine (world, food, organisms, brain, genome)
- [x] Visual viewer with egui controls and organism inspector
- [x] Headless runner with CSV logging and parallel parameter sweeps
- [ ] Performance optimization (target: 200+ ticks/s at 4000 pop)
- [ ] Parameter tuning (stable 100k-tick jars)
- [ ] CI pipeline (tests, clippy, WASM build)
- [ ] WASM build + GitHub Pages live demo
- [ ] NEAT-style brain topology evolution (v2)
- [ ] Social layer: signals, energy sharing, kin recognition
- [ ] Python analysis scripts and first experiment results

## License

[MIT](LICENSE)
