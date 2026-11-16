# Delayed-cue instrument check — 2026-11-16

Status: **implementation check passed on Pi; not an evolutionary result**.
Result ID: CHK-2026-11-16-DELAYED-CUE.

The manufactured recurrent reference recalled both balanced cues after 0, 1,
5, 20 and 100 blank ticks. The feed-forward and same-network state-cleared
controls scored 1/2 at every delay. The probe input was identically zero in
both cue conditions, and every trial began with zero state.

| Blank delay | Recurrent reference | Feed-forward | Same network, state cleared |
|---|---|---|---|
| 0 | 2/2 | 1/2 | 1/2 |
| 1 | 2/2 | 1/2 | 1/2 |
| 5 | 2/2 | 1/2 | 1/2 |
| 20 | 2/2 | 1/2 | 1/2 |
| 100 | 2/2 | 1/2 | 1/2 |

The deterministic balanced trials have no sampling confidence intervals or
significance test. A zero-response control always predicts the positive cue,
which explains its 1/2 score. The reference is a hand-built sign-persistence
latch, not an evolved or trained controller. It has one synthetic input rather
than the world's 20 sensors. This establishes a first test instrument; it does
not establish learning, useful evolved memory, UAL or intelligence. Testing
exported evolved genotypes and a memory-requiring ecology is still future work.

## Provenance and reproduction

- [Plan written before Pi execution](DELAYED_CUE_INSTRUMENT_PLAN_2026-11-16.md).
- [Full probe responses and machine-readable results](delayed_cue_instrument_2026-11-16.json).
- Source `d35dd2cf73261da5258e2978d94bd86798cec2e3`; `aarch64-unknown-linux-gnu`;
  `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; release runner SHA-256
  `e9fbf73b97a53320d8671212ec650c028b97e4154f3fcb3344e25e1fb0efc169`.
- Remote run directory: `/home/pi/Digital-Terrarium/output/delayed_cue_20261116T175013Z`; retained runner, build log and one
  JSON per delay. The first orchestration attempt stopped before building
  because it encountered an inaccessible system process; the scan was fixed
  to skip inaccessible processes. Its output folder remains preserved.
- The active ecology used its separate pinned executable throughout. A separate
  detached worktree built the instrument with one build job. The ecology was
  paused while the instrument cooled/built/ran, then returned to its thermal
  guard. Recorded temperature after the check was 72.15°C.

```bash
cargo build -p sim-runner --release
for delay in 0 1 5 20 100; do
  target/release/sim-runner delayed-cue --blank-delay "$delay"
done
```

Three instrument unit tests and two CLI tests verify controls, trial-state
reset, delay bounds and output shape. The full Rust suite passes 129 tests;
all-target Clippy and format checks pass. Tests also exercise 10,000 blank ticks.

The complete JSON outputs for all five delays exactly matched the macOS native
release build in a follow-up check. This is measured agreement for this
manufactured instrument, not a new general portability claim.
