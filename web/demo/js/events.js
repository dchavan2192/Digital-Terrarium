// "What's happening" event detection + anti-spam rate limiter.
// Pure module (node-testable). Spec: docs/research/DEMO_UX_DESIGN.md §6.4.
//
// Every threshold lives in EVENT_RULES. These are FRONT-END presentation
// constants, not simulation constants, so they do not violate the
// "every number in config" rule for sim-core.

import { compass, fmtInt, fmtPct, speciesName } from './format.js';

export const EVENT_RULES = {
  rateLimitMs: 3000, // 1 event per 3 s real time at 1x and 4x
  rateLimitFastMs: 2000, // 1 per 2 s at 16x
  maxQueueBeforeDropLow: 3,
  staleTicks: { high: 6000, med: 1500, low: 1000 }, // queued events older than this (sim ticks) are dropped
  maxQueue: 6, // beyond this the oldest lowest-priority event is dropped (keeps the feed current)
  maxKept: 50,
  hysteresisTicks: 200,
  cooldownTicks: { crash: 5000, boom: 5000, predation: 5000, cooperation: 2000, newSpecies: 1000, extinction: 1000, season: 1000, record: 2000 },
  crashDropFrac: 0.25,
  crashWindowTicks: 1000,
  boomCapFrac: 0.95,
  boomQuietTicks: 2000,
  boomGrowFrac: 0.3,
  boomWindowTicks: 1000,
  newSpeciesMinMembers: 10,
  extinctPeakMin: 10,
  predatorMult: 2,
  predatorMin: 10,
  predatorWindowTicks: 2000,
  trophicLevels: [1.1, 1.25, 1.4],
  coopLevels: [0.25, 0.5, 0.75],
  coopHysteresis: 0.05,
  genMilestones: [10, 25, 50, 100, 250, 500, 1000],
  recordBrainStep: 0.1,
  traitDiffSize: 0.1, // relative body-size difference worth mentioning
  traitDiffTrophic: 0.05,
  historyTicks: 2500, // how much stats history the detector keeps
};

export const ICONS = {
  opening: 'seed',
  drought: 'drought',
  droughtEnd: 'rain',
  crash: 'crash',
  boom: 'boom',
  newSpecies: 'fork',
  extinction: 'cut',
  predation: 'predation',
  cooperation: 'share',
  generation: 'generation',
  season: 'season',
  record: 'brain',
  creature: 'locate',
};

const PRIORITY_RANK = { high: 0, med: 1, low: 2 };

/**
 * Detector. Call `opening()` once, then `ingest(sample)` whenever a fresh stats
 * sample arrives (the demo samples every ~20 sim ticks). Returns new events.
 *
 * sample = { tick, population, hardCap, species: [{id, parent, population,
 *   meanBodySize, meanTrophic}], predator_count, mean_trophic_level,
 *   cooperation_rate, kin_share_ratio, max_generation, mean_connections,
 *   season, droughts: [{x, y, radius, severity, remaining}] }
 */
