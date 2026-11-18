#!/usr/bin/env python3
"""Independently check archived visible-validation motion and first-contact scoring.

Read-only: no controller evaluation, simulation, selection, or artifact mutation.
The companion summary auditor supplies provenance/coverage checks; this script
reconstructs geometry directly from archived actions using Python's math library.
It does not independently reconstruct neural activations or visual sensors.
"""
import argparse
import json
import math
from pathlib import Path

from summarize_visible_validation import audit as audit_metadata, read, require, sha


# Frozen before this validation, recorded in ACQUISITION_TRACE_DIAGNOSIS_2026-11-17.md.
FROZEN = {
    'original': [
        (2000, '7904b7ae7bd8af5bb0fa4780cd100b82627c956237fdc1e6fe2191a13cdda373'),
        (2001, 'fb8ab37be7c69d74d9889bccbaad07202e2cc4303b8ebffd83f5e4e575d8404d'),
        (2002, 'ab7bd01ee89163211cd43ea4e0c3f078942ca33f5ef0f8d08e6eda7c0b79d68f')],
    'terminal_only': [
        (2100, 'e3728bd0f1eb372d9939d7033f598dfec900eaad25e1cb773c5138bab7e3ecb3'),
        (2101, '73b347b14b4c83fc8f69a1b5213075257cf2fff7a89fdc50472a2966559f83c9'),
        (2102, '1881904ff05be72d6ac83d25673f8b5c2b2c99a5b2fc91dbd6ffa2865064ef26')],
    'terminal_progress': [
        (2100, '9826a8be7b838722daf3b1c6288531e570c17c81d73aa87258f2a874f0b2f045'),
        (2101, '67f2da7b6a10449f10745ba21b1c39c1b2269d7d2e117d8c838f1300ba0f230f'),
        (2102, 'f8cbb20adabcbe1d59dc4e8447486c4612b21658282eed571d85d18685e5b2c0')],
}


def close(actual, expected, context):
    require(math.isfinite(actual) and math.isclose(actual, expected,
            rel_tol=1e-10, abs_tol=1e-10), f'{context}: {actual} != {expected}')


def pose_close(actual, expected, context):
    for key in ('x', 'y', 'heading'):
        close(actual[key], expected[key], f'{context}/{key}')


def finite_tree(value):
    if isinstance(value, float):
        require(math.isfinite(value), 'nonfinite archived value')
    elif isinstance(value, dict):
        for child in value.values():
            finite_tree(child)
    elif isinstance(value, list):
        for child in value:
            finite_tree(child)


def contact(start, end, center, radius):
    """Earliest segment/disk contact via an independent quadratic solve."""
    dx, dy = end['x'] - start['x'], end['y'] - start['y']
    ox, oy = start['x'] - center[0], start['y'] - center[1]
    c = ox * ox + oy * oy - radius * radius
    if c <= 0:
        return 0.0
    a = dx * dx + dy * dy
    if a == 0:
        return None
    b = 2 * (ox * dx + oy * dy)
    discriminant = b * b - 4 * a * c
    if discriminant < 0:
        return None
    roots = [(-b - math.sqrt(discriminant)) / (2 * a),
             (-b + math.sqrt(discriminant)) / (2 * a)]
    hits = [root for root in roots if 0 <= root <= 1]
    return min(hits) if hits else None


def audit_trial(trial, arena):
    finite_tree(trial)
    cue = trial['cue']
    require(cue in (-1, 1), 'invalid cue')
    target = 'right' if cue == -1 else 'left'
    require(trial['target'] == target, 'wrong hidden label')
    pose = dict(x=arena['start_x'], y=arena['start_y'], heading=arena['start_heading'])
    pose_close(trial['initial_pose'], pose, 'initial pose')
    channel = 2 * arena['ray_count'] + 2
    hold = trial['hold_trace']
    require(len(hold) == arena['cue_ticks'] + arena['blank_delay'], 'hold length')
    for tick, frame in enumerate(hold, 1):
        presenting = tick <= arena['cue_ticks']
        require(frame['tick'] == tick and frame['phase'] == ('cue' if presenting else 'blank'),
                'hold phase/tick')
        pose_close(frame['pose'], pose, 'frozen hold pose')
        require(frame['inputs'][channel] == (cue if presenting else 0), 'hold cue')
    trajectory = trial['trajectory']
    require(len(trajectory) == trial['movement_ticks'] and
            1 <= len(trajectory) <= arena['horizon'], 'trajectory length')
    expected_choice = 'timeout'
    for tick, frame in enumerate(trajectory, 1):
        require(frame['tick'] == tick, 'movement tick')
        require(frame['inputs'][channel] == cue, 'visible movement cue')
        pose_close(frame['pose_before'], pose, 'movement continuity')
        heading = (pose['heading'] + arena['max_turn'] *
                   max(-1, min(1, frame['actions']['turn']))) % (2 * math.pi)
        distance = arena['max_speed'] * max(0, min(1, frame['actions']['speed']))
        end = dict(x=pose['x'] + distance * math.cos(heading),
                   y=pose['y'] + distance * math.sin(heading), heading=heading)
        hits = []
        for name in ('left', 'right'):
            fraction = contact(pose, end, (arena[f'goal_{name}_x'], arena[f'goal_{name}_y']),
                               arena['goal_radius'])
            if fraction is not None:
                hits.append((fraction, name))
        if hits:
            fraction, expected_choice = min(hits, key=lambda hit: hit[0])
            require(tick == len(trajectory), 'trajectory continued after first contact')
            close(frame['contact_fraction'], fraction, 'contact fraction')
            end['x'] = pose['x'] + fraction * (end['x'] - pose['x'])
            end['y'] = pose['y'] + fraction * (end['y'] - pose['y'])
        else:
            require(frame['contact_fraction'] is None, 'spurious contact')
        pose_close(frame['pose_after'], end, 'integrated pose')
        pose = end
    require(trial['choice'] == expected_choice, 'first-contact choice')
    require(trial['reward'] == int(expected_choice == target), 'terminal reward')
    if expected_choice == 'timeout':
        require(len(trajectory) == arena['horizon'], 'premature timeout')
    return len(trajectory)


