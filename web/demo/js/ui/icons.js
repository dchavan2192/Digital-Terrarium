// Inline SVG icons. UI controls use Lucide path data (ISC licence, lucide.dev),
// inlined so the page needs no icon CDN. Sim-concept glyphs (§8.3) are custom,
// drawn on the same 24-unit grid with the same 1.5 px stroke.

const LUCIDE = {
  play: '<polygon points="6 3 20 12 6 21 6 3"/>',
  pause: '<rect x="14" y="4" width="4" height="16" rx="1"/><rect x="6" y="4" width="4" height="16" rx="1"/>',
  reset: '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>',
  layers: '<path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z"/><path d="m22 17.65-9.17 4.16a2 2 0 0 1-1.66 0L2 17.65"/><path d="m22 12.65-9.17 4.16a2 2 0 0 1-1.66 0L2 12.65"/>',
  map: '<polygon points="3 6 9 3 15 6 21 3 21 18 15 21 9 18 3 21"/><line x1="9" x2="9" y1="3" y2="18"/><line x1="15" x2="15" y1="6" y2="21"/>',
  orbit: '<circle cx="12" cy="12" r="3"/><circle cx="19" cy="5" r="2"/><circle cx="5" cy="19" r="2"/><path d="M10.4 21.9a10 10 0 0 0 9.941-15.416"/><path d="M13.5 2.1a10 10 0 0 0-9.841 15.416"/>',
  locate: '<line x1="2" x2="5" y1="12" y2="12"/><line x1="19" x2="22" y1="12" y2="12"/><line x1="12" x2="12" y1="2" y2="5"/><line x1="12" x2="12" y1="19" y2="22"/><circle cx="12" cy="12" r="7"/>',
  help: '<circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><path d="M12 17h.01"/>',
  copy: '<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>',
  moon: '<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>',
  x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  menu: '<line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/>',
  github: '<path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/>',
  chevronUp: '<path d="m18 15-6-6-6 6"/>',
  arrowUp: '<path d="m5 12 7-7 7 7"/><path d="M12 19V5"/>',
};

const GLYPHS = {
  seed: '<ellipse cx="12" cy="13" rx="6.5" ry="8.5"/><circle cx="9.6" cy="8.2" r="1.1" fill="currentColor"/><circle cx="14.4" cy="8.2" r="1.1" fill="currentColor"/>',
  drought: '<circle cx="12" cy="12" r="9"/><path d="M12.5 3 10.5 8l3 3-2 4 1.5 6"/>',
  rain: '<path d="M7 15a4 4 0 0 1 .5-8 5 5 0 0 1 9.5 1.5A3.5 3.5 0 0 1 17 15"/><path d="m9 18-1 2"/><path d="m13 18-1 2"/><path d="m17 18-1 2"/>',
  fork: '<path d="M12 21v-9"/><path d="M12 12 6.5 5"/><path d="m12 12 5.5-7"/><circle cx="6" cy="4.2" r="1.5"/><circle cx="18" cy="4.2" r="1.5"/>',
  cut: '<path d="M12 21v-9"/><path d="M12 12 6.5 5"/><path d="m12 12 2.6-3.3"/><path d="m14 4.5 4.5 4.5"/><circle cx="6" cy="4.2" r="1.5"/>',
  predation: '<path d="m4 7 5 5-5 5"/><path d="m20 7-5 5 5 5"/>',
  share: '<circle cx="5" cy="16" r="2.2"/><circle cx="19" cy="16" r="2.2"/><path d="M6 12.5a7.5 7.5 0 0 1 12 0"/>',
  brain: '<circle cx="6" cy="7" r="2"/><circle cx="18" cy="7" r="2"/><circle cx="12" cy="18" r="2"/><path d="M8 7h8"/><path d="m7 9 4 7"/><path d="m17 9-4 7"/>',
  generation: '<path d="M4 18a8 8 0 0 1 16 0"/><path d="M7.5 18a4.5 4.5 0 0 1 9 0"/><path d="M11 18a1 1 0 0 1 2 0"/>',
  season: '<circle cx="12" cy="12" r="9"/><path d="M12 3a9 9 0 0 1 0 18z" fill="currentColor"/>',
  crash: '<path d="m22 17-8.5-8.5-5 5L2 7"/><path d="M16 17h6v-6"/>',
  boom: '<path d="m22 7-8.5 8.5-5-5L2 17"/><path d="M16 7h6v6"/>',
  dot: '<circle cx="12" cy="12" r="3" fill="currentColor"/>',
};

export function icon(name, size = 18, cls = '') {
  const body = LUCIDE[name] || GLYPHS[name] || GLYPHS.dot;
  return `<svg class="ic ${cls}" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">${body}</svg>`;
}

/** Creature silhouettes for the "Who eats whom" legend (§4.3). tier 0..2 */
export function podSilhouette(tier, size = 30) {
  const fin = tier >= 1 ? '<path d="M9 9.5 13 4.5 15 9.5" fill="currentColor" stroke="none"/>' : '';
  const jaws = tier >= 2 ? '<path d="M21.5 10.5 25 9.5M21.5 13.5 25 14.5" />' : '';
  return `<svg class="pod" width="${size}" height="${Math.round(size * 0.66)}" viewBox="0 0 27 18" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true">${fin}<ellipse cx="12.5" cy="12" rx="10" ry="5.2" fill="currentColor" fill-opacity="0.22"/>${jaws}<circle cx="19" cy="10" r="1.3" fill="currentColor" stroke="none"/></svg>`;
}