export function createEventEngine(world, rules = EVENT_RULES) {
  const W = world.width || 1000;
  const H = world.height || 1000;
  const hist = []; // [{tick, pop, pred, season}]
  const lastFired = {}; // kind -> tick
  const pendingConfirm = {}; // key -> first tick condition held
  const droughtKeys = new Map(); // key -> {x, y, radius}
  const species = new Map(); // id -> {firstSeen, peak, announced, founder, meanBodySize, meanTrophic, parent, dead}
  let first = true;
  let lastAtCapTick = -Infinity;
  let trophicBand = 0; // how many trophic levels crossed upward
  let coopBand = -1; // index of the highest coop level we're above (-1 = none)
  let genIdx = 0;
  let brainRecord = 0;
  let brainAnnounced = 0;
  let mergedNewSpecies = 0;
  let mergeStart = 0;
  let mergedExtinct = 0;
  let extinctMergeStart = 0;
  let lastSeasonPhase = null;

  function cooled(kind, tick) {
    const cd = rules.cooldownTicks[kind] ?? 0;
    const t = lastFired[kind];
    return t === undefined || tick - t >= cd;
  }

  function fire(out, kind, priority, tick, text, extra) {
    lastFired[kind] = tick;
    out.push({ kind, priority, tick, text, icon: ICONS[kind] || 'dot', ...extra });
  }

  /** Condition must hold continuously for hysteresisTicks before it counts. */
  function confirmed(key, cond, tick) {
    if (!cond) {
      delete pendingConfirm[key];
      return false;
    }
    if (pendingConfirm[key] === undefined) pendingConfirm[key] = tick;
    return tick - pendingConfirm[key] >= rules.hysteresisTicks;
  }

  function windowStats(tick, span) {
    let max = -Infinity;
    let min = Infinity;
    let predSum = 0;
    let n = 0;
    for (let i = hist.length - 1; i >= 0; i--) {
      const h = hist[i];
      if (tick - h.tick > span) break;
      if (h.pop > max) max = h.pop;
      if (h.pop < min) min = h.pop;
      predSum += h.pred;
      n++;
    }
    return { max, min, predMean: n ? predSum / n : 0, n };
  }

  function droughtStartedWithin(tick, span) {
    const t = lastFired.drought;
    return t !== undefined && tick - t <= span;
  }

  return {
    rules,

    opening(initialCount) {
      return {
        kind: 'opening',
        priority: 'high',
        tick: 0,
        icon: ICONS.opening,
        text: `${fmtInt(initialCount)} creatures with random brains have just been released. Watch which strategies survive.`,
      };
    },

    ingest(s) {
      const out = [];
      const tick = s.tick;
      hist.push({ tick, pop: s.population, pred: s.predator_count || 0, season: s.season ?? 1 });
      while (hist.length && tick - hist[0].tick > rules.historyTicks) hist.shift();

      // --- Droughts: diff the list (positions are fixed for a drought's life).
      const seen = new Set();
      for (const d of s.droughts || []) {
        const key = `${Math.round(d.x)}:${Math.round(d.y)}`;
        seen.add(key);
        if (!droughtKeys.has(key)) {
          droughtKeys.set(key, d);
          if (!first) {
            const where = compass(d.x, d.y, W, H);
            const place = where === 'centre' ? 'the centre of the world' : `the ${where}`;
            const pct = Math.round((d.severity ?? 0.05) * 100);
            const rem = d.remaining > 0 ? ` for the next ~${fmtInt(d.remaining)} ticks` : '';
            fire(out, 'drought', 'high', tick, `A drought has struck ${place}. Food there will regrow at ${pct}%${rem}.`, { x: d.x, y: d.y });
          }
        }
      }
      for (const [key, d] of droughtKeys) {
        if (!seen.has(key)) {
          droughtKeys.delete(key);
          const where = compass(d.x, d.y, W, H);
          const name = where === 'centre' ? 'central' : where;
          fire(out, 'droughtEnd', 'med', tick, `The ${name} drought has ended. Food is starting to return.`, { x: d.x, y: d.y });
        }
      }

      // --- Population crash / boom.
      if (!first) {
        const win = windowStats(tick, rules.crashWindowTicks);
        if (win.max > 0 && s.population <= win.max * (1 - rules.crashDropFrac) && cooled('crash', tick)) {
          const drop = 1 - s.population / win.max;
          let hint = '';
          if (droughtStartedWithin(tick, rules.crashWindowTicks)) hint = ' Likely the drought.';
          else if ((s.season ?? 1) < 0.8) hint = ' A winter food shortage.';
          fire(out, 'crash', 'high', tick, `Population crash: down ${fmtPct(drop)} to ${fmtInt(s.population)}.${hint}`);
        }
        const cap = s.hardCap || 0;
        const atCap = cap > 0 && s.population >= cap * rules.boomCapFrac;
        if (atCap) {
          if (tick - lastAtCapTick > rules.boomQuietTicks && cooled('boom', tick)) {
            fire(out, 'boom', 'low', tick, `Near the configured cap: ${fmtInt(s.population)} creatures. Births can be blocked at ${fmtInt(cap)}.`);
          }
          lastAtCapTick = tick;
        } else {
          const bw = windowStats(tick, rules.boomWindowTicks);
          if (bw.min > 0 && bw.n > 3 && s.population >= bw.min * (1 + rules.boomGrowFrac) && cooled('boom', tick)) {
            fire(out, 'boom', 'low', tick, `Population boom: up ${fmtPct(s.population / bw.min - 1)} to ${fmtInt(s.population)} in the last ${fmtInt(rules.boomWindowTicks)} ticks.`);
          }
        }
      }

      // --- Species births / extinctions (from the stable species list).
      const aliveIds = new Set();
      for (const sp of s.species || []) {
        if (sp.population <= 0) continue;
        aliveIds.add(sp.id);
        let rec = species.get(sp.id);
        if (!rec) {
          rec = { firstSeen: tick, peak: 0, announced: false, founder: first, parent: sp.parent ?? null };
          species.set(sp.id, rec);
        }
        rec.dead = false;
        rec.peak = Math.max(rec.peak, sp.population);
        rec.meanBodySize = sp.meanBodySize;
        rec.meanTrophic = sp.meanTrophic;
        if (sp.parent !== undefined && sp.parent !== null) rec.parent = sp.parent;
        if (!rec.announced && !rec.founder && sp.population >= rules.newSpeciesMinMembers) {
          rec.announced = true;
          if (cooled('newSpecies', tick)) {
            let text = `A new species has emerged: ${speciesName(sp.id)}`;
            const parent = rec.parent !== null ? species.get(rec.parent) : null;
            if (parent) {
              text += `, branching from ${speciesName(rec.parent)}.`;
              const bits = [];
              if (Number.isFinite(sp.meanBodySize) && Number.isFinite(parent.meanBodySize)) {
                const r = sp.meanBodySize / parent.meanBodySize - 1;
                if (r > rules.traitDiffSize) bits.push('larger');
                else if (r < -rules.traitDiffSize) bits.push('smaller');
              }
              if (Number.isFinite(sp.meanTrophic) && Number.isFinite(parent.meanTrophic)) {
                const d = sp.meanTrophic - parent.meanTrophic;
                if (d > rules.traitDiffTrophic) bits.push('hunts more');
                else if (d < -rules.traitDiffTrophic) bits.push('grazes more');
              }
              if (bits.length) text += ` It’s ${bits.join(' and ')}.`;
            } else {
              text += '.';
            }
            fire(out, 'newSpecies', 'high', tick, text, { species: sp.id });
            mergedNewSpecies = 0;
            mergeStart = tick;
          } else {
            mergedNewSpecies++;
          }
        }
      }
      if (mergedNewSpecies > 0 && cooled('newSpecies', tick)) {
        const n = mergedNewSpecies;
        mergedNewSpecies = 0;
        fire(out, 'newSpecies', 'high', tick,
          n === 1
            ? `Another new species has emerged in the last ${fmtInt(tick - mergeStart)} ticks.`
            : `${n} new species have emerged in the last ${fmtInt(tick - mergeStart)} ticks.`);
        mergeStart = tick;
      }
      for (const [id, rec] of species) {
        if (rec.dead || aliveIds.has(id)) continue;
        rec.dead = true;
        if (rec.peak >= rules.extinctPeakMin) {
          if (cooled('extinction', tick)) {
            fire(out, 'extinction', 'high', tick,
              `${speciesName(id)} has gone extinct after ${fmtInt(tick - rec.firstSeen)} ticks. At its peak it numbered ${fmtInt(rec.peak)}.`,
              { species: id });
            mergedExtinct = 0;
            extinctMergeStart = tick;
          } else {
            mergedExtinct++;
          }
        }
      }
      if (mergedExtinct > 0 && cooled('extinction', tick)) {
        const n = mergedExtinct;
        mergedExtinct = 0;
        fire(out, 'extinction', 'med', tick,
          n === 1
            ? `Another species has gone extinct in the last ${fmtInt(tick - extinctMergeStart)} ticks.`
            : `${n} more species have gone extinct in the last ${fmtInt(tick - extinctMergeStart)} ticks. Competition is fierce.`);
        extinctMergeStart = tick;
      }
      // Forget long-dead species so the map stays small.
      if (species.size > 400) for (const [id, rec] of species) if (rec.dead) species.delete(id);

      // --- Predation surge.
      if (!first) {
        const pw = windowStats(tick, rules.predatorWindowTicks);
        const pred = s.predator_count || 0;
        const surge = pred >= rules.predatorMin && pred >= pw.predMean * rules.predatorMult && pw.n > 5;
        const lvl = rules.trophicLevels[trophicBand];
        const crossed = lvl !== undefined && confirmed('trophic', s.mean_trophic_level >= lvl, tick);
        if (crossed) trophicBand++;
        if ((crossed || confirmed('predSurge', surge, tick)) && cooled('predation', tick)) {
          const frac = Math.max(0, (s.mean_trophic_level || 1) - 1);
          const text = frac >= 0.08
            ? `Hunting accounts for ${fmtPct(frac)} of intake on average, over creatures’ lifetimes.`
            : `Predation is rising: ${fmtInt(pred)} creatures now live mostly by hunting.`;
          fire(out, 'predation', 'med', tick, text);
        }
      }

      // --- Cooperation thresholds (either way, 5-pt hysteresis, confirmed).
      if (Number.isFinite(s.cooperation_rate)) {
        const r = s.cooperation_rate;
        const lv = rules.coopLevels;
        const up = coopBand + 1 < lv.length && r > lv[coopBand + 1] + rules.coopHysteresis;
        const down = coopBand >= 0 && r < lv[coopBand] - rules.coopHysteresis;
        if (confirmed('coopUp', up, tick)) {
          while (coopBand + 1 < lv.length && r > lv[coopBand + 1] + rules.coopHysteresis) coopBand++;
          delete pendingConfirm.coopUp;
          if (!first && cooled('cooperation', tick)) {
            const kin = s.kin_share_ratio >= 0.8 ? ', mostly between similar-hue creatures' : '';
            fire(out, 'cooperation', 'med', tick, `Energy sharing is more frequent${kin}.`);
          }
        } else if (confirmed('coopDown', down, tick)) {
          while (coopBand >= 0 && r < lv[coopBand] - rules.coopHysteresis) coopBand--;
          delete pendingConfirm.coopDown;
          if (cooled('cooperation', tick)) {
            fire(out, 'cooperation', 'med', tick, `Energy sharing is less frequent.`);
          }
        }
      }

      // --- Generation milestones.
      const g = s.max_generation || 0;
      while (genIdx < rules.genMilestones.length && g >= rules.genMilestones[genIdx]) {
        const m = rules.genMilestones[genIdx++];
        if (g < (rules.genMilestones[genIdx] ?? Infinity)) {
          fire(out, 'generation', 'low', tick, `Generation ${fmtInt(m)}: the deepest lineage is ${fmtInt(m)} generations removed from its random ancestor.`);
        }
      }

      // --- Record brain (low priority).
      const mc = s.mean_connections;
      if (Number.isFinite(mc)) {
        if (first) {
          brainRecord = mc;
          brainAnnounced = mc;
        } else if (mc > brainRecord) {
          brainRecord = mc;
          if (mc >= brainAnnounced * (1 + rules.recordBrainStep) && cooled('record', tick)) {
            brainAnnounced = mc;
            fire(out, 'record', 'low', tick, `Brain links reached a new record: ${fmtInt(mc)} enabled connections per creature.`);
          }
        }
      }

      // --- Season peaks (low priority). Phase from the multiplier history.
      if (Number.isFinite(s.seasonPhase)) {
        const p = s.seasonPhase;
        if (lastSeasonPhase !== null && !first) {
          const amp = s.seasonAmplitude || 0;
          if (lastSeasonPhase < 0.75 && p >= 0.75 && cooled('season', tick)) {
            fire(out, 'season', 'low', tick, `Winter is setting in: food is regrowing ${fmtPct(amp)} slower.`);
          } else if (lastSeasonPhase < 0.25 && p >= 0.25 && cooled('season', tick)) {
            fire(out, 'season', 'low', tick, `High summer: food is regrowing ${fmtPct(amp)} faster than average.`);
          }
        }
        lastSeasonPhase = p;
      }

      first = false;
      return out;
    },
  };
}

