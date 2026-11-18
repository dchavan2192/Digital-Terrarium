# Terrarium Web Demo — UX & Visual Design Spec (Phase 8)

Status: build-ready design spec, 2026-11-15. Owner of this doc: UX/visual design.
Consumers: bindings engineer (`crates/sim-wasm`), front-end engineer (`web/`), QA.

Architecture assumed (decision D7 in `docs/PROGRESS.md`): sim-core → `crates/sim-wasm` (plain `extern "C"`, zero-copy `Float32Array` views) → Three.js scene + HTML/CSS dashboard. The old demo stays at `web/classic.html`.

---

## 2026-11-17 metric-clarity update

The optional memory panel shows measured average enabled feed-forward, recurrent and total links per living creature, plus the actual configured recurrence-add attempt probability. Unknown/tick-zero averages stay unavailable. Inspector counts use the individual enabled-edge pool. Presence is distinct from useful recall, and the demo’s high mutation setting is disclosed. KPI/event wording describes model measurements rather than asserting intelligence, altruism, carrying capacity or adaptation. Literature citations indicate inspiration, not validation of those claims. Sharing is displayed as events per creature, and inspector static brain cost is labelled a baseline estimate.

Browser visual QA for this update was unavailable because the browser host was not accessible. Automated unit/real-WASM checks validate data wiring; layout remains unverified in this session. Existing GitHub Pages hosting and simulation settings are retained.

## 0. What the current demo gets wrong (diagnosis)

| Symptom (owner: "too game-like, no one knows what is happening") | Root cause in current viewer | Fix in this spec |
|---|---|---|
| Looks like an arcade game | Saturated triangles on near-black, no terrain, no legend (`renderer.rs`) | Naturalist "field-notebook" identity, terrain, calm palette (§3, §8) |
| Colour means nothing to a visitor | Creature colour = raw `genome.color_hue` modulated by energy; hue drifts continuously, so neighbours look "different" when they are the same species | Colour = *species* from a fixed 8-colour categorical palette; energy moved to a separate cue (§4) |
| Stats are unreadable | ~40 numbers in an egui side panel, jargon labels ("Troph", "Sig", "Mut") | 5 headline KPIs in plain English with tooltips + citations; everything else in a Charts tab (§6) |
| Nothing tells you what just happened | No events, no narrative | Rate-limited plain-English event feed ("A drought has hit the north-east…") (§6.4) |
| Brain is invisible | Inspector shows only scalars | Live brain graph with activations + "what is it doing" sentence (§7) |

---

## 1. Design principles and the 10-second goal

### 1.1 The 10-second comprehension goal

Within 10 seconds of landing, with no clicks, a first-time visitor (a recruiter, or a researcher outside ALife) should be able to say:

1. **"These are little creatures living on a landscape."** The scene reads as terrain plus agents at once, not as dots on black.
2. **"They need food to survive, and food is the green stuff."** Ground colour shows food, and creatures can be seen eating.
3. **"Colours are species, and the species change over time."** A legend chip next to the KPI strip says *"Colour = species"*, and the species count is a headline KPI.
4. **"Nobody programmed their behaviour. Their brains evolve."** The one-line hero subtitle says so, and the inspector's brain graph backs it up.
5. **"Something is happening right now."** The event feed shows at least one sentence in the first 10 s. The front end seeds it with a scripted opener: *"300 creatures with random brains have just been released. Watch which strategies survive."*

The hero subtitle (always visible, one line): **"An evolving ecosystem: 500 creatures with neural-network brains compete for food, hunt, share and evolve, live in your browser."**

### 1.2 Principles

