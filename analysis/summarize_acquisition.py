"""Audit the fixed three-seed visible acquisition pilot; no held-out evaluation.

python3 analysis/summarize_acquisition.py BATCH OUTPUT_JSON
Archives founder/final comparisons and checks all generations and training replays.
"""
import hashlib
import json
import math
from pathlib import Path
import statistics
import sys
import tomllib

SOURCE = '5504a20746bfe0dfcff54b95546324c02db1058d'
INPUTS = {'visible_acquisition.toml': '77e2a98d02c77a7eee05f6d571c9ad80f599e5bc5d0fac41d36a16a698407fce',
          'delayed_choice.toml': '7a47f36167419d92c0ab7414d11b4c399526dc90196fd64d7d938e1c6372f213'}


def require(value, message):
    if not value:
        raise ValueError(message)


def read(path):
    return json.loads(path.read_text())


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def check_genome(genome):
    require(genome['num_inputs'] == 20 and genome['num_outputs'] == 7
            and len(genome['hidden_nodes']) == 8 and len(genome['connections']) == 231, 'unexpected fixed topology')
    expected_nodes = [{'id': n, 'depth': 0.5} for n in range(28, 36)]
    expected_edges = sorted([(i, h) for i in range(21) for h in range(28, 36)]
                            + [(h, o) for h in range(28, 36) for o in range(21, 28)]
                            + [(0, o) for o in range(21, 28)])
    require(genome['hidden_nodes'] == expected_nodes
            and [(c['from'], c['to']) for c in genome['connections']] == expected_edges,
            'fixed hidden identities or connection keys differ')
    require(genome['mutation_rate'] == .1 and genome['mutation_step'] == .2, 'mutation genes changed')
    require(all(g['enabled'] and not g['recurrent'] and g['methylation'] == 0
                and math.isfinite(g['weight']) for g in genome['connections']), 'non-feed-forward or invalid expression')


