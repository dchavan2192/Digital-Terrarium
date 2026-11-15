# Prior-Art Audit for Terrarium (ALIFE 2027)

Status: research note. Nothing was run. Date: 2026-11-15. Author role: research scientist.
Question from the owner: "Have any of these been created before? I don't want to reinvent the wheel."

Short answer: **yes, every individual mechanism has been built before**, several of them in neural-agent ecologies. The closest prior platform is **Polyworld** (Yaeger 1994), which already had evolved neural brains with an energy cost per neuron and per synapse, sexual reproduction with crossover where both parents pay energy, predation, light signalling and, since 2008, a neutral "lockstep" null model. What is still defensible is narrow: **a controlled, shadow-normalised factorial test of three inheritance channels (genetic, epigenetic, ecological) inside one evolved-neural-agent ecology**. As far as this audit found, nobody has published that.

### Verification legend

Each citation is tagged:

- **[V]**: verified in this audit against the publisher, the arXiv record, or the paper's own text (authors, year and venue checked; quotes come from the text).
- **[P]**: partially verified. The work exists, but the venue, pages or a technical detail comes from a secondary source or remains uncertain.
- **[M]**: from memory, not re-checked here. **Check it before it goes into the paper.**

---

## 1. Closest existing platforms, compared with Terrarium

Key: Y = yes, N = no, ~ = partial or analogous, ? = not verified.

