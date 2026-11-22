# Demo review for first-time visitors — 2026-11-22

Scope: help outsiders understand and explore this personal artificial-life research project. Static review of README, `web/demo/index.html`, `js/main.js`, tour, inspector, feed, KPI and responsive styles, against `DEMO_UX_DESIGN.md`. No browser host was available: no rendered-layout, assistive-technology or user testing occurred. Recommendations below are design hypotheses, not measured usability improvements.

## Evidence and limits

- [Segel & Heer, Narrative Visualization (2010)](https://vis.stanford.edu/papers/narrative.html) characterizes the balance between an authored path and reader exploration. Application here: offer a short observation sequence, then free exploration. Its case-study framework does not prove a particular tour works for Terrarium.
- [National Academies, Communicating Science Effectively (2017), summary](https://www.nationalacademies.org/read/23674/chapter/2) emphasizes choosing communication approaches for the goal and audience. Our goal: visitors can explain the model's food–energy–offspring loop and distinguish observation from scientific evidence. Engagement alone is insufficient evidence of understanding.
- [Bret Victor, Explorable Explanations (2011)](https://worrydream.com/ExplorableExplanations/) connects authored explanation with inspectable examples and contextual information. This is an original design essay, not a controlled educational trial. Application: each suggested interaction should answer one stated question.
- W3C guidance establishes concrete checks for [logical focus order](https://www.w3.org/WAI/WCAG22/Understanding/focus-order.html) and [minimum target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum). Meeting these individual checks does not establish full accessibility conformance.

## Current strengths and mismatches

The live landscape, feed, energy accounting and inspector support learning through observation. The tour already explains designed rules, mutating controllers and operational genome clusters. Ecology correctly separates diet spread from distinct groups, and topology presence from intelligence. Task lab explicitly identifies saved native visible-cue evaluations, separate from live ecology; memory presence is not claimed as benefit. Preserve those distinctions.

| Existing surface | Concrete issue | Suggested adjustment |
| --- | --- | --- |
| Header | Task lab and Memory compete with the basic ecosystem story; the research purpose lives mainly in README. | Add a short orientation: “A designed world for studying how inherited bodies and neural controllers change across generations.” Prioritize a visible “Start here” action. |
| Phone | `.hero` is hidden below 700px. The short conceptual explanation disappears. | Keep a compact purpose sentence inside the tour or an always-available “About this model” disclosure; verify header overflow before adding text. |
| Tour step 3 | “Colour means species” targets `#kpis`, although `.legend-chip` carries the key. “Species” shorthand can outlive the cluster caveat. | Spotlight `.legend-chip`; consider “Colour groups similar genomes.” Retain the biological-species boundary. |
| Tour ending | “Click a creature” offers no observation question; moving targets require pointer precision. | Give one doable question first: “Where does the energy go?” Then add a general “Inspect a creature” button, independent of memory mode. |
| Inspector | Neural diagram precedes the energy detail; description sentences combine event flags and heuristic interpretation (`describe.js`). | Keep the short activity summary; label inferred descriptions “Activity summary,” and expose energy before full wiring for beginners. No intent claims. |
| Ecology | Diet variance, hidden neurons and topology windows answer different questions in one panel. | Group as “Diet differences” and “Brain structure”; put the neutral reference under “Advanced comparison.” Preserve scope and reset disclosure. |

The older design spec's “Nobody programmed their behaviour” and fixed release counts are unsuitable implementation copy. Rules, controller architecture and selection conditions are designed; current counts must come from measured state.

## Prioritized small implementation plan

1. **Now: a concrete tour exit.** Replace only the final tour title/body, as below. This teaches the energy constraint using existing controls without promising a particular outcome.
2. **Next: “Inspect a creature.”** Add a native button selecting a living creature deterministically through existing selection code; explain “Watch its energy while it moves.” Disable with a plain reason at zero population. Focus the inspector heading and restore focus on closing. Reuse existing Follow, death and restart handling.
3. **Then: progressive disclosure.** Keep Observe → Energy → Inspect as the essential route. Offer “Brain wiring,” “Ecology measurements,” and “Experimental comparisons” as optional depth. Rename Task lab's entry “Recorded task replays” or add that subtitle; keep provenance and partial-response cases. Before memory activation, visibly say “Restarts this seed; useful memory is unproven.” A same-seed restart is exploration, not a controlled benefit test.

## Bounded first change

File: `web/demo/js/ui/tour.js`, final `TOUR_STEPS` entry only; retain `target: '#side'`.

**Title:** “Start with energy.”

**Body:** “After this tour, use Pause, then open Energy. Compare food coming in with the costs of movement and upkeep in the latest sampled tick. Resume to watch the numbers change. These are measurements inside a designed model, not proof of intelligence.”

No new controls, engine changes, automatic pause or persistent settings. Existing tour reopening makes this available to returning visitors.

## Acceptance criteria

- Keyboard: complete, skip and reopen the tour; Back/Next and Escape work; closing restores focus. Reach Pause and Energy without pointer input; tab-arrow navigation works. Later inspect action must work without memory mode.
- Mobile: manually verify 360×640 and landscape, plus 200% zoom. Tour copy, controls and Energy remain reachable; panels do not cover focused controls. Use targets ≥24×24 CSS pixels or documented W3C spacing exceptions. Do not claim these pass from source inspection.
- Understanding: proposed moderated check with five new visitors, not yet conducted. After 90 seconds, ask what supplies/spends energy, how offspring differ, what colour groups mean, whether Task lab is live, and whether links establish useful memory. Target ≥4/5 answering each correctly without prompting; record misconceptions and revise. This small check diagnoses comprehension, not scientific validity or population-wide usability.
