// Live brain graph on a 2D canvas (§7.2). Three columns: grouped inputs,
// hidden nodes at x = NEAT depth, outputs. Edge width ∝ |effective weight|,
// negative weights dashed, methylated (silenced) weights faded. Node fill uses
// the diverging activation map (#4C8FD6 −1 → #2A2C28 0 → #E69F00 +1).
//
// ABI v1 has no live activations (SIM_WASM_ABI.md §10), so output nodes show
// sim-core's decayed output histogram mean ("recent average output", τ ≈ 100
// ticks) and inputs/hidden stay neutral. If a future ABI adds activations,
// pass them via set(..., {activations}) and every node lights up.

const GROUPS = [
  { name: 'Vision', match: (l) => l.startsWith('ray') },
  { name: 'Self', match: (l) => ['energy', 'heading', 'body_size'].includes(l) },
  { name: 'Social', match: (l) => ['heard_signal', 'kin_similarity'].includes(l) },
  { name: 'World', match: (l) => ['temperature', 'pheromone', 'light', 'niche_level', 'food_here'].includes(l) },
];
const SHORT = {
  energy: 'energy', heading: 'heading', body_size: 'size', heard_signal: 'signal', kin_similarity: 'kin',
  temperature: 'temp', pheromone: 'scent', light: 'light', niche_level: 'built', food_here: 'food here', construct: 'build', bias: 'bias',
};

function shortLabel(l) {
  const m = /^ray(\d+)_(dist|type)$/.exec(l);
  if (m) return `ray${Number(m[1]) + 1} ${m[2]}`;
  return SHORT[l] || l;
}

function actColor(a) {
  // diverging: blue (-1) -> graphite (0) -> amber (+1), sRGB space is fine for canvas
  const t = Math.max(-1, Math.min(1, a || 0));
  const lerp = (x, y, k) => Math.round(x + (y - x) * k);
  if (t < 0) return `rgb(${lerp(42, 76, -t)},${lerp(44, 143, -t)},${lerp(40, 214, -t)})`;
  return `rgb(${lerp(42, 230, t)},${lerp(44, 159, t)},${lerp(40, 0, t)})`;
}