| Platform (citation) | Controller | Topology evolves | Lifetime learning | Sexual repro | Brain energy cost | Predation | Signals | Env. modification / niche constr. | Epigenetics | OEE stats / neutral null |
|---|---|---|---|---|---|---|---|---|---|---|
| **Terrarium** | NEAT feed-forward | Y | N (planned 9c) | Y, both parents pay | structural + activity Σ\|w·a\| | Y | signal + pheromone | Y (construction grid → regrowth) | Y (per-connection, stress-induced, inherited) | Bedau activity; shadow designed, not built |
| **Polyworld**, Yaeger 1994 [V text; pages P] | evolved neural groups, vision input | Y (group counts/densities in genome) | **Y (Hebbian)** | **Y, crossover; each parent donates a gene-set fraction of energy** [V] | **Y: "determined linearly from the number of neurons and the number of synapses"** [V] | Y (attack; corpses become food) [V] | Y ("lighting") [V] | N (?) | N | complexity measures; **lockstep neutral null (2008)** [V] |
| **Geb**, Channon 2001/2003/2006 [V 2006; P 2001/2003] | neural nets (developmental encoding) | Y | ? | ~ (?) | ? | Y (kill) | ? | N | N | **Bedau activity + neutral shadow; "first autonomous system to pass" (2006)** [V] |
| **Avida**, Ofria & Wilke 2004 [M] | CPU programs | n/a | N | optional (Misevic et al. 2006 [V]) | n/a | added in later work [M] | N | resource depletion only | N | **MODES validated on Avida** (Dolson et al. 2019) [V] |
| **Tierra**, Ray 1991 [M] | machine code | n/a | N | N | CPU time | parasites | N | N | N | studied by Bedau et al. [M] |
| **Echo**, Holland 1995 [M] | tag/condition strings | n/a | N | Y (mating via tags) [M] | n/a | Y (combat) | N | N | N | **original neutral shadow** (Bedau, Snyder & Packard 1998) [V] |
| **Sugarscape**, Epstein & Axtell 1996 [M] | rules | N | ~ (culture) | Y | metabolism | Y (combat rule) | ~ (culture tags) | ~ (pollution) | N | N |
| **Framsticks**, Komosinski [M] | neural nets + 3D bodies | Y | ~ | Y | Y (energy) [M] | ~ | ? | N | N | N |
| **rtNEAT / NERO**, Stanley, Bryant & Miikkulainen 2005 [M] | NEAT | Y | N | Y (NEAT crossover) | N | task-driven, user fitness | N | N | N | N |
| **Critterding**, Winckelmans 2005– [P] | neural nets, 3D physics | Y | N | ? | energy-based | Y | ? | N | N | N |
| **Gene Pool**, Ventrella 1997/2005 [P] | motor genes (not neural, as believed) | N | N | **Y, mate preference / sexual selection** [P] | energy | N | N | N | N | N |
| **Darwinbots** [M] | DNA scripting language | n/a | N | ? | energy | Y | Y | N | N | N |
| **ALIEN**, Heinemann [P] | per-cell small neural nets, particle bodies, CUDA | Y | ? | ? | ? | Y ("hunt") | Y (communicator cells) | ~ (constructor cells build bodies) | N | N |
| **The Bibites**, Caussan 2016– [P] | neural nets, starting from empty brains | Y | ? | **N (planned)** [P] | ? (sensing costs energy [P]) | Y | **Y (pheromones)** [P] | N | N | N |
| **Neural MMO**, Suarez et al. 2019; 2.0 at NeurIPS 2023 D&B [M] | RL policies (deep) | N | Y (RL) | N | N | Y | ~ | ~ (resource depletion) | N | N (episodic, no evolution) |
| **EcoEvoJax**, Hamon, Nisioti & Moulin-Frier 2023, GECCO Companion [P] | evolvable RNNs, JAX | N (weights) | N | N (?) | ? | N (?) | N | resources regrow; niche work in companion paper | N | N |
| **JaxLife**, Lu, Beukman, Matthews & Foerster 2024, arXiv [V; venue unverified] | LSTM + attention deep nets | N | ~ (recurrent memory) | **N (asexual weight copy + noise)** [V] | action- and terrain-dependent, rising with age [V] | ? | **Y (communication)** | **Y: terraforming that "slowly regress[es]" to a base state** [V] | N | N (Kardashev-style energy metrics) |
| **Biomaker CA**, Randazzo & Mordvintsev 2023, arXiv [V] | NCA plant "agents", JAX | ~ | N | ~ | energy/nutrients | N | N | Y (plants alter the grid) | N | N |
| **Lenia** Chan 2019 [M] / **Flow-Lenia** Plantec et al. ALIFE 2023 [V] | continuous CA (no agents) | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | aims at OEE; contrast only |
| **ABMax predator–prey**, arXiv 2606.13639 [P] | RNN controllers, JAX | N | N | ? | internal energy | Y | N | N | N | N |
| **Amee**, Mack 2023 PhD thesis, Heidelberg (Utopia framework) [P] | not neural (as believed) | ~ | plasticity | ? | metabolic | ? | ? | **Y (niche construction)** | thesis discusses epigenetic inheritance; whether it is implemented is **unverified** | N |
| **Chromaria**, Soros & Stanley 2014 [M] | CPPN/NEAT | Y | N | ? | ? | N | N | ~ (bodies become environment) | N | studied conditions for OEE |
| **biosim4**, D. Miller 2021 (GitHub) [M] | small neural nets | N | N | Y | N | ~ | ~ (pheromone) | N | N | N |

I searched for "EvoSphere" and found no ALife platform by that name. Treat it as unverified or nonexistent.

**Reading the table.** Polyworld covers most of Terrarium's ecology and physiology columns, and it adds Hebbian learning, which Terrarium only plans. Geb covers the OEE-with-shadow column in a neural-agent system. JaxLife covers environment modification by neural agents. No row besides Terrarium has an implemented epigenetic column, and that is the one column with real white space.

---

## 2. Prior work for each claimed-novel mechanism

### 2.1 Niche construction (claim in PROGRESS.md: "No neural-agent ALife platform has this — completely novel")

**The claim is false. Remove it.**

