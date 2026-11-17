// Inspector drawer (§7): header, "what is it doing" sentence, brain graph,
// energy, traits, lineage, actions. Refreshed at 4 Hz by main.js.

import { createBrainGraph } from './brain-graph.js';
import { whatIsItDoing, sizeWord, tempWord, dietWords, entropyWord, agingWord, histogramMean } from '../describe.js';
import { fmtInt, fmtFixed, speciesName, TIER_NAMES, trophicTier } from '../format.js';
import { slotHex } from '../palette.js';
import { DEATH_CAUSES } from '../sim.js';
import { recurrentEdgeCount, connectionBreakdown } from '../memory.js';

export function createInspector(root, { compact, onClose, onFollow, onJump, onBack }) {
  const $ = (s) => root.querySelector(s);
  const graph = createBrainGraph($('#brainCanvas'), { compact });
  const strongest = $('#brainStrongest');
  strongest.checked = compact;
  strongest.addEventListener('change', () => graph.setStrongestOnly(strongest.checked));
  $('#inspClose').addEventListener('click', () => onClose());
  $('#inspFollow').addEventListener('click', () => onFollow());
  $('#inspBack').addEventListener('click', () => onBack());
  const jump = $('#inspJump');
  jump.addEventListener('click', () => onJump(Number(jump.dataset.id)));

  let openId = -1;
  let lastSentenceAt = 0;

  const api = {
    get id() {
      return openId;
    },
    get open() {
      return !root.hidden;
    },
    show(id) {
      openId = id;
      root.hidden = false;
      root.classList.remove('is-dead');
      $('#inspTitle').textContent = `Creature #${fmtInt(id)}`;
      $('#inspDoing').textContent = '…';
      graph.clear();
      lastSentenceAt = 0;
    },
    hide() {
      openId = -1;
      root.hidden = true;
      graph.clear();
    },
    /** Mark as dead but keep the last data visible. */
    died(cause, age) {
      root.classList.add('is-dead');
      $('#inspDoing').textContent = cause >= 0
        ? `This creature ${DEATH_CAUSES[cause] ?? 'died'}${Number.isFinite(age) ? ` at age ${fmtInt(age)}` : ''}.`
        : 'This creature is no longer alive.';
      $('#inspFollow').disabled = true;
    },
    /**
     * @param d normalised inspect() object
     * @param ctx { cfg, colors, now, isNight, reproducedRecently, onFruit, targetRelSize,
     *              targetSameSpecies, parentAlive, liveOutputs }
     */
    update(d, ctx) {
      if (!d || d.id !== openId) return;
      $('#inspFollow').disabled = false;
      const sp = d.species;
      const sw = `<i class="sw" style="--c:${slotHex(sp >= 0 ? ctx.colors.slot(sp) : -1)}"></i>`;
      $('#inspMeta').innerHTML = `${sp >= 0 ? `${speciesName(sp)} ${sw}` : 'Unclassified'} · Gen ${fmtInt(d.generation)} · Age <span class="tnum">${fmtInt(d.age)}</span> / ~${fmtInt(d.maxAge)}`;
      // Sentence at 2 Hz.
      if (ctx.now - lastSentenceAt > 480) {
        lastSentenceAt = ctx.now;
        const F = ctx.F;
        const energyFrac = d.energy.current / d.energy.maximum;
        const out = d.lastOutputs;
        const s = whatIsItDoing({
          attacked: !!(d.flags & F.ATTACKED),
          shared: !!(d.flags & F.SHARED),
          reproducedRecently: ctx.reproducedRecently,
          ateNow: !!(d.flags & F.ATE),
          onFruit: ctx.onFruit,
          constructing: out ? out[6] : undefined,
          speedNorm: Number.isFinite(d.speedNorm) ? d.speedNorm : 0,
          attackOut: out ? out[3] : undefined,
          energyFrac,
          isNight: ctx.isNight,
          targetRelSize: ctx.targetRelSize,
          targetSameSpecies: ctx.targetSameSpecies,
        });
        $('#inspDoing').textContent = `“${s}”`;
      }
      // Brain.
      const b = d.brain;
      let acts = null;
      if (d.activations) {
        acts = new Map();
        b.nodes.forEach((n, i) => acts.set(n.id, d.activations[i]));
      } else if (b.outputHistograms) {
        acts = new Map();
        const outs = b.nodes.filter((n) => n.kind === 'out');
        outs.forEach((n, i) => {
          const h = b.outputHistograms[i];
          if (h) acts.set(n.id, histogramMean(h.bins));
        });
      }
      graph.set(b, acts, d.id);
      const cost = d.energy.costBrain;
      const memoryLinks = recurrentEdgeCount(b);
      const links = connectionBreakdown(b.enabledConnections, memoryLinks);
      $('#brainCaption').textContent = `${fmtInt(b.enabledConnections)} enabled links${links ? ` = ${fmtInt(links.feedForward)} feed-forward + ${fmtInt(links.recurrent)} recurrent` : ''} · ${fmtInt(b.hidden)} hidden neurons${Number.isFinite(cost) ? ` · estimated baseline ${fmtFixed(cost, 2)} energy/tick` : ''}`;
      $('#brainNote').textContent = d.activations ? 'Node colour = live activation.' : 'Output colour = its recent average (last ~100 ticks). Live per-neuron activity needs a future sim-core hook.';
      if (memoryLinks) $('#brainNote').textContent += ' Dotted gold links read the previous tick; their presence does not show useful recall.';
      // Energy.
      const e = d.energy;
      const frac = Math.max(0, Math.min(1, e.current / e.maximum));
      $('#energyMeter').style.setProperty('--v', frac.toFixed(3));
      $('#energyText').textContent = `Energy ${fmtInt(e.current)} / ${fmtInt(e.maximum)}`;
      const intake = e.fromFood + e.fromAttacks;
      const pf = intake > 0 ? e.fromAttacks / intake : 0;
      $('#intakeFood').style.setProperty('--v', (1 - pf).toFixed(3));
      $('#intakeHunt').style.setProperty('--v', pf.toFixed(3));
      $('#intakeText').textContent = intake > 0 ? `Lifetime intake: ${fmtInt(e.fromFood)} from plants · ${fmtInt(e.fromAttacks)} from hunting` : 'Has not eaten yet.';
      const costs = [
        ['Body upkeep', e.costMetabolism],
        ['Brain', e.costBrain],
        ['Movement', e.costMovement],
        ['Wrong climate', e.costTemperature],
      ].filter((c) => Number.isFinite(c[1]));
      const maxC = Math.max(0.01, ...costs.map((c) => c[1]));
      $('#costList').innerHTML = costs
        .map((c) => `<li><span>${c[0]}</span><i class="bar" style="--v:${(c[1] / maxC).toFixed(3)}"></i><span class="tnum">−${fmtFixed(c[1], 2)}</span></li>`)
        .join('');
      // Traits.
      const t = d.traits;
      const tier = trophicTier(e.predationFraction);
      const rows = [
        ['Body size', Number.isFinite(t.bodySize) ? `${fmtFixed(t.bodySize, 1)} (${sizeWord(t.bodySize)})` : '–'],
        ['Diet', `${dietWords(e.predationFraction)} · ${TIER_NAMES[tier]}`],
        ['Prefers', Number.isFinite(t.preferredTemp) ? tempWord(t.preferredTemp) : '–'],
        ['Ages', agingWord(t.agingRate, ctx.cfg.defaultAgingRate)],
        ['Mutation rate', Number.isFinite(t.mutationRate) ? fmtFixed(t.mutationRate, 2) : '–'],
        ['Brain entropy', entropyWord(b.entropy)],
      ];
      if (ctx.cfg.fruitEnabled && Number.isFinite(t.diet)) rows.splice(2, 0, ['Food type', t.diet > 0.5 ? 'fruit specialist' : t.diet < 0.35 ? 'algae specialist' : 'generalist']);
      $('#traits').innerHTML = rows.map(([k, v]) => `<dt>${k}</dt><dd>${v}</dd>`).join('');
      // Lineage.
      const crumbs = [];
      if (d.rootId !== null && d.rootId !== d.id && d.rootId !== d.parentId) crumbs.push(`#${fmtInt(d.rootId)} <span class="muted">(founder)</span>`, '…');
      if (d.parentId !== null) crumbs.push(`#${fmtInt(d.parentId)} <span class="muted">(parent)</span>`);
      crumbs.push(`<strong>#${fmtInt(d.id)}</strong>`);
      if (d.generation === 0) crumbs[crumbs.length - 1] += ' <span class="muted">(founder)</span>';
      $('#lineage').innerHTML = crumbs.join(' → ');
      $('#children').textContent = Number.isFinite(d.childrenAlive) ? `Children alive: ${fmtInt(d.childrenAlive)}` : '';
      jump.hidden = !(d.parentId !== null && ctx.parentAlive);
      jump.dataset.id = String(d.parentId ?? -1);
    },
    setCompact(v) {
      strongest.checked = v;
      graph.setStrongestOnly(v);
    },
  };
  return api;
}
