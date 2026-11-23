import { COHORT_LABELS, CONTROL_LABELS, casesForPolicy, findTrial, poseAt, caseScore, outcomeText, plotBounds } from './task-lab-model.js';

const dialog = document.querySelector('#taskLab');
const openButton = document.querySelector('#taskLabBtn');
const $ = id => dialog.querySelector(`#${id}`);
let data = null, loading = null, timer = null, policy, trial, arena;

function option(value, label) {
  const element = document.createElement('option');
  element.value = value;
  element.textContent = label;
  return element;
}
function stop() {
  clearTimeout(timer);
  timer = null;
  $('taskPlay').textContent = 'Play replay';
}
function setPolicies() {
  const cohort = $('taskCohort').value;
  $('taskPolicy').replaceChildren(...data.policies.filter(p => p.cohort === cohort).map(p =>
    option(p.seed ?? p.name, p.seed ? `Seed ${p.seed} · champion ${p.champion}` : CONTROL_LABELS[p.name])));
  setCases();
}
function setCases() {
  policy = data.policies.find(p => p.cohort === $('taskCohort').value && String(p.seed ?? p.name) === $('taskPolicy').value);
  const previous = $('taskCase').value;
  const cases = casesForPolicy(data, policy);
  $('taskCase').replaceChildren(...cases.map(c => option(c.name, c.name.replaceAll('_', ' '))));
  if (cases.some(c => c.name === previous)) $('taskCase').value = previous;
  selectTrial();
}
function selectTrial() {
  stop();
  policy = data.policies.find(p => p.cohort === $('taskCohort').value && String(p.seed ?? p.name) === $('taskPolicy').value);
  const caseName = $('taskCase').value;
  arena = data.cases.find(c => c.study === policy.study && c.name === caseName).arena;
  trial = findTrial(policy, caseName, $('taskCue').value);
  $('taskTick').max = trial.movement_ticks;
  $('taskTick').value = 0;
  $('taskOutcome').textContent = `Recorded outcome: ${outcomeText(trial, arena.horizon)}`;
  $('taskScore').textContent = `${caseScore(policy, caseName)}/2 correct for this policy and case. Assigned goal: ${trial.target}.`;
  const study = data.studies.find(s => s.id === policy.study);
  $('taskStudySource').textContent = `${study.label} · native evaluator ${study.evaluator.slice(0, 7)}. ${study.note}`;
  $('taskResultsLink').href = study.report;
  $('taskPlanLink').href = study.plan;
  $('taskPolicyNote').textContent = policy.cohort === 'controls'
    ? 'Fixed manufactured control; not evolved or selected in this search. Passing a reference does not establish memory benefit.'
    : 'One final training-selected champion per search, including searches with partial task success. No validation-score selection.';
  $('taskInitial').textContent = `First movement action: turn ${trial.initial_action.turn.toFixed(3)} · speed ${trial.initial_action.speed.toFixed(3)} (normalized).`;
  render();
}
function render() {
  const { index, pose } = poseAt(trial, $('taskTick').value);
  const [x, y, heading] = pose;
  const bounds = plotBounds(arena, trial);
  const svg = $('taskArena');
  svg.setAttribute('viewBox', `${bounds.minX} ${-bounds.minY - bounds.height} ${bounds.width} ${bounds.height}`);
  const ns = 'http://www.w3.org/2000/svg';
  function node(tag, attrs, text) {
    const el = document.createElementNS(ns, tag);
    Object.entries(attrs).forEach(([key, value]) => el.setAttribute(key, value));
    if (text) el.textContent = text;
    svg.append(el);
    return el;
  }
  svg.replaceChildren();
  for (const side of ['left', 'right']) {
    node('circle', { cx: arena[`goal_${side}_x`], cy: -arena[`goal_${side}_y`], r: arena.goal_radius, class: 'task-goal' });
    node('text', { x: arena[`goal_${side}_x`], y: -arena[`goal_${side}_y`], class: 'task-goal-label' }, side === 'left' ? 'L' : 'R');
  }
  const points = poses => poses.map(p => `${p[0]},${-p[1]}`).join(' ');
  node('polyline', { points: points(trial.poses), class: 'task-path-full' });
  node('polyline', { points: points(trial.poses.slice(0, index + 1)), class: 'task-path' });
  node('circle', { cx: trial.poses[0][0], cy: -trial.poses[0][1], r: .14, class: 'task-start' });
  node('circle', { cx: x, cy: -y, r: .22, class: 'task-agent' });
  node('line', { x1: x, y1: -y, x2: x + .65 * Math.cos(heading), y2: -y - .65 * Math.sin(heading), class: 'task-path' });
  svg.setAttribute('aria-label', `Recorded movement, tick ${index}. Position ${x.toFixed(2)}, ${y.toFixed(2)}. Target ${trial.target}.`);
  $('taskTickRead').textContent = `Movement tick ${index} / ${trial.movement_ticks}`;
  $('taskTick').setAttribute('aria-valuetext', `Movement tick ${index} of ${trial.movement_ticks}`);
  $('taskStep').disabled = index === trial.movement_ticks;
}
function playStep() {
  const next = Number($('taskTick').value) + 1;
  $('taskTick').value = next;
  render();
  if (next >= trial.movement_ticks) stop();
  else timer = setTimeout(playStep, 220);
}
async function open() {
  dialog.showModal();
  $('taskClose').focus();
  if (data) return;
  $('taskStatus').hidden = false;
  $('taskStatus').textContent = 'Loading saved evaluations…';
  try {
    loading ??= fetch(new URL('../data/visible-task.json', import.meta.url)).then(response => {
      if (!response.ok) throw new Error('Replay data could not be loaded.');
      return response.json();
    });
    data = await loading;
    $('taskCohort').replaceChildren(...Object.entries(COHORT_LABELS).map(([id, label]) => option(id, label)));
    $('taskBody').hidden = false;
    $('taskStatus').hidden = true;
    setPolicies();
  } catch {
    loading = null;
    $('taskStatus').textContent = 'Saved evaluations are unavailable. Close and reopen Task lab to retry.';
  }
}
openButton.addEventListener('click', open);
$('taskClose').addEventListener('click', () => dialog.close());
dialog.addEventListener('close', () => { stop(); openButton.focus(); });
// Keep ecology keyboard shortcuts outside the modal; preserve native Escape/Tab.
dialog.addEventListener('keydown', event => event.stopPropagation());
$('taskCohort').addEventListener('change', setPolicies);
$('taskPolicy').addEventListener('change', setCases);
for (const id of ['taskCase', 'taskCue']) $(id).addEventListener('change', selectTrial);
$('taskTick').addEventListener('input', () => { stop(); render(); });
$('taskStep').addEventListener('click', () => { stop(); $('taskTick').value = Number($('taskTick').value) + 1; render(); });
$('taskRestart').addEventListener('click', () => { stop(); $('taskTick').value = 0; render(); });
$('taskPlay').addEventListener('click', () => {
  if (timer) { stop(); return; }
  if (Number($('taskTick').value) >= trial.movement_ticks) $('taskTick').value = 0;
  $('taskPlay').textContent = 'Pause replay';
  timer = setTimeout(playStep, 220);
});
