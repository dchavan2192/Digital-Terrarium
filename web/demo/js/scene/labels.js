// HTML overlay labels anchored to scene positions (0 draw calls, §3.7).
// Used for biome names and drought countdowns.

import * as THREE from 'three';

export function createLabels(container) {
  const items = [];
  const v = new THREE.Vector3();
  return {
    add(text, cls = '') {
      const el = document.createElement('div');
      el.className = `scene-label ${cls}`;
      el.textContent = text;
      el.hidden = true;
      container.appendChild(el);
      const item = { el, x: 0, y: 0, z: 0, visible: false, text, opacity: 1 };
      items.push(item);
      return item;
    },
    setText(item, text) {
      if (item.text !== text) {
        item.text = text;
        item.el.textContent = text;
      }
    },
    update(camera, w, h) {
      for (const it of items) {
        if (!it.visible) {
          if (!it.el.hidden) it.el.hidden = true;
          continue;
        }
        v.set(it.x, it.y, it.z).project(camera);
        const off = v.z > 1 || v.x < -1.1 || v.x > 1.1 || v.y < -1.1 || v.y > 1.1;
        if (off) {
          it.el.hidden = true;
          continue;
        }
        it.el.hidden = false;
        const px = ((v.x + 1) / 2) * w;
        const py = ((1 - v.y) / 2) * h;
        it.el.style.transform = `translate(-50%, -100%) translate(${px.toFixed(0)}px, ${py.toFixed(0)}px)`;
        const o = String(it.opacity);
        if (it.el.style.opacity !== o) it.el.style.opacity = o;
      }
    },
    clear() {
      for (const it of items) it.el.remove();
      items.length = 0;
    },
  };
}
