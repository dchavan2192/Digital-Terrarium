import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { casesForPolicy, findTrial, poseAt, caseScore, outcomeText, plotBounds } from '../js/task-lab-model.js';

const root = new URL('../../../', import.meta.url);
const data = JSON.parse(readFileSync(new URL('../data/visible-task.json', import.meta.url)));
const champions = data.policies.filter(p => p.cohort !== 'controls');
const seedSets = { original: [2000, 2001, 2002], terminal_only: [2100, 2101, 2102], terminal_progress: [2100, 2101, 2102], diversity_fixed: [5000, 5001, 5002, 5003], diversity_varied: [5000, 5001, 5002, 5003] };

test('replay retains all 17 champions, study-specific cases and both cues, including partial searches', () => {
  assert.equal(champions.length, 17);
  assert.equal(data.cases.length, 13);
  for (const [cohort, seeds] of Object.entries(seedSets)) {
    assert.deepEqual(champions.filter(p => p.cohort === cohort).map(p => p.seed), seeds);
  }
  let trials = 0;
  for (const policy of champions) {
    const cases = casesForPolicy(data, policy);
    assert.equal(cases.length, policy.study === 'diversity' ? 6 : 7);
    assert.ok(cases.every(c => c.study === policy.study));
    assert.deepEqual(policy.cases.map(c => c.name), cases.map(c => c.name));
    for (const c of policy.cases) {
      assert.deepEqual(c.trials.map(t => t.cue).sort(), [-1, 1]);
      for (const t of c.trials) {
        trials++;
        assert.equal(t.poses.length, t.movement_ticks + 1);
        assert.ok(t.poses.flat().every(Number.isFinite));
        assert.ok(['left', 'right', 'timeout'].includes(t.choice));
        assert.equal(t.reward, Number(t.choice === t.target));
        const arena = cases.find(x => x.name === c.name).arena;
        const b = plotBounds(arena, t);
        assert.ok(b.width > 0 && b.height > 0);
        assert.ok(t.poses.every(([x, y]) => x > b.minX && x < b.minX + b.width && y > b.minY && y < b.minY + b.height));
      }
    }
  }
  assert.equal(trials, 222);
  const partial = champions.find(p => p.cohort === 'terminal_progress' && p.seed === 2101);
  assert.equal(caseScore(partial, 'training_reference'), 1);
  assert.equal(findTrial(partial, 'training_reference', 1).choice, 'timeout');
});

test('scrubbing clamps to recorded poses and contact at the horizon stays a success', () => {
  const policy = champions.find(p => p.cohort === 'terminal_progress' && p.seed === 2100);
  const trial = findTrial(policy, 'training_reference', '-1');
  assert.equal(poseAt(trial, -3).pose, trial.poses[0]);
  assert.equal(poseAt(trial, 100).pose, trial.poses.at(-1));
  assert.equal(poseAt(trial, 2.9).index, 2);
  assert.equal(poseAt(trial, NaN).index, 0);
  assert.equal(trial.movement_ticks, 20);
  assert.ok(trial.contact_fraction > 0 && trial.contact_fraction < 1);
  assert.match(outcomeText(trial, 20), /Correct right goal.*tick 20 of 20/);
  assert.equal(findTrial(policy, 'missing', 1), null);
  assert.equal(data.policies.filter(p => p.cohort === 'controls').length, 3);
});

const sourcesPresent = data.sources.every(source => existsSync(new URL(source.path, root)));
test('export matches exact saved source identities, geometry, results and every pose', { skip: !sourcesPresent }, () => {
  for (const source of data.sources) {
    const raw = readFileSync(new URL(source.path, root));
    assert.equal(createHash('sha256').update(raw).digest('hex'), source.sha256);
    const saved = JSON.parse(raw);
    if (source.role !== 'evaluations') continue;
    const exported = champions.filter(p => p.study === source.study &&
      (source.cohorts ?? [source.cohort]).includes(p.cohort));
    const manifestSource = data.sources.find(s => s.study === source.study && s.role === 'manifest');
    const manifest = manifestSource ? JSON.parse(readFileSync(new URL(manifestSource.path, root))) : null;
    for (const policy of exported) {
      const original = source.study === 'diversity'
        ? saved.runs.find(p => p.seed === policy.seed && `diversity_${p.arm}` === policy.cohort)
        : saved.policies.find(p => p.seed === policy.seed);
      assert.equal(policy.champion, source.study === 'diversity' ? original.champion_id : original.training.champion_id);
      for (const c of policy.cases) {
        const sourceCase = (original.holdout_cases ?? original.cases).find(x => x.case === c.name);
        for (const t of c.trials) {
          const s = sourceCase.evaluation.intact.trials.find(x => x.cue === t.cue);
          for (const key of ['choice', 'target', 'reward', 'movement_ticks']) assert.equal(t[key], s[key]);
          const poses = [s.initial_pose, ...s.trajectory.map(step => step.pose_after)].map(p => [p.x, p.y, p.heading]);
          assert.deepEqual(t.poses, poses);
          assert.equal(t.contact_fraction, s.trajectory.at(-1).contact_fraction);
        }
      }
    }
    for (const c of data.cases.filter(c => c.study === source.study)) {
      const original = manifest
        ? manifest.holdout_cases.find(x => x.case === c.name).arena
        : saved.resolved_cases.find(x => x.case.name === c.name).arena;
      for (const [key, value] of Object.entries(c.arena)) assert.equal(value, original[key]);
    }
  }
});

test('exporter reproduces the committed small static payload without evaluating policies', { skip: !sourcesPresent }, () => {
  const script = fileURLToPath(new URL('analysis/export_visible_task_demo.py', root));
  const output = execFileSync('python3', ['-c', 'import json,runpy,sys; m=runpy.run_path(sys.argv[1]); print(json.dumps(m["export"](),separators=(",",":"),allow_nan=False))', script]);
  assert.equal(output.toString(), readFileSync(new URL('../data/visible-task.json', import.meta.url), 'utf8'));
  assert.ok(output.length < 400_000);
});


test('study selectors keep separate evaluators, plans, held-outs and locked champions', () => {
  assert.equal(data.studies.length, 2);
  const newer = data.studies.find(s => s.id === 'diversity');
  assert.match(newer.evaluator, /^c7324b6/);
  assert.match(newer.report, /VISIBLE_DIVERSITY_PILOT_2026-11-22.md$/);
  assert.match(newer.plan, /VISIBLE_DIVERSITY_PLAN_2026-11-18.md$/);
  assert.ok(data.sources.some(s => s.study === 'diversity' && s.role === 'selection_lock'));
  for (const policy of data.policies) {
    const cases = casesForPolicy(data, policy);
    for (const c of cases) for (const cue of [-1, 1]) assert.ok(findTrial(policy, c.name, cue));
    if (policy.study === 'diversity') assert.ok(cases.every(c => c.name !== 'training_reference'));
  }
  const newerPolicies = champions.filter(p => p.study === 'diversity');
  for (const [cohort, expected] of [['diversity_fixed', 12], ['diversity_varied', 14]]) {
    const correct = newerPolicies.filter(p => p.cohort === cohort).reduce((sum, p) => sum +
      p.cases.filter(c => c.name.startsWith('geometry_')).reduce((n, c) => n + caseScore(p, c.name), 0), 0);
    assert.equal(correct, expected);
  }
});
