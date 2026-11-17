# Separate diet-competition branch review — 2026-11-17

Read-only review of remote `science/diet-competition-kernel`, commit `703f89d`, by a separate agent. No checkout, merge, build or experiment was performed. This is separate ecological research, with no acquisition-method fix; retain it separately from the visible-task revision.

Before integration, resolve and test:

- The documented `food.fruit.diet_slope` sweep is not supported by the current sweep parameter dispatch.
- Zero/negative `diet_mutation_step` values can reach an invalid random range; add explicit domain handling/validation.
- Tiny positive competition sigma can underflow its squared denominator, producing NaN for identical diets; use a stable ratio/domain bound and tests.
- Add parameter/kernel tests and prove disabled-default trajectory preservation before claiming backward compatibility.
- A Gaussian diet separation of one sigma gives about 0.607 weight; correct the kernel comment.

Global finite-value validation already applies, but does not replace these domain checks. A linear feeding-efficiency slope alone does not establish a generalist fitness valley or demonstrate ecological specialization. Preserve this distinction in the branch's experimental claims. These are review findings and readiness requirements, not measured outcomes. The branch remains unmerged.
