// 4-step guided tour (§2.4): dimmed overlay with a spotlight cut-out and a
// short card. The sim keeps running underneath. Dismissal is remembered in
// localStorage (wrapped in try/catch: storage can be blocked or throw).

const KEY = 'terrarium.tour.v1';

export const TOUR_STEPS = [
  {
    target: '#viewport',
    title: 'Meet the creatures.',
    body: 'Each one has a small neural-network brain that started random. It senses food and neighbours, and decides how to move, eat, fight or share. Nobody programmed what they do.',
  },
  {
    target: '#legend',
    title: 'Food is energy, and energy is life.',
    body: 'Green ground is food. Moving, thinking and having babies all cost energy. Run out and you die. Lush areas regrow fast and barren ones barely regrow.',
  },
  {
    target: '#kpis',
    title: 'Colour means species.',
    body: 'When creatures eat well they reproduce, and their offspring’s brains mutate slightly. Lineages that drift far enough apart count as a new species and get a new colour.',
  },
  {
    target: '#side',
    title: 'Follow the story.',
    body: 'This feed narrates what evolution is doing right now. Click any creature to see its brain thinking, live.',
  },
];

export function tourSeen() {
  try {
    return localStorage.getItem(KEY) === 'done';
  } catch {
    return false;
  }
}

function markSeen() {
  try {
    localStorage.setItem(KEY, 'done');
  } catch {
    /* storage unavailable: the tour will simply be offered again */
  }
}

export function createTour(root, { onOpenSide } = {}) {
  const spot = root.querySelector('.tour-spot');
  const card = root.querySelector('.tour-card');
  const title = card.querySelector('.tour-title');
  const body = card.querySelector('.tour-body');
  const count = card.querySelector('.tour-count');
  const back = card.querySelector('[data-tour=back]');
  const next = card.querySelector('[data-tour=next]');
  const skip = card.querySelector('[data-tour=skip]');
  let step = -1;
  let lastFocus = null;

  function place() {
    if (step < 0) return;
    const s = TOUR_STEPS[step];
    let el = document.querySelector(s.target);
    if (el && el.offsetParent === null && s.target === '#side') el = document.querySelector('#phoneLatest') || el;
    const r = el ? el.getBoundingClientRect() : { left: 0, top: 0, width: innerWidth, height: innerHeight, right: innerWidth, bottom: innerHeight };
    const pad = 6;
    spot.style.transform = `translate(${Math.round(r.left - pad)}px, ${Math.round(r.top - pad)}px)`;
    spot.style.width = `${Math.round(r.width + pad * 2)}px`;
    spot.style.height = `${Math.round(r.height + pad * 2)}px`;
    // card: below the spotlight if it fits, else above, else centred inside it
    const cw = card.offsetWidth;
    const ch = card.offsetHeight;
    let x = Math.min(innerWidth - cw - 12, Math.max(12, r.left + r.width / 2 - cw / 2));
    let y = r.bottom + 14;
    if (y + ch > innerHeight - 12) y = r.top - ch - 14;
    if (y < 12) {
      y = Math.max(12, r.top + r.height / 2 - ch / 2);
      x = Math.min(innerWidth - cw - 12, Math.max(12, r.left + 24));
    }
    card.style.transform = `translate(${Math.round(x)}px, ${Math.round(y)}px)`;
  }

  function render() {
    const s = TOUR_STEPS[step];
    if (s.target === '#side' && onOpenSide) onOpenSide();
    title.textContent = s.title;
    body.textContent = s.body;
    count.textContent = `${step + 1} / ${TOUR_STEPS.length}`;
    back.disabled = step === 0;
    next.textContent = step === TOUR_STEPS.length - 1 ? 'Start exploring' : 'Next';
    requestAnimationFrame(place);
  }

  function close() {
    step = -1;
    root.hidden = true;
    markSeen();
    document.removeEventListener('keydown', onKey, true);
    window.removeEventListener('resize', place);
    if (lastFocus && lastFocus.focus) lastFocus.focus();
  }

  function onKey(e) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      close();
    } else if (e.key === 'ArrowRight') {
      e.stopPropagation();
      next.click();
    } else if (e.key === 'ArrowLeft') {
      e.stopPropagation();
      back.click();
    }
  }

  back.addEventListener('click', () => {
    if (step > 0) {
      step--;
      render();
    }
  });
  next.addEventListener('click', () => {
    if (step < TOUR_STEPS.length - 1) {
      step++;
      render();
    } else close();
  });
  skip.addEventListener('click', close);

  return {
    start() {
      lastFocus = document.activeElement;
      step = 0;
      root.hidden = false;
      render();
      document.addEventListener('keydown', onKey, true);
      window.addEventListener('resize', place);
      next.focus();
    },
    get active() {
      return step >= 0;
    },
    place,
  };
}
