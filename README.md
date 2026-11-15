# Terrarium

**An artificial-life laboratory where creatures evolve neural controllers in an energy-limited world.**

[**Live 3D demo**](https://dchavan2192.github.io/Digital-Terrarium/) · [Development plan](docs/PROGRESS.md) · [Results register](docs/RESULTS.md) · [Scientific review](docs/research/SCIENCE_COUNCIL_2026-11-15.md)

Watch a creature, inspect its brain, follow its lineage, or run controlled experiments headlessly. The Rust engine powers both the browser demo and the experimental runner.

> **Research question:** Which ecological conditions favor adaptive behavior, costly neural processing, and different forms of inheritance?

## What is implemented

A continuous 2D toroidal world contains algae, fruit, biomes, seasons, droughts, and terrain. Creatures have evolvable body traits and small feed-forward neural networks. With five vision rays, the current controller has **20 inputs and 7 outputs**: turn, speed, eat, attack, signal, share, and construct.

- **Energy economics:** food supplies energy; movement, neural processing, maintenance, and construction spend it. Feeding reduces movement speed by a configurable factor.
- **Predation and sharing:** attacks transfer a fraction of prey energy. Sharing transfers energy with configurable efficiency at most 1; it cannot amplify energy.
- **Evolution:** weights, connections, hidden nodes, and body traits mutate. Sexual reproduction combines matching genes, with asexual fallback. Historical node identity remains an open crossover issue.
- **Environmental modification:** construction changes local food regrowth and decays over time.
- **Epigenetic state:** low reserves increase connection methylation, suppressing effective weights; marks can be inherited and later decay. Whether this is adaptive is an experimental question.
- **Observation:** the web viewer provides an inspector, neural graph, data layers, and event feed. The headless runner records statistics across seeds and parameter settings.

The model is biologically inspired. Genome-distance clusters are operational species labels, hue similarity is a kin proxy, and the trophic statistic is an energy-source proxy. These are not direct demonstrations of reproductive isolation, genealogical relatedness, or a reconstructed food web. Network size and output entropy do not establish intelligence.

## Evidence status

**No current-model scientific result is established yet.** Earlier experiments used rules with energy-amplifying sharing; some also predated fruit and methylation fixes. Their plots and interpretations are historical records, not validated findings about the corrected engine.

In particular, the earlier “87% cooperation” and logarithmic activity-growth claims have been withdrawn as evidence. Activity accumulating during a finite run does not prove open-ended evolution. A neutral comparison and behavioral evaluation are planned; the current implementation has finite network-size limits.

The [results register](docs/RESULTS.md) defines the evidence required before a numerical claim enters this README. Historical material remains in [experiment findings](docs/EXPERIMENT_FINDINGS.md), [progress history](docs/PROGRESS.md), and Git history. The browser demo uses a presentation configuration; it is not an experimental control.

## Architecture

```text
sim-core    Rendering-independent world, organisms, neural controllers, evolution, statistics
sim-runner  Headless runs, CSV logging, and parallel parameter sweeps
sim-wasm    Thin C-ABI bridge from sim-core to the web viewer
sim-viewer  Native macroquad + egui development viewer
web/demo    Three.js visualization and HTML/CSS interface
analysis    Python plotting and experiment analysis
```

Seeded randomness supports repeatability for the same code, configuration, build environment, and execution mode. Bitwise agreement across Mac, Pi, and browser has not been established. The same seed across feature ablations also does not guarantee identical drought schedules while environment and organism events share a random stream.

## Quickstart

Install [Rust](https://rustup.rs/) stable.

```bash
# Native viewer
cargo run -p sim-viewer --release

# Short headless smoke run
cargo run -p sim-runner --release -- \
  --config configs/default.toml --ticks 1000 --output output/smoke

# Pilot sweep; choose a fresh output directory for each experiment
cargo run -p sim-runner --release -- sweep \
  --config configs/default.toml \
  --param food.regrowth_rate --values 0.003,0.006 \
  --set creature.hard_cap=2000 --seeds 3 --ticks 20000 \
  --output output/pilot
```

Long experiments in this project's workflow run on the Raspberry Pi 5. The [Pi launcher](analysis/launch_pi_phase9a.sh) and CLI `--help` describe the current runner options. Do not treat pilot configurations as validated defaults.

Native controls: scroll to zoom, right-drag to pan, space to pause, click to inspect. The browser viewer has its own visible controls and guided tour. GitHub Pages builds the WASM engine from the same repository; the demo header identifies its deployed revision.

```bash
# Development checks
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace

# Plot a CSV, using the path produced by the runner
python analysis/plot_readme.py path/to/stats.csv
```

Plotting requires a Python environment with matplotlib, pandas, and scipy. Generated run data stays in the ignored `output/` directory.

## Configuration and research method

Start with [default.toml](configs/default.toml), or the separate [demo.toml](configs/demo.toml). Important controls include food regrowth, neural structure/activity costs, reproduction investment, transfer efficiency, construction cost, and environmental variability. The defaults are modeling choices undergoing calibration, not measured biological constants.

The staged plan is to validate energy accounting and mechanism effects, establish a food-limited baseline, add persistence and evaluation, and then test mechanisms individually before studying interactions. Memory and lifetime learning are future work. Informative null results count as progress.

Primary methods and prior art include [NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf), [Polyworld's selection-versus-drift comparison](https://arxiv.org/html/1112.4906), [MODES metric definitions](https://doi.org/10.1162/artl_a_00280), and the [ODD model-description protocol](https://www.jasss.org/23/2/7.html). See the [prior-art review](docs/research/PRIOR_ART.md) for boundaries on novelty claims and the [scientific council](docs/research/SCIENCE_COUNCIL_2026-11-15.md) for design decisions and unresolved alternatives.

## License

[MIT](LICENSE)