- **JaxLife** (Lu et al. 2024) [V]: neural (LSTM) agents spend energy to terraform cells, changing terrain properties. Changes "slowly regress" toward a base state at a rate proportional to the cell's maximum energy. That is structurally the same as Terrarium's construction grid with decay feeding food regrowth.
- **Chiba, Suzuki & Arita 2020**, "Evolution of Complex Niche-Constructing Behaviors and Ecological Inheritance of Adaptive Structures in a Physically Grounded Environment," *Frontiers in Robotics and AI* 7, doi:10.3389/frobt.2020.00045 [V title/venue; controller type unverified]. Prey build structures that protect them from predators, and the paper explicitly manipulates **ecological inheritance** of those structures across generations.
- **Jaafar, Suzuki & Arita 2025**, "Eco-Evo-Devo in the Adaptive Evolution of Artificial Creatures Within a 3D Physical Environment," *Electronics* 14(2):354 [P]: creatures build objects and develop morphologically. It reports that too much environmental modification can hinder adaptation.
- **Nisioti & Moulin-Frier 2023**, "Dynamics of niche construction in adaptable populations evolving in diverse environments," arXiv:2305.09369 [V; no published venue found]: plasticity, evolvability and niche construction co-evolve.
- **Taylor 2004**, "Niche Construction and the Evolution of Complexity," *Artificial Life IX*, pp. 375–380 [V]: an individual-based ALife model in which niche construction drives organisms toward more genes. It is not neural.
- **Silver & Di Paolo 2006**, "Spatial effects favour the evolution of niche construction," *Theoretical Population Biology* 70(4):387–400 [V]: an individual-based spatial version of the Laland two-locus model.
- **Hayashi, Suzuki & Arita 2015**, a minimal model of learning plus niche construction, BICT 2015 [P].
- Theory: Odling-Smee, Laland & Feldman 2003 [M]; Laland, Odling-Smee & Feldman 1999, *PNAS* 96:10242–10247 [M].

**Narrow, defensible version:** an *evolvable, continuous construct output* of a topology-evolving (NEAT) brain that **multiplies** a cell's biome × season × drought regrowth. That design makes niche construction an evolved buffer against droughts, which is testable. The closest prior work is JaxLife and Chiba et al. 2020.

### 2.2 Epigenetic inheritance (claim: "Almost completely absent from ALife literature (only EpiAL)")

**Overstated, and the EpiAL citation is wrong.**

- **Citation error.** The outline cites "Sousa & Costa (2010) Proc ALIFE XII, 131–138". The correct reference is **Sousa, J. & Costa, E. (2010), "EpiAL – An Epigenetic Approach for an Artificial Life Model," *Proc. ICAART 2010* (Vol. 1), pp. 90–97, doi:10.5220/0002732500900097** [V]. An extended version appeared as "Designing an Epigenetic Approach in Artificial Life: The EpiAL Model," ICAART 2010 Revised Selected Papers (Springer CCIS), pp. 78–90 [P]. EpiAL agents use **binary-string** behaviour, not neural nets [P, via Ricalde 2019].
- **Turner, Lones, Fuente, Stepney, Caves & Tyrrell 2013**, "The incorporation of epigenetics in artificial gene regulatory networks," *BioSystems* 112(2):56–62 [V]: an epigenetic control layer on artificial GRNs (AERN).
- **Brawer, Hill, Livingston, Aaron, Bongard & Long 2017**, "Epigenetic Operators and the Evolution of Physically Embodied Robots," *Frontiers in Robotics and AI* 4:1 [V]: an epigenetic operator applied to evolving Braitenberg-style robot circuits.
- **Tanev & Yuta 2008**, "Epigenetic programming," *Information Sciences* [M].
- **Ricalde 2019**, GP with an epigenetic mechanism (PhD thesis, Memorial University, arXiv:1903.03854) [V].
- A 2008 *ALife XI* paper from Southampton on methylation patterns as an evolvable inheritance channel and genetic assimilation (eprints.soton.ac.uk/266571) [P; authors not confirmed, page returned 403].
- **Hadjiivanov & Blair 2019**, epigenetic neuroevolution of CNNs, IEEE CEC [P]: neural, but for image classification rather than agents.
- **Already inside NEAT**: Stanley & Miikkulainen 2002 genes carry a heritable *disabled* flag [V via search], which is binary, heritable silencing of connections. Terrarium's methylation generalises it to a continuous mask that is *environmentally induced* and *noisy when inherited*. Reviewers will make this comparison, so pre-empt it.
- Related non-genetic inheritance: Lamarckian evolution of robots (e.g. Jelisavcic et al. 2019, *Frontiers in Robotics and AI*) [M]. Biology: Jablonka & Lamb 2005 [M]; Day & Bonduriansky 2011 *Am Nat* [M]; Klironomos, Berg & Collins 2013 *BioEssays* [M].