/**
 * Anti-spam: at most one event per interval (real time), priority queue, and
 * low-priority events are dropped while the queue is longer than 3.
 */
export function createRateLimiter(rules = EVENT_RULES) {
  let queue = [];
  let lastEmit = -Infinity;
  let seq = 0;
  return {
    push(ev) {
      ev._seq = seq++;
      queue.push(ev);
      queue.sort((a, b) => PRIORITY_RANK[a.priority] - PRIORITY_RANK[b.priority] || a._seq - b._seq);
      while (queue.length > rules.maxQueueBeforeDropLow) {
        const idx = queue.findLastIndex((e) => e.priority === 'low');
        if (idx < 0) break;
        queue.splice(idx, 1);
      }
      while (queue.length > rules.maxQueue) {
        // drop the oldest event of the lowest priority present
        const worst = Math.max(...queue.map((e) => PRIORITY_RANK[e.priority]));
        let idx = -1;
        for (let i = 0; i < queue.length; i++) if (PRIORITY_RANK[queue[i].priority] === worst && (idx < 0 || queue[i]._seq < queue[idx]._seq)) idx = i;
        queue.splice(idx, 1);
      }
    },
    /** Emit immediately (bypasses the limiter, e.g. the opening line). */
    force(nowMs) {
      lastEmit = nowMs;
    },
    /** @param tick current sim tick (optional): stale queued events are dropped. */
    poll(nowMs, fast = false, tick = undefined) {
      if (tick !== undefined) {
        for (let i = queue.length - 1; i >= 0; i--) {
          if (tick - queue[i].tick > (rules.staleTicks[queue[i].priority] ?? Infinity)) queue.splice(i, 1);
        }
      }
      if (!queue.length) return null;
      const gap = fast ? rules.rateLimitFastMs : rules.rateLimitMs;
      if (nowMs - lastEmit < gap) return null;
      lastEmit = nowMs;
      return queue.shift();
    },
    get length() {
      return queue.length;
    },
    clear() {
      queue = [];
      lastEmit = -Infinity;
    },
  };
}
