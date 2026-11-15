// Instanced "seed-pod" creatures, eyes, inverted-hull outline, state halos,
// attack sparks + death glyphs, selection ring, death ghosts.
// Spec §4 (creature design), §4.4 (state cues), §4.5 (animation), §3.7 (budget).
//
// Draw calls: bodies 1, outlines 1, eyes 1, halos 1, lines 1, selection 1 = 6.
// Trophic shape cue (fin / mandibles) is ONE geometry: a per-vertex `aPart`
// (0 pod, 1 fin, 2 mandible) is collapsed in the vertex shader when it exceeds
// the per-instance `aTier` (0 grazer, 1 omnivore, 2 hunter). No extra calls.
//
// The render-loop path (update) allocates nothing: all scratch objects and
// typed arrays are created here once.

import * as THREE from 'three';
import { SPECIES_LINEAR, OTHER_SLOT, SCENE_HEX, hexToLinear } from '../palette.js';

export const VIS = {
  scale: 2.2, // multiplier on the spec's body length; 1.0 left creatures a few pixels wide (tune via ?scale=)
  bodyLen: 0.07, // §4.1: world-space body length 0.07 × √body_size scene units
  eatMs: 250,
  attackMs: 200,
  reproMs: 600,
  dieMs: 700,
  bornMs: 600,
  glyphMs: 1200,
};

const GEO_LEN = 2.7; // pod geometry length along Z (2 × 1.35)
const SLOT_BITS = 14;
const SLOTS = 1 << SLOT_BITS;
const MASK = SLOTS - 1;
const MAX_GHOSTS = 64;
const MAX_SPARKS = 64;
const CUE_NONE = 0;
const CUE_EAT = 1;
const CUE_ATTACK = 2;
const CUE_REPRO = 3;

function buildBodyGeometry() {
  const parts = [];
  const pod = new THREE.IcosahedronGeometry(1, 1);
  pod.scale(1, 0.62, 1.35);
  parts.push([pod, 0]);
  // Dorsal fin: a thin 3-face wedge.
  const A = [0, 0.42, 0.35];
  const B = [0, 0.42, -0.85];
  const C = [0, 1.0, -0.62];
  const w = 0.06;
  const fin = new THREE.BufferGeometry();
  fin.setAttribute('position', new THREE.Float32BufferAttribute([
    A[0] + w, A[1], A[2], B[0] + w, B[1], B[2], C[0], C[1], C[2],
    A[0] - w, A[1], A[2], C[0], C[1], C[2], B[0] - w, B[1], B[2],
    B[0] + w, B[1], B[2], B[0] - w, B[1], B[2], C[0], C[1], C[2],
  ], 3));
  fin.computeVertexNormals();
  parts.push([fin, 1]);
  // Mandibles: two small forward cones converging in front of the eyes.
  for (const sx of [-1, 1]) {
    const c = new THREE.ConeGeometry(0.11, 0.45, 5).toNonIndexed();
    c.rotateX(Math.PI / 2); // apex -> +Z (forward)
    c.rotateY(-sx * 0.35);
    c.translate(sx * 0.27, -0.05, 1.35);
    parts.push([c, 2]);
  }
  let count = 0;
  for (const [g] of parts) count += g.attributes.position.count;
  const P = new Float32Array(count * 3);
  const N = new Float32Array(count * 3);
  const T = new Float32Array(count);
  let k = 0;
  for (const [g, part] of parts) {
    const p = g.attributes.position.array;
    const nrm = g.attributes.normal.array;
    P.set(p, k * 3);
    N.set(nrm, k * 3);
    T.fill(part, k, k + g.attributes.position.count);
    k += g.attributes.position.count;
    g.dispose();
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(P, 3));
  geo.setAttribute('normal', new THREE.BufferAttribute(N, 3));
  geo.setAttribute('aPart', new THREE.BufferAttribute(T, 1));
  geo.computeBoundingSphere();
  return geo;
}

