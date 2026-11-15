// Sim-space <-> scene-space mapping and terrain height sampling.
// Spec §3.1: the world is a 10 x 10 scene-unit plane centred at the origin
// (sim x -> scene X, sim y -> scene Z), so 1 sim unit = 0.01 scene units for a
// 1000-unit world. Pure (no three.js), allocation-free hot paths.

export const WORLD_SCENE_SIZE = 10;
export const HEIGHT_EXAGGERATION = 0.35; // §3.2: y = h_norm * 0.35 scene units

export function createWorldMap(cfg, elevation) {
  const W = cfg.width;
  const H = cfg.height;
  const cols = cfg.cols;
  const rows = cfg.rows;
  const sx = WORLD_SCENE_SIZE / W;
  const sz = WORLD_SCENE_SIZE / H;
  const half = WORLD_SCENE_SIZE / 2;
  // Normalise raw elevation (ABI gives min/max) to roughly [-1, 1].
  const lo = cfg.elevationMin;
  const hi = cfg.elevationMax;
  const mid = (lo + hi) / 2;
  const halfRange = Math.max(1e-6, (hi - lo) / 2);
  let exaggeration = HEIGHT_EXAGGERATION;

  // Heights at vertex (c, r) = cell centre (c, r): row-major like the grid.
  const heights = new Float32Array(cols * rows);
  function rebuildHeights() {
    for (let i = 0; i < heights.length; i++) {
      heights[i] = elevation ? ((elevation[i] - mid) / halfRange) * exaggeration : 0;
    }
  }
  rebuildHeights();

  const map = {
    W,
    H,
    cols,
    rows,
    heights,
    /** sim x -> scene X */
    sceneX: (x) => x * sx - half,
    /** sim y -> scene Z */
    sceneZ: (y) => y * sz - half,
    /** scene X -> sim x (wrapped into [0, W)) */
    simX: (X) => ((((X + half) / sx) % W) + W) % W,
    simY: (Z) => ((((Z + half) / sz) % H) + H) % H,
    unitScale: sx,
    /** Scene height at a sim position, bilinear between cell centres, wrap-aware. */
    heightAt(x, y) {
      if (!elevation) return 0;
      const fx = x / (W / cols) - 0.5;
      const fy = y / (H / rows) - 0.5;
      let c0 = Math.floor(fx);
      let r0 = Math.floor(fy);
      const tx = fx - c0;
      const ty = fy - r0;
      c0 = ((c0 % cols) + cols) % cols;
      r0 = ((r0 % rows) + rows) % rows;
      const c1 = (c0 + 1) % cols;
      const r1 = (r0 + 1) % rows;
      const a = heights[r0 * cols + c0];
      const b = heights[r0 * cols + c1];
      const c = heights[r1 * cols + c0];
      const d = heights[r1 * cols + c1];
      return (a + (b - a) * tx) * (1 - ty) + (c + (d - c) * tx) * ty;
    },
    /** Shortest wrapped delta b - a on an axis of length m. */
    wrapDelta(a, b, m) {
      let d = b - a;
      if (d > m * 0.5) d -= m;
      else if (d < -m * 0.5) d += m;
      return d;
    },
    get exaggeration() {
      return exaggeration;
    },
    setExaggeration(k) {
      exaggeration = k;
      rebuildHeights();
    },
  };
  return map;
}
