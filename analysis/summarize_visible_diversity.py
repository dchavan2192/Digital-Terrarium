#!/usr/bin/env python3
"""Read-only audit of the frozen visible diversity pilot; never select by test score."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import statistics
import tomllib

from audit_visible_validation_physics import audit_trial

SOURCE = 'c7324b6ba6d2c6a57523c83cad2575d7a8ee75be'
BINARY_SHA = '682bb415d26dd5009220e6b6a775d9a153e30b97d8c97055cf3c023c5e6df02b'
CORE_BUILD = '76b23bc91f012cd7'
COMPILER = 'rustc 1.99.1 (58cf8f9c3 2026-10-13)'
EXTERNAL_RECORD = Path(__file__).resolve().parents[1] / 'docs/results/visible_diversity_launch_2026-11-22.json'
PROTOCOL_SHA = 'a009ba4ad6467850e1270504474766262a4f68dbec59d341251efc5dcd896c67'
ARENA_SHA = '7a47f36167419d92c0ab7414d11b4c399526dc90196fd64d7d938e1c6372f213'
PINNED_FILES = {'visible_diversity.toml': PROTOCOL_SHA, 'delayed_choice.toml': ARENA_SHA,
                'pi_thermal_guard.py': 'e25611f799fac2537df86738e99cd29ad801a87bf4f4408df0c6d20e56a6176c',
                'run_guarded_acquisition.py': '2f08b9c443b917026203d17cc3fc75d6a5f6b0b229e139c085e9c65a1a58c7ec'}
SEEDS = [5000, 5001, 5002, 5003]
ORDER = [(seed, arm) for seed in SEEDS for arm in
         (['fixed', 'varied'] if seed % 2 == 0 else ['varied', 'fixed'])]
HOLDOUTS = ['geometry_forward_shift', 'geometry_wide_shift', 'geometry_near_narrow',
            'geometry_far_diagonal', 'heading_minus30', 'heading_plus30']
TRAINING = {'fixed': ['reference_copy_1', 'reference_copy_2', 'reference_copy_3'],
            'varied': ['reference', 'near', 'narrow']}
LOCK_FILES = {'founder_genomes.json', 'founders.json', 'generations.jsonl',
              'final_population.json', 'innovations.json', 'selection.json', 'training_summary.json'}


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def finite(value):
    if isinstance(value, float):
        require(math.isfinite(value), 'nonfinite archived JSON')
    elif isinstance(value, dict):
        for child in value.values():
            finite(child)
    elif isinstance(value, list):
        for child in value:
            finite(child)


def close(a, b, reason):
    require(math.isclose(a, b, rel_tol=1e-11, abs_tol=1e-11), reason)


def fnv(data):
    result = 0xcbf29ce484222325
    for byte in data:
        result = ((result ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{result:016x}'


class Reader:
    def __init__(self, batch):
        self.batch = batch.resolve()
        self.hashes = {}

    def raw(self, path):
        path = Path(path)
        require(path.is_file() and not path.is_symlink() and path.resolve().is_relative_to(self.batch),
                'audit input is not a contained regular file')
        require(path.stat().st_size <= 256 * 1024**2, 'audit file exceeds256MiB')
        data = path.read_bytes()
        key = str(path.relative_to(self.batch))
        digest = hashlib.sha256(data).hexdigest()
        require(key not in self.hashes or self.hashes[key] == digest, 'input changed during audit')
        self.hashes[key] = digest
        return data

    def json(self, path):
        value = json.loads(self.raw(path))
        finite(value)
        return value

    def identity(self, session, identity):
        relative = Path(identity['path'])
        require(not relative.is_absolute() and '..' not in relative.parts, 'unsafe identity path')
        data = self.raw(session / relative)
        require(identity['bytes'] == len(data) and identity['fnv1a64_noncryptographic'] == fnv(data),
                'locked artifact identity mismatch')


def rank_key(candidate):
    return (-candidate['correct'], -candidate['terminal_progress'], candidate['id'])


def fixed_genotype(genome):
    result = copy.deepcopy(genome)
    result.pop('color_hue')  # Native neutral hue drift is explicitly retained.
    for edge in result['connections']:
        edge.pop('weight')
    return result


def audit_scores(scores, arm, size=64):
    require(len(scores) == size and len({c['id'] for c in scores}) == size, 'score count/ID mismatch')
    for candidate in scores:
        require(type(candidate['id']) is int and candidate['id'] >= 0, 'invalid candidate ID')
        cases = candidate['case_scores']
        require([case['case'] for case in cases] == TRAINING[arm], 'training slot order differs')
        for case in cases:
            require(type(case['correct']) is int and 0 <= case['correct'] <= 2, 'case correct outside0..2')
            require(math.isfinite(case['terminal_progress']) and 0 <= case['terminal_progress'] <= 1,
                    'invalid case progress')
        require(candidate['correct'] == sum(c['correct'] for c in cases), 'incorrect total correct')
        close(candidate['terminal_progress'], sum(c['terminal_progress'] for c in cases) / 3,
              'incorrect aggregate progress')
        if arm == 'fixed':
            require(all((c['correct'], c['terminal_progress']) ==
                        (cases[0]['correct'], cases[0]['terminal_progress']) for c in cases),
                    'fixed repeated reference slots differ')
    require(scores == sorted(scores, key=rank_key), 'incorrect correct/progress/ID ranking')


def audit_generations(generations, arm, population=64, replacements=25, elites=8):
    require(len(generations) == replacements + 1, 'generation coverage mismatch')
    parents = {}
    previous = None
    for index, generation in enumerate(generations):
        require(generation['generation'] == index, 'generation order mismatch')
        scores = generation['scores']
        audit_scores(scores, arm, population)
        require(generation['champion_id'] == scores[0]['id'] and
                generation['best_correct'] == scores[0]['correct'] and
                generation['champion_terminal_progress'] == scores[0]['terminal_progress'] and
                generation['all_six_correct'] == sum(c['correct'] == 6 for c in scores),
                'generation champion metadata mismatch')
        if previous is None:
            require({c['id'] for c in scores} == set(range(population)), 'founder IDs differ')
            require(all(c['parent_id'] is None and c['born_generation'] == 0 for c in scores), 'founder lineage')
        else:
            old = {c['id']: c for c in previous}
            now = {c['id']: c for c in scores}
            retained = {c['id'] for c in previous[:elites]}
            require(set(old) & set(now) == retained and all(now[i] == old[i] for i in retained),
                    'elite identity/score changed or unselected survivor')
            children = [c for c in scores if c['id'] not in old]
            first = population + (index - 1) * (population - elites)
            require({c['id'] for c in children} == set(range(first, first + population - elites)),
                    'new persistent IDs differ')
            require(all(c['parent_id'] in old and c['born_generation'] == index for c in children),
                    'offspring parent/birth generation mismatch')
        for candidate in scores:
            parent = candidate['parent_id']
            require(candidate['id'] not in parents or parents[candidate['id']] == parent, 'lineage changed')
            parents[candidate['id']] = parent
        previous = scores
    return parents


def evaluation(case, arena):
    require(case['status'] == 'completed', 'technical case failure retained: cannot publish final summary')
    result = case['evaluation']
    require(result['cue_visible'] is True, 'unexpected hidden cue')
    require(result['intact']['trials'] == result['same_network_state_cleared']['trials'], 'FF reset inequality')
    for name in ('intact', 'same_network_state_cleared'):
        condition = result[name]
        trials = condition['trials']
        require([t['cue'] for t in trials] == [-1, 1], 'cue coverage mismatch')
        for trial in trials:
            audit_trial(trial, arena)
            side = trial['target']
            start = trial['initial_pose']
            end = trial['trajectory'][-1]['pose_after']
            gx, gy = arena[f'goal_{side}_x'], arena[f'goal_{side}_y']
            denominator = math.hypot(start['x'] - gx, start['y'] - gy) - arena['goal_radius']
            require(denominator > 0, 'invalid progress denominator')
            remaining = max(0, math.hypot(end['x'] - gx, end['y'] - gy) - arena['goal_radius'])
            expected = 1 if trial['reward'] == 1 else max(0, min(1, (denominator - remaining) / denominator))
            close(trial['terminal_progress'], expected, 'terminal progress does not match final pose')
        close(condition['mean_reward'], sum(t['reward'] for t in trials) / 2, 'mean reward mismatch')
        close(condition['mean_terminal_progress'], sum(t['terminal_progress'] for t in trials) / 2,
              'mean terminal progress mismatch')
    return result['intact']['trials']


def endpoints(cases):
    require([c['case'] for c in cases] == HOLDOUTS, 'held-out coverage/order differs')
    return dict(geometry_correct=sum(t['reward'] for c in cases[:4] for t in c['evaluation']['intact']['trials']),
                geometry_denominator=8,
                heading_correct=sum(t['reward'] for c in cases[4:] for t in c['evaluation']['intact']['trials']),
                heading_denominator=4,
                both_correct_cases=sum(all(t['reward'] == 1 for t in c['evaluation']['intact']['trials']) for c in cases))


def resolved_arena(base, case):
    expected = copy.deepcopy(base)
    if case['kind'] == 'relative_goals':
        forward, lateral = case['forward'], case['lateral']
        sine, cosine = math.sin(base['start_heading']), math.cos(base['start_heading'])
        for side, sign in [('left', 1), ('right', -1)]:
            expected[f'goal_{side}_x'] = base['start_x'] + forward * cosine - sign * lateral * sine
            expected[f'goal_{side}_y'] = base['start_y'] + forward * sine + sign * lateral * cosine
    else:
        require(case['kind'] == 'rigid_rotation', 'unknown held-out family')
        sine, cosine = math.sin(case['radians']), math.cos(case['radians'])
        for side in ('left', 'right'):
            x, y = base[f'goal_{side}_x'] - base['start_x'], base[f'goal_{side}_y'] - base['start_y']
            expected[f'goal_{side}_x'] = base['start_x'] + x * cosine - y * sine
            expected[f'goal_{side}_y'] = base['start_y'] + x * sine + y * cosine
        expected['start_heading'] = (base['start_heading'] + case['radians']) % (2 * math.pi)
    return expected


def check_resolved(actual, expected):
    require(set(actual) == set(expected), 'resolved arena fields differ')
    for key in expected:
        close(actual[key], expected[key], f'resolved arena differs: {key}')


def summarize(batch):
    batch = Path(batch).resolve()
    reader = Reader(batch)
    require(reader.raw(batch / 'status').decode().strip() == 'completed' and
            reader.raw(batch / 'exit_code').decode().strip() == '0', 'guarded batch not completed')
    state = reader.json(batch / 'supervisor.json')
    require(state['state'] == 'completed' and all(state[k] == 0 for k in
            ('runner_exit_code', 'guard_exit_code', 'supervisor_exit_code')) and not state['runner_left_paused'],
            'unclean supervisor completion')
    guard = reader.json(batch / 'thermal_guard/status.json')
    require(guard['state'] == 'finished' and guard['running_matches'] == 0 and guard['paused'] == [], 'unclean guard')
    operations = reader.json(batch / 'supervisor_manifest.json')
    launch = reader.json(batch / 'launch.json')
    require(EXTERNAL_RECORD.is_file() and EXTERNAL_RECORD.stat().st_size <= 65536,
            'missing/bounded external prelaunch record required')
    external_bytes = EXTERNAL_RECORD.read_bytes()
    external = json.loads(external_bytes)
    finite(external)
    require(launch == external, 'batch launch record differs from committed prospective record')
    reader.hashes['external:docs/results/visible_diversity_launch_2026-11-22.json'] = hashlib.sha256(external_bytes).hexdigest()
    require(launch['schema_version'] == 1 and launch['source_revision'] == SOURCE and
            launch['source_dirty'] is False and launch['status'] == 'prepared_not_launched',
            'prelaunch source/provenance mismatch')
    require(set(launch['artifacts_sha256']) == set(PINNED_FILES) | {'sim-runner'},
            'prelaunch pinned artifact coverage differs')
    require(launch['artifacts_sha256']['sim-runner'] == BINARY_SHA and
            launch['build_metadata']['TERRARIUM_CORE_BUILD'] == CORE_BUILD,
            'frozen binary/core pin differs')
    for name, digest in launch['artifacts_sha256'].items():
        require(hashlib.sha256(reader.raw(batch / name)).hexdigest() == digest, 'prelaunch artifact changed')
        if name in PINNED_FILES:
            require(digest == PINNED_FILES[name], 'source-pinned script/config differs')
    require(launch['resources']['free_disk_bytes'] >= 20 * 1024**3 and
            launch['resources']['available_ram_bytes'] >= 3 * 1024**3, 'prelaunch resource check failed')
    require(operations['sha256'] == {name: digest for name, digest in launch['artifacts_sha256'].items()
                                    if name != 'run_guarded_acquisition.py'}, 'operation/prelaunch pins differ')
    require(operations['task'] == 'acquire-visible-diversity' and operations['wall_seconds'] == 1800 and
            operations['environment_override'] == {'RAYON_NUM_THREADS': '1'} and
            operations['temperature_high_c'] == 75 and operations['temperature_low_c'] == 65 and
            operations['guard_poll_seconds'] == 5, 'operational policy differs')
    require(operations['sha256']['visible_diversity.toml'] == PROTOCOL_SHA and
            operations['sha256']['delayed_choice.toml'] == ARENA_SHA, 'frozen config pin differs')
    for name, digest in operations['sha256'].items():
        data = reader.raw(batch / name)
        require(hashlib.sha256(data).hexdigest() == digest, 'pinned task input changed')
    protocol_data = reader.raw(batch / 'visible_diversity.toml')
    protocol = tomllib.loads(protocol_data.decode())
    base = tomllib.loads(reader.raw(batch / 'delayed_choice.toml').decode())
    sessions = list((batch / 'runs').glob('session_*'))
    require(len(sessions) == 1 and sessions[0].is_dir(), 'native session count differs')
    session = sessions[0]
    manifest = reader.json(session / 'manifest.json')
    require(manifest['source_revision'] == SOURCE and manifest['source_dirty'] == 'false' and
            manifest['target'] == 'aarch64-unknown-linux-gnu' and manifest['build_profile'] == 'release' and
            manifest['core_build'] == CORE_BUILD and manifest['compiler'] == COMPILER,
            'native source/build not pinned clean Pi release')
    require(manifest['protocol'] == protocol and manifest['arena'] == base and
            manifest['protocol_fnv1a64_noncryptographic'] == fnv(protocol_data) and
            manifest['isolated_diet_mutation_probability'] == 0, 'native config/weight-only mismatch')
    require(manifest['schedule'] == [{'seed': seed, 'arms': [arm for s, arm in ORDER if s == seed]}
                                     for seed in SEEDS], 'balanced schedule differs')
    require(manifest['budget']['training_candidate_evaluations'] == 13312 and
            manifest['budget']['scheduled_training_trials'] == 79872 and
            manifest['budget']['training_trials_per_candidate'] == 6, 'training budget differs')
    require(manifest['budget']['diagnostic_trials'] == 294 and
            manifest['budget']['max_neural_steps'] == 2084316 and
            manifest['budget']['max_edge_steps'] == 2084316 * 231 and
            manifest['budget']['retained_trace_samples_upper_bound'] == 7938,
            'complete diagnostic/training work budget differs')
    for arm in ('fixed', 'varied'):
        resolved = manifest['training_cases'][arm]
        require([c['case'] for c in resolved] == TRAINING[arm], 'resolved training names differ')
        expected = [base] * 3 if arm == 'fixed' else [base,
            resolved_arena(base, dict(kind='relative_goals', forward=3, lateral=3)),
            resolved_arena(base, dict(kind='relative_goals', forward=4, lateral=2.5))]
        for case, arena in zip(resolved, expected):
            check_resolved(case['arena'], arena)
    require(len(manifest['holdout_cases']) == 6, 'resolved held-out count differs')
    for resolved, case in zip(manifest['holdout_cases'], protocol['holdouts']):
        require(resolved['case'] == case['name'], 'frozen held-out name differs')
        check_resolved(resolved['arena'], resolved_arena(base, case))
        require(all(g['initial_ray_hits'] > 0 and g['witness_within_horizon'] is True
                    for g in resolved['geometry_preflight']['goals']), 'held-out geometry preflight failed')
    controls = reader.json(session / 'controls.json')
    require(controls['instrument_validated'] is True and controls['cue_visible'] is True and
            all(v is True for v in controls['checks'].values()) and len(controls['controls']) == 3,
            'manufactured instrument check failed')
    for condition in controls['controls']:
        require(condition['mean_reward'] == 1, 'visible manufactured control failed')
        require([t['cue'] for t in condition['trials']] == [-1, 1], 'manufactured cue coverage')
        for trial in condition['trials']:
            audit_trial(trial, base)
            require(trial['reward'] == 1, 'manufactured terminal response failed')
    root = reader.json(session / 'summary.json')
    lock = reader.json(session / 'selection_lock.json')
    require(root['status'] == 'completed' and root['technical_failures'] == 0, 'technical search failure')
    require(lock['status'] == 'locked' and lock['expected_searches'] == 8 and
            lock['selected_champions'] == 8 and lock['failed_searches'] == 0, 'incomplete selection lock')
    require([(r['seed'], r['arm']) for r in root['runs']] == ORDER and
            [(r['seed'], r['arm']) for r in lock['runs']] == ORDER, 'eight-search coverage/order mismatch')
    reader.identity(session, root['selection_lock'])
    rows, founder_inputs = [], {}
    for locked, result in zip(lock['runs'], root['runs']):
        seed, arm = result['seed'], result['arm']
        directory = session / arm / f'seed_{seed}'
        require(result == reader.json(directory / 'summary.json') and result['status'] == 'completed' and
                result['technical_case_failures'] == 0, 'root/per-search result differs or technical failure')
        require(result['selection_lock'] == root['selection_lock'], 'different selection lock')
        expected_paths = {f'{arm}/seed_{seed}/{name}' for name in LOCK_FILES}
        require(len(locked['artifacts']) == 7 and {i['path'] for i in locked['artifacts']} == expected_paths,
                'training artifact lock coverage differs')
        for identity in locked['artifacts']:
            reader.identity(session, identity)
        raw_founders = reader.json(directory / 'founder_genomes.json')
        require([c['id'] for c in raw_founders] == list(range(64)), 'raw founder order differs')
        if seed in founder_inputs:
            require(raw_founders == founder_inputs[seed], 'paired founder genomes differ')
        founder_inputs[seed] = raw_founders
        founders = reader.json(directory / 'founders.json')
        final = reader.json(directory / 'final_population.json')
        reader.json(directory / 'innovations.json')
        lines = reader.raw(directory / 'generations.jsonl').decode().splitlines()
        generations = [json.loads(line) for line in lines]
        finite(generations)
        parents = audit_generations(generations, arm)
        for population, scores in ((founders, generations[0]['scores']), (final, generations[-1]['scores'])):
            audit_scores(population, arm)
            require([{k: v for k, v in c.items() if k != 'genome'} for c in population] == scores,
                    'population and generation scores differ')
        original = {c['id']: c['genome'] for c in raw_founders}
        require({c['id']: c['genome'] for c in founders} == original, 'scored founders changed genotype')
        for genome in original.values():
            require(genome['num_inputs'] == 20 and genome['num_outputs'] == 7 and
                    len(genome['hidden_nodes']) == 8 and len(genome['connections']) == 231,
                    'founder controller dimensions/topology differ')
        for candidate in final:
            ancestor = candidate['id']
            while parents[ancestor] is not None:
                ancestor = parents[ancestor]
            genome = candidate['genome']
            require(fixed_genotype(genome) == fixed_genotype(original[ancestor]),
                    'nonweight genotype change beyond allowed neutral hue drift')
            require(genome['diet_preference'] == original[ancestor]['diet_preference'], 'diet mutated in weight-only search')
            require(all(not e['recurrent'] and e['methylation'] == 0 for e in genome['connections']), 'non-FF/plastic final genotype')
        champion = final[0]
        selection = reader.json(directory / 'selection.json')
        training_summary = reader.json(directory / 'training_summary.json')
        require(training_summary.get('holdout_cases') is None and training_summary['status'] == 'training_completed',
                'training archive contains test results')
        for item in (selection, locked):
            require(item['champion_id'] == champion['id'] and item['correct'] == champion['correct'] and
                    item['terminal_progress'] == champion['terminal_progress'], 'selection not final training rank')
        require(result['champion_id'] == champion['id'] and result['training_correct'] == champion['correct'] and
                result['training_terminal_progress'] == champion['terminal_progress'] and
                result['evaluations'] == 1664 and result['scheduled_training_trials'] == 9984,
                'training summary/budget mismatch')
        training = result['training_cases']
        require([c['case'] for c in training] == TRAINING[arm] and training == training_summary['training_cases'],
                'training replay coverage differs')
        for case, resolved, score in zip(training, manifest['training_cases'][arm], champion['case_scores']):
            trials = evaluation(case, resolved['arena'])
            require(sum(t['reward'] for t in trials) == score['correct'], 'training correct replay mismatch')
            close(sum(t['terminal_progress'] for t in trials) / 2, score['terminal_progress'], 'training progress replay mismatch')
        heldout = result['holdout_cases']
        require([c['case'] for c in manifest['holdout_cases']] == HOLDOUTS, 'resolved held-out order mismatch')
        for case, resolved in zip(heldout, manifest['holdout_cases']):
            require(case['case'] == resolved['case'], 'test case identity mismatch')
            evaluation(case, resolved['arena'])
        rows.append(dict(seed=seed, arm=arm, champion_id=champion['id'],
                         founder_correct=founders[0]['correct'], training_correct=champion['correct'],
                         training_progress=champion['terminal_progress'], **endpoints(heldout),
                         first_generation_all_six_correct=next((g['generation'] for g in generations if g['all_six_correct']), None),
                         training_curve=[{k: g[k] for k in ('generation', 'champion_id', 'best_correct',
                                                           'champion_terminal_progress', 'all_six_correct')} for g in generations],
                         cases=[dict(case=c['case'], trials=[{k: t[k] for k in
                             ('cue', 'target', 'choice', 'reward', 'movement_ticks', 'terminal_progress')}
                             for t in c['evaluation']['intact']['trials']]) for c in heldout]))
    pairs = []
    for seed in SEEDS:
        off, on = [next(r for r in rows if r['seed'] == seed and r['arm'] == arm) for arm in ('fixed', 'varied')]
        pairs.append(dict(seed=seed, geometry_difference=(on['geometry_correct'] - off['geometry_correct']) / 8,
                          heading_difference=(on['heading_correct'] - off['heading_correct']) / 4))
    paired = {key: dict(mean=statistics.mean(p[key] for p in pairs),
                       range=[min(p[key] for p in pairs), max(p[key] for p in pairs)])
              for key in ('geometry_difference', 'heading_difference')}
    groups = [dict(arm=arm, available_searches=4, planned_searches=4,
                   geometry_mean_fraction=statistics.mean(r['geometry_correct'] / 8 for r in rows if r['arm'] == arm),
                   heading_mean_fraction=statistics.mean(r['heading_correct'] / 4 for r in rows if r['arm'] == arm))
              for arm in ('fixed', 'varied')]
    reader.raw(batch / 'guard.log')
    telemetry = [json.loads(line) for line in reader.raw(batch / 'thermal_supervisor.jsonl').decode().splitlines()]
    finite(telemetry)
    temperatures = [t['temperature_c'] for t in telemetry if t.get('temperature_c') is not None]
    require(all(0 <= t <= 150 for t in temperatures), 'invalid sampled temperature')
    return dict(schema_version=1, complete=True, source=SOURCE, core_build=manifest['core_build'],
                available_searches=8, planned_searches=8, available_pairs=4, planned_pairs=4,
                rows=rows, pairs=pairs, paired_summary=paired, by_arm=groups, artifact_sha256=reader.hashes,
                telemetry=dict(samples=len(telemetry), maximum_sampled_temperature_c=max(temperatures) if temperatures else None,
                               snapshots=telemetry),
                interpretation='Four paired searches; original training-selected champions retained. '
                'Geometry and heading endpoints separate. No memory, ecology or confirmatory inference.',
                thermal_note='Clean guard lifecycle only; sampled telemetry cannot establish continuous peak temperature or absence of throttling.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('batch', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        result = summarize(args.batch)
        with args.output.open('x') as stream:
            json.dump(result, stream, indent=2, allow_nan=False)
            stream.write('\n')
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f'audit failed; preserve technical/partial artifacts: {error}\n')