function buildEyeGeometry() {
  const g = new THREE.IcosahedronGeometry(0.18, 0); // non-indexed, 20 faces
  const p = g.attributes.position;
  const col = new Float32Array(p.count * 3);
  const white = hexToLinear('#F4F1E8');
  const pupil = hexToLinear('#141613');
  for (let f = 0; f < p.count / 3; f++) {
    const cz = (p.getZ(f * 3) + p.getZ(f * 3 + 1) + p.getZ(f * 3 + 2)) / 3;
    const c = cz > 0.1 ? pupil : white;
    for (let v = 0; v < 3; v++) col.set(c, (f * 3 + v) * 3);
  }
  g.setAttribute('color', new THREE.BufferAttribute(col, 3));
  return g;
}

function tierPatch(shader) {
  shader.vertexShader = shader.vertexShader
    .replace('#include <common>', '#include <common>\nattribute float aPart;\nattribute float aTier;')
    .replace('#include <begin_vertex>', '#include <begin_vertex>\nif (aPart > aTier + 0.5) transformed *= 0.0;');
}

export function createCreatures(scene, map, cfg, sim, speciesColors) {
  const { L, F } = sim;
  const CAP = cfg.hardCap + 64 + MAX_GHOSTS;
  const bodyGeo = buildBodyGeometry();
  const tierAttr = new THREE.InstancedBufferAttribute(new Float32Array(CAP), 1);
  tierAttr.setUsage(THREE.DynamicDrawUsage);
  bodyGeo.setAttribute('aTier', tierAttr);

  // --- bodies
  const bodyMat = new THREE.MeshLambertMaterial({ flatShading: true });
  bodyMat.onBeforeCompile = tierPatch;
  bodyMat.customProgramCacheKey = () => 'terrarium-pod';
  const bodies = new THREE.InstancedMesh(bodyGeo, bodyMat, CAP);
  bodies.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
  bodies.instanceColor = new THREE.InstancedBufferAttribute(new Float32Array(CAP * 3), 3).setUsage(THREE.DynamicDrawUsage);
  bodies.castShadow = true;
  bodies.frustumCulled = false;
  bodies.count = 0;
  const depthMat = new THREE.MeshDepthMaterial({ depthPacking: THREE.RGBADepthPacking });
  depthMat.onBeforeCompile = tierPatch;
  depthMat.customProgramCacheKey = () => 'terrarium-pod-depth';
  bodies.customDepthMaterial = depthMat;
  scene.add(bodies);

  // --- inverted-hull outline, constant pixel width (§4.2: ship in MVP)
  const outlineMat = new THREE.ShaderMaterial({
    side: THREE.BackSide,
    uniforms: {
      uPx: { value: 1.4 },
      uRes: { value: new THREE.Vector2(1, 1) },
      uColor: { value: new THREE.Color(SCENE_HEX.outline) },
      uColorPred: { value: new THREE.Color(SCENE_HEX.outlinePredator) },
    },
    vertexShader: /* glsl */ `
      attribute float aPart;
      attribute float aTier;
      uniform float uPx;
      uniform vec2 uRes;
      varying float vTier;
      void main() {
        vec3 p = position;
        if (aPart > aTier + 0.5) p *= 0.0;
        vec4 mv = modelViewMatrix * instanceMatrix * vec4(p, 1.0);
        vec3 nv = normalize(normalMatrix * mat3(instanceMatrix) * normal);
        vec4 clip = projectionMatrix * mv;
        vec2 dir = length(nv.xy) > 1e-4 ? normalize(nv.xy) : vec2(0.0);
        clip.xy += dir * uPx * 2.0 / uRes * clip.w;
        vTier = aTier;
        gl_Position = clip;
      }`,
    fragmentShader: /* glsl */ `
      uniform vec3 uColor;
      uniform vec3 uColorPred;
      varying float vTier;
      void main() {
        gl_FragColor = vec4(vTier > 1.5 ? uColorPred : uColor, 1.0);
        #include <colorspace_fragment>
      }`,
  });
  const outlines = new THREE.InstancedMesh(bodyGeo, outlineMat, CAP);
  outlines.instanceMatrix = bodies.instanceMatrix; // share the matrices
  outlines.frustumCulled = false;
  outlines.count = 0;
  scene.add(outlines);

  // --- eyes (2 per creature)
  const eyeMat = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true });
  const eyes = new THREE.InstancedMesh(buildEyeGeometry(), eyeMat, CAP * 2);
  eyes.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
  eyes.instanceColor = new THREE.InstancedBufferAttribute(new Float32Array(CAP * 6), 3).setUsage(THREE.DynamicDrawUsage);
  eyes.frustumCulled = false;
  eyes.count = 0;
  scene.add(eyes);

  // --- halos (flat ground rings, additive)
  const ringGeo = new THREE.RingGeometry(0.9, 1.0, 24);
  ringGeo.rotateX(-Math.PI / 2);
  const haloMat = new THREE.MeshBasicMaterial({ transparent: true, blending: THREE.AdditiveBlending, depthWrite: false, fog: false });
  const halos = new THREE.InstancedMesh(ringGeo, haloMat, CAP);
  halos.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
  halos.instanceColor = new THREE.InstancedBufferAttribute(new Float32Array(CAP * 3), 3).setUsage(THREE.DynamicDrawUsage);
  halos.frustumCulled = false;
  halos.count = 0;
  halos.renderOrder = 3;
  scene.add(halos);

  // --- lines: attack sparks + death glyphs (one LineSegments)
  const LINE_VERTS = MAX_SPARKS * 2 + MAX_GHOSTS * 4;
  const linePos = new Float32Array(LINE_VERTS * 3);
  const lineCol = new Float32Array(LINE_VERTS * 3);
  const lineGeo = new THREE.BufferGeometry();
  lineGeo.setAttribute('position', new THREE.BufferAttribute(linePos, 3).setUsage(THREE.DynamicDrawUsage));
  lineGeo.setAttribute('color', new THREE.BufferAttribute(lineCol, 3).setUsage(THREE.DynamicDrawUsage));
  lineGeo.setDrawRange(0, 0);
  const lines = new THREE.LineSegments(lineGeo, new THREE.LineBasicMaterial({ vertexColors: true, transparent: true, blending: THREE.AdditiveBlending, depthWrite: false, fog: false }));
  lines.frustumCulled = false;
  lines.renderOrder = 4;
  scene.add(lines);

  // --- selection ring
  const selRing = new THREE.Mesh(new THREE.TorusGeometry(1, 0.07, 6, 40), new THREE.MeshBasicMaterial({ color: new THREE.Color(SCENE_HEX.selection), transparent: true, opacity: 0.9, depthWrite: false, fog: false }));
  selRing.rotation.x = Math.PI / 2;
  selRing.visible = false;
  selRing.renderOrder = 5;
  scene.add(selRing);

  // --- per-id slot state (direct-mapped by id; collisions just skip the lerp)
  const sId = new Int32Array(SLOTS).fill(-1);
  const sX = new Float32Array(SLOTS);
  const sY = new Float32Array(SLOTS);
  const sH = new Float32Array(SLOTS);
  const sPX = new Float32Array(SLOTS);
  const sPY = new Float32Array(SLOTS);
  const sPH = new Float32Array(SLOTS);
  const sPhase = new Float32Array(SLOTS);
  const sBorn = new Float64Array(SLOTS);
  const sCue = new Uint8Array(SLOTS);
  const sCueT = new Float64Array(SLOTS);
  const sSize = new Float32Array(SLOTS);
  const sReproTick = new Float64Array(SLOTS);
  const sIdx = new Int32Array(SLOTS);
  const sFrame = new Int32Array(SLOTS);
  // per-instance display state (for picking / follow / labels)
  const dispX = new Float32Array(CAP);
  const dispY = new Float32Array(CAP);
  const dispId = new Int32Array(CAP);
  const dispSpecies = new Int32Array(CAP);
  let dispCount = 0;
  // ghosts
  const gX = new Float32Array(MAX_GHOSTS);
  const gY = new Float32Array(MAX_GHOSTS);
  const gH = new Float32Array(MAX_GHOSTS);
  const gSize = new Float32Array(MAX_GHOSTS);
  const gSpecies = new Int32Array(MAX_GHOSTS);
  const gCause = new Int8Array(MAX_GHOSTS);
  const gT = new Float64Array(MAX_GHOSTS).fill(-1e12);
  let gNext = 0;
  // sparks
  const spA = new Int32Array(MAX_SPARKS);
  const spV = new Int32Array(MAX_SPARKS);
  const spT = new Float64Array(MAX_SPARKS).fill(-1e12);
  let spNext = 0;

  // scratch (no allocation in update)
  const _m = new THREE.Matrix4();
  const _p = new THREE.Vector3();
  const _q = new THREE.Quaternion();
  const _e = new THREE.Euler(0, 0, 0, 'YXZ');
  const _s = new THREE.Vector3();
  const _v = new THREE.Vector3();
  const BG = hexToLinear(SCENE_HEX.bg);
  const LEAF = hexToLinear(SCENE_HEX.leaf);
  const BONE = hexToLinear(SCENE_HEX.bone);
  const ALERT = hexToLinear(SCENE_HEX.alert);
  const OCHRE = hexToLinear(SCENE_HEX.dryOchre);
  const EYE_OFF_X = 0.32;
  const EYE_OFF_Y = 0.28;
  const EYE_OFF_Z = 0.95;
  const W = map.W;
  const H = map.H;
  const rm = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;

  let frameNo = 1;
  let selectedId = -1;
  let selectedSpecies = -2;
  let highlightSpecies = -2; // -2 = none
  let fast = false;
  let lastTick = 0;

  const api = {
    bodies,
    outlines,
    eyes,
    halos,
    lines,
    selRing,
    onSelectedDied: null,
    get selectedId() {
      return selectedId;
    },
    select(id) {
      selectedId = id;
      selRing.visible = id >= 0;
      if (id < 0) selectedSpecies = -2;
    },
    highlight(speciesId) {
      highlightSpecies = speciesId;
    },
    get highlighted() {
      return highlightSpecies;
    },
    setResolution(w, h, dpr) {
      outlineMat.uniforms.uRes.value.set(w * dpr, h * dpr);
      outlineMat.uniforms.uPx.value = 1.2 * dpr;
    },
    /** Tick of the last reproduction seen for an id (for "Just had offspring"). */
    lastReproTick(id) {
      const s = id & MASK;
      return sId[s] === id ? sReproTick[s] : -1e12;
    },
    reset() {
      sId.fill(-1);
      gT.fill(-1e12);
      spT.fill(-1e12);
      selectedId = -1;
      selRing.visible = false;
      highlightSpecies = -2;
      dispCount = 0;
    },

    /** After each sim batch: roll interpolation state, trigger cues, ghosts, sparks. */
    ingest(cr, deaths, DTH, now, isFast, tick) {
      fast = isFast;
      lastTick = tick;
      const S = cr.stride;
      const d = cr.data;
      const ids = cr.ids;
      let sparks = 0;
      for (let i = 0; i < cr.count; i++) {
        const o = i * S;
        const id = ids ? ids[i] : d[o + L.ID] | 0;
        const s = id & MASK;
        const flags = d[o + L.FLAGS] | 0;
        const x = d[o + L.X];
        const y = d[o + L.Y];
        const h = d[o + L.HEADING];
        if (sId[s] !== id) {
          sId[s] = id;
          sPX[s] = x;
          sPY[s] = y;
          sPH[s] = h;
          sPhase[s] = (id * 2.399) % 6.283;
          sCue[s] = CUE_NONE;
          sReproTick[s] = -1e12;
          sBorn[s] = flags & F.NEWBORN ? now : -1e12;
        } else {
          sPX[s] = sX[s];
          sPY[s] = sY[s];
          sPH[s] = sH[s];
          if (flags & F.NEWBORN) sBorn[s] = now;
        }
        sX[s] = x;
        sY[s] = y;
        sH[s] = h;
        sSize[s] = d[o + L.BODY];
        // Dominant cue: reproducing > attacking > eating (dying handled by ghosts).
        const busy = now - sCueT[s] < cueDuration(sCue[s]);
        if (flags & F.REPRODUCED) {
          sReproTick[s] = tick;
          sCue[s] = CUE_REPRO;
          sCueT[s] = now;
        } else if (flags & F.ATTACKED) {
          if (!busy || sCue[s] === CUE_EAT) {
            sCue[s] = CUE_ATTACK;
            sCueT[s] = now;
          }
          if (sparks < 24) {
            const v = nearestOther(cr, i, x, y, cfg.attackRange * 2.5);
            if (v >= 0) {
              spA[spNext] = id;
              spV[spNext] = v;
              spT[spNext] = now;
              spNext = (spNext + 1) % MAX_SPARKS;
              sparks++;
            }
          }
        } else if (flags & F.ATE && !fast && !busy) {
          sCue[s] = CUE_EAT;
          sCueT[s] = now;
        }
      }
      // Deaths -> ghosts (+ notify if the selected creature died).
      if (deaths && deaths.count) {
        const dd = deaths.data;
        const DS = deaths.stride;
        for (let j = 0; j < deaths.count; j++) {
          const o = j * DS;
          const id = deaths.ids ? deaths.ids[j] : -1;
          const s = id & MASK;
          const known = id >= 0 && sId[s] === id;
          const cause = dd[o + DTH.CAUSE] | 0;
          if (id === selectedId && api.onSelectedDied) api.onSelectedDied(id, cause, dd[o + DTH.X], dd[o + DTH.Y]);
          if (!known) continue; // born and died inside one batch: nothing to fade
          const g = gNext;
          gNext = (gNext + 1) % MAX_GHOSTS;
          gX[g] = dd[o + DTH.X];
          gY[g] = dd[o + DTH.Y];
          gH[g] = sH[s];
          gSize[g] = sSize[s];
          gSpecies[g] = dd[o + DTH.SPECIES] | 0;
          gCause[g] = cause;
          gT[g] = now;
          sId[s] = -1;
        }
      }
    },

    /** Every frame. alpha ∈ [0,1] = progress between the last two sim states. */
    update(cr, now, dt, alpha) {
      frameNo++;
      const S = cr.stride;
      const d = cr.data;
      const ids = cr.ids;
      const n = Math.min(cr.count, cfg.hardCap + 64);
      const mArr = bodies.instanceMatrix.array;
      const cArr = bodies.instanceColor.array;
      const tArr = tierAttr.array;
      const eArr = eyes.instanceMatrix.array;
      const ecArr = eyes.instanceColor.array;
      const hArr = halos.instanceMatrix.array;
      const hcArr = halos.instanceColor.array;
      const scaleK = VIS.scale * VIS.bodyLen / GEO_LEN;
      const inspecting = selectedId >= 0;
      let hi = 0;
      let selIdx = -1;
      if (inspecting) selectedSpecies = -2;
      for (let i = 0; i < n; i++) {
        const o = i * S;
        const id = ids ? ids[i] : d[o + L.ID] | 0;
        if (id === selectedId) {
          selIdx = i;
          selectedSpecies = d[o + L.SPECIES] | 0;
        }
      }
      for (let i = 0; i < n; i++) {
        const o = i * S;
        const id = ids ? ids[i] : d[o + L.ID] | 0;
        const s = id & MASK;
        let x = d[o + L.X];
        let y = d[o + L.Y];
        let h = d[o + L.HEADING];
        if (sId[s] === id && alpha < 1) {
          let dx = sX[s] - sPX[s];
          let dy = sY[s] - sPY[s];
          if (dx > W * 0.5) dx -= W;
          else if (dx < -W * 0.5) dx += W;
          if (dy > H * 0.5) dy -= H;
          else if (dy < -H * 0.5) dy += H;
          x = sPX[s] + dx * alpha;
          y = sPY[s] + dy * alpha;
          if (x < 0) x += W;
          else if (x >= W) x -= W;
          if (y < 0) y += H;
          else if (y >= H) y -= H;
          let dh = sH[s] - sPH[s];
          if (dh > Math.PI) dh -= 2 * Math.PI;
          else if (dh < -Math.PI) dh += 2 * Math.PI;
          h = sPH[s] + dh * alpha;
        }
        sIdx[s] = i;
        sFrame[s] = frameNo;
        dispX[i] = x;
        dispY[i] = y;
        dispId[i] = id;
        const species = d[o + L.SPECIES] | 0;
        dispSpecies[i] = species;
        const bs = d[o + L.BODY];
        const speedN = d[o + L.SPEED_NORM];
        const turn = d[o + L.TURN];
        const energy = d[o + L.ENERGY_FRAC];
        const pf = d[o + L.PREDATION];
        let sc = scaleK * Math.sqrt(bs);
        // newborn scale-up (ease-out-back)
        const bt = (now - sBorn[s]) / VIS.bornMs;
        if (bt < 1 && !rm) {
          const c1 = 1.70158;
          const u = Math.max(0, bt) - 1;
          sc *= Math.max(0.05, 1 + (c1 + 1) * u * u * u + c1 * u * u);
        }
        // gait
        let yoff = 0;
        let breath = 1;
        if (speedN >= 0.05) {
          sPhase[s] += speedN * 18 * dt;
          yoff = 0.25 * sc * speedN * Math.abs(Math.sin(sPhase[s]));
        } else {
          breath = 1 + 0.03 * Math.sin(now * 0.00377 + sPhase[s]);
        }
        const gy = map.heightAt(x, y);
        _p.set(map.sceneX(x), gy + 0.55 * sc + yoff, map.sceneZ(y));
        _e.set(0, Math.PI / 2 - h, (turn || 0) * 0.21);
        _q.setFromEuler(_e);
        _s.set(sc, sc * breath, sc);
        _m.compose(_p, _q, _s);
        _m.toArray(mArr, i * 16);
        tArr[i] = pf > 0.5 ? 2 : pf >= 0.25 ? 1 : 0;
        // eyes
        for (let k = 0; k < 2; k++) {
          _v.set((k ? EYE_OFF_X : -EYE_OFF_X) * sc, EYE_OFF_Y * sc * breath, EYE_OFF_Z * sc).applyQuaternion(_q).add(_p);
          _s.set(sc, sc, sc);
          _m.compose(_v, _q, _s);
          _m.toArray(eArr, (i * 2 + k) * 16);
        }
        // colour: species hue, energy fade, selection / highlight dimming
        const slot = species >= 0 ? speciesColors.slot(species) : OTHER_SLOT;
        let fade = (1 - Math.min(1, Math.max(0, energy))) * 0.45;
        let dim = 0;
        if (inspecting && id !== selectedId) dim = species === selectedSpecies && species >= 0 ? 0.15 : 0.4;
        else if (highlightSpecies > -2 && species !== highlightSpecies) dim = 0.6;
        fade = 1 - (1 - fade) * (1 - dim);
        const r = SPECIES_LINEAR[slot * 3];
        const g = SPECIES_LINEAR[slot * 3 + 1];
        const b = SPECIES_LINEAR[slot * 3 + 2];
        cArr[i * 3] = r + (BG[0] - r) * fade;
        cArr[i * 3 + 1] = g + (BG[1] - g) * fade;
        cArr[i * 3 + 2] = b + (BG[2] - b) * fade;
        const eb = 1 - dim * 0.75;
        for (let k = 0; k < 6; k++) ecArr[i * 6 + k] = eb;
        // halo cue
        const cue = sCue[s];
        if (cue !== CUE_NONE && hi < CAP) {
          const dur = cueDuration(cue);
          const t = (now - sCueT[s]) / dur;
          if (t >= 0 && t < 1) {
            const len = sc * GEO_LEN;
            let rad = len * 0.55;
            let a = 0;
            let C = LEAF;
            if (cue === CUE_EAT) {
              rad = len * (0.45 + 0.15 * t);
              a = 0.35 * (1 - t);
            } else if (cue === CUE_ATTACK) {
              C = ALERT;
              rad = len * 0.6;
              a = 0.8 * (1 - t);
            } else if (cue === CUE_REPRO) {
              C = BONE;
              rad = rm ? len : len * (0.5 + 1.6 * t);
              a = 0.7 * (1 - t);
            }
            a *= 1 - dim;
            _p.set(map.sceneX(x), gy + 0.006, map.sceneZ(y));
            _q.identity();
            _s.set(rad, 1, rad);
            _m.compose(_p, _q, _s);
            _m.toArray(hArr, hi * 16);
            hcArr[hi * 3] = C[0] * a;
            hcArr[hi * 3 + 1] = C[1] * a;
            hcArr[hi * 3 + 2] = C[2] * a;
            hi++;
          } else if (t >= 1) {
            sCue[s] = CUE_NONE;
          }
        }
      }
      dispCount = n;
      // ghosts appended after the live creatures
      let gi = n;
      let lv = 0;
      for (let g = 0; g < MAX_GHOSTS; g++) {
        const age = now - gT[g];
        if (age < 0 || age > VIS.glyphMs) continue;
        const t = Math.min(1, age / (fast ? VIS.dieMs * 0.5 : VIS.dieMs));
        const gx = gX[g];
        const gyW = gY[g];
        const ground = map.heightAt(gx, gyW);
        const sc = scaleK * Math.sqrt(gSize[g]) * (1 - 0.2 * t);
        if (t < 1 && gi < CAP) {
          _p.set(map.sceneX(gx), ground + 0.55 * sc - 0.3 * 0.62 * 2 * sc * t, map.sceneZ(gyW));
          _e.set(0, Math.PI / 2 - gH[g], 0);
          _q.setFromEuler(_e);
          _s.set(sc, sc, sc);
          _m.compose(_p, _q, _s);
          _m.toArray(mArr, gi * 16);
          tArr[gi] = 0;
          const slot = gSpecies[g] >= 0 ? speciesColors.slot(gSpecies[g]) : OTHER_SLOT;
          const r = SPECIES_LINEAR[slot * 3];
          const gg = SPECIES_LINEAR[slot * 3 + 1];
          const b = SPECIES_LINEAR[slot * 3 + 2];
          const lum = r * 0.3 + gg * 0.59 + b * 0.11;
          const ds = Math.min(1, t * 1.6); // desaturate first, then fade into the ground
          const R = r + (lum - r) * ds;
          const G = gg + (lum - gg) * ds;
          const B = b + (lum - b) * ds;
          cArr[gi * 3] = R + (BG[0] - R) * t;
          cArr[gi * 3 + 1] = G + (BG[1] - G) * t;
          cArr[gi * 3 + 2] = B + (BG[2] - B) * t;
          // eyes: park them inside the body (scale 0)
          _s.set(0, 0, 0);
          _m.compose(_p, _q, _s);
          _m.toArray(eArr, gi * 2 * 16);
          _m.toArray(eArr, (gi * 2 + 1) * 16);
          gi++;
        }
        // death glyph ✕ (ochre = starved, red = killed)
        const cause = gCause[g];
        if ((cause === 0 || cause === 3) && lv + 4 <= LINE_VERTS) {
          const C = cause === 3 ? ALERT : OCHRE;
          const a = 0.9 * (1 - age / VIS.glyphMs);
          const X = map.sceneX(gx);
          const Z = map.sceneZ(gyW);
          const k = scaleK * 1.6;
          const Y = ground + 0.01;
          lv = pushLine(lv, X - k, Y, Z - k, X + k, Y, Z + k, C, a);
          lv = pushLine(lv, X - k, Y, Z + k, X + k, Y, Z - k, C, a);
        }
      }
      // attack sparks attacker -> victim (displayed positions)
      const sparkMs = fast ? VIS.attackMs * 0.5 : VIS.attackMs;
      for (let k = 0; k < MAX_SPARKS; k++) {
        const age = now - spT[k];
        if (age < 0 || age > sparkMs || lv + 2 > LINE_VERTS) continue;
        const ia = indexOf(spA[k]);
        const iv = indexOf(spV[k]);
        if (ia < 0 || iv < 0) continue;
        const ax = dispX[ia];
        const ay = dispY[ia];
        let dx = dispX[iv] - ax;
        let dy = dispY[iv] - ay;
        if (dx > W * 0.5) dx -= W;
        else if (dx < -W * 0.5) dx += W;
        if (dy > H * 0.5) dy -= H;
        else if (dy < -H * 0.5) dy += H;
        const a = 1 - age / sparkMs;
        const y0 = map.heightAt(ax, ay) + 0.03;
        lv = pushLine(lv, map.sceneX(ax), y0, map.sceneZ(ay), map.sceneX(ax + dx), y0, map.sceneZ(ay + dy), ALERT, a);
      }
      lineGeo.setDrawRange(0, lv);
      if (lv) {
        lineGeo.attributes.position.needsUpdate = true;
        lineGeo.attributes.color.needsUpdate = true;
      }
      bodies.count = gi;
      outlines.count = gi;
      eyes.count = gi * 2;
      halos.count = hi;
      bodies.instanceMatrix.needsUpdate = true;
      bodies.instanceColor.needsUpdate = true;
      tierAttr.needsUpdate = true;
      eyes.instanceMatrix.needsUpdate = true;
      eyes.instanceColor.needsUpdate = true;
      if (hi) {
        halos.instanceMatrix.needsUpdate = true;
        halos.instanceColor.needsUpdate = true;
      }
      // selection ring floats above the selected creature, 0.5 Hz pulse
      if (selIdx >= 0) {
        const x = dispX[selIdx];
        const y = dispY[selIdx];
        const sc = scaleK * Math.sqrt(d[selIdx * S + L.BODY]) * GEO_LEN;
        selRing.visible = true;
        selRing.position.set(map.sceneX(x), map.heightAt(x, y) + sc * 0.9, map.sceneZ(y));
        const pulse = rm ? 1 : 1 + 0.12 * Math.sin(now * 0.00314);
        selRing.scale.setScalar(sc * 0.75 * pulse);
      } else {
        selRing.visible = false;
      }
    },

    /** Nearest displayed creature to a sim position within r sim units, or -1. */
    pick(x, y, r) {
      let best = -1;
      let bd = r * r;
      for (let i = 0; i < dispCount; i++) {
        let dx = dispX[i] - x;
        let dy = dispY[i] - y;
        if (dx > W * 0.5) dx -= W;
        else if (dx < -W * 0.5) dx += W;
        if (dy > H * 0.5) dy -= H;
        else if (dy < -H * 0.5) dy += H;
        const d2 = dx * dx + dy * dy;
        if (d2 < bd) {
          bd = d2;
          best = i;
        }
      }
      return best < 0 ? -1 : dispId[best];
    },

    /** Instance row of an id in the current frame, or -1. */
    indexOf,

    /** Displayed scene pose of an id: writes out.{x,y,z,heading,simX,simY}. */
    pose(id, out) {
      const i = indexOf(id);
      if (i < 0) return false;
      const s = id & MASK;
      out.simX = dispX[i];
      out.simY = dispY[i];
      out.x = map.sceneX(dispX[i]);
      out.z = map.sceneZ(dispY[i]);
      out.y = map.heightAt(dispX[i], dispY[i]);
      out.heading = sH[s];
      return true;
    },
  };

  function indexOf(id) {
    const s = id & MASK;
    return sId[s] === id && sFrame[s] === frameNo ? sIdx[s] : -1;
  }

  function cueDuration(cue) {
    const k = fast ? 0.5 : 1;
    if (cue === CUE_EAT) return VIS.eatMs * k;
    if (cue === CUE_ATTACK) return VIS.attackMs * k * 2;
    if (cue === CUE_REPRO) return VIS.reproMs * k;
    return 0;
  }

  function nearestOther(cr, self, x, y, r) {
    const S = cr.stride;
    const d = cr.data;
    let best = -1;
    let bd = r * r;
    for (let j = 0; j < cr.count; j++) {
      if (j === self) continue;
      let dx = d[j * S + L.X] - x;
      let dy = d[j * S + L.Y] - y;
      if (dx > W * 0.5) dx -= W;
      else if (dx < -W * 0.5) dx += W;
      if (dy > H * 0.5) dy -= H;
      else if (dy < -H * 0.5) dy += H;
      const d2 = dx * dx + dy * dy;
      if (d2 < bd) {
        bd = d2;
        best = j;
      }
    }
    if (best < 0) return -1;
    return cr.ids ? cr.ids[best] : d[best * S + L.ID] | 0;
  }

  function pushLine(v, x0, y0, z0, x1, y1, z1, C, a) {
    linePos[v * 3] = x0;
    linePos[v * 3 + 1] = y0;
    linePos[v * 3 + 2] = z0;
    lineCol[v * 3] = C[0] * a;
    lineCol[v * 3 + 1] = C[1] * a;
    lineCol[v * 3 + 2] = C[2] * a;
    v++;
    linePos[v * 3] = x1;
    linePos[v * 3 + 1] = y1;
    linePos[v * 3 + 2] = z1;
    lineCol[v * 3] = C[0] * a;
    lineCol[v * 3 + 1] = C[1] * a;
    lineCol[v * 3 + 2] = C[2] * a;
    return v + 1;
  }

  return api;
}
