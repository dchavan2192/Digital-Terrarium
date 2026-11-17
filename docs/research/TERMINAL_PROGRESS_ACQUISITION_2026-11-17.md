# Terminal progress as an acquisition tiebreak — 2026-11-17

Development decision, not an experimental result. The original [fixed-budget pilot](../results/VISIBLE_ACQUISITION_PILOT_2026-11-17.md) remains complete and unchanged: no search acquired both visible responses. This revision tests a different search objective rather than extending that pilot.

## Mechanism

`acquire-visible` accepts an optional protocol `fitness` enum:

- `terminal_only` (default): correct count descending, then persistent ID ascending. Old protocol files remain valid; they do not calculate progress or change their random draws.
- `terminal_progress` (explicit opt-in): correct count descending, then mean terminal progress descending, then persistent ID ascending. A lower correct count can never outrank a higher one. Mutation, founders, tournament, elites and evaluation budget remain unchanged.

For each cue, only after first contact or horizon termination, calculate surface distance from the terminal pose to that cue's assigned goal disc. Progress is `clamp((start_surface_distance - terminal_surface_distance) / start_surface_distance, 0, 1)`, averaged over the balanced cues. Starting distance must be finite and positive. Confirmed correct contact gives exactly 1, avoiding boundary-rounding differences among fully successful policies. This uses terminal distance, never closest approach along the path.

The target label remains absent from visual goal appearance and is not consulted during movement except as the already visible cue. No distance enters neural inputs, brain state, online reward or within-life genotype updates. This is teacher-informed genetic selection across generations. The terminal reward remains 0/1 per trial and the primary acquisition endpoint remains two correct choices. Progress gains alone are not competence.

Opt-in candidate and generation records include the secondary value; summary includes founder/final champion progress, and the same four final replay trials include per-cue/mean progress. No additional neural trials are introduced. The existing `improvement_over_founders` flag continues to mean correct-count improvement only; use the explicit progress fields for secondary-objective changes. Default terminal-only candidate/generation files omit these extra fields. Provenance strings explicitly distinguish the two selection rules.

## Research limits and independent design review

An independent council reviewer endorsed a separately frozen paired engineering comparison and required identical founder genomes by ID, all failed searches retained, finite distances, confirmed-contact normalization, terminal-only scoring and actual correctness as the primary endpoint.

[Ng, Harada & Russell, ICML 1999](https://ai.stanford.edu/~ang/papers/shaping-icml99.pdf) studies policy invariance for specified potential-based additive MDP reward transformations. It does **not** prove invariance or improvement for this lexicographic evolutionary tiebreak. Do not describe this implementation as policy-invariant reward shaping.

[Lehman & Stanley, Evolutionary Computation 2011](https://doi.org/10.1162/EVCO_a_00025) provides objective-deception examples in different search tasks ([primary paper](https://www.cs.swarthmore.edu/~meeden/cs81/f17/papers/lehman_ecj11.pdf)). It supports skepticism that closeness will necessarily guide discovery, not a prediction of success in Terrarium's arena. These sources motivate a controlled engineering comparison; no novelty or biological-learning claim follows.

A prospective paired pilot uses unchanged 64/25 budgets and fresh seeds 2100–2102 in `configs/visible_progress_control.toml` and `configs/visible_progress.toml`. Freeze its exact implementation/input hashes and analysis rules before running. Matched initial genomes do not imply matched later histories once selection differs. No delayed-task selection, new ecology mechanism or broad cost sweep is part of this unit.
