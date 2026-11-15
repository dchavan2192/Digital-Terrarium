Work autonomously through the roadmap in docs/DESIGN.md, starting from the
first phase that isn't finished (Phase 1 is done and tuned).

For each phase:
1. Research first: look up how established artificial-life projects and papers
   handle this phase (e.g. NEAT, Avida, Polyworld, Lenia, The Bibites), and
   write what you learned and what you chose in docs/research/phase-N.md.
2. Implement it, with every tunable number in configs/default.toml.
3. Add tests and run cargo test and cargo clippy.
4. Check the phase's gate from the roadmap with the headless runner across
   several seeds. Only move on when the gate passes.
5. Commit locally with a clear message. Do not push to GitHub.

Never wait for input. If unsure, make the best decision and note why in
NIGHT_LOG.md. If a gate can't be passed after real effort, stop there and
explain what you tried. Keep a running log of progress in NIGHT_LOG.md.