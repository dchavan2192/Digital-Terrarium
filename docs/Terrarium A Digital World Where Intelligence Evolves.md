# Terrarium: A Digital World Where Intelligence Evolves

Nov 7, 2026 · @Dhruv Chavan

## Vision and research question

Build a digital terrarium: a sealed world where tiny creatures with evolvable neural brains live, compete, cooperate and evolve, while you set only the conditions. Then run many jars side by side to find which conditions push evolution toward intelligence.

- **Soil, light and water:** food, seasons and energy costs, set in a config file.
- **The sealed glass:** once a run starts, nobody steers; the creatures evolve on their own.
- **Looking through the glass:** the visual viewer, on desktop or in the browser.
- **A shelf of jars:** many runs with one condition changed, which is where the science comes from.

**Core research question:** What environmental conditions (scarcity, seasonality, predation, group living) make larger brains and cooperation evolve?

**Why it matters:** One run is a demo. Thousands of controlled runs turn "how does intelligence emerge?" into a testable question with data, plots and a clear story for a README, a paper or grad applications.

**Name:** *Terrarium* (fallbacks if taken on GitHub: *Vivarium*, *Glassjar*). The repo layout below uses `terrarium`.

## Scope

The sim covers evolution, brains and social behavior; it deliberately skips real chemistry and stops short of civilization.

| Level | In scope? | How it's modeled |
| --- | --- | --- |
| Chemistry / origin of life | No (v1) | Organisms exist from tick 0; a "primordial soup" mode is a stretch goal |
| Evolution | Yes, core | Genome with mutation, reproduction costs energy, limited food |
| Brains | Yes, core | Small evolvable neural nets (fixed topology first, NEAT-style later) |
| Social behavior | Yes | Signals between agents, kin recognition, food sharing |
| Civilization | No | Possible future "promotion" to LLM agents, out of scope here |

**Non-goals for v1:** 3D, realistic physics, multiplayer, GPU compute. Each can come later without a rewrite if the core is clean.

## Decisions

Locked in: Rust, continuous 2D space, a 1000 x 1000 world with 500 to 2,000 creatures, and every tunable number in one config file so a jar's lifespan can be tested against its settings.

| Decision | Choice |
| --- | --- |
| Language | Rust |
| Space | Continuous 2D, wraparound edges |
| World size and population | 1000 x 1000 units, 1,000 creatures at start, hard cap 4,000 |
| Reproduction | Asexual first; sexual reproduction as a later experiment |
| Brain v1 | Fixed feed-forward net, 13 inputs, 8 hidden, 3 outputs; only weights evolve |
| Mutation | Each weight mutates with some probability; the mutation rate is itself a gene |
| Intelligence measures | Brain size, foraging efficiency (energy gained per tick), later signal information |
| First experiment | Food scarcity vs brain size |
| Look | Dark background, dim green food field, creatures as small triangles pointing where they head, colored by lineage |
| License and visibility | MIT, public repo from day one |
| Direction | Portfolio first, built rigorously enough for a later write-up |

### Starting parameters (global config)

Every value lives in `configs/default.toml`, so no number is hard-coded. These are first guesses, to be tuned in Phase 1 until a jar survives 100k ticks.

| Group | Parameter | Start value |
| --- | --- | --- |
| World | Width, height | 1000, 1000 units |
| World | Seed | 42 |
| Food | Field cell size | 10 units (100 x 100 cells) |
| Food | Max food per cell | 1.0 |
| Food | Regrowth per cell per tick | 0.002 |
| Food | Energy per unit of food | 40 |
| Creature | Body radius | 3 units |
| Creature | Max speed, max turn | 2 units/tick, 0.2 rad/tick |
| Creature | Vision | 5 rays over a 120 degree cone, 60 units long |
| Creature | Max age | 5,000 ticks |
| Energy | Starting, maximum | 100, 200 |
| Energy | Base metabolism | 0.1 per tick |
| Energy | Movement cost | 0.05 x speed squared per tick |
| Energy | Brain cost | 0.002 per neuron per tick |
| Reproduction | Threshold, parent pays, child gets | 150, 60, 50 |
| Mutation | Per-weight probability, step size | 0.1, 0.2 |

### Lifespan tracking

The runner records the tick at which a jar goes extinct (or that it survived the full run), alongside the config that produced it. Sweeping one parameter at a time then shows directly which settings let a culture last.

## Tech stack

Recommendation: **Rust**, because it compiles to WebAssembly, so the GitHub repo can link a live browser demo anyone can open with no install.