**Narrow, defensible version:** stress-induced, per-connection, heritable methylation that silences synapses of an evolved NEAT controller and also acts on the *living* brain, inside an open-ended ecology. No exact prior was found. The closest prior work is EpiAL (agent-based, non-neural), Brawer et al. 2017 (embodied, non-heritable operator) and NEAT's disabled-gene flag.

**Caveat a reviewer will raise:** if marks never reset and are inherited with small noise, the mechanism is close to a *second genetic channel with a higher mutation rate plus Lamarckian writes*. You must show that it behaves differently from a genetic mask, for example through reversibility after stress ends.

### 2.3 Activity-dependent neural energy cost (claim: "not yet applied to ALife neural costs")

**Partly true, and the scope is narrow.**

- **Polyworld already charges for brains** [V, Yaeger 1994 text]: "The energy expended by an organism's neural processing is determined linearly from the number of neurons and the number of synapses it has." That is a **structural** cost, the same as Terrarium's `brain_cost` and `connection_cost` terms. Yaeger et al. (2008) loosely call it energy for "neural activity" [V], so phrase the contrast carefully.
- **Per-spike energy cost in evolved spiking controllers**: Jones, Jin, Sendhoff & Yao, "Evolution of Neural Organization in a Hydra-Like Animat," ICONIP 2008, LNCS 5506, pp. 216–223 (Springer 2009) [P]. Each spike costs energy proportional to connection length. This is the closest prior for an *activity* cost on an evolved controller, though it is not an open-ended ecology.
- **Connection cost** (structural): Clune, Mouret & Lipson 2013, *Proc. R. Soc. B* 280:20122863 [M].
- **Energy-aware fitness** in evolutionary robotics: McHale & Husbands, *ALife XII* 2010 [P].
- **JaxLife**: energy depends on actions and terrain, not on network activity [V].
- In ML, Σ\|w·a\| is an L1-style activity regulariser. Say so explicitly, because reviewers will.

**Narrow, defensible version:** a metabolic cost proportional to *realised synaptic work* Σ\|w·a\| (Attwell & Laughlin 2001), paid out of the same energy budget that funds reproduction, in an open-ended ecology. No exact prior was found in neural-agent ALife ecologies. The closest prior work is Polyworld (structural) and Jones et al. 2008 (spike cost). Whether it changes anything is an empirical question, and CLAUDE.md lists "brain cost vs complexity" as untested.

### 2.4 Neutral shadow models for OEE

**Not novel. Terrarium is adopting an existing method, which is the right thing to do.**

- **Bedau, Snyder & Packard 1998**, "A Classification of Long-Term Evolutionary Dynamics," *Artificial Life VI*, MIT Press [V; pages disputed: 189–198 in the Reed copy, 228–237 in bibliographic records]. This paper introduced the neutral shadow of Echo.
- **Channon 2001** (*ECAL 2001*, LNCS 2159) [P], **2003** (*Artificial Life VIII*, pp. 173–181; printed 2002) [P] and **2006** (*Genetic Programming and Evolvable Machines* 7(3):253–281, doi:10.1007/s10710-006-9009-3) [V] applied shadowed activity statistics to **Geb, a neural-agent world**.
- **Polyworld "lockstep" mode**: Yaeger, Griffith & Sporns 2008, "Passive and Driven Trends in the Evolution of Complexity," *Artificial Life XI* (arXiv:1112.4906) [V]. Quote: "every time a birth occurred in the original natural-selection run, a birth is forced to occur in the lockstep run, only instead of being produced by the original parents, the birth is produced by two agents chosen at random from the population." This is a **neural-agent neutral shadow with sexual reproduction**, which is exactly what OEE_SHADOW_DESIGN.md proposes. The design note does not cite it yet and should.
- **MODES**: Dolson, Vostinar, Wiser & Ofria 2019, *Artificial Life* 25(1):50–73, doi:10.1162/artl_a_00280 [V]. It provides change, novelty, ecology and complexity potentials, a C++ implementation, and validation on NK landscapes and Avida.
- **Channon 2019**, "Maximum Individual Complexity is Indefinitely Scalable in Geb," *Artificial Life* 25(2):134–144, doi:10.1162/artl_a_00285 [V].
- Taylor et al. 2016, *Artificial Life* 22(3):408–423 [M]; Bedau & Packard 1991, *Artificial Life II*. The repo gives the pages as 431–461 in one place and 431–462 in another; resolve this [M].

