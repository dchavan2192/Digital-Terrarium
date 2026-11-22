"""Small data-independent auditor regressions; no actual policy evaluation."""
import copy
import hashlib
import json
from pathlib import Path
import unittest
import uuid

import summarize_visible_diversity as audit


def candidate(identifier, parent=None, generation=0, correct=1, progress=.4):
    return dict(id=identifier, parent_id=parent, born_generation=generation,
                correct=correct * 3, terminal_progress=progress,
                case_scores=[dict(case=name, correct=correct, terminal_progress=progress)
                             for name in audit.TRAINING['fixed']])


def generation(index, scores):
    return dict(generation=index, champion_id=scores[0]['id'], best_correct=scores[0]['correct'],
                champion_terminal_progress=scores[0]['terminal_progress'],
                all_six_correct=sum(c['correct'] == 6 for c in scores), scores=scores)


class AuditTests(unittest.TestCase):
    def test_correctness_strictly_precedes_progress_then_id(self):
        scores = [candidate(3, correct=2, progress=.1), candidate(1, progress=.8),
                  candidate(2, progress=.8)]
        audit.audit_scores(scores, 'fixed', size=3)
        for corrupt in ([scores[1], scores[0], scores[2]], [scores[0], scores[2], scores[1]]):
            with self.assertRaisesRegex(ValueError, 'ranking'):
                audit.audit_scores(corrupt, 'fixed', size=3)

    def test_six_trial_accounting_and_repeated_fixed_slots(self):
        scores = [candidate(0), candidate(1)]
        for key, value in [('correct', 2), ('terminal_progress', .9)]:
            bad = copy.deepcopy(scores)
            bad[0][key] = value
            with self.assertRaises(ValueError):
                audit.audit_scores(bad, 'fixed', size=2)
        bad = copy.deepcopy(scores)
        bad[0]['case_scores'][1]['terminal_progress'] = .6
        bad[0]['terminal_progress'] = (.4 + .6 + .4) / 3
        with self.assertRaisesRegex(ValueError, 'repeated'):
            audit.audit_scores(bad, 'fixed', size=2)

    def test_generation_elites_parents_birth_and_persistent_ids(self):
        founders = [candidate(i) for i in range(3)]
        children = [candidate(0), candidate(3, 1, 1), candidate(4, 2, 1)]
        generations = [generation(0, founders), generation(1, children)]
        parents = audit.audit_generations(generations, 'fixed', population=3, replacements=1, elites=1)
        self.assertEqual(parents, {0: None, 1: None, 2: None, 3: 1, 4: 2})
        for field, value in [('parent_id', 999), ('born_generation', 0), ('id', 7)]:
            bad = copy.deepcopy(generations)
            bad[1]['scores'][1][field] = value
            with self.assertRaises(ValueError):
                audit.audit_generations(bad, 'fixed', population=3, replacements=1, elites=1)

    def test_endpoint_families_have_fixed_denominators(self):
        cases = [dict(case=name, evaluation=dict(intact=dict(trials=[dict(reward=1), dict(reward=0)])))
                 for name in audit.HOLDOUTS]
        result = audit.endpoints(cases)
        self.assertEqual(result['geometry_correct'], 4)
        self.assertEqual(result['geometry_denominator'], 8)
        self.assertEqual(result['heading_correct'], 2)
        self.assertEqual(result['heading_denominator'], 4)
        self.assertEqual(result['both_correct_cases'], 0)
        with self.assertRaisesRegex(ValueError, 'coverage'):
            audit.endpoints(cases[:-1])

    def test_locked_identity_uses_exact_bytes_and_rejects_changes(self):
        root = Path(__file__).resolve().parents[1] / 'output/diversity-summary-tests' / uuid.uuid4().hex
        root.mkdir(parents=True)
        path = root / 'training.json'
        raw = b'{"champion_id":0}\n'
        path.write_bytes(raw)
        identity = dict(path=path.name, bytes=len(raw), fnv1a64_noncryptographic=audit.fnv(raw))
        reader = audit.Reader(root)
        reader.identity(root, identity)
        self.assertEqual(reader.hashes[path.name], hashlib.sha256(raw).hexdigest())
        path.write_bytes(raw + b' ')
        with self.assertRaisesRegex(ValueError, 'changed'):
            reader.identity(root, identity)
        with self.assertRaisesRegex(ValueError, 'unsafe'):
            reader.identity(root, dict(identity, path='../outside.json'))

    def test_nonfinite_and_technical_failure_rejected(self):
        with self.assertRaises(ValueError):
            audit.finite({'nested': [float('nan')]})
        with self.assertRaisesRegex(ValueError, 'technical'):
            audit.evaluation(dict(status='failed', error='fixture'), {})
        root = Path(__file__).resolve().parents[1] / 'output/diversity-summary-tests' / uuid.uuid4().hex
        root.mkdir(parents=True)
        (root / 'status').write_text('failed\n')
        (root / 'exit_code').write_text('1\n')
        with self.assertRaisesRegex(ValueError, 'not completed'):
            audit.summarize(root)


if __name__ == '__main__':
    unittest.main()