| Concern | Rust (recommended) | C++ alternative |
| --- | --- | --- |
| Rendering | macroquad (simple) or Bevy (bigger, ECS built in) | raylib |
| Debug UI / sliders | egui | Dear ImGui |
| Parallelism | rayon | OpenMP or std::execution |
| Entity storage | Plain struct-of-arrays, or Bevy ECS | EnTT |
| Config | serde + TOML | toml++ |
| Web demo | wasm32 target, host on GitHub Pages | Emscripten (harder) |
| Build / deps | cargo | CMake + vcpkg |

**Analysis side:** Python (polars or pandas, matplotlib) reads the sim's CSV/Parquet logs and makes plots. Keeps the Rust core focused on speed.

**Key rule:** the simulation core has zero rendering code. It runs **headless** for experiments (fast, many runs in parallel) and with a **viewer** for demos. Same code, same results, same seed.

## Architecture

&#91;embedded content: architecture · 1 core, 2 apps, 3 outputs\]

The viewer and the runner call the same sim-core, so a run watched in the demo and a run in an experiment follow identical rules for the same seed.

## Hardware and deployment

A Raspberry Pi 5 (16 GB RAM, 256 GB storage) runs the always-on jar headless; laptops and phones watch it through the browser viewer.

| Machine | Role | Notes |
| --- | --- | --- |
| Laptop | Development, quick runs, viewer | Code and test here; same Rust code builds for the Pi |
| Raspberry Pi 5 | Always-on jar, long runs, up to 4 jars in parallel | Headless only; a few watts, so it can run 24/7 |
| OSC or laptop (optional) | Large parameter sweeps | 30 seeds x 5 conditions is slow on 4 cores |

### Pi setup

- **Build:** compile natively on the Pi (`cargo build --release`) or cross-compile for aarch64 from the laptop.
- **Run as a service:** a systemd unit starts the runner on boot and restarts it after a crash.
- **Snapshots:** save the full world state every N ticks, so a power cut never loses the evolution so far; the runner resumes from the latest snapshot.
- **Cooling:** use the active cooler; sustained full load otherwise throttles the CPU.
- **Storage:** if the 256 GB is a microSD card, batch and compress writes (Parquet) to limit wear; an NVMe SSD is better if available.

### Live view

- The runner on the Pi serves the world state over a WebSocket (a few updates per second, positions and colors only).
- The browser viewer (the same WASM build) connects and draws it, from a laptop or a phone on the home network.
- Later option: expose it publicly through a tunnel, so repo visitors can peek into a jar that has been evolving for weeks.

### AI HAT (v1)

Not used. It accelerates fixed, pre-compiled vision models, while this sim has thousands of small brains whose weights change every generation; those run faster on the CPU. It also occupies the PCIe slot an NVMe SSD could use.

## World and organism design

Everything runs on energy: organisms spend it to move, think and reproduce, and only food brings it back, so every trait has a cost.

### World

- 2D continuous space (wraparound), with a uniform grid for fast neighbor lookup.
- Food grows on a resource field; regrowth rate and patchiness are tunable.
- Optional environment pressures, each a toggle for experiments: seasons (food cycles), droughts, a slowly shifting climate, terrain zones.
- Fixed timestep and a seeded RNG, so any run replays exactly from its seed.

### Organism

- **Body:** position, heading, energy, age, size.
- **Senses:** a few vision rays (distance + what was hit: food, kin, stranger, predator), own energy, signals heard nearby.
- **Actions:** turn, move speed, eat, attack, reproduce, emit a signal, share energy.
- **Costs:** base metabolism + movement + **brain cost per neuron**. That last one is the key tension: smarter costs more.

### Genome

- Body traits: size, speed, vision range, color (doubles as kin marker).
- Brain: network weights, then later topology.
- Mutation rate is itself a gene, so evolvability can evolve.
- Reproduction: asexual first; sexual (crossover) as a later experiment.

### Brain

1. **v1:** fixed small feed-forward net (≈ 12 inputs, 8 hidden, 7 outputs). Only weights evolve. Easy to make fast.
2. **v2:** NEAT-style growth: neurons and connections can be added or removed, so brain size can actually evolve.
3. **v3:** recurrent connections, giving organisms short-term memory.

### Social layer

- A signal channel: each organism emits a value; neighbors hear it. Meaning is not defined, so any "language" must evolve.
- Energy sharing: giving costs the giver, which is exactly where kin selection and reciprocity can show up.
- Similar color = likely kin, which lets organisms evolve to treat relatives differently.

