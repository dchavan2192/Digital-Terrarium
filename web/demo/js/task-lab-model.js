// Saved poses only: this module never advances a neural policy or the ecology.
export const COHORT_LABELS = {
  original: 'Original pilot', terminal_only: 'Terminal-only comparison',
  terminal_progress: 'Terminal-progress comparison',
  diversity_fixed: 'One-layout training (new)', diversity_varied: 'Three-layout training (new)',
  controls: 'Fixed reference controls (earlier study)',
};
export const CONTROL_LABELS = {
  cue_independent: 'Cue-independent straight mover',
  manufactured_feed_forward: 'Manufactured feed-forward reference',
  manufactured_recurrent: 'Manufactured recurrent reference',
};
export function findTrial(policy, caseName, cue) {
  return policy.cases.find(c => c.name === caseName)?.trials.find(t => t.cue === Number(cue)) ?? null;
}
export function casesForPolicy(data, policy) {
  return policy.cases.map(c => data.cases.find(x => x.study === policy.study && x.name === c.name));
}
export function poseAt(trial, tick) {
  const index = Math.max(0, Math.min(trial.movement_ticks, Math.trunc(Number(tick) || 0)));
  return { index, pose: trial.poses[index] };
}
export function caseScore(policy, caseName) {
  return policy.cases.find(c => c.name === caseName).trials.reduce((n, t) => n + t.reward, 0);
}
export function outcomeText(trial, horizon) {
  if (trial.choice === 'timeout') return `No goal contact through tick ${trial.movement_ticks} of ${horizon}.`;
  return `${trial.reward ? 'Correct' : 'Wrong'} ${trial.choice} goal · first contact tick ${trial.movement_ticks} of ${horizon}.`;
}
export function plotBounds(arena, trial) {
  const r = arena.goal_radius;
  const xs = trial.poses.map(p => p[0]);
  const ys = trial.poses.map(p => p[1]);
  for (const side of ['left', 'right']) {
    xs.push(arena[`goal_${side}_x`] - r, arena[`goal_${side}_x`] + r);
    ys.push(arena[`goal_${side}_y`] - r, arena[`goal_${side}_y`] + r);
  }
  const minX = Math.min(...xs), maxX = Math.max(...xs);
  const minY = Math.min(...ys), maxY = Math.max(...ys);
  const pad = Math.max(1, (maxX - minX) * .1, (maxY - minY) * .1);
  return { minX: minX - pad, minY: minY - pad, width: maxX - minX + 2 * pad, height: maxY - minY + 2 * pad };
}
