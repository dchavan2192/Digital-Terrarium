"""Data-independent tests of the paired pilot auditor; no evaluator is run."""
import copy
import math
import unittest

import summarize_progress_comparison as audit


class ProgressAuditTests(unittest.TestCase):
    def setUp(self):
        self.arena = dict(start_x=0.0, start_y=0.0, goal_left_x=-4.0,
                          goal_left_y=4.0, goal_right_x=4.0, goal_right_y=4.0,
                          goal_radius=1.5)

    def trial(self, terminal, cue=1, reward=0):
        return dict(cue=cue, reward=reward,
                    trajectory=[dict(pose_after=dict(x=terminal[0], y=terminal[1]))])

    def test_terminal_progress_uses_last_pose_not_best_approach(self):
        trial = self.trial((-1, 1))
        trial['trajectory'].insert(0, dict(pose_after=dict(x=-2.8, y=2.8)))
        expected = ((math.sqrt(32) - 1.5) - (math.sqrt(18) - 1.5)) / (math.sqrt(32) - 1.5)
        self.assertAlmostEqual(audit.terminal_progress(trial, self.arena), expected)
        self.assertLess(audit.terminal_progress(trial, self.arena), .5)

    def test_progress_clips_retreat_and_exact_correct_contact(self):
        self.assertEqual(audit.terminal_progress(self.trial((0, -5)), self.arena), 0)
        # The confirmed contact flag gives exactly 1 despite floating boundary residue.
        hit = (-4 + 1.5 + 1e-14, 4)
        self.assertEqual(audit.terminal_progress(self.trial(hit, reward=1), self.arena), 1)
        # The other cue's target is right: a left contact is not progress to it.
        self.assertEqual(audit.terminal_progress(self.trial((-4, 2.5), cue=-1), self.arena), 0)

    def test_invalid_normalization_or_nonfinite_terminal_is_rejected(self):
        invalid = dict(self.arena, start_x=-4.0, start_y=4.0)
        with self.assertRaises(ValueError):
            audit.terminal_progress(self.trial((-1, 1)), invalid)
        with self.assertRaises(ValueError):
            audit.terminal_progress(self.trial((math.inf, 1)), self.arena)

    def test_true_reward_precedes_progress_and_progress_precedes_id(self):
        records = [dict(id=1, parent_id=None, correct=0, terminal_progress=1.0),
                   dict(id=20, parent_id=None, correct=1, terminal_progress=.1),
                   dict(id=10, parent_id=None, correct=1, terminal_progress=.2),
                   dict(id=5, parent_id=None, correct=1, terminal_progress=.2)]
        self.assertEqual([r['id'] for r in sorted(records, key=lambda r: audit.rank_key(r, 'terminal_progress'))],
                         [5, 10, 20, 1])

    def test_finite_progress_and_mode_compatibility_are_mandatory(self):
        base = dict(id=3, parent_id=None, correct=1)
        audit.check_metric(base, 'terminal_only')
        with self.assertRaises(ValueError):
            audit.check_metric(base, 'terminal_progress')
        for value in [None, True, math.nan, math.inf, -0.1, 1.1]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                audit.check_metric(dict(base, terminal_progress=value), 'terminal_progress')
        with self.assertRaises(ValueError):
            audit.check_metric(dict(base, terminal_progress=0.0), 'terminal_only')
        audit.check_metric(dict(base, terminal_progress=.5), 'terminal_progress')

    def test_json_rejects_nonfinite_literals_overflow_and_duplicate_keys(self):
        for text in ['{"a":NaN}', '{"a":Infinity}', '{"a":1e999}', '{"a":1,"a":2}']:
            with self.subTest(text=text), self.assertRaises(ValueError):
                audit.parse_json(text)
        self.assertEqual(audit.parse_json('{"a":[0,0.25,1]}'), {'a': [0, .25, 1]})

    def test_founder_identity_excludes_fitness_rank_and_includes_full_genome(self):
        founders = {0: {'weight': 1.0}, 1: {'weight': 2.0}}
        reordered = {1: {'weight': 2.0}, 0: {'weight': 1.0}}
        self.assertEqual(audit.digest_canonical(founders), audit.digest_canonical(reordered))
        modified = copy.deepcopy(founders)
        modified[1]['weight'] = 2.001
        self.assertNotEqual(audit.digest_canonical(founders), audit.digest_canonical(modified))


if __name__ == '__main__':
    unittest.main()