**Narrow, defensible version:** none on its own. Present the shadow as a *method* borrowed from Bedau, Channon and Yaeger, and claim novelty only for *what it is used to test* (§4b).

### 2.5 Genetic + epigenetic + ecological inheritance (Extended Evolutionary Synthesis) in one simulation

**No exact prior found. This is the strongest candidate.**

- Searches combining "epigenetic", "niche construction" and "neural agents / ALife" (standard and extended web search) found **no implemented simulation** with all three channels and evolved neural agents.
- Closest: **Amee** (Mack 2023, Heidelberg PhD thesis, on the Utopia framework) [P] unifies development, niche construction and phenotypic plasticity with selection, and discusses genetic, epigenetic and ecological inheritance. **Check whether epigenetic inheritance is actually implemented**, and whether the agents are neural (believed not). Also EpiAL (epigenetic only), JaxLife (ecological only) and Chiba et al. 2020 (ecological inheritance only).
- Conceptual and theoretical frameworks: Jablonka & Lamb 2005 [M]; Danchin et al. 2011, "Beyond DNA," *Nat. Rev. Genet.* [M]; Day & Bonduriansky 2011 [M]; Laland et al. 2015 "The extended evolutionary synthesis," *Proc. R. Soc. B* [M]. These frame the question but do not simulate it in ALife.

### 2.6 Twofold cost of sex in neural ALife

**Not novel. Polyworld already has biparental energy investment.**

- Polyworld: "The offspring's total available energy on birth is the sum of these contributions from the two parents" [V]. That is the same scheme as Terrarium's `mate_cost`.
- Avida: Misevic, Ofria & Lenski 2006, *Proc. R. Soc. B*, "Sexual reproduction reshapes the genetic architecture of digital organisms" [P]. Sex can evolve de novo in changing environments (Misevic thesis) [P], and later Avida work added configurable sex-specific costs [P].
- Conceptual caution: the classical twofold cost (Maynard Smith 1978) comes from **producing males and from anisogamy**, not from both parents paying (Lehtonen, Jennions & Kokko 2012, "The many costs of sex," *TREE* [P]). In Terrarium's hermaphroditic agents, "mate pays too" doubles the **per-offspring energetic cost**. Call it that ("a twofold per-offspring investment cost"), not *the* twofold cost of sex, or a reviewer will object.

### 2.7 Other mechanisms (brief)

- **Kin-directed sharing / Hamilton's rule**: well studied in ALife and evolutionary robotics, for example Waibel, Floreano & Keller 2011, "A quantitative test of Hamilton's rule for the evolution of altruism," *PLoS Biology* [M]. Not novel.
- **Pheromones, seasons, day/night, droughts, biomes**: standard in hobby and research sims (The Bibites pheromones [P], JaxLife weather and climate [V]). Not novel.
- **Gompertz-Makeham ageing**: less common than energy-only death, but JaxLife already raises energy costs with age [V]. At most a minor differentiator.

---

## 3. Existing tools to reuse instead of rebuilding

