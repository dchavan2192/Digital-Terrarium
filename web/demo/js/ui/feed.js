// "What's happening" feed (§6.4): newest on top, max 50, Day N stamp (tick in
// the tooltip), icon + one sentence. Events with a location fly the camera
// there; events with a species highlight it. The list is aria-live="polite".

import { icon } from './icons.js';
import { dayOf, fmtInt } from '../format.js';
import { slotHex } from '../palette.js';
import { EVENT_RULES } from '../events.js';

function esc(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
}

export function createFeed(list, latestEl, { onLocate, onSpecies, colors }) {
  let dayPeriod = 600;
  const items = [];

  function html(ev) {
    let text = esc(ev.text);
    if (ev.species !== undefined && ev.species !== null) {
      const sw = `<i class="sw" style="--c:${slotHex(colors.slot(ev.species))}"></i>`;
      text = text.replace(`Sp. ${ev.species}`, `Sp. ${ev.species} ${sw}`);
    }
    return text;
  }

  list.addEventListener('click', (e) => {
    const li = e.target.closest('li[data-i]');
    if (!li) return;
    const ev = items.find((x) => String(x._id) === li.dataset.i);
    if (!ev) return;
    if (ev.x !== undefined && onLocate) onLocate(ev.x, ev.y);
    if (ev.species !== undefined && onSpecies) onSpecies(ev.species);
  });

  let uid = 0;
  return {
    setDayPeriod(p) {
      dayPeriod = p;
    },
    add(ev) {
      ev._id = uid++;
      items.unshift(ev);
      if (items.length > EVENT_RULES.maxKept) items.pop();
      const li = document.createElement('li');
      li.dataset.i = String(ev._id);
      li.className = `ev ev-${ev.priority || 'low'}${ev.x !== undefined || ev.species !== undefined ? ' ev-link' : ''}`;
      const day = dayOf(ev.tick, dayPeriod);
      li.innerHTML = `<span class="ev-icon">${icon(ev.icon, 16)}</span>
        <span class="ev-body"><span class="ev-day tnum" title="Tick ${fmtInt(ev.tick)}">Day ${fmtInt(day)}</span> ${html(ev)}</span>`;
      if (ev.x !== undefined || ev.species !== undefined) {
        li.tabIndex = 0;
        li.setAttribute('role', 'button');
        li.addEventListener('keydown', (k) => {
          if (k.key === 'Enter' || k.key === ' ') {
            k.preventDefault();
            li.click();
          }
        });
      }
      list.prepend(li);
      while (list.children.length > EVENT_RULES.maxKept) list.lastElementChild.remove();
      if (latestEl) latestEl.innerHTML = `<span class="ev-day tnum">Day ${fmtInt(day)}</span> · ${html(ev)}`;
    },
    clear() {
      items.length = 0;
      list.replaceChildren();
      if (latestEl) latestEl.textContent = '';
    },
  };
}