export function createBrainGraph(canvas, { compact = false } = {}) {
  const ctx = canvas.getContext('2d');
  let brain = null;
  let acts = null; // Map nodeId -> activation
  let pos = new Map(); // nodeId -> {x, y, r, label, kind}
  let hover = -1;
  let strongestOnly = compact;
  let cssW = 0;
  let cssH = 0;

  function layout() {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    cssW = canvas.clientWidth || 340;
    cssH = canvas.clientHeight || 300;
    canvas.width = Math.round(cssW * dpr);
    canvas.height = Math.round(cssH * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    pos = new Map();
    if (!brain) return;
    const padT = 18;
    const padB = 8;
    const inX = 62;
    const outX = cssW - 46;
    const inputs = brain.nodes.filter((n) => n.kind === 'in' || n.kind === 'bias');
    const outputs = brain.nodes.filter((n) => n.kind === 'out');
    const hidden = brain.nodes.filter((n) => n.kind === 'hidden');
    // inputs: grouped with gaps
    const ordered = [];
    const groupStarts = [];
    const bias = inputs.find((n) => n.kind === 'bias');
    for (const g of GROUPS) {
      const members = inputs.filter((n) => n.kind === 'in' && g.match(n.label));
      if (!members.length) continue;
      groupStarts.push({ name: g.name, at: ordered.length });
      ordered.push(...members, null);
    }
    const rest = inputs.filter((n) => n.kind === 'in' && !GROUPS.some((g) => g.match(n.label)));
    if (rest.length) ordered.push(...rest, null);
    if (bias) ordered.push(bias);
    const slots = ordered.length;
    const step = (cssH - padT - padB) / Math.max(1, slots - 1);
    ordered.forEach((n, i) => {
      if (n) pos.set(n.id, { x: inX, y: padT + i * step, r: 4.5, label: shortLabel(n.label), kind: n.kind });
    });
    pos.groups = groupStarts.map((g) => ({ name: g.name, y: padT + g.at * step - 9 }));
    const oStep = (cssH - padT - padB) / Math.max(1, outputs.length - 1);
    outputs.forEach((n, i) => pos.set(n.id, { x: outX, y: padT + 10 + i * (oStep * 0.9), r: 6, label: shortLabel(n.label), kind: 'out' }));
    // hidden: x by depth, spread vertically within a depth layer
    const byLayer = new Map();
    for (const h of hidden) {
      const k = h.layer ?? Math.round(h.depth * 10);
      if (!byLayer.has(k)) byLayer.set(k, []);
      byLayer.get(k).push(h);
    }
    for (const nodes of byLayer.values()) {
      nodes.sort((a, b) => a.id - b.id);
      const hs = (cssH - padT - padB - 20) / Math.max(1, nodes.length);
      nodes.forEach((h, i) => {
        const d = Math.min(0.85, Math.max(0.15, h.depth));
        pos.set(h.id, { x: inX + (outX - inX) * d, y: padT + 10 + hs * (i + 0.5), r: 5, label: '', kind: 'hidden' });
      });
    }
  }

  function edgesToDraw() {
    const all = brain.edges.filter((e) => e.enabled !== false && pos.has(e.from) && pos.has(e.to));
    if (!strongestOnly) return all;
    return all.slice().sort((a, b) => Math.abs(b.effective_weight ?? b.weight) - Math.abs(a.effective_weight ?? a.weight)).slice(0, 30);
  }

  function draw() {
    ctx.clearRect(0, 0, cssW, cssH);
    if (!brain) return;
    const edges = edgesToDraw();
    const base = edges.length > 60 ? 0.25 : 0.55;
    for (const e of edges) {
      const a = pos.get(e.from);
      const b = pos.get(e.to);
      const w = e.effective_weight ?? e.weight;
      const meth = e.methylation || 0;
      let alpha = base * (1 - meth);
      if (hover >= 0) alpha = e.from === hover || e.to === hover ? 0.95 * (1 - meth * 0.7) : 0.05;
      ctx.lineWidth = 0.5 + 2.5 * Math.min(1, Math.abs(w));
      ctx.strokeStyle = w >= 0 ? `rgba(184,179,166,${alpha})` : `rgba(143,163,184,${alpha})`;
      ctx.setLineDash(w >= 0 ? [] : [3, 3]);
      const mx = (a.x + b.x) / 2;
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.bezierCurveTo(mx, a.y, mx, b.y, b.x, b.y);
      ctx.stroke();
    }
    ctx.setLineDash([]);
    ctx.font = '10px Inter, system-ui, sans-serif';
    ctx.textBaseline = 'middle';
    for (const [id, p] of pos) {
      const act = acts ? acts.get(id) : undefined;
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
      ctx.fillStyle = act === undefined ? '#2A2C28' : actColor(act);
      ctx.fill();
      ctx.lineWidth = id === hover ? 2 : 1;
      ctx.strokeStyle = id === hover ? '#C9A65B' : '#858174';
      ctx.stroke();
      if (p.label) {
        ctx.fillStyle = id === hover ? '#ECE8DD' : '#B8B3A6';
        ctx.textAlign = p.kind === 'out' ? 'left' : 'right';
        ctx.fillText(p.label, p.kind === 'out' ? p.x + 10 : p.x - 8, p.y);
      }
    }
    ctx.fillStyle = '#858174';
    ctx.font = '600 9px Inter, system-ui, sans-serif';
    ctx.textAlign = 'left';
    for (const g of pos.groups || []) ctx.fillText(g.name.toUpperCase(), 4, Math.max(6, g.y));
    ctx.textAlign = 'center';
    ctx.fillText('INPUTS', 40, 6);
    ctx.fillText('HIDDEN', cssW / 2, 6);
    ctx.fillText('OUTPUTS', cssW - 26, 6);
  }

  function nodeAt(x, y) {
    let best = -1;
    let bd = 100;
    for (const [id, p] of pos) {
      const d = (p.x - x) ** 2 + (p.y - y) ** 2;
      if (d < bd) {
        bd = d;
        best = id;
      }
    }
    return best;
  }

  canvas.addEventListener('mousemove', (e) => {
    const r = canvas.getBoundingClientRect();
    const h = nodeAt(e.clientX - r.left, e.clientY - r.top);
    if (h !== hover) {
      hover = h;
      const p = pos.get(h);
      canvas.title = p ? `${p.kind === 'hidden' ? 'hidden neuron' : p.label}${acts && acts.has(h) ? ` · ${acts.get(h).toFixed(2)}` : ''}` : '';
      draw();
    }
  });
  canvas.addEventListener('mouseleave', () => {
    hover = -1;
    draw();
  });

  let lastKey = '';
  return {
    /**
     * @param b normalised inspector brain {nodes, edges, outputHistograms}
     * @param a Map(nodeId -> activation) or null
     * @param owner creature id (layout is rebuilt when it or the topology changes)
     */
    set(b, a, owner) {
      const key = b ? `${owner}:${b.nodes.length}:${b.edges.length}` : '';
      brain = b;
      acts = a;
      if (key !== lastKey || canvas.clientWidth !== cssW || canvas.clientHeight !== cssH) {
        lastKey = key;
        layout();
      }
      draw();
    },
    setStrongestOnly(v) {
      strongestOnly = v;
      draw();
    },
    get strongestOnly() {
      return strongestOnly;
    },
    clear() {
      brain = null;
      lastKey = '';
      ctx.clearRect(0, 0, canvas.width, canvas.height);
    },
  };
}