| Need | Reuse | Citation | Recommendation |
|---|---|---|---|
| Describing the model | **ODD protocol** | Grimm et al. 2006, *Ecol. Model.* 198:115–126 [M]; 2010 update, *Ecol. Model.* 221:2760–2768 [M]; **2020 second update, *JASSS* 23(2):7, doi:10.18564/jasss.4259** [V] | **Adopt.** Write the paper's §3 and a supplementary ODD document. Reviewers from ecology and ABM expect it. |
| Validating against patterns | Pattern-oriented modelling | Grimm et al. 2005, *Science* 310:987–991 [M]; DeAngelis & Grimm 2014, "Individual-based models in ecology after four decades," *F1000Prime Rep.* 6:39 [M]; Grimm & Railsback 2005 (Princeton UP) [M] | **Adopt** for the ecological realism claims (trophic levels, Lotka–Volterra cycles). |
| OEE metrics | **MODES** definitions + **Empirical** C++ library (github.com/devosoft/Empirical) | Dolson et al. 2019 [V] | **Reuse the definitions**: change, novelty, ecology and complexity potentials, plus the persistence filter. Re-implement them in Python over exported snapshots, or call Empirical's systematics tools through a dump. Do not invent new metric names. |
| Phylogeny output | ALife data standards (phylogeny CSV) | Lalejini et al. 2019, ALIFE [M] | **Adopt.** Export the lineage tree in the standard format so existing tools (phylogenetic diversity, MRCA depth) work unchanged. |
| Neutral null | Bedau shadow / **Polyworld lockstep** | §2.4 | Already designed. Cite Polyworld lockstep as the closest neural-agent precedent and compare designs (lockstep replays recorded births and deaths; the Terrarium shadow is online). |
| NEAT reference | neat-python [M], SharpNEAT [M], MultiNEAT [M], **TensorNEAT** (Wang, Zhao, Liu, Sun & Cheng, GECCO 2024, arXiv:2404.01817) [V] | Stanley & Miikkulainen 2002 | **Use as a test oracle** to fix audit item G (crossover only mixes weights; no global innovation numbers). Keep the Rust implementation for speed and determinism; Rust NEAT crates are [M] unverified and not recommended without review. |
| GPU scaling (if hard_cap must grow) | FLAME GPU 2 [M: Richmond et al. 2023, *Softw. Pract. Exp.*]; JAX (EcoEvoJax/JaxLife-style) | | **Not now.** PROGRESS.md already decided GPU is premature. |
| General ABM frameworks | NetLogo (Wilensky 1999) [M], Mesa (Kazil et al. 2020; Mesa 3 in JOSS 2025) [M], MASON (Luke et al. 2005, *Simulation*) [M], Repast (North et al. 2013) [M], Agents.jl (Datseris, Vahdati & DuBois, *Simulation*) [M] | | **Do not migrate.** Terrarium's deterministic, allocation-free Rust core is a real asset. One option: a ~200-line NetLogo or Mesa toy of the niche-construction-plus-drought mechanism, as an independent replication for reviewers. |
| Epidemiology ABMs | Covasim (Kerr et al. 2021, *PLOS Comp Biol*) [M], OpenABM-Covid19 (Hinch et al. 2021, *PLOS Comp Biol*) [M], FRED (Grefenstette et al. 2013, *BMC Public Health*) [M] | | **Nothing to reuse scientifically.** Different domain. They are useful only as engineering examples (calibration, regression tests, documentation). |
| Exploration of sim space | ASAL (Kumar et al. 2024, arXiv:2412.17799) [P] | | Optional later work: foundation-model novelty search over Terrarium configs. |
| Baseline results to compare against | Polyworld complexity trends (Yaeger & Sporns 2006; Yaeger et al. 2008, 2010) [P]; Geb activity classes (Channon 2006) [V] | | **Compare directly** (see §4d). |

---

## 4. Verdict

### (a) What is reinventing the wheel (blunt)