## Experiments and metrics

Each experiment changes one environment setting, runs about 30 seeds per setting headless, and compares brain size and cooperation at the end.

### What gets logged (every N ticks, per run)

- Population size, births, deaths, mean energy.
- Mean and max brain size (neurons, connections), plus the brain-cost share of energy.
- Cooperation rate: share of energy transfers, and how often they go to kin vs strangers.
- Signal use: whether signals carry information (mutual information between a signal and what the sender sees).
- Lineage tree and species count (clusters of similar genomes).

### First experiment set

| Experiment | Changed setting | Hypothesis |
| --- | --- | --- |
| Scarcity | Food regrowth rate | Moderate scarcity favors bigger brains; extreme scarcity favors small, cheap ones |
| Seasonality | Food cycle amplitude | Changing environments favor memory and larger brains |
| Patchiness | Food clustering | Patchy food favors signaling about food location |
| Predation | Predator species on/off | Predators favor grouping and alarm signals |
| Kin structure | Offspring spawn near vs far from parent | Spawning near kin favors energy sharing |

### Rigor

- Fixed seeds and a config file saved with every result, so any figure can be reproduced.
- Report means with confidence intervals across seeds, never a single run.
- A batch runner script that sweeps parameters across all CPU cores (or OSC, if available).

## Roadmap

&#91;embedded content: roadmap · 6 phases, 5 gates, about 16 weeks\]

The headless runner and a first real result come in phase 3, before social behavior and NEAT, so the project has a finding to show even if later phases slip. Week counts are rough estimates for part-time work alongside classes.

## GitHub presentation

The repo should sell itself in 10 seconds: a moving GIF at the top, a "try it in your browser" link, then one plot that answers the research question.

### Repo layout (Cargo workspace)

```
terrarium/
├── crates/
│   ├── sim-core/      # world, organisms, genome, brain, rules (no graphics)
│   ├── sim-viewer/    # macroquad/Bevy + egui; native and WASM
│   └── sim-runner/    # headless CLI: run configs, sweep params, write logs
├── configs/           # TOML experiment configs
├── analysis/          # Python notebooks and plot scripts
├── docs/              # design doc, results write-up, GIFs
├── .github/workflows/ # CI: tests, clippy, build WASM, deploy Pages
└── README.md
```

### README order

1. Title + one-line pitch ("A digital terrarium where tiny creatures evolve brains") + hero GIF of a jar in motion.
2. "Live demo" link (GitHub Pages, WASM build).
3. The question and the headline result plot.
4. How it works: one diagram + short sections.
5. Quickstart: `cargo run -p sim-viewer` and `cargo run -p sim-runner -- configs/scarcity.toml`.
6. Reproducing the experiments, then contributing and license (MIT or Apache-2.0).

### Visual features worth building

- Color organisms by lineage, so you can watch species rise and fall.
- Click an organism to see its brain lighting up live.
- Live charts panel: population, average brain size, cooperation rate.
- Speed slider and a record-GIF button for easy sharing.

### Terrarium features

- **Jar shelf view:** several runs shown side by side in a grid, each with its conditions labeled, so differences in what evolved are visible at a glance.
- **Build your own jar (web demo):** visitors set food, seasons and predators with sliders, seal the jar, and watch what grows.
- **Share a jar:** a jar's seed and settings fit in a URL, so anyone can open the exact same world.

## Risks and open questions

The biggest risk is a world where nothing interesting evolves; the fix is fast iteration on the rules before any big experiment.

| Risk | Mitigation |
| --- | --- |
| Population explodes or dies out | Tune energy costs early; add a population-health dashboard |
| Evolution stalls (everyone stays simple) | Make the environment demand behavior (moving food, predators) |
| Agents exploit a bug instead of evolving (e.g. infinite energy) | Energy conservation tests in CI |
| Results depend on one lucky seed | Always 30+ seeds and confidence intervals |
| Scope creep (3D, chemistry, LLMs) | Non-goals list in Scope; park ideas in GitHub issues |
| Learning Rust slows progress | Start with macroquad (small API), defer Bevy |

### Open questions

- Rust or C++? Rust is recommended for the WASM demo; C++ if you already know it well.
- Fixed brains first, or go straight to NEAT?
- Target scale for v1: 1,000 organisms? 10,000?
- Is this a portfolio piece only, or aimed at a write-up (workshop paper, arXiv, ALIFE conference)?
