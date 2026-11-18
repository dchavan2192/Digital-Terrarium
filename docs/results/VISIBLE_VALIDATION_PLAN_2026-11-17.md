# Saved-policy visible validation — prospective plan and stopping point

Status: implementation and synthetic-fixture checks only. **No archived pilot policy has been evaluated on the new cases. No experiment is running.** Commit this plan and suite before any such evaluation. Owner requested a usage-limit stopping point; do not launch a study in this wrap-up.

## Frozen scope

Use `configs/visible_validation.toml`, SHA-256 `595d8892befd21e25368fa9a0ce13ecdb62e4958e01e18ef8c3fedee9b4eff8a`. Seven cases: original training reference; rigid rotations by minus/plus 45 degrees; relative goal offsets (forward, lateral) `(3,3)`, `(5.5,4)`, `(4,2.5)`, `(4,5)`. Preserve cue mapping, goal radius, horizon, movement rules, hold phases and controller weights. Rotations move both goals and release heading together, preserving relative goal labels; they mainly probe the absolute-heading input. Relative cases change both distance and bearing and cannot isolate either cause.

Cases were chosen using policy-free initial ray coverage and a privileged turn-in-place/straight-action reachability witness, before evaluating any actual saved policy. An initially considered `(5,5)` offset had initial ray blind spots and was rejected during instrument design. The witness is a geometry diagnostic, not a fair neural controller or a proof that a policy can solve the task. Record its output for every case.

Evaluate every declared final training champion, including partial and failed searches. Keep the original seeds 2000–2002 as a separate descriptive cohort; compare paired terminal-only/progress seeds 2100–2102 without pooling the original cohort. Do not select policies using validation scores, extend searches, add seeds or tune the suite after outcomes. Source locations and nine champion-summary SHA-256 values are recorded in [the trace diagnosis](ACQUISITION_TRACE_DIAGNOSIS_2026-11-17.md); complete inputs remain in the linked parent reports/releases.

## Instrument behavior

`sim-runner validate-visible --session SOURCE_SESSION --suite configs/visible_validation.toml --output OUTPUT`

The loader checks manifests, completion, protocol, final-population topology/ranking and champion summaries; all declared failed seeds remain explicit. It bounds source bytes and trace/edge work. Before any changed-case evaluation, every completed champion must exactly reproduce its saved original-case trials, removing only the acquisition-only progress fields from the comparison. Reject mismatches as technical failures. Preserve source files and pinned historical binaries.

Each case reports fixed manufactured feed-forward, recurrent and cue-independent controls without retuning or filtering cases according to their outcomes. Intact and state-cleared trials are retained; feed-forward equality is an instrument check, not independent replication. Reference replay is reused rather than recomputed. Three policies and seven cases require at most 174 trials, 4,524 neural steps and 4,698 trace samples per invocation, including controls. No acquisition or ecological simulation is performed.

Report all cases and both cues: correctness, wrong-goal contacts, timeouts, first-contact tick and trajectories. Primary descriptive endpoint is both cues correct per policy/case. Keep heading rotations and changed relative geometries identifiable. Retain source failures and distinguish them from behavioral failures. FNV fingerprints in output are noncryptographic; archive full SHA-256 input/output inventories externally.

## Resume checklist

1. Read the newest `docs/PROGRESS.md` entry, check Pi activity/temperature and fetch GitHub changes.
2. Review this committed plan/suite and pin a clean evaluator revision/build. Finish a bounded thermal-supervised validation launch path before execution; the existing acquisition supervisor currently launches `acquire-visible`, so it must not be reused unchanged. Preserve its default behavior and test any extension.
3. Freeze complete source-session SHA-256 inventories and copy inputs into separate immutable batch directories. Run the three cohorts serially, with thermal guard and recorded watchdog; retain all failures and telemetry. No outcome-driven changes.
4. Independently audit baseline replay, source identity, full cohort denominators, controls and every changed-case result. Save raw data and an appropriately qualified report.
5. Design fresh-seed repeatability with a stated precision target before confirmatory acquisition or delayed-memory selection. This saved-policy diagnostic cannot establish fresh-run acquisition, adaptive memory, lifetime learning or ecological benefit.

The separate `science/diet-competition-kernel` branch remains unmerged pending its documented validation fixes. Demo and ecological defaults are unchanged by this harness.

## Post-execution interpretation clarification

The frozen offsets were executed unchanged. The near `(3,3)` case preserves the original center bearing, changing distance and apparent angular size; the other three relative cases change distance and bearing together. The earlier general wording about relative cases should not be read as saying that every case changes center bearing. See the separate final report for outcomes; this clarification changes no prospective condition or endpoint.