def summarize(batch):
    batch = Path(batch)
    require((batch / 'status').read_text().strip() == 'completed'
            and (batch / 'exit_code').read_text().strip() == '0', 'batch not completed')
    require((batch / 'source_revision').read_text().strip() == SOURCE, 'unexpected execution source')
    for name, digest in INPUTS.items():
        require(sha(batch / name) == digest, f'{name}: frozen input changed')
    sessions = list((batch / 'runs').glob('session_*'))
    require(len(sessions) == 1, 'expected one session')
    session = sessions[0]
    manifest = read(session / 'manifest.json')
    require(manifest['source_revision'] == SOURCE and manifest['total_evaluations'] == 4992, 'manifest source/budget differs')
    require(manifest['source_dirty'] == 'false' and not (batch / 'source_status').read_text().strip(), 'pinned build source not clean')
    require(manifest['target'] == 'aarch64-unknown-linux-gnu' and manifest['build_profile'] == 'release', 'unexpected execution build')
    operational = read(batch / 'supervisor.json')
    require(operational['state'] == 'completed' and operational['supervisor_exit_code'] == 0
            and operational['runner_exit_code'] == 0, 'supervisor incomplete')
    operations = read(batch / 'supervisor_manifest.json')
    require(operations['wall_seconds'] == 1800 and operations['wall_includes_startup_and_cooling']
            and operations['environment_override'] == {'RAYON_NUM_THREADS': '1'}, 'operational limits differ')
    for name, digest in operations['sha256'].items():
        require(sha(batch / name) == digest, f'operational pinned artifact differs: {name}')
    require(set(operations['sha256']) == {'sim-runner', 'visible_acquisition.toml', 'delayed_choice.toml', 'pi_thermal_guard.py'}, 'missing operational artifact hashes')
    for key, name in [('protocol', 'visible_acquisition.toml'), ('arena', 'delayed_choice.toml')]:
        require(manifest[key] == tomllib.loads((batch / name).read_text()), 'resolved input differs')
    controls = read(session / 'controls.json')
    require([c['name'] for c in controls['controls']] == ['recurrent_reference', 'feed_forward_control', 'same_network_state_cleared'], 'control names/count differ')
    require(controls['checks'] and all(v is True for v in controls['checks'].values()), 'control checks failed')
    require(controls['instrument_validated'] and controls['cue_visible']
            and all(c['mean_reward'] == 1 for c in controls['controls']), 'visible controls failed')
    completed = read(session / 'summary.json')
    require(completed['status'] == 'completed' and completed['technical_failures'] == 0
            and [r['seed'] for r in completed['runs']] == [2000, 2001, 2002], 'incomplete declared seeds')
    runs = []
    for seed in [2000, 2001, 2002]:
        root = session / f'seed_{seed}'
        founders, final = [read(root / name) for name in ['founders.json', 'final_population.json']]
        generations = [json.loads(line) for line in (root / 'generations.jsonl').read_text().splitlines()]
        require([g['generation'] for g in generations] == list(range(26)), 'missing/duplicate generation')
        require(len(founders) == len(final) == 64, 'wrong population size')
        require({c['id'] for c in founders} == set(range(64)), 'founder IDs differ')
        for c in founders + final:
            check_genome(c['genome'])
            require(type(c['correct']) is int and c['correct'] in (0, 1, 2), 'invalid fitness')
        first_seen = {}
        previous = None
        for generation in generations:
            g, scores = generation['generation'], generation['scores']
            require(len(scores) == 64 and len({c['id'] for c in scores}) == 64, 'generation IDs/count differ')
            require(all(type(c['correct']) is int and c['correct'] in (0, 1, 2) for c in scores), 'invalid generation fitness')
            require(scores == sorted(scores, key=lambda c: (-c['correct'], c['id'])), 'rank/tie order differs')
            if previous is None:
                require(scores == [{k: c[k] for k in ['id', 'parent_id', 'correct']} for c in founders], 'founder scores differ')
            else:
                require(all(c in scores for c in previous[:8]), 'elites not retained unchanged')
                children = [c for c in scores if c['id'] not in {p['id'] for p in previous}]
                require({c['id'] for c in children} == set(range(64 + (g - 1) * 56, 64 + g * 56)), 'offspring ID allocation differs')
                require(all(c['parent_id'] in {p['id'] for p in previous} for c in children), 'unknown parent')
            for c in scores:
                first_seen.setdefault(c['id'], g)
            require(generation['champion_id'] == scores[0]['id']
                    and generation['best_mean_reward'] == scores[0]['correct'] / 2
                    and generation['both_correct'] == sum(c['correct'] == 2 for c in scores)
                    and math.isclose(generation['population_mean_reward'], statistics.mean(c['correct'] / 2 for c in scores)), 'generation aggregate differs')
            previous = scores
        require(previous == [{k: c[k] for k in ['id', 'parent_id', 'correct']} for c in final], 'final population/scores differ')
        require(all(c['born_generation'] == first_seen[c['id']] for c in final), 'final birth generation differs')
        summary = read(root / 'summary.json')
        require(summary == next(r for r in completed['runs'] if r['seed'] == seed), 'root/run summary mismatch')
        require(summary['status'] == 'completed' and summary['evaluations'] == 1664
                and summary['champion_id'] == final[0]['id']
                and summary['founder_best_mean_reward'] == founders[0]['correct'] / 2
                and summary['final_best_mean_reward'] == final[0]['correct'] / 2, 'champion/budget mismatch')
        evaluation = summary['champion_training_trials']
        trials = evaluation['intact']['trials']
        require(evaluation['cue_visible'] and [t['cue'] for t in trials] == [-1, 1]
                and trials == evaluation['same_network_state_cleared']['trials'], 'feed-forward replay differs')
        arena = manifest['arena']
        initial = {'x': arena['start_x'], 'y': arena['start_y'], 'heading': arena['start_heading']}
        for trial in trials:
            cue = trial['cue']
            target = 'right' if cue == -1 else 'left'
            require(trial['target'] == target and trial['choice'] in ('left', 'right', 'timeout')
                    and trial['reward'] == int(trial['choice'] == target), 'cue mapping/reward differs')
            hold, movement = trial['hold_trace'], trial['trajectory']
            require(trial['initial_pose'] == initial and len(hold) == 6
                    and all(t['pose'] == initial and t['inputs'][12] == (cue if i == 0 else 0)
                            for i, t in enumerate(hold)), 'hold pose/cue changed')
            require(len(movement) == trial['movement_ticks'] and 1 <= len(movement) <= 20
                    and [t['tick'] for t in movement] == list(range(1, len(movement) + 1))
                    and all(t['inputs'][12] == cue for t in movement), 'visible movement trace differs')
            require(movement[0]['pose_before'] == initial
                    and all(b['pose_before'] == a['pose_after'] for a, b in zip(movement, movement[1:])), 'pose continuity differs')
            require(all(t['contact_fraction'] is None for t in movement[:-1]), 'nonterminal goal contact')
            if trial['choice'] == 'timeout':
                require(len(movement) == 20 and movement[-1]['contact_fraction'] is None, 'invalid timeout')
            else:
                require(movement[-1]['contact_fraction'] is not None
                        and 0 <= movement[-1]['contact_fraction'] <= 1, 'missing terminal contact')
        require(sum(t['reward'] for t in trials) == final[0]['correct'], 'training replay fitness differs')
        require(summary['both_correct_at_founding'] == (founders[0]['correct'] == 2)
                and summary['both_correct_at_end'] == (final[0]['correct'] == 2)
                and summary['improvement_over_founders'] == (final[0]['correct'] > founders[0]['correct']), 'acquisition flags differ')
        runs.append(dict(seed=seed, founder_best=founders[0]['correct'] / 2, final_best=final[0]['correct'] / 2,
                         champion_id=final[0]['id'], improved=final[0]['correct'] > founders[0]['correct'],
                         founder_score_counts={str(i): sum(c['correct'] == i for c in founders) for i in range(3)},
                         final_score_counts={str(i): sum(c['correct'] == i for c in final) for i in range(3)},
                         generations=generations, elapsed_seconds=summary['elapsed_seconds'],
                         champion_choices=[{k: t[k] for k in ['cue', 'target', 'choice', 'reward', 'movement_ticks']} for t in trials]))
    guard = read(batch / 'thermal_guard/status.json')
    require(guard['state'] == 'finished' and guard['running_matches'] == 0 and not guard['paused'], 'guard not finished')
    return dict(schema_version=1, complete=True, execution_source=SOURCE,
                source_dirty=manifest['source_dirty'], core_build=manifest['core_build'],
                compiler=manifest['compiler'], target=manifest['target'], build_profile=manifest['build_profile'],
                independent_unit='founder-population search run; n=3, not organisms or cue trials',
                interpretation='fixed visible training acquisition only; no held-out validation, memory use, lifetime learning or ecological benefit',
                total_candidate_evaluations=4992, runs=runs,
                operational=operational,
                artifact_sha256={str(p.relative_to(batch)): sha(p) for p in sorted(batch.rglob('*')) if p.is_file()})


if __name__ == '__main__':
    result = summarize(sys.argv[1])
    path = Path(sys.argv[2])
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')
    print(json.dumps([{k: r[k] for k in ['seed', 'founder_best', 'final_best', 'improved', 'champion_choices']} for r in result['runs']], indent=2))