1. **The overall platform concept.** "Neural-net creatures with an energy budget, vision-like sensors, predation, signalling and sexual reproduction in a 2D world" is Polyworld (1994), and also Framsticks, Critterding and The Bibites. Terrarium is a modern, deterministic, faster Polyworld minus Hebbian learning. It is not a new kind of platform.
2. **Brain energy cost (structural part).** Polyworld charges per neuron and per synapse. Terrarium's `brain_cost` and `connection_cost` are the same idea.
3. **Biparental cost of sex.** This is the Polyworld scheme.
4. **Neutral shadow / OEE activity statistics.** These are Bedau 1998, Channon 2001–2006 and Polyworld lockstep 2008. Terrarium's current home-grown `activity.rs` metrics were mislabelled (OEE_SHADOW_DESIGN.md §1). Reuse the Bedau, Channon and MODES definitions verbatim.
5. **Niche construction by neural agents.** JaxLife terraforming and Chiba et al. 2020 already have it.
6. **Kin sharing, pheromones, seasons, droughts, biomes, day/night.** Standard ingredients.
7. **NEAT itself**, including a buggy crossover (item G). Check it against a reference implementation rather than re-deriving it.

### (b) What is plausibly novel, stated narrowly

| # | Narrow claim | Closest prior | Confidence |
|---|---|---|---|
| 1 | **Factorial ablation of three inheritance channels (genetic recombination × epigenetic marks × ecological construction) in one evolved-neural-agent ecology, each scored by excess evolutionary activity over a neutral shadow.** | Channon 2006 (shadow, one channel); Polyworld lockstep (null, one channel); Amee (multiple channels, no shadow, likely non-neural); EpiAL; JaxLife | **Medium–high.** No implementation found. Novelty rests on the *experiment*, not the features. |
| 2 | Stress-induced, heritable, per-synapse methylation in an evolved NEAT controller that acts within the lifetime *and* across generations. | EpiAL (non-neural); NEAT disabled flag; Brawer et al. 2017 | **Medium.** Novel as a mechanism, but a reviewer can call it "a noisy heritable weight mask". You need evidence that it behaves epigenetically. |
| 3 | Metabolic cost of *realised synaptic activity* Σ\|w·a\| in an open-ended ecology, as distinct from Polyworld's structural cost. | Polyworld (structural); Jones et al. 2008 (spike cost, task-based) | **Low–medium.** Novel in setting but incremental. Worth a section only if it measurably changes evolved sparsity or topology. |
| 4 | Niche construction as an evolved buffer against stochastic local droughts (construction multiplies drought-suppressed regrowth). | JaxLife; Chiba et al. 2020; Silver & Di Paolo 2006 | **Low–medium.** It is a hypothesis test, not a mechanism claim. |

**Withdraw now:** "No neural-agent ALife platform has [niche construction] — completely novel"; "only EpiAL"; "not yet applied to ALife neural costs"; "Implement features no other ALife platform has". Also withdraw the abstract's "D(t) grows as ~15.5·ln(t) … satisfying Bedau's criterion". Those runs predate the fixes, the metric was mislabelled (not Bedau's D), and no shadow exists yet (PROGRESS.md item H).

### (c) What to reuse

1. **ODD (Grimm et al. 2020)** for the model description, plus pattern-oriented validation.
2. **MODES metric definitions** (Dolson et al. 2019) and **Bedau/Channon activity definitions with a shadow**. Name them exactly as in the literature: D = diversity, Ā_cum, median activity, A_new.
3. **Polyworld lockstep as the design reference** for the shadow, and Polyworld and Geb **results as baselines**.
4. **ALife data-standard phylogeny export**, so existing analysis tools apply.
5. **A reference NEAT implementation** (neat-python or TensorNEAT) as a test oracle for crossover and innovation numbers.

### (d) Three concrete recommendations to sharpen the novelty claim

