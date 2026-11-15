// Terrain heightfield, data-layer colouring, biome contours, fruit dots, drought rings.
// Spec §3.2 (terrain), §3.3 (droughts), §5 (layers).
// Draw calls: terrain 1, biome contours 1, drought rings 1, fruit points 1.

import * as THREE from 'three';
import { SCENE_HEX, hexToLinear } from '../palette.js';
import { WORLD_SCENE_SIZE } from './world.js';

const RING_SEGMENTS = 72;
const MAX_RINGS = 8;
const RING_GROW_MS = 600;
const RING_FADE_MS = 1000;
const LOW_TINT = 0.28; // how much the lowest ground is pulled toward the cool lowland tone
const VIGNETTE = 0.04; // §3.1: 4% edge band fades into the page

export function createTerrain(scene, map, cfg) {
  const cols = map.cols;
  const rows = map.rows;
  const n = cols * rows;
  // The plane spans cell CENTRES so vertex (c, r) == cell (c, r) exactly.
  const spanX = (WORLD_SCENE_SIZE * (cols - 1)) / cols;
  const spanZ = (WORLD_SCENE_SIZE * (rows - 1)) / rows;
  const geo = new THREE.PlaneGeometry(spanX, spanZ, cols - 1, rows - 1);
  geo.rotateX(-Math.PI / 2); // PlaneGeometry row iy -> +Z, so vertex index = r * cols + c
  const pos = geo.attributes.position;
  for (let i = 0; i < n; i++) pos.setY(i, map.heights[i]);
  pos.needsUpdate = true;
  geo.computeVertexNormals();
  const colors = new Float32Array(n * 3);
  const colorAttr = new THREE.BufferAttribute(colors, 3);
  colorAttr.setUsage(THREE.DynamicDrawUsage);
  geo.setAttribute('color', colorAttr);

  const mat = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.receiveShadow = true;
  mesh.name = 'terrain';
  scene.add(mesh);

  // Precomputed per-vertex constants.
  const lowMix = new Float32Array(n);
  const edgeMix = new Float32Array(n);
  let hMin = Infinity;
  let hMax = -Infinity;
  for (let i = 0; i < n; i++) {
    hMin = Math.min(hMin, map.heights[i]);
    hMax = Math.max(hMax, map.heights[i]);
  }
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      const i = r * cols + c;
      const h01 = hMax > hMin ? (map.heights[i] - hMin) / (hMax - hMin) : 0.5;
      lowMix[i] = Math.max(0, 0.45 - h01) / 0.45 * LOW_TINT;
      const u = (c + 0.5) / cols;
      const v = (r + 0.5) / rows;
      const e = Math.min(u, 1 - u, v, 1 - v);
      edgeMix[i] = e < VIGNETTE ? (1 - e / VIGNETTE) * 0.85 : 0;
    }
  }
  const bg = hexToLinear(SCENE_HEX.bg);
  const low = hexToLinear(SCENE_HEX.lowland);
  const dry = hexToLinear(SCENE_HEX.dryOchre);

  // ------------------------------------------------------------- droughts
  // Ring slots keyed by position (fixed for a drought's life).
  const rings = [];
  for (let k = 0; k < MAX_RINGS; k++) rings.push({ active: false, key: 0, x: 0, y: 0, r: 0, sev: 1, remaining: 0, born: 0, ended: 0, seen: false, label: null });
  const ringVerts = MAX_RINGS * RING_SEGMENTS * 2;
  const ringPos = new Float32Array(ringVerts * 3);
  const ringCol = new Float32Array(ringVerts * 3);
  const ringDist = new Float32Array(ringVerts);
  const ringGeo = new THREE.BufferGeometry();
  ringGeo.setAttribute('position', new THREE.BufferAttribute(ringPos, 3).setUsage(THREE.DynamicDrawUsage));
  ringGeo.setAttribute('color', new THREE.BufferAttribute(ringCol, 3).setUsage(THREE.DynamicDrawUsage));
  ringGeo.setAttribute('lineDistance', new THREE.BufferAttribute(ringDist, 1).setUsage(THREE.DynamicDrawUsage));
  ringGeo.setDrawRange(0, 0);
  const ringMat = new THREE.LineDashedMaterial({ vertexColors: true, dashSize: 0.09, gapSize: 0.06, transparent: true, blending: THREE.AdditiveBlending, depthWrite: false, fog: false });
  const ringLines = new THREE.LineSegments(ringGeo, ringMat);
  ringLines.frustumCulled = false;
  ringLines.renderOrder = 2;
  scene.add(ringLines);

  // ------------------------------------------------------------- biomes
  const biomeGeo = new THREE.BufferGeometry();
  const bSeg = 96;
  const bPos = new Float32Array(Math.max(1, cfg.biomes.length) * bSeg * 2 * 3);
  let bv = 0;
  for (const b of cfg.biomes) {
    for (let s = 0; s < bSeg; s++) {
      for (const k of [s, s + 1]) {
        const a = (k / bSeg) * Math.PI * 2;
        const x = b.x + Math.cos(a) * b.radius;
        const y = b.y + Math.sin(a) * b.radius;
        bPos[bv++] = map.sceneX(x);
        bPos[bv++] = map.heightAt(((x % map.W) + map.W) % map.W, ((y % map.H) + map.H) % map.H) + 0.015;
        bPos[bv++] = map.sceneZ(y);
      }
    }
  }
  biomeGeo.setAttribute('position', new THREE.BufferAttribute(bPos, 3));
  const biomeMat = new THREE.LineDashedMaterial({ color: 0xece8dd, dashSize: 0.025, gapSize: 0.05, transparent: true, opacity: 0.55, depthWrite: false });
  const biomeLines = new THREE.LineSegments(biomeGeo, biomeMat);
  biomeLines.computeLineDistances(); // once, at init
  biomeLines.renderOrder = 1;
  scene.add(biomeLines);

  // ------------------------------------------------------------- fruit
  const fruitPos = new Float32Array(n * 3);
  const fruitGeo = new THREE.BufferGeometry();
  fruitGeo.setAttribute('position', new THREE.BufferAttribute(fruitPos, 3).setUsage(THREE.DynamicDrawUsage));
  fruitGeo.setDrawRange(0, 0);
  const fruitMat = new THREE.PointsMaterial({ color: new THREE.Color(SCENE_HEX.berry), size: 0.06, sizeAttenuation: true, map: dotTexture(), alphaTest: 0.5, transparent: true, depthWrite: false });
  const fruitPts = new THREE.Points(fruitGeo, fruitMat);
  fruitPts.frustumCulled = false;
  fruitPts.visible = !!cfg.fruitEnabled;
  scene.add(fruitPts);

  // ------------------------------------------------------------- colour pass
  /**
   * @param layer { lut: Float32Array(256*3), values: Float32Array|null, scale, offset, droughts: bool }
   *   t = clamp((values[i] - offset) * scale, 0, 1)
   */
  function updateColors(layer, now) {
    const lut = layer.lut;
    const vals = layer.values;
    const scale = layer.scale;
    const off = layer.offset;
    // Active drought discs in sim units (≤ MAX_RINGS).
    let nd = 0;
    if (layer.droughts) for (let k = 0; k < MAX_RINGS; k++) if (rings[k].active || rings[k].ended) nd++;
    const W = map.W;
    const H = map.H;
    const cw = W / cols;
    const ch = H / rows;
    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        const i = r * cols + c;
        let t = vals ? (vals[i] - off) * scale : 0.5;
        t = t < 0 ? 0 : t > 1 ? 1 : t;
        const li = (t * 255) | 0;
        let R = lut[li * 3];
        let G = lut[li * 3 + 1];
        let B = lut[li * 3 + 2];
        // Low ground: darker, cooler (no water mechanic, so no lakes).
        const lm = lowMix[i];
        R += (low[0] - R) * lm;
        G += (low[1] - G) * lm;
        B += (low[2] - B) * lm;
        // Droughts: desaturate + warm, smooth falloff over the outer 15%.
        if (nd) {
          const x = (c + 0.5) * cw;
          const y = (r + 0.5) * ch;
          for (let k = 0; k < MAX_RINGS; k++) {
            const d = rings[k];
            if (!d.active && !d.ended) continue;
            const vis = ringVisibility(d, now);
            if (vis <= 0) continue;
            let dx = x - d.x;
            let dy = y - d.y;
            if (dx > W / 2) dx -= W;
            else if (dx < -W / 2) dx += W;
            if (dy > H / 2) dy -= H;
            else if (dy < -H / 2) dy += H;
            const rr = d.r * ringGrowth(d, now);
            const dist = Math.sqrt(dx * dx + dy * dy);
            if (dist >= rr) continue;
            const inner = rr * 0.85;
            const fall = dist <= inner ? 1 : 1 - (dist - inner) / (rr - inner);
            const m = 0.55 * (1 - d.sev) * fall * vis;
            const lum = R * 0.3 + G * 0.59 + B * 0.11;
            R += (lum - R) * m * 0.6;
            G += (lum - G) * m * 0.6;
            B += (lum - B) * m * 0.6;
            R += (dry[0] - R) * m;
            G += (dry[1] - G) * m;
            B += (dry[2] - B) * m;
          }
        }
        const em = edgeMix[i];
        R += (bg[0] - R) * em;
        G += (bg[1] - G) * em;
        B += (bg[2] - B) * em;
        colors[i * 3] = R;
        colors[i * 3 + 1] = G;
        colors[i * 3 + 2] = B;
      }
    }
    colorAttr.needsUpdate = true;
  }

  function ringGrowth(d, now) {
    if (now < 0 || d.born <= 0) return 1;
    const t = Math.min(1, (now - d.born) / RING_GROW_MS);
    return 1 - (1 - t) * (1 - t);
  }
  function ringVisibility(d, now) {
    if (d.active) return 1;
    if (!d.ended) return 0;
    return Math.max(0, 1 - (now - d.ended) / RING_FADE_MS);
  }

  /** Sync ring slots with the sim's drought table (call every frame; tiny). */
  function syncDroughts(dt, DL, now, animate) {
    for (let k = 0; k < MAX_RINGS; k++) rings[k].seen = false;
    for (let j = 0; j < dt.count; j++) {
      const o = j * dt.stride;
      const x = dt.data[o + DL.X];
      const y = dt.data[o + DL.Y];
      const key = Math.round(x) * 100003 + Math.round(y);
      let slot = null;
      for (let k = 0; k < MAX_RINGS; k++) if (rings[k].active && rings[k].key === key) slot = rings[k];
      if (!slot) {
        for (let k = 0; k < MAX_RINGS && !slot; k++) if (!rings[k].active && !rings[k].ended) slot = rings[k];
        for (let k = 0; k < MAX_RINGS && !slot; k++) if (!rings[k].active) slot = rings[k];
        if (!slot) continue;
        slot.active = true;
        slot.ended = 0;
        slot.key = key;
        slot.born = animate ? now : -1;
      }
      slot.seen = true;
      slot.x = x;
      slot.y = y;
      slot.r = dt.data[o + DL.R];
      slot.sev = dt.data[o + DL.SEVERITY];
      slot.remaining = dt.data[o + DL.REMAINING];
    }
    for (let k = 0; k < MAX_RINGS; k++) {
      const d = rings[k];
      if (d.active && !d.seen) {
        d.active = false;
        d.ended = animate ? now : now - RING_FADE_MS;
      } else if (!d.active && d.ended && now - d.ended > RING_FADE_MS) {
        d.ended = 0;
      }
    }
    // Rebuild ring vertices.
    const warn = dry;
    let v = 0;
    let dist = 0;
    for (let k = 0; k < MAX_RINGS; k++) {
      const d = rings[k];
      if (!d.active && !d.ended) continue;
      const vis = ringVisibility(d, now) * 0.7; // 70% opacity
      const rr = d.r * ringGrowth(d, now);
      for (let s = 0; s < RING_SEGMENTS; s++) {
        for (let e = 0; e < 2; e++) {
          const a = ((s + e) / RING_SEGMENTS) * Math.PI * 2;
          const sx = d.x + Math.cos(a) * rr;
          const sy = d.y + Math.sin(a) * rr;
          const wx = ((sx % map.W) + map.W) % map.W;
          const wy = ((sy % map.H) + map.H) % map.H;
          ringPos[v * 3] = map.sceneX(sx);
          ringPos[v * 3 + 1] = map.heightAt(wx, wy) + 0.02;
          ringPos[v * 3 + 2] = map.sceneZ(sy);
          ringCol[v * 3] = warn[0] * vis;
          ringCol[v * 3 + 1] = warn[1] * vis;
          ringCol[v * 3 + 2] = warn[2] * vis;
          if (e === 0) ringDist[v] = dist;
          else {
            dist += (2 * Math.PI * rr * map.unitScale) / RING_SEGMENTS;
            ringDist[v] = dist;
          }
          v++;
        }
      }
    }
    ringGeo.setDrawRange(0, v);
    ringGeo.attributes.position.needsUpdate = true;
    ringGeo.attributes.color.needsUpdate = true;
    ringGeo.attributes.lineDistance.needsUpdate = true;
  }

  /** Fruit dots on cells with fruit > 20% of max (§5). */
  function updateFruit(fruitGrid) {
    if (!fruitGrid || !fruitPts.visible) return;
    const thr = cfg.maxPerCell * 0.2;
    let k = 0;
    for (let i = 0; i < n; i++) {
      if (fruitGrid[i] > thr) {
        fruitPos[k * 3] = pos.getX(i);
        fruitPos[k * 3 + 1] = pos.getY(i) + 0.03;
        fruitPos[k * 3 + 2] = pos.getZ(i);
        k++;
      }
    }
    fruitGeo.setDrawRange(0, k);
    fruitGeo.attributes.position.needsUpdate = true;
  }

  // Map-mode "world edge, wraps around" dashed outline (§3.1).
  const e = WORLD_SCENE_SIZE / 2;
  const edgeGeo = new THREE.BufferGeometry().setFromPoints([
    new THREE.Vector3(-e, 0.03, -e), new THREE.Vector3(e, 0.03, -e),
    new THREE.Vector3(e, 0.03, -e), new THREE.Vector3(e, 0.03, e),
    new THREE.Vector3(e, 0.03, e), new THREE.Vector3(-e, 0.03, e),
    new THREE.Vector3(-e, 0.03, e), new THREE.Vector3(-e, 0.03, -e),
  ]);
  const edge = new THREE.LineSegments(edgeGeo, new THREE.LineDashedMaterial({ color: 0x858174, dashSize: 0.12, gapSize: 0.1, depthTest: false, fog: false }));
  edge.computeLineDistances();
  edge.visible = false;
  edge.renderOrder = 6;
  scene.add(edge);

  return {
    mesh,
    rings,
    edge,
    biomeLines,
    fruitPts,
    updateColors,
    syncDroughts,
    updateFruit,
    setBiomeOpacity(o) {
      biomeMat.opacity = o;
      biomeLines.visible = o > 0.01;
    },
    setFruitVisible(v) {
      fruitPts.visible = v && !!cfg.fruitEnabled;
    },
    dispose() {
      for (const o of [mesh, ringLines, biomeLines, fruitPts]) {
        scene.remove(o);
        o.geometry.dispose();
        o.material.dispose();
      }
    },
  };
}

function dotTexture() {
  const c = document.createElement('canvas');
  c.width = c.height = 32;
  const g = c.getContext('2d');
  g.fillStyle = '#fff';
  g.beginPath();
  g.arc(16, 16, 14, 0, Math.PI * 2);
  g.fill();
  const t = new THREE.CanvasTexture(c);
  t.colorSpace = THREE.SRGBColorSpace;
  return t;
}
