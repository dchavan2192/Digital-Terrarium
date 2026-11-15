// Species tab (§6.3): up to 12 rows by population. Swatch, name + generated
// epithet, population with trend arrow, mean body size, mean links, trophic
// level, age of the species. Clicking a row highlights that species in the scene.

import { fmtInt, fmtFixed, speciesEpithet, speciesName } from '../format.js';
import { slotHex } from '../palette.js';

export function createSpeciesTab(tbody, { colors, onToggle }) {
  let highlighted = -2;
  tbody.addEventListener('click', (e) => {
    const tr = e.target.closest('tr[data-id]');
    if (!tr) return;
    const id = Number(tr.dataset.id);
    highlighted = highlighted === id ? -2 : id;
    onToggle(highlighted);
    mark();
  });
  tbody.addEventListener('keydown', (e) => {
    if ((e.key === 'Enter' || e.key === ' ') && e.target.closest('tr[data-id]')) {
      e.preventDefault();
      e.target.closest('tr[data-id]').click();
    }
  });

  function mark() {
    for (const tr of tbody.children) tr.classList.toggle('is-on', Number(tr.dataset.id) === highlighted);
  }

  return {
    update(species, tick) {
      const rows = species
        .filter((s) => s.population > 0 && s.established)
        .sort((a, b) => b.population - a.population)
        .slice(0, 12);
      const html = rows
        .map((s) => {
          const arrow = s.trend > 0.05 ? '↑' : s.trend < -0.05 ? '↓' : '→';
          const age = Number.isFinite(s.firstSeen) ? fmtInt(tick - s.firstSeen) : '–';
          return `<tr data-id="${s.id}" tabindex="0">
            <td><i class="sw" style="--c:${slotHex(colors.slot(s.id))}"></i></td>
            <td><span class="sp-name">${speciesName(s.id)}</span><span class="sp-epithet">${speciesEpithet(s)}</span></td>
            <td class="tnum">${fmtInt(s.population)} <span class="trend" data-dir="${arrow}">${arrow}</span></td>
            <td class="tnum">${fmtFixed(s.meanBodySize, 1)}</td>
            <td class="tnum">${fmtInt(s.meanConnections)}</td>
            <td class="tnum">${fmtFixed(s.meanTrophic, 2)}</td>
            <td class="tnum">${age}</td>
          </tr>`;
        })
        .join('');
      tbody.innerHTML = html || '<tr><td colspan="7" class="empty">No established species yet.</td></tr>';
      mark();
    },
    clearHighlight() {
      highlighted = -2;
      mark();
    },
    set(id) {
      highlighted = id;
      mark();
    },
  };
}