def audit(batch):
    metadata = audit_metadata(batch)
    counts = dict(frozen_summary_hashes=0, pinned_and_source_hashes=0,
                  policy_case_rows=0, reconstructed_trials=0, movement_frames=0,
                  equivalent_baseline_control_trials=0)
    for cohort, frozen in FROZEN.items():
        child = batch / cohort
        result = read(next((child / 'runs').glob('session_*/result.json')))
        require([p['seed'] for p in result['policies']] == [seed for seed, _ in frozen],
                'frozen policy order')
        for seed, digest in frozen:
            require(sha(child / 'input' / f'seed_{seed}' / 'summary.json') == digest,
                    'frozen pre-validation summary hash mismatch')
            counts['frozen_summary_hashes'] += 1
        source = next(c for c in metadata['campaign']['cohorts'] if c['cohort'] == cohort)
        counts['pinned_and_source_hashes'] += (len(source['source_sha256']) +
            len(read(child / 'supervisor_manifest.json')['sha256']))
        arenas = {case['case']['name']: case['arena'] for case in result['resolved_cases']}
        evaluations = []
        for policy in result['policies']:
            for case in policy['cases']:
                evaluations.append((case['evaluation'], arenas[case['case']]))
                counts['policy_case_rows'] += 1
        for case in result['case_controls']:
            for name in ('manufactured_feed_forward', 'manufactured_recurrent', 'cue_independent'):
                evaluation = case[name]
                expected = 0 if name == 'cue_independent' else 1
                for condition in ('intact', 'same_network_state_cleared'):
                    require(evaluation[condition]['mean_reward'] == expected, 'fixed control score')
                if name == 'cue_independent':
                    trials = evaluation['intact']['trials']
                    paths = [[{key: frame[key] for key in ('pose_before', 'pose_after', 'actions')}
                              for frame in trial['trajectory']] for trial in trials]
                    require(paths[0] == paths[1], 'cue-independent movement differs')
                evaluations.append((evaluation, arenas[case['case']]))
        for evaluation, arena in evaluations:
            require(evaluation['cue_visible'] is True, 'unexpected hidden mode')
            for condition in ('intact', 'same_network_state_cleared'):
                for trial in evaluation[condition]['trials']:
                    counts['movement_frames'] += audit_trial(trial, arena)
                    counts['reconstructed_trials'] += 1
        reference = result['case_controls'][0]
        counterparts = (reference['manufactured_recurrent']['intact'],
                        reference['manufactured_feed_forward']['intact'],
                        reference['manufactured_recurrent']['same_network_state_cleared'])
        for baseline, counterpart in zip(result['baseline_instrument_controls']['controls'], counterparts):
            # Baseline control names are additional labels; trial contents must match.
            require(baseline['trials'] == counterpart['trials'], 'baseline control trace mismatch')
            counts['equivalent_baseline_control_trials'] += len(baseline['trials'])
    counts['total_trials_covered'] = (counts['reconstructed_trials'] +
                                    counts['equivalent_baseline_control_trials'])
    return dict(schema_version=1, status='passed', batch=str(batch), checks=counts,
                result_sha256=metadata['result_sha256'], aggregate=metadata['aggregate'],
                limits='Archived actions and geometry only; no neural/sensor reconstruction, '
                       'new experiment, independent population replication, or continuous thermal proof.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('batch', type=Path)
    args = parser.parse_args()
    try:
        print(json.dumps(audit(args.batch), indent=2, allow_nan=False))
    except (ValueError, KeyError, TypeError, OSError, StopIteration) as error:
        parser.exit(1, f'audit failed: {error}\n')