1. **Retitle the paper and lead with the experiment, not the feature list.** For example: "Which inheritance channels make evolution non-neutral? A shadow-controlled factorial test of genetic, epigenetic and ecological inheritance in a neural-agent ecology." Run the 2×2×2 (sex × epigenetics × niche construction) design at the Phase 9a food-limited setting, report excess median activity over the shadow (Channon class 4b), and test interaction terms. An interaction such as "epigenetics helps only when niche construction is on" is the result that would justify the "combination" claim. Related Work must cite Polyworld (1994, 2008 lockstep), Geb (2006, 2019), JaxLife (2024), Chiba et al. (2020), EpiAL (ICAART 2010) and Amee (2023).
2. **Make the epigenetic channel distinguishable from a genetic one by design and by control.** Add an "epigenetic shadow" control in which offspring inherit *shuffled* or *randomised* marks with the same distribution. Also check that marks are reversible (they decay or reset when stress ends). Then show a transgenerational priming effect after droughts that disappears under shuffling. Without this, reviewers will call methylation "just more genome".
3. **Benchmark against Polyworld directly, and fix the record.** Reproduce Polyworld's driven-versus-passive complexity analysis (Yaeger et al. 2008) on Terrarium brains using the shadow, so readers get an apples-to-apples comparison with the closest prior system. Also correct the citations: Sousa & Costa is ICAART 2010 pp. 90–97, not ALIFE XII pp. 131–138; check the Bedau, Snyder & Packard 1998 pages and the Bedau & Packard 1991 pages; add Yaeger et al. 2008 to OEE_SHADOW_DESIGN.md; and re-check every [M] item above before submission.

---

## Sources consulted (this audit)

- Yaeger 1994 PolyWorld chapter (author PDF): https://shinyverse.org/larryy/Yaeger.ALife3.pdf
- Yaeger, Griffith & Sporns 2008: https://arxiv.org/abs/1112.4906
- Channon 2006 GPEM: https://link.springer.com/doi/10.1007/s10710-006-9009-3 ; Channon 2019: https://eprints.keele.ac.uk/5900/ ; Channon 2003 record: https://eprints.keele.ac.uk/36/
- Bedau, Snyder & Packard 1998: https://people.reed.edu/~mab/papers/alife6.pdf
- Dolson et al. 2019 MODES: https://par.nsf.gov/biblio/10104628 , https://peerj.com/preprints/27249
- JaxLife: https://arxiv.org/html/2409.00853v1 , https://github.com/luchris429/jaxlife
- EcoEvoJax: https://arxiv.org/abs/2302.09334v1
- Nisioti & Moulin-Frier 2023: https://arxiv.org/abs/2305.09369v1
- Chiba, Suzuki & Arita 2020: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7805702/
- Jaafar, Suzuki & Arita 2025: https://mdpi.com/2079-9292/14/2/354
- Taylor 2004: https://tim-taylor.com/paper-details/taylor2004niche.html
- Silver & Di Paolo 2006: https://catalog.comses.net/publications/142136
- EpiAL (ICAART 2010): https://www.scitepress.org/Papers/2010/27325/
- Turner et al. 2013: https://gpbib.cs.ucl.ac.uk/gp-html/turner2013incorporation.html
- Brawer et al. 2017: https://www.frontiersin.org/articles/10.3389/frobt.2017.00001
- Ricalde 2019: https://arxiv.org/abs/1903.03854
- Jones et al. (hydra animat): https://rd.springer.com/chapter/10.1007/978-3-642-02490-0_27
- Amee thesis: https://archiv.ub.uni-heidelberg.de/volltextserver/id/eprint/33914/contents
- Misevic et al. / Avida sex: https://d.lib.msu.edu/etd/34067/OBJ/download ; Lehtonen et al. 2012: https://openresearch-repository.anu.edu.au/bitstream/1885/64549/2/01_Lehtonen_The_many_costs_of_sex_2012.pdf
- Biomaker CA: https://arxiv.org/pdf/2307.09320 ; Flow-Lenia: https://arxiv.org/abs/2212.07906
- The Bibites: https://thebibites.itch.io/the-bibites ; ALIEN: https://alien-project.org , https://github.com/chrxh/alien ; Gene Pool: https://swimbots.com/About.html ; Critterding: https://sourceforge.net/projects/critterding/
- ODD 2020: https://www.jasss.org/23/2/7.html
- TensorNEAT: https://arxiv.org/abs/2404.01817v1
- ABMax predator–prey: https://arxiv.org/pdf/2606.13639 ; ASAL: https://arxiv.org/html/2412.17799v2
