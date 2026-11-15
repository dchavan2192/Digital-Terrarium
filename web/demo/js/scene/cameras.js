// Orbit / Map / Follow cameras with 700 ms eased transitions. Spec §3.5, §8.4.

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

const DEG = Math.PI / 180;
const TWEEN_MS = 700;
const ORBIT_HOME = new THREE.Vector3(0, 7.2, 9.0);
const MAP_HEIGHT = 30;

function ease(t) {
  // close to the spec's cubic-bezier(.2,.7,.2,1): fast start, soft landing
  return 1 - Math.pow(1 - t, 3);
}

export function createCameras(dom, { reducedMotion = false } = {}) {
  const persp = new THREE.PerspectiveCamera(40, 1, 0.02, 120);
  persp.position.copy(ORBIT_HOME);
  const ortho = new THREE.OrthographicCamera(-5.5, 5.5, 5.5, -5.5, 0.1, 100);
  ortho.position.set(0, MAP_HEIGHT, 0);
  ortho.up.set(0, 0, -1); // north (sim y = 0) at the top of the screen
  ortho.lookAt(0, 0, 0);

  const controls = new OrbitControls(persp, dom);
  controls.target.set(0, 0, 0);
  controls.minPolarAngle = 20 * DEG;
  controls.maxPolarAngle = 75 * DEG;
  controls.minDistance = 6;
  controls.maxDistance = 16;
  controls.enableDamping = true;
  controls.dampingFactor = 0.08;
  controls.autoRotate = !reducedMotion;
  controls.autoRotateSpeed = (0.15 * 60) / (2 * Math.PI); // ≈ 0.15 rad/s when update(dt) is used
  controls.addEventListener('start', () => {
    controls.autoRotate = false;
  });
  controls.update();

  let mode = 'orbit';
  let active = persp;
  let aspect = 1;
  let mapHalf = 5.6;
  const tween = { on: false, t0: 0, fp: new THREE.Vector3(), ft: new THREE.Vector3(), tp: new THREE.Vector3(), tt: new THREE.Vector3(), done: null };
  const followTarget = new THREE.Vector3();
  const followPos = new THREE.Vector3();
  const _v = new THREE.Vector3();
  let followInit = false;

  function startTween(toPos, toTarget, done) {
    tween.fp.copy(persp.position);
    tween.ft.copy(controls.target);
    tween.tp.copy(toPos);
    tween.tt.copy(toTarget);
    tween.done = done || null;
    if (reducedMotion) {
      persp.position.copy(toPos);
      controls.target.copy(toTarget);
      persp.lookAt(toTarget);
      tween.on = false;
      if (done) done();
      return;
    }
    tween.t0 = performance.now();
    tween.on = true;
  }

  function fitOrtho() {
    // Fit the whole 10x10 world (plus margin) on both axes.
    const h = Math.max(mapHalf, mapHalf / aspect);
    ortho.left = -h * aspect;
    ortho.right = h * aspect;
    ortho.top = h;
    ortho.bottom = -h;
    ortho.updateProjectionMatrix();
  }

  // --- Map mode pan / zoom (OrbitControls is disabled there)
  const ptrs = new Map();
  let pinchD = 0;
  function onDown(e) {
    if (mode !== 'map') return;
    ptrs.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (ptrs.size === 2) {
      const [a, b] = [...ptrs.values()];
      pinchD = Math.hypot(a.x - b.x, a.y - b.y);
    }
  }
  function onMove(e) {
    if (mode !== 'map' || !ptrs.has(e.pointerId)) return;
    const p = ptrs.get(e.pointerId);
    const wpp = (ortho.top - ortho.bottom) / ortho.zoom / dom.clientHeight;
    if (ptrs.size === 1) {
      ortho.position.x -= (e.clientX - p.x) * wpp;
      ortho.position.z -= (e.clientY - p.y) * wpp;
      clampMap();
    }
    p.x = e.clientX;
    p.y = e.clientY;
    if (ptrs.size === 2) {
      const [a, b] = [...ptrs.values()];
      const d = Math.hypot(a.x - b.x, a.y - b.y);
      if (pinchD > 0) zoomMap(pinchD / d);
      pinchD = d;
    }
  }
  function onUp(e) {
    ptrs.delete(e.pointerId);
    if (ptrs.size < 2) pinchD = 0;
  }
  function onWheel(e) {
    if (mode !== 'map') return;
    e.preventDefault();
    zoomMap(Math.exp(e.deltaY * 0.0015));
  }
  function zoomMap(k) {
    ortho.zoom = Math.min(6, Math.max(0.9, ortho.zoom / k));
    ortho.updateProjectionMatrix();
    clampMap();
  }
  function clampMap() {
    const lim = 5;
    ortho.position.x = Math.max(-lim, Math.min(lim, ortho.position.x));
    ortho.position.z = Math.max(-lim, Math.min(lim, ortho.position.z));
  }
  dom.addEventListener('pointerdown', onDown);
  dom.addEventListener('pointermove', onMove);
  dom.addEventListener('pointerup', onUp);
  dom.addEventListener('pointercancel', onUp);
  dom.addEventListener('wheel', onWheel, { passive: false });

  const api = {
    persp,
    ortho,
    controls,
    get mode() {
      return mode;
    },
    get camera() {
      return active;
    },
    get tweening() {
      return tween.on;
    },
    stopAutoRotate() {
      controls.autoRotate = false;
    },
    resize(w, h) {
      aspect = w / Math.max(1, h);
      persp.aspect = aspect;
      persp.updateProjectionMatrix();
      fitOrtho();
    },
    /** mode: 'orbit' | 'map' | 'follow' */
    setMode(next) {
      if (next === mode) return;
      const prev = mode;
      mode = next;
      controls.enabled = false;
      if (next === 'map') {
        // Rise to straight overhead, then cut to the orthographic camera.
        startTween(_v.set(0, 15, 0.001), new THREE.Vector3(0, 0, 0), () => {
          if (mode !== 'map') return;
          active = ortho;
          ortho.position.set(0, MAP_HEIGHT, 0);
          ortho.zoom = 1;
          fitOrtho();
        });
      } else if (next === 'orbit') {
        if (prev === 'map') {
          active = persp;
          persp.position.set(0, 15, 0.001);
          controls.target.set(0, 0, 0);
        }
        startTween(ORBIT_HOME, new THREE.Vector3(0, 0, 0), () => {
          if (mode === 'orbit') controls.enabled = true;
        });
      } else if (next === 'follow') {
        if (prev === 'map') {
          active = persp;
          persp.position.set(0, 15, 0.001);
        }
        tween.on = false;
        followInit = false;
        controls.autoRotate = false;
      }
    },
    /** Map-mode: centre on a sim-space point (scene coords). */
    flyTo(x, z) {
      if (mode === 'map') {
        ortho.position.x = x;
        ortho.position.z = z;
        clampMap();
      } else if (mode === 'orbit') {
        const off = _v.copy(persp.position).sub(controls.target);
        const t = new THREE.Vector3(x, 0, z);
        startTween(t.clone().add(off), t, () => {
          if (mode === 'orbit') controls.enabled = true;
        });
      }
    },
    /**
     * @param pose {x,y,z,heading} of the followed creature (scene units) or null.
     */
    update(dt, pose) {
      if (tween.on) {
        const t = Math.min(1, (performance.now() - tween.t0) / TWEEN_MS);
        const k = ease(t);
        persp.position.lerpVectors(tween.fp, tween.tp, k);
        controls.target.lerpVectors(tween.ft, tween.tt, k);
        persp.lookAt(controls.target);
        if (t >= 1) {
          tween.on = false;
          if (tween.done) tween.done();
        }
        return;
      }
      if (mode === 'orbit') {
        controls.enabled = true;
        controls.update(dt);
      } else if (mode === 'follow' && pose) {
        const fx = Math.cos(pose.heading);
        const fz = Math.sin(pose.heading);
        const bx = pose.x - fx * 0.6;
        const by = pose.y + 0.45;
        const bz = pose.z - fz * 0.6;
        const jump = followInit && followTarget.distanceToSquared(_v.set(pose.x, pose.y, pose.z)) > 4;
        if (!followInit || jump) {
          // first frame, or the creature wrapped around the torus: snap the look target
          followTarget.set(pose.x, pose.y, pose.z);
          if (jump) followPos.set(bx, by, bz);
          else followPos.copy(persp.position);
          followInit = true;
        }
        followTarget.x += (pose.x - followTarget.x) * 0.12;
        followTarget.y += (pose.y - followTarget.y) * 0.12;
        followTarget.z += (pose.z - followTarget.z) * 0.12;
        const k = reducedMotion ? 1 : 0.08;
        followPos.x += (bx - followPos.x) * k;
        followPos.y += (by - followPos.y) * k;
        followPos.z += (bz - followPos.z) * k;
        persp.position.copy(followPos);
        persp.lookAt(followTarget);
        controls.target.copy(followTarget);
      }
    },
    dispose() {
      controls.dispose();
      dom.removeEventListener('pointerdown', onDown);
      dom.removeEventListener('pointermove', onMove);
      dom.removeEventListener('pointerup', onUp);
      dom.removeEventListener('pointercancel', onUp);
      dom.removeEventListener('wheel', onWheel);
    },
  };
  return api;
}
