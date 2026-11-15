// Renderer, scene, fog, lights, muted day/night, adaptive pixel ratio, picking.
// Spec §3.4 (lighting), §3.6 (picking), §3.7 (performance), §8.1 (scene colours).

import * as THREE from 'three';
import { SCENE_HEX } from '../palette.js';

const DEG = Math.PI / 180;
// Lambert in three r155+ divides by π, so intensities carry a π factor.
// LIGHT_GAIN keeps (sun + ambient) ≈ 1 so vertex-colour data stays honest.
const LIGHT_GAIN = 0.82;

export function createStage(container, { phone = false, reducedMotion = false } = {}) {
  const renderer = new THREE.WebGLRenderer({ antialias: !phone, powerPreference: 'high-performance', alpha: false });
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  const dprCap = phone ? 1.5 : 2;
  let dpr = Math.min(window.devicePixelRatio || 1, dprCap);
  renderer.setPixelRatio(dpr);
  renderer.shadowMap.enabled = !phone;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  renderer.domElement.className = 'viewport-canvas';
  renderer.domElement.setAttribute('aria-label', 'Live 3D view of the simulated world');
  renderer.domElement.setAttribute('role', 'img');
  container.appendChild(renderer.domElement);

  const scene = new THREE.Scene();
  const bg = new THREE.Color(SCENE_HEX.bg);
  scene.background = bg;
  scene.fog = new THREE.FogExp2(bg, 0.035);

  const hemi = new THREE.HemisphereLight(0xece8dd, 0x2a2620, 0.45 * Math.PI * LIGHT_GAIN);
  scene.add(hemi);
  const sun = new THREE.DirectionalLight(new THREE.Color(SCENE_HEX.sun), Math.PI * LIGHT_GAIN);
  // Fixed sun: elevation 50°, azimuth 135° (from the south-east).
  const el = 50 * DEG;
  const az = 135 * DEG;
  sun.position.set(Math.cos(el) * Math.sin(az) * 12, Math.sin(el) * 12, Math.cos(el) * Math.cos(az) * 12);
  sun.target.position.set(0, 0, 0);
  sun.castShadow = !phone;
  sun.shadow.mapSize.set(1024, 1024);
  const sc = sun.shadow.camera;
  sc.left = -6;
  sc.right = 6;
  sc.top = 6;
  sc.bottom = -6;
  sc.near = 1;
  sc.far = 30;
  sun.shadow.bias = -0.0008;
  sun.shadow.normalBias = 0.01;
  scene.add(sun);
  scene.add(sun.target);

  const moon = new THREE.Color(SCENE_HEX.moon);
  const sunCol = new THREE.Color(SCENE_HEX.sun);
  let lightVis = 1;

  // Adaptive quality (§3.7): rolling 2 s fps < 45 -> dpr -0.25 (floor 1.0), then shadows off.
  let fpsT0 = performance.now();
  let fpsFrames = 0;
  let fps = 60;
  let shadowsOn = renderer.shadowMap.enabled;
  let onQuality = null;

  function sizeTo() {
    const w = Math.max(1, container.clientWidth);
    const h = Math.max(1, container.clientHeight);
    renderer.setSize(w, h, false);
    renderer.domElement.style.width = `${w}px`;
    renderer.domElement.style.height = `${h}px`;
    return { w, h };
  }

  const raycaster = new THREE.Raycaster();
  const ndc = new THREE.Vector2();
  const plane = new THREE.Plane(new THREE.Vector3(0, 1, 0), 0);
  const hit = new THREE.Vector3();

  return {
    renderer,
    scene,
    sun,
    hemi,
    get dpr() {
      return dpr;
    },
    get fps() {
      return fps;
    },
    set onQuality(fn) {
      onQuality = fn;
    },
    size: sizeTo,
    /**
     * Muted day/night (§3.4). light = sim light level [0,1]; at >= 4x speed the
     * modulation is disabled (L = 1) because a 600-tick day would strobe.
     */
    setDaylight(light, dt, enabled) {
      const target = enabled && !reducedMotion ? light : 1;
      const k = 1 - Math.exp(-dt / 0.5); // EMA, tau = 0.5 s real time
      lightVis += (target - lightVis) * k;
      sun.intensity = (0.55 + 0.45 * lightVis) * Math.PI * LIGHT_GAIN;
      sun.color.copy(moon).lerp(sunCol, lightVis);
    },
    render(camera) {
      renderer.render(scene, camera);
      fpsFrames++;
      const now = performance.now();
      if (now - fpsT0 >= 2000) {
        fps = (fpsFrames * 1000) / (now - fpsT0);
        fpsFrames = 0;
        fpsT0 = now;
        if (fps < 45 && document.visibilityState === 'visible') {
          if (dpr > 1.0) {
            dpr = Math.max(1.0, dpr - 0.25);
            renderer.setPixelRatio(dpr);
            sizeTo();
            if (onQuality) onQuality({ dpr, shadows: shadowsOn, fps });
          } else if (shadowsOn) {
            shadowsOn = false;
            renderer.shadowMap.enabled = false;
            sun.castShadow = false;
            scene.traverse((o) => {
              if (o.material) o.material.needsUpdate = true;
            });
            if (onQuality) onQuality({ dpr, shadows: shadowsOn, fps });
          }
        }
      }
    },
    /**
     * CPU picking (§3.6): ray -> terrain surface (fixed-point iterations on the
     * height map) -> sim coords. Returns {x, y} in sim units or null.
     */
    screenToSim(clientX, clientY, camera, map, out) {
      const r = renderer.domElement.getBoundingClientRect();
      ndc.set(((clientX - r.left) / r.width) * 2 - 1, -((clientY - r.top) / r.height) * 2 + 1);
      raycaster.setFromCamera(ndc, camera);
      plane.constant = 0;
      for (let it = 0; it < 4; it++) {
        if (!raycaster.ray.intersectPlane(plane, hit)) return null;
        const sx = map.simX(hit.x);
        const sy = map.simY(hit.z);
        plane.constant = -map.heightAt(sx, sy);
      }
      if (Math.abs(hit.x) > 5.2 || Math.abs(hit.z) > 5.2) return null;
      out.x = map.simX(hit.x);
      out.y = map.simY(hit.z);
      return out;
    },
  };
}
