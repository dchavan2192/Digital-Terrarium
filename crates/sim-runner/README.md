# Experiment artifacts

Every single run or sweep creates a new `session_<unix-nanoseconds>_<pid>_<counter>` directory beneath `--output`. It never appends to or truncates an earlier run. Sweeps validate **every generated configuration** after overrides before creating that session or starting jobs. Integer parameters require integers; toggles require 0 or 1. Values that collide in the six-decimal output labels are rejected.

Single-run artifacts live directly in the session; sweep artifacts live in `session_.../sweep/`. Existing `stats_seed...csv` and `config_seed...toml` basenames are preserved. Each run also writes an adjacent `manifest_seed...toml`: full configuration, compiled source revision and dirty flag, compiler, target, profile, schema/hash version, seed, requested/completed ticks, log interval, elapsed time, and completion status.

- `completed`: requested duration reached and artifacts flushed.
- `extinct`: finished early because the population died out; this is an experimental outcome, not execution failure.
- `failed`: caught simulation panic or artifact I/O error; the CLI returns a nonzero exit status.
- `running`, absent, or malformed manifest: incomplete/unverified; never treat this as a finished result. Abrupt process termination or power loss may leave this state. Manifest updates are not yet atomic checkpoints.

The stats CSV begins after the first simulation step, avoiding the old synthetic tick-zero all-zero row, and always includes the actual final tick even when it is not a log interval boundary. Source provenance is captured at build time, so running the binary from another checkout does not relabel it. A dirty build is explicitly marked and should not be used for a published result without preserving its source.

The Phase 9a Pi script uses `pi@dchavan`, defaults to two worker threads, refuses to overwrite tracked remote edits or launch over an active simulation, and creates `output/phase9a/launch_.../session_.../`. Its `status`, `exit_code`, and `launcher.log` describe the detached job. The Phase 7 script similarly creates `output/phase7/launch_.../{sex,niche,epi}/session_.../sweep/`; this is four targeted arms, not a factorial study. Point analysis at the specific launch/session, rather than mixing repeated experiments.

**Remaining scientific limitation:** the environment still shares an RNG stream with evolution. Equal seeds across feature arms do not guarantee identical drought schedules. Causal environmental comparisons need independent environmental streams or replayed schedules. Shared random inputs also do not automatically reduce variance in nonlinear ecological systems; see [Glasserman and Yao (1992), Some Guidelines and Guarantees for Common Random Numbers](https://business.columbia.edu/sites/default/files-efs/pubfiles/4261/glasserman_yao_guidelines.pdf).
