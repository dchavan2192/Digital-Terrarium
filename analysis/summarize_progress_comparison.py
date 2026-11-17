"""Audit the fixed paired visible-training pilot, never run an evaluator.

Usage: python3 analysis/summarize_progress_comparison.py PARENT OUTPUT_JSON
       [--source FULL_COMMIT_SHA]

PARENT contains completed terminal_only/ and terminal_progress/ supervised
batches. OUTPUT_JSON must be outside the raw parent. All six searches must be
complete; partial or failed runs cannot silently become a comparison.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import statistics
import tomllib

from summarize_acquisition import check_genome, require, sha


SEEDS = [2100, 2101, 2102]
MODES = ('terminal_only', 'terminal_progress')
PROTOCOL_HASHES = {
    'terminal_only': 'bf979d245a715c86a5160dc15b5f67245e0bf3654ecb718dba7783bef495ce46',
    'terminal_progress': '244decdcccbfb190501a979ff954d3be7a37dce431c9f9fe87892447085dea15',
}
ARENA_HASH = '7a47f36167419d92c0ab7414d11b4c399526dc90196fd64d7d938e1c6372f213'
PROTOCOL_COMMON = dict(
    population=64, generations=25, elites=8, tournament=3, seeds=SEEDS, hidden=8,
    mutation=dict(weight_probability=.1, weight_step=.2,
                  add_connection_probability=0.0, add_node_probability=0.0,
                  toggle_probability=0.0, prune_probability=0.0,
                  add_recurrent_probability=0.0, trait_probability=0.0),
)
CONTROL_NAMES = ['recurrent_reference', 'feed_forward_control', 'same_network_state_cleared']
BUILD_FIELDS = ('source_revision', 'source_dirty', 'core_build', 'compiler', 'target', 'build_profile')


def finite_tree(value):
    """Reject JSON NaN/Infinity, including overflowed numeric literals."""
    if isinstance(value, float):
        require(math.isfinite(value), 'nonfinite JSON number')
    elif isinstance(value, dict):
        for item in value.values():
            finite_tree(item)
    elif isinstance(value, list):
        for item in value:
            finite_tree(item)


def parse_json(text):
    def unique_object(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, f'duplicate JSON key: {key}')
            result[key] = value
        return result
    result = json.loads(text, object_pairs_hook=unique_object)
    finite_tree(result)
    return result


def read_json(path):
    try:
        return parse_json(path.read_text())
    except (ValueError, KeyError, TypeError) as error:
        raise ValueError(f'{path}: {error}') from error


def close(actual, expected, message):
    require(type(actual) in (int, float) and math.isfinite(actual)
            and math.isclose(actual, expected, rel_tol=0, abs_tol=1e-12), message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False)


def digest_canonical(value):
    return hashlib.sha256(canonical(value).encode()).hexdigest()


def check_metric(record, mode):
    require(type(record['id']) is int and record['id'] >= 0, 'invalid candidate ID')
    require(record['parent_id'] is None or type(record['parent_id']) is int,
            'invalid parent ID')
    require(type(record['correct']) is int and record['correct'] in (0, 1, 2),
            'invalid true terminal fitness')
    if mode == 'terminal_progress':
        value = record.get('terminal_progress')
        require(type(value) in (int, float) and math.isfinite(value) and 0 <= value <= 1,
                'missing or invalid progress fitness')
    else:
        require('terminal_progress' not in record, 'progress field in terminal-only mode')


def rank_key(record, mode):
    return (-record['correct'],
            -record['terminal_progress'] if mode == 'terminal_progress' else 0,
            record['id'])


def score_row(candidate, mode):
    keys = ['id', 'parent_id', 'correct']
    if mode == 'terminal_progress':
        keys.append('terminal_progress')
    return {key: candidate[key] for key in keys}


def check_population(population, mode):
    require(len(population) == 64 and len({c['id'] for c in population}) == 64,
            'population count/unique IDs differ')
    for candidate in population:
        check_metric(candidate, mode)
        check_genome(candidate['genome'])
    require(population == sorted(population, key=lambda c: rank_key(c, mode)),
            'population rank/tie order differs')


def terminal_progress(trial, arena):
    """Recompute the specified terminal-only diagnostic from the archived pose."""
    target = 'left' if trial['cue'] == 1 else 'right'
    x, y = arena[f'goal_{target}_x'], arena[f'goal_{target}_y']
    radius = arena['goal_radius']

    def surface(pose):
        return math.sqrt((pose['x'] - x)**2 + (pose['y'] - y)**2) - radius

    start = surface(dict(x=arena['start_x'], y=arena['start_y']))
    require(math.isfinite(start) and start > 0, 'invalid progress normalization distance')
    terminal = surface(trial['trajectory'][-1]['pose_after'])
    require(math.isfinite(terminal), 'nonfinite terminal surface distance')
    if trial['reward'] == 1:
        return 1.0
    return min(1.0, max(0.0, (start - max(terminal, 0.0)) / start))


def check_condition(condition, arena, progress):
    trials = condition['trials']
    require([t['cue'] for t in trials] == [-1, 1], 'balanced trial order differs')
    initial = dict(x=arena['start_x'], y=arena['start_y'], heading=arena['start_heading'])
    recomputed_progress = []
    common_release_inputs = None
    for trial in trials:
        cue = trial['cue']
        target = 'left' if cue == 1 else 'right'
        require(trial['target'] == target and trial['choice'] in ('left', 'right', 'timeout')
                and type(trial['reward']) is int
                and trial['reward'] == int(trial['choice'] == target), 'cue/target/reward mismatch')
        hold, movement = trial['hold_trace'], trial['trajectory']
        require(trial['initial_pose'] == initial and len(hold) == 6,
                'hold count or initial pose differs')
        for index, tick in enumerate(hold):
            require(tick['tick'] == index + 1 and tick['pose'] == initial
                    and tick['phase'] == ('cue' if index == 0 else 'blank')
                    and len(tick['inputs']) == 20
                    and tick['inputs'][12] == (cue if index == 0 else 0), 'hold trace changed')
        require(type(trial['movement_ticks']) is int and len(movement) == trial['movement_ticks']
                and 1 <= len(movement) <= 20
                and [t['tick'] for t in movement] == list(range(1, len(movement) + 1)),
                'movement count/tick order differs')
        require(movement[0]['pose_before'] == initial
                and all(b['pose_before'] == a['pose_after'] for a, b in zip(movement, movement[1:])),
                'movement pose continuity differs')
        for tick in movement:
            require(len(tick['inputs']) == 20 and tick['inputs'][12] == cue,
                    'visible response cue differs')
            require(not {'reward', 'target', 'cue'} & tick.keys(), 'answer metadata inside response frame')
        release_inputs = list(movement[0]['inputs'])
        release_inputs[12] = 0
        if common_release_inputs is None:
            common_release_inputs = release_inputs
        require(release_inputs == common_release_inputs, 'non-cue release inputs differ across cues')
        for tick in hold:
            held_inputs = list(tick['inputs'])
            held_inputs[12] = 0
            require(held_inputs == common_release_inputs, 'non-cue sensors changed during fixed-pose hold')
        require(all(t['contact_fraction'] is None for t in movement[:-1]), 'nonterminal contact')
        if trial['choice'] == 'timeout':
            require(len(movement) == 20 and movement[-1]['contact_fraction'] is None,
                    'invalid timeout termination')
        else:
            fraction = movement[-1]['contact_fraction']
            require(type(fraction) in (int, float) and 0 <= fraction <= 1, 'invalid terminal contact')
            pose = movement[-1]['pose_after']
            x, y = arena[f"goal_{trial['choice']}_x"], arena[f"goal_{trial['choice']}_y"]
            distance = math.sqrt((pose['x'] - x)**2 + (pose['y'] - y)**2)
            require(math.isclose(distance, arena['goal_radius'], rel_tol=0, abs_tol=1e-9),
                    'contact pose is not on chosen goal boundary')
        recomputed = terminal_progress(trial, arena)
        recomputed_progress.append(recomputed)
        if progress:
            close(trial.get('terminal_progress'), recomputed, 'trial progress does not match terminal pose')
        else:
            require('terminal_progress' not in trial, 'terminal-only trial contains progress')
    close(condition['mean_reward'], sum(t['reward'] for t in trials) / 2, 'mean true reward differs')
    mean_progress = statistics.mean(recomputed_progress)
    if progress:
        close(condition.get('mean_terminal_progress'), mean_progress, 'condition progress mean differs')
    else:
        require('mean_terminal_progress' not in condition, 'terminal-only condition contains progress')
    return mean_progress


def check_sha_file(path, batch):
    require(path.is_file(), f'missing pinned inventory: {path.name}')
    seen = set()
    for line in path.read_text().splitlines():
        digest, filename = line.split(maxsplit=1)
        name = Path(filename.lstrip('*')).name
        require(name not in seen and re.fullmatch('[0-9a-f]{64}', digest), 'invalid pinned inventory')
        seen.add(name)
        require(sha(batch / name) == digest, f'pinned artifact hash mismatch: {name}')
    require(seen, 'empty pinned inventory')


def check_operations(batch):
    require((batch / 'status').read_text().strip() == 'completed'
            and (batch / 'exit_code').read_text().strip() == '0', 'sub-batch incomplete or failed')
    require(not (batch / 'source_status').read_text().strip(), 'execution source is dirty')
    supervisor = read_json(batch / 'supervisor.json')
    require(supervisor['state'] == 'completed' and supervisor['runner_exit_code'] == 0
            and supervisor['supervisor_exit_code'] == 0 and supervisor['guard_exit_code'] == 0
            and supervisor['runner_left_paused'] is False and supervisor['reason'] is None,
            'supervisor not cleanly completed')
    operations = read_json(batch / 'supervisor_manifest.json')
    require(type(operations['wall_seconds']) in (int, float) and operations['wall_seconds'] == 1800
            and operations['wall_includes_startup_and_cooling'] is True
            and operations['environment_override'] == {'RAYON_NUM_THREADS': '1'}, 'unsafe operational bounds')
    require((operations['temperature_high_c'], operations['temperature_low_c'], operations['guard_poll_seconds'])
            == (75, 65, 5), 'thermal policy differs')
    names = {'sim-runner', 'visible_acquisition.toml', 'delayed_choice.toml', 'pi_thermal_guard.py'}
    require(set(operations['sha256']) == names, 'operational pinned files differ')
    for name, digest in operations['sha256'].items():
        require(re.fullmatch('[0-9a-f]{64}', digest) and sha(batch / name) == digest,
                f'operational hash mismatch: {name}')
    for filename in ('pinned_build.sha256', 'operational_inputs.sha256'):
        check_sha_file(batch / filename, batch)
    guard = read_json(batch / 'thermal_guard/status.json')
    require(guard['state'] == 'finished' and guard['running_matches'] == 0 and guard['paused'] == [],
            'guard not finished or owned processes remain')
    require((guard['high_c'], guard['low_c'], guard['poll_seconds']) == (75, 65, 5), 'guard thresholds differ')
    return dict(supervisor=supervisor, limits=operations, final_guard=guard)


def audit_run(root, seed, mode, arena, parent_summary):
    founders, final = (read_json(root / name) for name in ('founders.json', 'final_population.json'))
    for population in (founders, final):
        check_population(population, mode)
    require({c['id'] for c in founders} == set(range(64))
            and all(c['parent_id'] is None and c['born_generation'] == 0 for c in founders),
            'founder identities differ')
    generations = [parse_json(line) for line in (root / 'generations.jsonl').read_text().splitlines()]
    require([g['generation'] for g in generations] == list(range(26)), 'generation grid incomplete/duplicated')
    previous, first_seen = None, {}
    for generation in generations:
        index, scores = generation['generation'], generation['scores']
        require(len(scores) == 64 and len({c['id'] for c in scores}) == 64, 'generation population IDs differ')
        for score in scores:
            check_metric(score, mode)
            require(set(score) == set(score_row(score, mode)), 'unknown generation score field')
        require(scores == sorted(scores, key=lambda c: rank_key(c, mode)), 'generation ranking/ties differ')
        if previous is None:
            require(scores == [score_row(c, mode) for c in founders], 'founder log mismatch')
        else:
            require(all(c in scores for c in previous[:8]), 'elite scores/IDs not retained unchanged')
            previous_ids = {c['id'] for c in previous}
            children = [c for c in scores if c['id'] not in previous_ids]
            require({c['id'] for c in children} == set(range(64 + (index - 1)*56, 64 + index*56)),
                    'child ID allocation differs')
            require(all(c['parent_id'] in previous_ids for c in children), 'unknown child parent')
        for score in scores:
            first_seen.setdefault(score['id'], index)
        require(generation['champion_id'] == scores[0]['id']
                and generation['best_mean_reward'] == scores[0]['correct']/2
                and generation['population_mean_reward'] == statistics.mean(c['correct']/2 for c in scores)
                and generation['both_correct'] == sum(c['correct'] == 2 for c in scores),
                'generation true-fitness aggregate differs')
        if mode == 'terminal_progress':
            close(generation.get('champion_terminal_progress'), scores[0]['terminal_progress'],
                  'generation selected-champion progress differs')
        else:
            require('champion_terminal_progress' not in generation, 'progress aggregate in control')
        previous = scores
    require(previous == [score_row(c, mode) for c in final], 'final score log mismatch')
    require(all(c['born_generation'] == first_seen[c['id']] for c in final), 'final birth generation differs')
    summary = read_json(root / 'summary.json')
    require(summary == parent_summary and summary['status'] == 'completed' and summary['seed'] == seed,
            'run/root completion summary mismatch')
    require(type(summary['elapsed_seconds']) in (int, float) and summary['elapsed_seconds'] >= 0,
            'invalid elapsed search time')
    require(summary['evaluations'] == 1664 and summary['champion_id'] == final[0]['id']
            and summary['founder_best_mean_reward'] == founders[0]['correct']/2
            and summary['final_best_mean_reward'] == final[0]['correct']/2, 'champion or work budget differs')
    require(summary['both_correct_at_founding'] == (founders[0]['correct'] == 2)
            and summary['both_correct_at_end'] == (final[0]['correct'] == 2)
            and summary['improvement_over_founders'] == (final[0]['correct'] > founders[0]['correct']),
            'true-acquisition flags differ')
    evaluation = summary['champion_training_trials']
    require(evaluation['cue_visible'] is True, 'champion replay is not visible')
    intact, cleared = evaluation['intact'], evaluation['same_network_state_cleared']
    require(intact['clear_state_each_tick'] is False and cleared['clear_state_each_tick'] is True
            and intact['trials'] == cleared['trials'], 'feed-forward reset replay differs')
    progress = mode == 'terminal_progress'
    recomputed = check_condition(intact, arena, progress)
    check_condition(cleared, arena, progress)
    close(intact['mean_reward'], final[0]['correct']/2, 'champion replay true fitness differs')
    if progress:
        close(summary.get('founder_best_terminal_progress'), founders[0]['terminal_progress'], 'founder progress differs')
        close(summary.get('final_best_terminal_progress'), final[0]['terminal_progress'], 'final progress differs')
        close(final[0]['terminal_progress'], recomputed, 'selected champion progress differs from geometry')
    else:
        require('founder_best_terminal_progress' not in summary and 'final_best_terminal_progress' not in summary,
                'control summary unexpectedly includes progress')
    counts = lambda pop: {str(i): sum(c['correct'] == i for c in pop) for i in range(3)}
    result = dict(seed=seed, founder_best=founders[0]['correct']/2, final_best=final[0]['correct']/2,
                  both_correct_at_founding=founders[0]['correct'] == 2, both_correct_at_end=final[0]['correct'] == 2,
                  improved=final[0]['correct'] > founders[0]['correct'], champion_id=final[0]['id'],
                  founder_score_counts=counts(founders), final_score_counts=counts(final),
                  final_population_mean_reward=statistics.mean(c['correct']/2 for c in final),
                  recomputed_champion_terminal_progress=recomputed,
                  generations=[{k: v for k, v in g.items() if k != 'scores'} for g in generations],
                  elapsed_seconds=summary['elapsed_seconds'],
                  champion_choices=[{k: t[k] for k in ('cue', 'target', 'choice', 'reward', 'movement_ticks')}
                                    for t in intact['trials']])
    if progress:
        result['founder_selected_champion_terminal_progress'] = founders[0]['terminal_progress']
        result['final_selected_champion_terminal_progress'] = final[0]['terminal_progress']
    return result, {c['id']: c['genome'] for c in founders}


def audit_mode(batch, mode, expected_source):
    source = (batch / 'source_revision').read_text().strip()
    require(re.fullmatch('[0-9a-f]{40}', source) and source == expected_source, 'execution source differs')
    require(sha(batch / 'visible_acquisition.toml') == PROTOCOL_HASHES[mode]
            and sha(batch / 'delayed_choice.toml') == ARENA_HASH, 'frozen input hash differs')
    protocol = tomllib.loads((batch / 'visible_acquisition.toml').read_text())
    require(protocol == dict(PROTOCOL_COMMON, fitness=mode), 'protocol table differs')
    arena = tomllib.loads((batch / 'delayed_choice.toml').read_text())
    operations = check_operations(batch)
    sessions = sorted((batch / 'runs').glob('session_*'))
    require(len(sessions) == 1 and sessions[0].is_dir(), 'require exactly one session per mode')
    session = sessions[0]
    require({p.name for p in session.glob('seed_*')} == {f'seed_{seed}' for seed in SEEDS}, 'unexpected/missing seed directories')
    manifest = read_json(session / 'manifest.json')
    require(manifest['source_revision'] == source and manifest['source_dirty'] == 'false'
            and manifest['build_profile'] == 'release' and manifest['target'] == 'aarch64-unknown-linux-gnu',
            'manifest source/build not the declared clean Pi release')
    require(manifest['protocol'] == protocol and manifest['arena'] == arena
            and manifest['total_evaluations'] == 4992, 'manifest config/work budget differs')
    controls = read_json(session / 'controls.json')
    require(controls['arena'] == arena, 'control arena differs from frozen input')
    require([c['name'] for c in controls['controls']] == CONTROL_NAMES
            and controls['instrument_validated'] is True and controls['cue_visible'] is True,
            'visible instrument controls missing or invalid')
    require(controls['checks'] and all(v is True for v in controls['checks'].values()), 'instrument check failed')
    for control in controls['controls']:
        require(control['mean_reward'] == 1, 'manufactured visible reference did not solve task')
        check_condition(control, arena, False)
    completed = read_json(session / 'summary.json')
    require(completed['status'] == 'completed' and completed['technical_failures'] == 0
            and [r['seed'] for r in completed['runs']] == SEEDS, 'failed or incomplete declared searches')
    runs, founders = [], {}
    for seed, parent_summary in zip(SEEDS, completed['runs']):
        result, genomes = audit_run(session / f'seed_{seed}', seed, mode, arena, parent_summary)
        runs.append(result)
        founders[seed] = genomes
    return dict(mode=mode, build={k: manifest[k] for k in BUILD_FIELDS}, protocol=protocol, arena=arena,
                candidate_evaluations=4992, runs=runs, operational=operations), founders


def summarize(parent, source=None):
    parent = Path(parent)
    if (parent / 'status').exists():
        require((parent / 'status').read_text().strip() == 'completed', 'parent queue not completed')
    if (parent / 'exit_code').exists():
        require((parent / 'exit_code').read_text().strip() == '0', 'parent queue failed')
    require((parent / 'status').read_text().strip() == 'completed'
            and (parent / 'exit_code').read_text().strip() == '0', 'parent comparison incomplete or failed')
    sources = [(parent / mode / 'source_revision').read_text().strip() for mode in MODES]
    require(sources[0] == sources[1], 'paired execution revisions differ')
    expected_source = source or sources[0]
    require(re.fullmatch('[0-9a-f]{40}', expected_source), '--source must be a full commit SHA')
    modes, founders = {}, {}
    for mode in MODES:
        try:
            modes[mode], founders[mode] = audit_mode(parent / mode, mode, expected_source)
        except (ValueError, KeyError, TypeError) as error:
            raise ValueError(f'{mode}: {error}') from error
    require(modes['terminal_only']['build'] == modes['terminal_progress']['build'], 'paired build metadata differs')
    require(sha(parent / 'terminal_only/sim-runner') == sha(parent / 'terminal_progress/sim-runner'),
            'paired executable bytes differ')
    require(modes['terminal_only']['operational']['limits']['wall_seconds']
            == modes['terminal_progress']['operational']['limits']['wall_seconds'],
            'paired watchdog limits differ')
    identities, paired = {}, []
    for index, seed in enumerate(SEEDS):
        control_founders, progress_founders = (founders[mode][seed] for mode in MODES)
        require(canonical(control_founders) == canonical(progress_founders), f'seed {seed}: founder genomes differ by mode')
        identities[str(seed)] = digest_canonical(control_founders)
        control, progress = (modes[mode]['runs'][index] for mode in MODES)
        paired.append(dict(seed=seed, terminal_only_final_best=control['final_best'],
                           terminal_progress_final_best=progress['final_best'],
                           final_true_reward_difference=progress['final_best'] - control['final_best'],
                           terminal_only_both_correct=control['both_correct_at_end'],
                           terminal_progress_both_correct=progress['both_correct_at_end'],
                           final_population_mean_reward_difference=progress['final_population_mean_reward']
                           - control['final_population_mean_reward']))
    return dict(schema_version=1, complete=True, execution_source=expected_source,
                independent_unit='paired founder-population search seed; n=3, not candidates, generations or cue trials',
                interpretation='fixed visible training search-method pilot; no held-out validation, memory use, lifetime learning or ecological benefit',
                primary_endpoint='true correct first-contact count/both-cue acquisition, distinct from progress tiebreak',
                total_candidate_evaluations=9984, paired_seed_count=3,
                frozen_protocol_sha256=PROTOCOL_HASHES, arena_sha256=ARENA_HASH,
                canonical_founder_genome_sha256=identities, modes=modes, paired=paired,
                mean_paired_final_true_reward_difference=statistics.mean(p['final_true_reward_difference'] for p in paired),
                artifact_sha256={str(p.relative_to(parent)): sha(p)
                                 for p in sorted(parent.rglob('*')) if p.is_file()})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('output_json', type=Path)
    parser.add_argument('--source', help='Expected full execution revision; otherwise require matching recorded revisions')
    args = parser.parse_args()
    require(args.parent.resolve() not in [args.output_json.resolve(), *args.output_json.resolve().parents],
            'write the analysis outside the raw parent to preserve a stable inventory')
    result = summarize(args.parent, args.source)
    args.output_json.parent.mkdir(parents=True, exist_ok=True)
    args.output_json.write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')
    print(json.dumps(result['paired'], indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
