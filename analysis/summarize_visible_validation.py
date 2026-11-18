#!/usr/bin/env python3
"""Strict all-cohort audit and descriptive summary; never select by test score."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path

CASES = ['training_reference', 'rotation_minus_45', 'rotation_plus_45',
         'near', 'far_forward', 'narrow', 'wide']
COHORTS = ['original', 'terminal_only', 'terminal_progress']


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def read(path):
    return json.loads(path.read_text(), parse_constant=lambda value: (_ for _ in ()).throw(
        ValueError(f'nonfinite JSON: {value}')))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def strip_progress(evaluation):
    result = deepcopy(evaluation)
    for name in ('intact', 'same_network_state_cleared'):
        result[name].pop('mean_terminal_progress', None)
        for trial in result[name]['trials']:
            trial.pop('terminal_progress', None)
    return result


def trial_counts(condition, horizon):
    trials = condition['trials']
    require(len(trials) == 2 and [t['cue'] for t in trials] == [-1.0, 1.0],
            'missing/duplicated/unbalanced cues')
    counts = dict(correct=0, wrong=0, timeout=0)
    for trial in trials:
        require(type(trial['movement_ticks']) is int and 1 <= trial['movement_ticks'] <= horizon,
                'invalid movement tick')
        require(trial['target'] == ('right' if trial['cue'] == -1 else 'left'), 'cue target mismatch')
        choice = trial['choice']
        if choice == 'timeout':
            require(trial['reward'] == 0 and trial['movement_ticks'] == horizon, 'invalid timeout')
            counts['timeout'] += 1
        else:
            require(choice in ('left', 'right'), 'invalid choice')
            correct = choice == trial['target']
            require(trial['reward'] == int(correct), 'choice/reward mismatch')
            counts['correct' if correct else 'wrong'] += 1
    require(condition['mean_reward'] == counts['correct'] / 2, 'mean reward mismatch')
    return counts


def audit(batch):
    require((batch / 'status').read_text().strip() == 'completed', 'campaign incomplete')
    campaign = read(batch / 'campaign.json')
    require(campaign['execution_order'] == COHORTS, 'cohort order/coverage mismatch')
    require([c['cohort'] for c in campaign['cohorts']] == COHORTS, 'source cohort coverage')
    rows, controls, operational, result_hashes = [], [], [], {}
    for cohort in campaign['cohorts']:
        label = cohort['cohort']
        child = batch / label
        require((child / 'status').read_text().strip() == 'completed', 'child incomplete')
        state = read(child / 'supervisor.json')
        require(state['state'] == 'completed' and state['runner_exit_code'] == 0
                and state['guard_exit_code'] == 0 and state['supervisor_exit_code'] == 0,
                'supervision failed')
        manifest = read(child / 'supervisor_manifest.json')
        require(manifest['task'] == 'validate-visible' and manifest['wall_seconds'] == 1800,
                'supervisor task/watchdog mismatch')
        for name, digest in manifest['sha256'].items():
            require(sha(child / name) == digest, f'pinned input changed: {label}/{name}')
        require(manifest['sha256']['sim-runner'] == campaign['binary_sha256'], 'binary mismatch')
        for name, digest in cohort['source_sha256'].items():
            require(sha(child / 'input' / name) == digest, 'source-copy identity changed')
        sessions = list((child / 'runs').glob('session_*/result.json'))
        require(len(sessions) == 1, 'result session count')
        result = read(sessions[0])
        result_hashes[str(sessions[0].relative_to(batch))] = sha(sessions[0])
        require(result['status'] == 'completed' and result['task'] == 'saved_policy_visible_validation',
                'result incomplete/wrong task')
        evaluator = result['evaluator']
        require(evaluator['source_revision'] == campaign['evaluator_revision']
                and evaluator['source_dirty'] == 'false', 'evaluator not pinned clean source')
        require(result['source']['technical_failures'] == 0, 'unexpected source failure')
        require([c['case']['name'] for c in result['resolved_cases']] == CASES, 'case coverage')
        require([c['case'] for c in result['case_controls']] == CASES, 'control coverage')
        require([p['seed'] for p in result['policies']] == cohort['seeds'], 'policy seed coverage')
        for entry in result['source']['input_files']:
            # The copied campaign may now be on a different machine.
            relative = entry['path'].split('/input/', 1)[1]
            data = (child / 'input' / relative).read_bytes()
            fingerprint = 0xcbf29ce484222325
            for byte in data:
                fingerprint = ((fingerprint ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
            require(entry['bytes'] == len(data) and entry['fnv1a64_noncryptographic'] == f'{fingerprint:016x}',
                    'loader input identity mismatch')
        for case in result['resolved_cases']:
            require(all(g['witness_within_horizon'] and g['initial_ray_hits'] > 0
                        for g in case['geometry_preflight']['goals']), 'geometry preflight mismatch')
        horizon = result['resolved_cases'][0]['arena']['horizon']
        for policy in result['policies']:
            training = read(child / 'input' / f"seed_{policy['seed']}" / 'summary.json')
            require(policy['status'] == 'evaluated' and policy['training'] == training, 'policy source mismatch')
            require([c['case'] for c in policy['cases']] == CASES, 'policy case coverage')
            require(policy['cases'][0]['matches_training_replay'] is True and
                    policy['cases'][0]['evaluation'] == strip_progress(training['champion_training_trials']),
                    'baseline replay mismatch')
            for case in policy['cases']:
                evaluation = case['evaluation']
                require(evaluation['cue_visible'] is True, 'cue visibility mismatch')
                require(evaluation['intact']['trials'] == evaluation['same_network_state_cleared']['trials'],
                        'feed-forward reset mismatch')
                counts = trial_counts(evaluation['intact'], horizon)
                trial_counts(evaluation['same_network_state_cleared'], horizon)
                rows.append(dict(cohort=label, seed=policy['seed'], champion_id=training['champion_id'],
                                 case=case['case'], **counts, both_correct=counts['correct'] == 2,
                                 trials=[{k: t[k] for k in ('cue', 'choice', 'reward', 'movement_ticks')}
                                         for t in evaluation['intact']['trials']]))
        for case in result['case_controls']:
            require(case['cue_independent_paths_equal'] is True, 'cue independent path mismatch')
            for name in ('manufactured_feed_forward', 'manufactured_recurrent', 'cue_independent'):
                for condition in ('intact', 'same_network_state_cleared'):
                    controls.append(dict(cohort=label, case=case['case'], controller=name,
                                         condition=condition, **trial_counts(case[name][condition], horizon)))
        guard = read(child / 'thermal_guard/status.json')
        operational.append(dict(cohort=label, supervisor=state, final_guard=guard,
                                evaluator=evaluator))
    aggregate = []
    for cohort in COHORTS:
        for case in CASES:
            selected = [r for r in rows if r['cohort'] == cohort and r['case'] == case]
            require(len(selected) == 3, 'missing policy/case rows')
            aggregate.append(dict(cohort=cohort, case=case, policies=3,
                                  both_correct=sum(r['both_correct'] for r in selected),
                                  **{k: sum(r[k] for r in selected) for k in ('correct', 'wrong', 'timeout')}))
    return dict(schema_version=1, campaign=campaign, result_sha256=result_hashes,
                aggregate=aggregate, policies=rows, controls=controls, operational=operational,
                interpretation='Saved training-selected policies only; original cohort separate. No fresh-run, memory or ecology inference.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('batch', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        summary = audit(args.batch)
        with args.output.open('x') as output:
            json.dump(summary, output, indent=2, allow_nan=False)
            output.write('\n')
        print(json.dumps(summary['aggregate'], indent=2))
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f'validation audit refused: {error}\n')