1. **Explain before you impress.** Every visual channel has exactly one meaning, and that meaning is written on screen (legend, tooltip, or feed). No unexplained colour, glow or number. (After Bret Victor's *Explorable Explanations*: the reader should be able to check the claim against the thing on screen.)
2. **One story at a time.** The default view shows one data layer, 5 KPIs and one feed. Depth (charts, brain graph, layers) is opt-in through progressive disclosure, as in Distill.pub articles and Nicky Case's explorables, which build up one mechanic at a time.
3. **The science is the decoration.** No particles, bloom or screen-shake for their own sake. Motion and highlights only fire when the simulation does something (eat, attack, share, build, birth, death). This is what makes it read as a lab instrument and not a game.
4. **Honest, perceptual colour.** Continuous data uses perceptually uniform, colour-blind-safe maps (viridis/cividis family, after Smith & van der Walt 2015, Nuñez et al. 2018, Crameri et al. 2020). Categorical data uses a fixed palette that stays distinguishable under deuteranopia (Okabe & Ito 2008). No rainbow/jet anywhere.
5. **Same seed, same story.** The seed is always visible and shareable (`?seed=1234` in the URL). This puts the project's determinism guarantee on screen as a feature, which researchers will notice and recruiters can try.

---

## 2. Layout

### 2.1 Desktop (≥ 1100 px wide)

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ TERRARIUM  An evolving ecosystem: 500 creatures with neural-network brains…   [?Tour] │ ← header 48px
├──────────────────────────────────────────────────────────────────────────────────────┤
│ ┌Population──┐┌Species────┐┌Brain size──┐┌Cooperation─┐┌Food chain──┐  Colour=species │ ← KPI strip 72px
│ │ 482  ▁▃▅▆▇ ││ 7  ▂▂▃▅▄  ││ 41 links ▃▄││ 64% ▅▆▆▇▇  ││ 1.18 ▁▁▂▃  │  ● ● ● ● ● ●   │
│ └────────────┘└───────────┘└────────────┘└────────────┘└────────────┘                 │
├───────────────────────────────────────────────────────────┬──────────────────────────┤
│                                                           │ WHAT'S HAPPENING         │
│                                                           │ ─────────────────────    │
│                3D VIEWPORT (Three.js canvas)              │ Day 41 · A drought hit   │
│                                                           │   the north-east. Food   │
│   ┌Layers──────────┐                     ┌View──────┐     │   there is collapsing.   │
│   │◉ Food          │                     │ ⟲ Orbit  │     │ Day 38 · New species     │
│   │○ Temperature   │                     │ ▦ Map    │     │   "Sp. 9" branched off   │
│   │○ Scent trails  │                     │ ◎ Follow │     │   from Sp. 3.            │
│   │○ Built ground  │                     └──────────┘     │ Day 30 · Predators now   │
│   │○ Elevation     │                                      │   get 1 in 5 meals by    │
│   └────────────────┘                                      │   hunting.               │
│                                     ┌Legend (for layer)┐  │ …                        │
│                                     │ food ▕▁▂▃▄▅▆▇█▏  │  │ [Feed] [Charts] [Species]│ ← tabs
│                                     │ none        full │  │                          │
│                                     └──────────────────┘  │                          │
├───────────────────────────────────────────────────────────┴──────────────────────────┤
│ ▶❚❚  1× 4× 16×   Tick 41,230 · Day 41 ☀︎/☾ · Season: spring   Seed 1234 [↻ New] [⧉] │ ← time bar 48px
└──────────────────────────────────────────────────────────────────────────────────────┘

Inspector drawer (slides in from the right over the feed column, 380px, when a creature is clicked):
┌──────────────────────────┐
│ ✕  Creature #18 342      │
│ Sp. 3 ● · Gen 57 · Age 812│
│ "Hunting: chasing a      │
│  smaller creature."      │
│ ┌ Brain ───────────────┐ │
│ │ inputs  hidden outputs│ │
│ │  ○─┐    ●      ○ turn │ │
│ │  ○─┼──● ●──────● speed│ │
│ │  …                    │ │
│ └───────────────────────┘ │
│ Energy budget (last 100t) │
│  +food ████████  +hunt ██ │
│  −move ███ −brain ██ −… █ │
│ Lineage  #4 → … → #18342  │
│ Diet: 78% plants · 22% meat│
│ [Follow] [Back to overview]│
└──────────────────────────┘
```

Grid: CSS grid `grid-template-columns: 1fr 360px; grid-template-rows: 48px 72px 1fr 48px`. The viewport is the only element that grows. The right column holds the tabs Feed, Charts and Species, plus the Inspector overlay.

### 2.2 Tablet (700–1099 px)

The right column collapses into a bottom sheet (40% height, draggable). The KPI strip stays on one row with a horizontal scroll-snap if it overflows. Layer and view controls collapse to icon buttons.

### 2.3 Phone (< 700 px), graceful degradation

```
┌────────────────────────────┐
│ TERRARIUM            [?] ≡ │ 44px
├────────────────────────────┤
│ 482 creatures · 7 species  │ ← 2 KPIs inline; swipe → for the other 3
│ ◀ ●Brain 41 ●Coop 64% … ▶  │   (scroll-snap chips) 56px
├────────────────────────────┤
│                            │
│     3D VIEWPORT            │  ~55vh, defaults to TOP-DOWN map view
│     (top-down default)     │  (orbit is awkward with thumbs)
│                    [▦][◉]  │  ← view + layer FABs
├────────────────────────────┤
│ Day 41 · A drought hit the │ ← latest event only, tap to expand feed
│ north-east…            ⌃   │
├────────────────────────────┤
│ ▶❚❚  1× 4× 16×  Seed 1234 ↻│ 48px
└────────────────────────────┘
Inspector, charts and feed open as full-height bottom sheets.
```

Phone degradations: pixel ratio capped at 1.5, creature count is unchanged (the sim must stay the same) but trails are off, shadows are off, terrain is 64×64, and the brain graph shows outputs plus top-10 strongest connections only.

### 2.4 Guided tour (4 steps, auto-offered on first visit, skippable, remembered in `localStorage` inside try/catch)

The tour is a dimmed overlay with a spotlight cut-out and a 1–2 sentence card. The sim keeps running at 1× underneath, because a moving world explains more than a paused one.

| # | Spotlight | Copy |
|---|---|---|
| 1 | Viewport | **Meet the creatures.** Each one has a small neural-network brain that started random. It senses food and neighbours, and decides how to move, eat, fight or share. Nobody programmed what they do. |
| 2 | Ground + Food legend | **Food is energy, and energy is life.** Green ground is food. Moving, thinking and having babies all cost energy. Run out and you die. Lush areas regrow fast and barren ones barely regrow. |
| 3 | KPI strip + species legend | **Colour means species.** When creatures eat well they reproduce, and their offspring's brains mutate slightly. Lineages that drift far enough apart count as a new species and get a new colour. |
| 4 | Feed + a creature click hint | **Follow the story.** This feed narrates what evolution is doing right now. Click any creature to see its brain thinking, live. |

A "?" button in the header replays the tour.

---

## 3. 3D scene spec

### 3.1 World mapping

- Sim space is 1000 × 1000 on a torus. Scene units: 1 sim unit = 0.01 scene units, so the world is a 10 × 10 plane centred at the origin (x → X, y → Z). Keeping the world small avoids float jitter and makes camera presets simple.
- **Toroidal wrap:** no visible walls. Creatures crossing an edge teleport. To hide the jump, fade the terrain edges with a 4% vignette band and draw a thin dashed "world edge, wraps around" outline in the Map view. Trails must break (not be drawn) when a segment's length exceeds half the world.

### 3.2 Terrain

- **Geometry:** one `PlaneGeometry(10, 10, 99, 99)`, so 100×100 vertices match the 100×100 food grid 1:1. That is about 19.6k triangles in one draw call. Heights are sampled once at init from `ElevationMap::height_at` (range about [-1, 1]) through a new WASM export (see §9).
- **Height exaggeration:** `y = h × 0.35` scene units, about 3.5% of the world width. That is enough to read hills under orbit view without hiding creatures behind ridges. A slider (0–2×) lives in the Elevation layer.
- **Style:** **smooth-shaded low-poly hybrid.** Use `flatShading: true` on the 100×100 mesh for a faceted, paper-model look that suits the naturalist identity and makes slopes readable, and use smooth vertex colours for the data. Do not use photoreal textures.
- **Water:** **none.** sim-core has no water mechanic, and a lake would imply one (principle 1). Low elevation is drawn in a darker, cooler tone only.
- **Ground colouring:** per-vertex colour attribute updated from the active layer (§5). The default layer is **Food**. Biomes appear indirectly: lush oases show as persistently greener regions and the barren zone as pale soil. Biome outlines (from config `food.biomes`) are drawn as faint dotted contour rings with small labels such as "Lush oasis ×3 regrowth" and "Barren ×0.3", toggleable, on by default for the first 30 s and then faded to 30% opacity.
- **Update cadence:** colours update every 4 frames (100×100 = 10k vertices; writing a `Float32Array` of 30k values is cheap). Use `attribute.needsUpdate = true` with `usage = DynamicDrawUsage`.

### 3.3 Droughts

- Each `ActiveDrought {x, y, radius, severity, ticks_remaining}` is drawn as:
  1. **Ground desaturation + warm tint** inside the radius, computed in the terrain colour pass: `mix(colour, dryOchre, 0.55 × (1 − severity))` with a smooth falloff over the outer 15% of the radius.
  2. A **ring** on the ground, `RingGeometry` with dashed ochre `#C98B3C` at 70% opacity, plus a small floating label `Drought · 2,300 ticks left`.
  3. On spawn, the ring expands from 0 to the radius over 600 ms. On expiry it fades over 1 s.
- No red. Red is reserved for attack cues and reads as danger or error.

### 3.4 Day/night lighting

- **Important sim fact:** `daynight.period = 40` ticks. At 16× (see §3.7) that is several day cycles per second, which would strobe.
- Rule: **lighting follows a smoothed light level**, `L_vis` = EMA of `light_level`, τ = 0.5 s real time, and the effect is **muted**:
  - Directional "sun" intensity `0.55 + 0.45·L_vis`, colour lerped from moonlight `#9DB2D6` to sun `#FFF1DC`.
  - Hemisphere ambient fixed at 0.45, so night never goes dark and creatures are always visible.
  - **At 4× and above, disable lighting modulation** (L_vis = 1) and show day/night only in the time-bar dial (☀︎/☾ icon with a 24-segment ring). Tooltip: "At night creatures see half as far (Kronfeld-Schor & Dayan 2003)."
- Sun direction is fixed (elevation 50°, azimuth 135°) so terrain shading stays stable. Moving shadows would be distracting.
- Shadows: one 1024² shadow map from the sun, terrain receives and creatures cast. Off on phones and on any device that falls below 45 fps (see §3.7).

### 3.5 Camera

| Mode | Behaviour | Control |
|---|---|---|
| **Orbit** (desktop default) | `OrbitControls`, target at the world centre, polar angle clamped 20°–75°, distance 6–16, damping 0.08. On load, run a slow auto-rotate (0.15 rad/s) until the first user input. | drag, wheel, pinch |
| **Map** (phone default) | Orthographic top-down camera framing the full 10×10 world. Pan and zoom only. Best for reading data layers. | `M` key / ▦ button |
| **Follow** | Third-person: the camera sits behind and above the selected creature (offset 0.6 back, 0.45 up), with the look target lerped (k = 0.12/frame) so the creature's turning doesn't make the view jitter. If the creature dies, hold the camera for 1.5 s on the death spot, show "Creature #N died (starved)", and return to Orbit. | click creature, `F` |

Transitions between modes: a 700 ms ease-in-out tween of position and target. No cuts.

### 3.6 Picking

- Raycast against an invisible low-res proxy: one `InstancedMesh` of icosahedrons (radius ×1.6 of the body) or, cheaper, a CPU nearest-neighbour search in sim space after projecting the click onto the terrain plane. **Prefer the CPU approach:** project the click ray onto y=0, convert to sim coords, and pick the nearest creature within 15 sim units.
- Hover (desktop only) shows a tooltip after 250 ms: `Sp. 3 · Gen 57 · Grazing`.

### 3.7 Performance budget

| Item | Budget |
|---|---|
| Target | **60 fps** on a 2020 integrated-GPU laptop at 1×–4×, **≥ 30 fps** at 16× and on mid-range phones |
| Draw calls | **≤ 15** total: terrain 1, creature bodies 1, creature eyes 1, state halos 1, trails 1, drought rings ≤ 4 (or 1 instanced), selection ring 1, biome contours 1, labels as an HTML overlay (0) |
| Triangles | terrain ~20k + creatures 500 × 64 = ~32k + trails ~10k, so **< 70k** |
| Pixel ratio | `min(devicePixelRatio, 2)` desktop, `min(dpr, 1.5)` phone. **Adaptive:** if the rolling 2 s fps < 45, step down by 0.25 to a floor of 1.0, then turn shadows off, then trails off |
| Per-frame JS allocations | zero in the render loop. Reuse a `Matrix4`/`Color`, write directly into `instanceMatrix.array` and `instanceColor.array` |
| Sim stepping | runs in the main thread inside `requestAnimationFrame` with a time budget of **≤ 8 ms per frame**. If a 16× batch exceeds it, carry the remainder over (the display tick rate drops, determinism is unaffected). Optional v2: move the sim to a Web Worker with `SharedArrayBuffer` (requires COOP/COEP headers, which GitHub Pages does not set, so it stays v2) |
| Interpolation | positions are lerped between the last two sim states with alpha = time-since-step / step-interval, so 1× motion looks smooth. Headings are slerped, with wrap-aware deltas |
| Speeds | **1× = 15 ticks/s** (a day lasts about 2.7 s, generations take tens of seconds), **4× = 60 ticks/s** (1 tick/frame), **16× = 240 ticks/s** (4 ticks/frame). Label them "1×/4×/16×" with a tooltip giving ticks per second |
| Three.js | pin an exact version via ES module import map from cdn.jsdelivr.net, e.g. `three@0.160.0` and `three@0.160.0/examples/jsm/controls/OrbitControls.js` |

---

## 4. Creature ("mini agent") design

### 4.1 Form (original, low-poly)

A **"seed-pod" body**: a slightly elongated, flattened low-poly ovoid, a little like a beetle or a tardigrade. It is unlike a game character and reads as an organism, with a clear front.

- **Body:** `IcosahedronGeometry(1, 1)` (80 tris, 42 verts) scaled (1.0, 0.62, 1.35), so it is longer along +Z (the forward direction) and flattened vertically. Flat-shaded.
- **Eyes:** two small spheres, `IcosahedronGeometry(0.18, 0)` (20 tris each), placed at (±0.32, 0.28, 0.95). They form a **separate InstancedMesh** with 2 instances per creature (1000 instances, 1 draw call), coloured off-white `#F4F1E8` with a dark pupil done via vertex colour on the front faces. Eyes do most of the work of making the shape read as a creature.
- **Optional v2 detail:** a tiny dorsal ridge (3-tri fin) on predators, see §4.3.
- **Vertex budget:** body ≤ 42 verts / 80 tris, eyes 2 × 12 verts / 20 tris, so **≤ 66 verts / 120 tris per creature**, and 500 creatures come to about 60k tris across 2 draw calls. Merge the eyes into the body geometry if draw calls matter more than per-instance eye animation (they do not at 15 calls).
- **Base scale:** world-space body length `0.07 × body_size^0.5` scene units (= 7–12 sim units for body_size 0.5–3.0). Use the square root so large creatures don't swamp the view while size still reads.

### 4.2 Encodings (one channel = one meaning)

| Channel | Encodes | Spec |
|---|---|---|
| **Size** | `genome.body_size` [0.5, 3.0] | scale ∝ √body_size (above). Legend: "Bigger = bigger body: stronger in fights, costs more to run (Kleiber 1932)." |
| **Body colour (hue)** | **species** | Fixed 8-colour categorical palette (below), assigned by **stable species ID** (requires new sim-core tracking, see §9.5). Species ranked 9+ by population share one neutral "other" grey-taupe `#9A948A`. Never raw `color_hue`. |
| **Lightness / saturation** | **energy** (subtle) | Body colour mixed toward the background tone by `(1 − energy/max_energy) × 0.45`. A starving creature looks faded or ghostly, a well-fed one rich. Gives an immediate sense of health without a second hue. |
| **Shape cue** | **trophic role** | predators (predation fraction > 0.5) get a **dorsal fin + forward-pointing "mandibles"** (2 small cones near the eyes), done by swapping to a second InstancedMesh "predator body" geometry. Omnivores (0.25–0.5) get a fin only. Herbivores are a plain pod. Shape, not colour, so it works for colour-blind viewers and doesn't clash with the species hue. |
| **Halo ring** (state) | current action | see 4.4 |
| **Bob / gait** | speed | see 4.5 |
| **Heading** | direction | body +Z aligned with `heading`, eyes at the front |

**Species palette (dark background), adapted from Okabe & Ito (2008) and lifted in lightness for a dark UI.** Order matters: rank 1 (most populous) gets index 0.

| idx | token | hex | name |
|---|---|---|---|
| 0 | `--sp-0` | `#E69F00` | amber |
| 1 | `--sp-1` | `#56B4E9` | sky |
| 2 | `--sp-2` | `#7FD1AE` | mint (lighter than O&I green so it shows on green food) |
| 3 | `--sp-3` | `#F0E442` | pollen |
| 4 | `--sp-4` | `#4C8FD6` | lake blue |
| 5 | `--sp-5` | `#E07B39` | vermilion (O&I D55E00 lifted) |
| 6 | `--sp-6` | `#CC79A7` | orchid |
| 7 | `--sp-7` | `#D9D2C3` | bone |
| — | `--sp-other` | `#9A948A` | other / unclassified |

**Colour stability rule:** a species keeps its colour for life. Colours are assigned on species birth to the lowest free palette slot. When a species goes extinct its slot is freed after a 10 s cooldown, so a recycled colour never appears to "resurrect" a dead species. If all 8 slots are taken, new species are `other` until a slot frees.

Mint on green food: food ground colour is low-chroma (see §5), and every creature gets a 1-px dark outline via a back-face-expanded hull (inverted-hull outline, 1 extra draw call) so it separates from any ground colour. **Ship the outline in MVP.** It is the single biggest legibility win.

### 4.3 Predator vs herbivore summary

- Herbivore: smooth pod.
- Omnivore: pod with a fin.
- Predator: pod with a fin and mandibles, plus a slightly **darker outline** (`#0E0F0C` vs `#1E211C`).
- Legend card "Who eats whom": three silhouettes with "Grazer · Omnivore · Hunter" and the tooltip "Based on where its lifetime energy came from: plants vs. other creatures (Lindeman 1942 trophic levels)."

### 4.4 State cues (action events)

Rendered with **one InstancedMesh of flat ground rings** (`RingGeometry(0.9, 1.0, 24)`, laid on the terrain under each creature, additive, depth-write off). Per instance: colour, scale and opacity. Only the **dominant action this tick** is shown, with priority **dying > reproducing > attacking > sharing > constructing > eating**.

| State | Cue | Colour | Duration |
|---|---|---|---|
| Eating | small soft ring pulse at 0.8× body, plus the food cell under it visibly dims (it is actually depleted) | `#B9D88A` leaf, 35% | 250 ms |
| Attacking | quick **line spark** from attacker to victim (instanced line segment, 1 draw call) + victim flashes white for 1 frame | `#E5484D` signal-red, the only red in the UI | 200 ms |
| Sharing | short **arc of 3 dots travelling donor→recipient** (instanced points, reuse the trail buffer) | `#F2C94C` honey | 500 ms |
| Constructing | ring stays on the ground and fades slowly, a sense of "marking ground" | `#A58AD6` lavender | 800 ms |
| Reproducing | expanding ring from the parent + offspring scales up 0 → 1 with an ease-out-back | `#F4F1E8` bone-white | 600 ms |
| Dying | body sinks 30% into the terrain, desaturates to grey and fades over 700 ms. A **small ochre ✕ glyph** at starvation deaths or a red one at kills (needs death cause, §9.5) | — | 700 ms |

At 16× the cues would flicker, so **at 16× show only attacks, births and deaths**, and use a 50% shorter duration.

### 4.5 Animation

- **Bobbing gait:** `y_offset = A × |sin(φ)|`, where `φ += speed_norm × 18 rad/s × dt` and amplitude `A = 0.012 × scale × speed_norm`. A per-creature phase offset (from id) stops them moving in lockstep. Idle creatures (speed < 0.05) instead "breathe": a scale-y oscillation of ±3% at 0.6 Hz.
- **Banking:** roll ∝ turn output (`±12°` max), giving a subtle sense of steering.
- **Terrain following:** creature y = terrain height at (x, y), sampled from the cached 100×100 height array by bilinear interpolation, plus half the body height.
- **Trails:** last **12 positions** per creature (sampled every 3 ticks) in one `LineSegments` with a vertex-colour alpha ramp from 0 to 40% of the species colour. Default **on** in Orbit, **off** in Map (cleaner layer reading), and always on (36 positions, brighter) for the selected creature. Must break at wrap edges.
- **Selection highlight:** a floating ring above the creature (`TorusGeometry` thin, bone-white, slow 0.5 Hz pulse), plus all non-selected creatures dimmed to 60% opacity while the inspector is open. Same-species creatures stay at 85% so the lineage can be seen around it.

---

## 5. Data layers

One layer is active at a time (radio, not checkboxes, to avoid blended unreadable colours). Each layer recolours the terrain vertices. Biome contours and drought rings sit on top of all layers.

| Layer (UI label) | Source | Colormap | Domain / legend | Tooltip |
|---|---|---|---|---|
| **Food** (default) | `food_grid.cells` (algae) + `fruit_cells` | **custom sequential "lichen":** `#2A2620` bare soil → `#4E5A34` → `#7E9A4A` → `#B9D88A` (monotone lightness, single green hue). Fruit is drawn as small **instanced dots** (berries `#C86B9A`) only on cells with fruit > 20% of max, so the two food types are separable without a 2D colormap | 0 → `max_per_cell`. Legend: gradient bar "no food — full" + "● fruit (grows only in oases)" | "Food regrows each tick, faster in oases, slower in droughts and winter. Creatures specialise on algae or fruit (Grant & Grant 2006)." |
| **Temperature** | analytic: `y / height` (north cold, south warm). Plus optional per-cell creature mismatch in v2 | **diverging, colour-blind-safe:** Crameri *vik*-like, `#2B5C8A` cold → `#E8E4D8` mild → `#B5562E` warm | cold north ↔ warm south. Labels "N cold", "S warm" | "Each creature has an evolved preferred temperature. Living far from it costs extra energy (Bergmann 1847)." |
| **Scent trails** (pheromones) | `pheromone_grid.cells` / `max_level` | **sequential "magma-ish" on dark, clipped:** transparent → `#5B3A7A` → `#C46FA8` → `#F6C7A0`. Low values transparent over the neutral terrain, so the trails look like they were painted on | 0 → max_level. "faint — strong" | "Creatures leave scent wherever they walk; others can smell it. Indirect communication like ant trails (Wilson 1975; Deneubourg et al. 1990)." |
| **Built ground** (niche construction) | `construction_grid.cells` [-max, +max] | **diverging:** degraded `#8A5A3C` ← neutral `#3A3A34` → enriched `#6FB3A0` | −1 degraded ↔ +1 enriched | "Creatures can spend energy to improve the ground (more regrowth) or degrade it. Their changes outlive them (Odling-Smee, Laland & Feldman 2003)." |
| **Elevation** | cached height map | **cividis** (Nuñez et al. 2018), plus **contour lines** every 0.2 units via a shader `fract()` band | low → high | "Hills slow creatures down. Climbing costs speed (MacArthur & Wilson 1967)." |

**Legend design:** a single floating card in the bottom-left of the viewport (above the layer picker). It contains the layer name, a 160×10 px gradient bar with min/max labels in plain words (not numbers by default), a "?" that expands the tooltip text and citation, and a units line ("cell max = 1.0 food units") in the expanded state. On hover over terrain (desktop), a crosshair readout shows the exact cell value: `Food 0.62 · Temp mild · Elev 0.31`.

Colormap implementation: precompute 256-entry LUTs as `Uint8Array` at startup. Terrain colour = `lut[floor(v×255)]` lightly multiplied by the hillshade, so the data stays readable and the terrain stays legible.

---

## 6. Stats and storytelling

### 6.1 Headline KPIs (5)

Each tile shows a **plain-English label**, a big number, a unit or qualifier, a **sparkline of the last 2 000 ticks** (sampled every 20 ticks, so 100 points), and a delta arrow vs. 500 ticks ago. Hovering or tapping opens a tooltip with "What it means", "Why it matters" and a citation.

| # | Label | Value (from `TickStats`) | Format | Tooltip (what · why · cite) |
|---|---|---|---|---|
| 1 | **Creatures alive** | `population` | `482` (+ "/ 500 cap" small) | How many creatures are alive. Populations boom when food is plentiful and crash in droughts. The cap is a configured limit, not a measured carrying capacity. *Lotka 1925; Volterra 1926.* |
| 2 | **Species** | `species_count` | `7` | Similarity groups defined by the model’s inherited brain/body distance rule. Their count alone does not establish biological speciation or adaptation. *Stanley & Miikkulainen 2002 (NEAT); Schluter 2000.* |
| 3 | **Brain links** | `mean_connections` (primary), `mean_brain_neurons` (secondary) | `41 links · 9 neurons` | Average enabled brain links, including recurrence. Mutation adds links and the model charges energy for them; count alone does not establish intelligence or useful memory. Show measured feed-forward/recurrent decomposition when available. *Aiello & Wheeler 1995; Laughlin et al. 1998.* |
| 4 | **Sharing** | `cooperation_rate` (+ `kin_share_ratio` in tooltip) | `0.64 events / creature` | Recorded sharing events per living creature at the sampled tick. Hue similarity is a model kin proxy; it is not genealogy or evidence of evolved altruism. *Hamilton 1964 (rB > C); Trivers 1971.* |
| 5 | **Food chain** | `mean_trophic_level` (+ `predator_count`) | `1.18` with a qualifier: < 1.1 "Mostly grazers", 1.1–1.4 "Some hunting", > 1.4 "Predators common" | Average of 1 plus each creature’s lifetime predation-intake fraction. Changes alone do not establish evolutionary adaptation. *Lindeman 1942.* |

A sixth "Generations" counter sits in the time bar, not the strip: `Gen 57 (deepest lineage)` from `max_generation`.

Sparkline spec: 120×28 px SVG polyline, 1.5 px stroke in `--ink-2`, last point as a 3 px dot in the accent colour, no axes. Min and max values appear in the tooltip. Use `prefers-reduced-motion` to disable value tweening.

### 6.2 Charts tab

Built with plain SVG or uPlot (pinned version from cdnjs, tiny and fast). Each chart has a title written as a question, x = **ticks** (with "Day N" secondary labels), a full history ring of 5 000 points, and a downsample toggle.

1. **"Who is winning?"**: stacked area of population by species (stable species colours), y = creatures. This is the Muller-style chart the project needs most.
2. **"Are brains getting smarter or cheaper?"**: lines for `mean_connections` and `mean_brain_neurons` (dual axis, clearly labelled), plus a faint `mean_brain_entropy` line.
3. **"Hunters vs. grazers"**: `herbivore_count` vs `predator_count` lines (plus `mean_trophic_level` on the right axis). Shows Lotka–Volterra-style cycles if they emerge.
4. **"Do they help each other?"**: `cooperation_rate` and `kin_share_ratio`, y 0–100%.
5. **"Is evolution still inventing?"**: `novel_traits` (bars per window) and `mean_cumulative_activity` D(t) line. Tooltip: "Unbounded growth suggests open-ended evolution (Bedau & Packard 1991)."
6. **"Environment"**: `active_droughts` as shaded bands across all charts (a shared x-band annotation), plus the season sine as a thin strip. Drought bands appear on **every** chart, so cause and effect can be seen at a glance.

Every chart's x-axis shares a hover crosshair (synchronised cursors).

### 6.3 Species tab

A table of up to 12 rows, sorted by population: swatch, name ("Sp. 3", plus an optional generated binomial such as *"Sp. 3, Lowland grazer"* derived from traits: trophic → grazer/omnivore/hunter, temp pref → northern/southern, size → small/large), population with trend arrow (`trend`), mean body size, mean links, trophic level, and age of the species in ticks. Clicking a row highlights that species in the scene (others dim).

### 6.4 Event feed ("What's happening")

**Format:** `Day N` timestamp (Day = tick / daynight.period, which gives readable numbers; show the tick in the tooltip), an icon, and one sentence of at most 140 chars. Newest on top, max 50 kept. Clicking an event with a location flies the camera there; clicking one with a species highlights that species.

**Detection runs in JS** from per-second stats plus WASM events (§9.4). All thresholds below are front-end constants in one `events.js` table, not sim constants, so they don't violate the "every number in config" rule for sim-core.

| Event | Trigger (evaluated once per second of sim-time-batch) | Priority | Example copy |
|---|---|---|---|
| Opening | t = 0 | — | "300 creatures with random brains have just been released. Watch which strategies survive." |
| Drought starts | drought list gains an entry | high | "A drought has struck the north-east. Food there will regrow at 5% for the next ~5,000 ticks." (compass from x,y: N/NE/E/…/centre) |
| Drought ends | entry disappears | med | "The north-east drought has ended. Food is starting to return." |
| Population crash | population falls ≥ 25% vs. the max over the last 1 000 ticks | high | "Population crash: down 31% to 330. {cause hint}". Cause hint: if a drought started in the window, "likely the drought"; if season multiplier < 0.8, "winter food shortage"; else omitted |
| Population boom | population ≥ 95% of hard_cap for the first time in 2 000 ticks, or +30% in 1 000 ticks | low | "Near the configured cap: 500 creatures. Births can be blocked at 500." |
| New species | stable species ID first seen **and** reaches ≥ 10 members (avoid noise from 1-member clusters) | high | "A new species has emerged: Sp. 9 ●, branching from Sp. 3. It's larger and hunts more." (diff vs. parent species' mean size and trophic level, if available) |
| Extinction | species with peak ≥ 10 members reaches 0 | high | "Sp. 4 ● has gone extinct after 8,200 ticks. At its peak it numbered 140." |
| Predation surge | `predator_count` ≥ 2× its 2 000-tick mean and ≥ 10, or trophic level crosses 1.1 / 1.25 / 1.4 upward | med | "Hunting accounts for 20% of intake on average, over creatures’ lifetimes." |
| Cooperation surge / collapse | `cooperation_rate` crosses 25 / 50 / 75% (either way, with 5-pt hysteresis) | med | "Energy sharing is more frequent, mostly between similar-hue creatures." |
| Record brain | `mean_connections` new all-time max that is ≥ 10% above the last announced record | low | "Brain links reached a new record: 52 enabled connections per creature." |
| Generation milestone | `max_generation` crosses 10, 25, 50, 100, 250, 500, 1000 | low | "Generation 100: the deepest lineage is 100 generations removed from its random ancestor." |
| Season | season multiplier crosses its max/min (seasons.period = 10 000 ticks) | low | "Winter is setting in: food is regrowing 30% slower." |
| Selected creature | follow mode only: births, kills, death of the followed creature | med | "#18342 just had offspring (#18990)." / "#18342 starved at age 812." |

**Anti-spam rules:**
- **Global rate limit:** at most 1 event per 3 s real time at 1× and 4×, and 1 per 2 s at 16×. Extra events go into a queue, and **low-priority events are dropped** if the queue is longer than 3.
- **Per-type cooldown:** the same event type can't repeat within 5 000 ticks (crash, boom, surge) or 1 000 ticks (new species). Multiple new species in one cooldown are merged: "3 new species have emerged in the last 1,000 ticks."
- **Hysteresis** on all threshold crossings (crossing must be confirmed for 200 ticks).
- **Coalescing:** extinction plus new species within 200 ticks of each other with similar traits is reported as "Sp. 4 has been replaced by its descendant Sp. 9" (v2).

---

## 7. Inspector

Opens on creature click as a drawer (desktop) or bottom sheet (phone). Data comes from `tw_inspect(id)` (§9.3), refreshed **4×/s** for scalars and **every frame** for brain activations while open.

### 7.1 Header and plain-English sentence

`Creature #18342 · Sp. 3 ● · Gen 57 · Age 812 / ~2000`

**"What is it doing?"** is a single sentence, regenerated 2×/s from the last action and the nearest sensed object. Rules, in priority order:

| Condition | Sentence |
|---|---|
| attacked this tick | "Attacking a {smaller/larger} {same-species/other-species} creature." |
| shared this tick | "Sharing energy with a {relative/stranger} (kin similarity {k})." |
| reproduced in the last 20 ticks | "Just had offspring." |
| eating and food > 0 under it | "Grazing on {algae/fruit}." |
| constructing > 0.5 | "Improving the ground here." |
| speed > 0.6 and nearest ray sees food | "Heading toward food it can see." |
| speed > 0.6 and nearest ray sees a creature and attack output > 0 | "Chasing another creature." |
| energy < 25% | "Running low on energy and searching for food." |
| night and speed < 0.2 | "Resting in the dark." |
| else | "Wandering." |

### 7.2 Brain graph (SVG, live)

- **Layout:** three columns. **20 inputs** on the left, grouped and labelled: Vision (5 rays × dist/type), Self (energy, heading, size), Social (signal, kin), World (temperature, scent, light, built ground, food here; the last was added on 2026-11-15). **Hidden** nodes in the middle, x-position = NEAT `depth` (0..1), y = order. **7 outputs** on the right: turn, speed, eat, attack, signal, share, build.
- **Nodes:** circles r = 5 px (inputs/outputs r = 6). Fill = activation on a diverging map (`#4C8FD6` −1 → `#2A2C28` 0 → `#E69F00` +1), stroke `--ink-3`.
- **Edges:** cubic curves. Width = `0.5 + 2.5 × |w|` (clamped), colour by sign (positive `--ink-2` warm grey, negative dashed). **Methylated** (epigenetically silenced) weights are drawn at `opacity × (1 − methylation)`, with a tooltip "silenced by stress (Jablonka & Lamb 2005)".
- **Live activations:** node fills update every frame. On edges, optionally animate a 2-px dot moving along strongly-active edges (|w·a| > 0.5), v2.
- **Scale:** with up to ~180 connections, draw edges at 25% opacity and highlight a node's in/out edges on hover. "Show strongest 30 connections" is a default-on toggle on phone.
- **Caption under the graph:** "41 connections · 9 hidden neurons · costs 0.18 energy/tick to run". Explains the trade-off in one line.

### 7.3 Energy budget

A **horizontal diverging bar** over the last 100 ticks: income on the right (food-algae, fruit, hunting, received shares), spending on the left (base metabolism, movement, brain, temperature mismatch, construction, attacks, given shares, reproduction). Below it, `Energy 42 / 100` as a thin meter. Tooltip: "Energy is the only currency: food in; moving, thinking and breeding out."

### 7.4 Lineage

- Breadcrumb: `#4 (founder) → … → #17020 (parent) → #18342`, plus `children: 3 alive, 5 total`.
- `Lineage: descends from founder #4, one of 23 surviving founder lines` (from `root_ancestor_id`, `lineage_diversity`).
- Buttons: **Follow**, **Highlight family** (dims all but same root ancestor), **Jump to parent** (if alive).

### 7.5 Traits card

Two-column key/value in plain words: Body size "1.8 (large)", Diet "78% plants · 22% meat", Prefers "cool north", Ages "slowly" (aging_rate percentile), Mutation rate "0.03", Brain entropy "varied behaviour (0.62)".

---

## 8. Visual identity: "Field Notebook, after dark"

The look of a naturalist's notebook on a lab bench at night: graphite and moss, warm paper ink, measured accents. Dark default, with a light "paper" theme available via a toggle (good for screenshots on the portfolio).

### 8.1 Palette tokens

```css
:root {                       /* dark (default) */
  --bg:        #141613;  /* deep graphite-moss, page + scene clear colour */
  --bg-2:      #1C1F1B;  /* panels */
  --bg-3:      #252923;  /* cards, inputs, hover */
  --line:      #343A32;  /* hairlines, dividers */
  --ink-1:     #ECE8DD;  /* primary text, "paper"   (contrast on --bg ≈ 15:1) */
  --ink-2:     #B8B3A6;  /* secondary text          (≈ 8.6:1) */
  --ink-3:     #858174;  /* tertiary/labels         (≈ 4.6:1, AA for ≥14px) */
  --accent:    #C9A65B;  /* brass: focus rings, active toggles, sparkline dot */
  --accent-2:  #7FB59A;  /* lichen: links, "on" states */
  --alert:     #E5484D;  /* attacks only */
  --warn:      #C98B3C;  /* droughts */
  --honey:     #F2C94C;  /* sharing */
  --sp-0 … --sp-7, --sp-other   /* species palette, §4.2 */
}
:root[data-theme="light"] {   /* "paper" */
  --bg: #F4F1E8; --bg-2: #EBE6D9; --bg-3: #E0DACB; --line: #CFC7B4;
  --ink-1: #1E211C; --ink-2: #4A4D44; --ink-3: #6E6E62;
  --accent: #8A6A22; --accent-2: #3E7A5E;
}
```

Scene: clear colour `--bg`, with exponential fog (`FogExp2`, density 0.035) toward `--bg` so the far terrain dissolves into the page and the 3D view feels part of the document rather than a game window.

### 8.2 Typography (Google Fonts)

- **UI and body:** **Inter** (400/500/600). Tabular numerals (`font-feature-settings: "tnum"`) on every KPI and counter so the numbers don't jitter as they change.
- **Headings / wordmark / chart titles:** **Fraunces** (soft optical serif, 600, opsz 48), the "naturalist" voice. Use it for the wordmark "Terrarium", tour cards and chart titles only.
- **Data / mono:** **JetBrains Mono** (400) for tick, seed, IDs and cell readouts.
- Scale: 12 / 14 / 16 / 20 / 28 / 40 px. KPI numbers 28 px Inter 600 tnum, labels 12 px uppercase 0.06em tracking in `--ink-3`.

### 8.3 Iconography

- **Lucide** icons (pinned version, 1.5 px stroke, 18 px) for UI controls: play, pause, rotate-ccw (reset), layers, map, locate (follow), help-circle, copy (seed link), sun/moon.
- **Custom 16 px glyphs** (inline SVG, same 1.5 px stroke) for sim concepts, also used in feed entries and legends: drought (cracked circle), new species (branching fork), extinction (fork with a cut branch), predation (two chevrons converging), sharing (two dots with an arc), brain (node triad), generation (stacked arcs), season (half-filled circle).
- The **wordmark** is "Terrarium" in Fraunces next to a small seed-pod silhouette (the creature shape from §4), which doubles as the favicon.

### 8.4 Motion

Durations are 150 ms (UI), 300 ms (panels) and 700 ms (camera), with easing `cubic-bezier(.2,.7,.2,1)`. Respect `prefers-reduced-motion`: no auto-rotate, no camera tweens (cut instead), no number tweening, and state cues reduced to static icons.

### 8.5 Accessibility

- All colour encodings have a redundant non-colour cue: shape for trophic role, icons for events, labels in legends, and a dash pattern for negative brain weights.
- The species palette was chosen from Okabe–Ito for deuteranopia and protanopia distinctness. Before shipping, QA must check it with a CVD simulator (Chrome DevTools "Emulate vision deficiencies").
- Keyboard: `Space` play/pause, `1/2/3` speed, `M` map, `F` follow, `L` cycle layers, `Esc` close inspector, `?` tour. Visible brass focus ring.
- The KPI strip and feed are real DOM text (screen-readable). The feed is an `aria-live="polite"` region, rate-limited by the same rules.

---

## 9. Data contract: what the front end needs from WASM

Conventions: all numeric buffers are `f32` (or `u32` where noted) in **linear WASM memory**, exposed as a `(ptr, len)` pair. JS builds `new Float32Array(memory.buffer, ptr, len)` and **must re-create the view after any call that may grow memory** (check `memory.buffer !== lastBuffer`). Sim coordinates stay in sim units (0..1000). The front end does the scene mapping.

### 9.1 Lifecycle and control exports

| Export | Purpose |
|---|---|
| `tw_init(cfg_ptr, cfg_len, seed: u64) -> i32` | Build `Simulation` from TOML (the default config is embedded at compile time, and JS may pass overrides). Returns 0 on OK |
| `tw_step(n: u32)` | Advance n ticks. Must also accumulate events (§9.4) across those n ticks |
| `tw_alloc(len) / tw_free(ptr,len)` | for passing config strings |
| `tw_tick() -> f64`, `tw_seed() -> u64` | |
| `tw_config_json_ptr/len()` | **once at init**: world w/h, food cols/rows/cell_size/max_per_cell, `creature.hard_cap`, `max_energy`, `max_age`, `daynight.period`, `seasons.period/amplitude`, `pheromones.max_level`, `niche_construction.max_level`, `brain.inputs/outputs`, `food.biomes[]` (x, y, radius, multiplier), which subsystems are enabled. This lets the front end hard-code nothing |

### 9.2 Per frame (after each `tw_step`)

**Creature buffer** `tw_creatures_ptr() / tw_creature_count() / tw_creature_stride()`, stride **16 f32 per creature** (64 B × 500 = 32 KB):

| off | field | source | status |
|---|---|---|---|
| 0 | id (as f32 bits via a parallel `u32` view, or split into lo/hi) | `org.id` | exists. **Use a separate `Uint32Array` ids buffer** (`tw_creature_ids_ptr`) to avoid f32 precision loss above 2^24 |
| 1 | x | `pos_x` | exists |
| 2 | y | `pos_y` | exists |
| 3 | heading (rad) | `heading` | exists |
| 4 | energy / max_energy [0,1] | `energy` | exists |
| 5 | body_size | `genome.body_size` | exists |
| 6 | speed_norm [0,1] | last `Action.speed` | **NEW: Action buffer is private and transient; persist it** |
| 7 | turn [-1,1] | last `Action.turn` | **NEW** |
| 8 | stable species id (u32 as f32, < 2^24 fine) | — | **NEW (§9.5)** |
| 9 | trophic: predation fraction [0,1] | `energy_from_attacks / (food + attacks)` | derivable (compute in WASM) |
| 10 | action flags bitmask (as f32 int): 1=ate, 2=attacked, 4=was attacked, 8=shared, 16=received share, 32=constructed, 64=reproduced, 128=newborn | — | **NEW: per-tick action outcomes are not recorded on `Organism`** |
| 11 | interaction target index (row in this buffer, −1 none), for attack/share line endpoints | — | **NEW** |
| 12 | generation | `generation` | exists |
| 13 | age / max_age | `age` | exists |
| 14 | signal [-1,1] | `signal` | exists |
| 15 | reserved (v2: diet_preference or preferred_temp) | genome | exists |

Flags must be **OR-accumulated across the n ticks of a `tw_step(n)` batch** and cleared at the start of each `tw_step`. Otherwise cues are lost at 16×.

**Dead-this-step buffer** `tw_deaths_ptr/len()`: `[id_u32, x, y, cause_u8, species_id]` per creature that died during the last `tw_step`, needed for death animations because `organisms.retain` removes them before JS can see them. **NEW: death cause not tracked.** Causes: 0 starvation (energy ≤ 0), 1 old age (age ≥ max_age), 2 senescence (Gompertz roll), 3 killed (energy ≤ 0 *and* attacked this tick, so killed-by needs attack tracking).

**Global per-frame scalars** (`tw_frame_ptr()`, 8 f32): tick (f64 split or separate `tw_tick`), `light_level` (**NEW export: currently a local in `step()`**), `seasonal_multiplier(tick)` (function exists in world.rs), population, active_droughts.

### 9.3 Grids (100×100, f32, row-major, y-major rows)

| Grid | Source | Cadence | Status |
|---|---|---|---|
| food (algae) | `food_grid.cells` (f64) | every 4 frames | exists. **Store or mirror as f32** (or WASM converts into a persistent f32 scratch buffer, 40 KB) |
| fruit | `food_grid.fruit_cells` | every 4 frames | exists |
| pheromone | `pheromone_grid.cells` | every 4 frames, only when layer active | exists |
| construction | `construction_grid.cells` | every 4 frames, only when layer active | exists |
| regrowth rate (biome map) | `food_grid.regrowth_rates` | **once** at init | **private field: needs a getter** |
| elevation | sample `elevation_map.height_at` at cell centres | **once** at init (static) | **NEW export `tw_elevation_grid(cols, rows)`**: the map is analytic with no stored grid |
| temperature | analytic `y/height` | front end computes | none needed |
| droughts | `food_grid.droughts`: `[x, y, radius, severity, ticks_remaining]` × n | every frame (tiny) | exists |

JS takes the grid dims from the config JSON.

### 9.4 Per second (JSON is fine at 1 Hz)

`tw_stats_json_ptr/len()`, which serialises `TickStats` (all fields exist and are `Serialize`) **plus**:
- `species: [{stable_id, population, mean_body_size, mean_connections, mean_trophic_level, predator_fraction, trend, first_seen_tick, parent_stable_id}]`. **`TickStats.species` is currently `#[serde(skip)]`**: add a serialisable variant. `stable_id`, `first_seen_tick` and `parent_stable_id` are **NEW**.
- `season_multiplier`, `light_level`.

`tw_events_json_ptr/len()` drains an **event queue accumulated inside sim-core/sim-wasm since the last drain** (NEW):
- `{"t":tick,"kind":"species_born","species":9,"parent":3,"size":12}`. **NEW**: clustering already knows when a "New species" branch is taken (sim.rs ~L1205), but the event is dropped and the index is not stable.
- `{"kind":"species_extinct","species":4,"peak":140,"lifespan":8200}`. **NEW**
- `{"kind":"drought_start","x":..,"y":..,"radius":..,"severity":..,"duration":..}` / `drought_end`. **NEW** (derivable in JS by diffing the drought list, so optional)
- `{"kind":"record","what":"max_generation","value":100}`. Optional, JS can do it.

Population crash/boom, predation/cooperation surges, records, milestones and seasons are **computed in JS** from the stats history. They need no sim-core change.

### 9.5 Inspect (on demand, 4 Hz while open; activations every frame)

`tw_inspect(id) -> json`:
- scalars: id, parent_id, root_ancestor_id, generation, age, birth_tick, energy, max_energy, body_size, color_hue, aging_rate, preferred_temp, diet_preference, mutation_rate, mutation_step, energy_from_food, energy_from_attacks, signal, brain entropy (from `output_bins`), species stable id. **All exist except the species id.**
- brain topology: `nodes: [{id, kind: in|hidden|out|bias, depth}]`, `edges: [{from, to, weight, enabled, methylation}]` from `genome.connections` and `hidden_nodes`. Exists.
- `children_alive`, `children_total`. **NEW**: there is no child index. `children_alive` can be computed by scanning `parent_id` (O(n), fine at 4 Hz). `children_total` needs a counter on `Organism`.
- **energy budget over the last 100 ticks**, per category (food, fruit, hunting, shares received / base metabolism, movement, brain, temperature, construction, attack cost, shares given, reproduction). **NEW**: none of these per-organism flows are recorded except lifetime `energy_from_food` and `energy_from_attacks`. Proposal: an `[f32; 12]` EMA accumulator on `Organism` (allocation-free, deterministic, no RNG use, so it doesn't affect determinism).
- `last_inputs[20]`, `last_outputs[7]` for the "what is it doing" sentence. **NEW**: inputs are built on the stack and discarded.

`tw_brain_activations(id) -> ptr` (per frame while the inspector is open): `[f32; MAX_NODES]` node activations from the last forward pass. **NEW**: `Brain::forward` discards its stack activations. Options: (a) re-run `forward` on the stored `last_inputs` in sim-wasm, which is pure and doesn't touch the RNG, so it is safe, and needs a `forward_with_activations` variant; or (b) store activations for one "watched" id only. **Prefer (a).**

### 9.6 Summary of sim-core gaps (for the bindings engineer)

1. **Stable species IDs.** Today `SpeciesInfo.id` is a per-tick index into `representatives`, which reorders when earlier species vanish, and infos are sorted by population. Need a monotonically increasing `stable_id` carried with each representative, plus a per-organism `species_id` field (or a parallel `Vec<u32>`).
2. **Species birth/extinction events** with parent species (parent = species of the organism that founded the new cluster, looked up via `parent_id`).
3. **Per-organism last action** (speed, turn, and the outcome flags ate/attacked/attacked-by/shared/received/constructed/reproduced) plus the interaction target.
4. **Death records with cause** (starvation / old age / senescence / killed), emitted before `retain`.
5. **Light level** exposed (currently a local), plus the season multiplier (the function exists).
6. **Elevation grid sampling** and a **regrowth-rate (biome) getter**.
7. **Per-organism energy-flow EMA** (12 categories) for the energy budget.
8. **Last inputs/outputs and node activations** for the inspector.
9. **Serialisable species list** (`#[serde(skip)]` today).
10. **f32 mirrors** of the f64 grids (or conversion in sim-wasm into persistent scratch buffers, which keeps sim-core untouched).

All of these are observational only: no RNG draws and no behaviour change, so **same seed = same run is preserved**. Items 3, 4 and 7 add a few fields to `Organism`. Keep them fixed-size (no per-tick `Vec` allocation).

Note for docs: CLAUDE.md still says 15 inputs / 6 outputs. Current config is **19 inputs / 7 outputs** (adds temperature, pheromone, light, niche level and the `construct` output). The brain-graph labels in §7.2 follow the current config and should be read from `tw_config_json` (`brain.inputs/outputs`) rather than hard-coded.

---

## 10. MVP vs. v2

### MVP (ship this for the portfolio)

1. Layout: header + hero subtitle, KPI strip (5 tiles + sparklines + tooltips), viewport, Feed tab, time bar (play/pause, 1×/4×/16×, seed display, new seed, copy link `?seed=`). Phone layout with top-down default.
2. Terrain: 100×100 flat-shaded heightfield, Food layer (algae + fruit dots), biome contour rings, drought rings + desaturation, muted day/night (off at ≥4×), fog.
3. Creatures: instanced seed-pod body + eyes + inverted-hull outline, species palette with stable IDs, energy fade, size, heading, bobbing gait, predator silhouette swap, death fade-out.
4. State cues: eating, attacking (line spark), reproducing, dying. Selection ring.
5. Camera: Orbit + Map + Follow, click-picking (CPU nearest).
6. Event feed: opening, drought start/end, crash/boom, new species, extinction, predation surge, cooperation threshold, generation milestones, with the rate limiter.
7. Inspector: header, "what is it doing" sentence, **static brain graph with live output-node activations** (outputs only need `last_outputs`), traits card, lineage breadcrumb (parent/root only), Follow.
8. Guided tour (4 steps), legend card, keyboard shortcuts, reduced-motion support, adaptive pixel ratio.
9. WASM gaps needed for MVP: §9.6 items **1, 2, 3, 4, 5, 6, 9, 10**.

### v2

- Remaining layers: Temperature, Scent trails, Built ground, Elevation with contours and an exaggeration slider. (Data is cheap, so promote to MVP if time allows. Elevation and Scent are the most visually rewarding.)
- Charts tab (6 charts, synced crosshair, drought bands) and the Species tab with generated names.
- Sharing and constructing cues. Trails for all creatures.
- Inspector: full live hidden-node activations (§9.5 option a), energy-budget bar (needs gap 7), children counts, "highlight family", methylation rendering.
- Event coalescing (replacement narratives), followed-creature events, season events, record-brain events, camera fly-to on event click.
- Light "paper" theme toggle and a screenshot/export button (PNG of the canvas with the KPI strip, for sharing).
- "Experiment mode": 2–3 sliders (food regrowth, brain cost, sharing benefit) that reset with the same seed, an explorable "what if" in the spirit of Nicky Case. It needs a config-override path in `tw_init`.
- Web Worker + SharedArrayBuffer sim (needs COOP/COEP; GitHub Pages can't set these headers, so it would need a service-worker shim).

---

## Task lab: saved native evaluation replay

The optional **Task lab** toolbar button opens a keyboard-accessible native dialog, separate from the live ecology. It replays saved native visible-cue evaluations for all nine declared final training champions across seven frozen cases and both cues, including partial and unsuccessful searches. A separate cohort exposes three fixed manufactured controls. Select cohort, policy, case and cue; play, step, restart or scrub exact saved movement poses. The display reports recorded choice, first-contact tick and two-cue correctness. Goal labels and path overlays are replay annotations, not additional policy inputs.

`analysis/export_visible_task_demo.py` deterministically derives `web/demo/data/visible-task.json` from the three archived robustness result files. It retains source SHA-256 identities and exact poses, goal geometry and outcomes; it runs no policy evaluations. The 190 KB payload loads only when the dialog opens. The page links the evaluation plan, results/controls and replay provenance. Labels explicitly say prerecorded native evaluation and visible cue; a memory benefit is not established. Opening the lab changes no engine configuration or live-world defaults. On phones, a two-row header keeps the toolbar controls available.

## References (design and visualisation)

- Victor, B. (2011). *Explorable Explanations*. worrydream.com/ExplorableExplanations
- Case, N. (2017). *Explorable Explanations* collection and *The Evolution of Trust*. ncase.me
- Olah, C. & Carter, S. (2017). Research Debt. *Distill*. (Distill.pub interaction norms: inline figures, progressive disclosure)
- Chan, B. W.-C. (2019). Lenia: Biology of Artificial Life. *Complex Systems* 28(3). (Live-demo presentation of ALife)
- Sims, K. (1994). Evolving Virtual Creatures. *SIGGRAPH '94*. (Showing evolved behaviour through simple bodies)
- Segel, E. & Heer, J. (2010). Narrative Visualization: Telling Stories with Data. *IEEE TVCG* 16(6). (Martini-glass structure: guided tour → free exploration)
- Smith, N. & van der Walt, S. (2015). *A Better Default Colormap for Matplotlib* (viridis). SciPy 2015.
- Nuñez, J. R., Anderton, C. R. & Renslow, R. S. (2018). Optimizing colormaps with consideration for color vision deficiency (cividis). *PLOS ONE* 13(7).
- Crameri, F., Shephard, G. E. & Heron, P. J. (2020). The misuse of colour in science communication. *Nature Communications* 11, 5444.
- Okabe, M. & Ito, K. (2008). *Color Universal Design: How to make figures and presentations that are friendly to colorblind people*.
- Borland, D. & Taylor, R. M. (2007). Rainbow Color Map (Still) Considered Harmful. *IEEE CG&A* 27(2).
