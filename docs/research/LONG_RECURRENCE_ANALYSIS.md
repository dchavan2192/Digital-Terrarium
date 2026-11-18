# Long recurrence campaign: audit and analysis handoff

`analysis/summarize_long_recurrence.py` implements the existing [frozen study plan](../results/BASELINE_RECURRENCE_LONG_PLAN_2026-11-17.md). Development began while five of eight worlds were complete. This implementation changes no experimental setting, run order, endpoint or sampling window. Synthetic fixtures cover every status; current real data has only been used for format/provenance compatibility and an explicitly incomplete audit. No final campaign result is registered yet.

## Safe collection

Read Pi `status` and `worker.json` first. Do not rerun the campaign, replace its executable, resume a checkpoint or start a second simulation as part of analysis. Use a local mirror; checkpoints remain on the Pi.

```bash
rsync -az --exclude 'checkpoint*' pi@100.127.61.93:/home/pi/Digital-Terrarium/output/long_recurrence/baseline_long_20261118T023800Z/ output/long-recurrence/baseline_long_20261118T023800Z/
python3 analysis/summarize_long_recurrence.py output/long-recurrence/baseline_long_20261118T023800Z --allow-incomplete --output output/long-recurrence/observed-snapshot.json
```

All pinned job inputs, including binary and code copies, must be present. The output is created exclusively; use a fresh filename for each snapshot. Without `--allow-incomplete`, the auditor rejects a noncompleted campaign. A terminal failed campaign also requires explicit incomplete-report mode; this preserves failed and unstarted slots without calling the study complete.

The mirror is not an atomic filesystem snapshot. Each reported SHA-256 identifies the exact bytes consumed by the parser, not an independently reread later version. Repeated reads of an input must agree. Queue/status contradictions cause rejection: refresh the mirror and retry rather than dropping a world or repairing scientific data. On completion, refresh again, run without the incomplete flag, build an immutable archive and verify its payload hashes before publication. No existing output is overwritten.

## Checks and output

- Retain all eight prescribed jobs and all four paired seed labels. Require pinned engine/core/clean status, exact base and whole effective config, input inventories, fixed command budgets, consistent compiler/target identity, one native session per job and clean terminal supervisor/guard state. Reject resumed or duplicated sessions and impossible queue ordering.
- Classify `completed`, `extinct`, `active`, `technical_failed` and `not_started` separately. A completed campaign can contain biological extinction; an extinct world is not a successful living late-window observation. Explicit arm and pair denominators accompany every available mean/range.
- Require finite metrics, nonnegative activity/brain energy costs, integer nonnegative ticks/populations/event counters, monotone cumulative counters, exact scheduled prefixes and no recurrence in off controls. A current-tick event count cannot exceed the cumulative increment since the preceding saved row. Complete worlds require all 500 scheduled rows. Final off-grid extinction rows are retained.
- Early window50,000–100,000 contains51 saved observations; late375,000–500,000 contains126. Full living windows only: no partial-window extrapolation. Report per-world means, late-minus-early, sampled population variability/range, near-cap fraction and exact inclusive event counts. The inclusive count is `C(end) - C(start) + events_at_start`; summing sampled single-tick events would omit unsampled ticks.
- `observed_tick` means the last complete saved CSV record. Native live manifests may still show zero completed ticks, and CSV buffering can lag actual execution. Ignore only an incomplete trailing CSV record in nonterminal data; reject malformed interior rows or truncated terminal records. Observed population ranges describe saved snapshots, not inter-snapshot extrema.
- Cross-check final exported living-population size and expressed enabled-edge counts against terminal CSV values. Normalize only the two known config serialization differences: optional drought schedule seed and deprecated epigenetic stress threshold. Empty final exports are valid at extinction. Living edge/fraction means become null at population zero; raw pre-action brain activity retains its separate timing and units.
- Retain raw observed snapshots and trajectories. Only audited completed living worlds contribute late windows and complete-pair contrasts. Partial early/final observations remain visible. Partial-report means are available-world descriptions with explicit missingness, not final experiment results, confirmatory estimates or extra replicates.

```bash
python3 -m unittest discover -s analysis -p 'test_long_recurrence_summary.py'
```

The tests use synthetic worlds, including extinct/failed/active/unstarted cases, corrupted provenance/exports/CSV, counter-boundary examples, and consumed-byte hash behavior. They do not simulate new organisms or tune the study. The live Pi worker and browser demo are unchanged by these analysis tools.
