# Exported-genome delayed-cue negative control — 2026-11-16

Status: **implementation check passed; not an evolutionary result**.
ID: CHK-2026-11-16-EXPORTED-DELAYED-CUE.

The new `evaluate-delayed-cue` command evaluated all **898 exported genomes**
from the prespecified first donor seed. Every controller had zero enabled
recurrent connections, scored **1/2** on the two balanced cue trials, and gave
identical intact/state-cleared trial responses. Input record order was preserved.
These are the expected negative-control results for feed-forward controllers
under identical probe inputs.

| Check | Outcome |
|---|---|
| All exported records evaluated | 898 / 898 |
| Enabled recurrence | 0 in every record |
| Intact accuracy | 1/2 in every record |
| Same-network state-cleared accuracy | 1/2 in every record |
| Actual cue/probe responses match between conditions | Every record |

The [protocol](EXPORTED_GENOME_ASSAY_PLAN_2026-11-16.md) was saved before the
actual evaluation. Cue channel 12 is the signed heard-signal sensor in the
20-input donor configuration. Five blank ticks and one probe followed each
one-tick cue. All other inputs were zero. Weights and exported snapshot
methylation were fixed; the assay did not restore living expressed brains or
neural state, update marks, move organisms, or simulate fitness. Snapshot marks
may include lifetime changes and do not isolate inherited epigenetic effects.

This is one donor world's instrument check. Its organisms and deterministic
trials are not independent evolutionary replicates; no significance test or
confidence interval is appropriate. The result neither shows a memory benefit
nor rules out memory under a different channel, readout or natural task in a
memory-enabled genotype. Exported-genome task evaluation is now available, but
a memory-requiring environment and evolved recurrent-donor evaluation remain
separate prerequisites for a memory-selection experiment.

## Provenance and reproduction

[Machine-readable summary and SHA-256 identities](exported_genome_assay_2026-11-16.json).

- Donor: seed 100, source `0da954fa2911e5a0f075375b41196f8dafa6e4c8`, Pi release,
  `aarch64-unknown-linux-gnu`, core build `abad322241615ca2`.
  Its dirty-build flag remains `true`; the archived batch source-status file
  records only untracked `nohup.out` and `run_overnight.sh`, not tracked edits.
- The donor's pinned binary resumed the final 200,000-tick checkpoint for exactly
  one tick into a new directory, exporting all living genomes at tick 200,001.
  This export bridge left the original validation outputs intact. The current
  full-duration recheck was still running; no acceptance decision is implied.
- Checkpoint SHA-256: `9716b5a69642ba3055eef9993de8f837acace79631afdee26c137b2d93363070`.
- Export: `output/exported_assay_donor_20261116T212115Z/session_1794864076739181191_30731_0/genomes_seed100_tick200001.json`, 29,763,649 bytes,
  SHA-256 `0bcab908515cbbb6da5fbc216a52c432f0ddb3dc1f2098e012bc872ff4cf1077`. The source export and bridge
  configuration/manifest remain on both Pi and local project output storage.
- Evaluator: source `aa6e844bb8234c111add54f01e18610f3581e333`, clean tree at build,
  core `83292ca5eb55ebf5`, `aarch64-apple-darwin`, release,
  `rustc 1.99.1 (58cf8f9c3 2026-10-13)`. Full output is retained at
  `output/exported_assay_donor_20261116T212115Z/assay_signal12_delay5.json`; SHA-256 `ad6f9d9dcd1d105cff7ccc905660d8640f2f9e744669b496f747fd650b4460be`.
- Observed local evaluation wall time: 0.175 seconds including
  input parsing and output generation; not a general throughput benchmark. The
  evaluator was not rebuilt on the busy Pi and added no sustained Pi workload.

```bash
cargo build -p sim-runner --release
target/release/sim-runner evaluate-delayed-cue \
  --input output/exported_assay_donor_20261116T212115Z/session_1794864076739181191_30731_0/genomes_seed100_tick200001.json \
  --cue-channel 12 --blank-delay 5 > output/exported-genome-assay.json
```

## Implementation verification

142 Rust tests, 10 JS tests and 10 thermal tests passed. Formatting and all-target
Clippy passed. Tests cover genuine exports, state resets, snapshot methylation,
ignored expressed state, malformed IDs/topology/configuration, dimension
overflow and bounded work. Finite weights producing non-finite activity are
rejected with no stdout report. Thermal policy tests now run in CI.

The engine's checked configuration/seed trajectory is unchanged: complete-state
fingerprints before and after at ticks 0, 1 and 100 match (`4ed03498157ea3c5`,
`b92d14f490a0cd65`, `2066d844942fa4de`; default config, seed 42, macOS release).
This limited measurement complements the tests; it is not a proof for every
possible state. Checkpoint compatibility still requires the original pinned
build, while validated genotype exports can be evaluated across builds.

Independent review recomputed both export and assay output hashes, checked all
898 records and exact trial equality, and confirmed configuration/provenance.
It matched the checkpoint hash to the retained checksum record; it did not
recompute that hash from checkpoint bytes locally.
