"""Data-independent checks for balanced-cue accounting and exact replay stripping."""
import unittest
from copy import deepcopy
from pathlib import Path
from unittest.mock import patch
import summarize_visible_validation as audit_module
from summarize_visible_validation import trial_counts, strip_progress


class ValidationAuditTests(unittest.TestCase):
    def fixture(self):
        return dict(mean_reward=.5, trials=[
            dict(cue=-1, target='right', choice='right', reward=1, movement_ticks=20),
            dict(cue=1, target='left', choice='timeout', reward=0, movement_ticks=20)])

    def test_counts_keep_timeouts_separate_from_wrong_contacts(self):
        condition = self.fixture()
        self.assertEqual(trial_counts(condition, 20), dict(correct=1, wrong=0, timeout=1))
        condition['trials'][1]['choice'] = 'right'
        condition['trials'][1]['movement_ticks'] = 7
        self.assertEqual(trial_counts(condition, 20), dict(correct=1, wrong=1, timeout=0))

    def test_rejects_duplicate_cue_bad_reward_early_timeout_and_bad_mean(self):
        for field, value in [('cue', -1), ('reward', 1), ('movement_ticks', 19)]:
            condition = self.fixture()
            condition['trials'][1][field] = value
            with self.assertRaises(ValueError):
                trial_counts(condition, 20)
        condition = self.fixture()
        condition['mean_reward'] = 1
        with self.assertRaises(ValueError):
            trial_counts(condition, 20)

    def test_stripping_preserves_every_nonprogress_field_and_source(self):
        source = {key: dict(self.fixture(), mean_terminal_progress=.5)
                  for key in ('intact', 'same_network_state_cleared')}
        source['intact']['trials'][0]['terminal_progress'] = 1
        stripped = strip_progress(source)
        self.assertEqual(stripped['intact'], self.fixture())
        self.assertIn('terminal_progress', source['intact']['trials'][0])


DATASET = Path(__file__).resolve().parents[1] / 'output/visible-validation/robustness_20261118T020500Z'


@unittest.skipUnless(DATASET.is_dir(), 'archived diagnostic is not installed')
class DatasetAuditTests(unittest.TestCase):
    def test_complete_cohort_and_case_denominators(self):
        result = audit_module.audit(DATASET)
        self.assertEqual(len(result['policies']), 63)
        self.assertEqual(len(result['aggregate']), 21)
        self.assertEqual(len(result['controls']), 126)

    def test_missing_case_or_changed_baseline_is_rejected(self):
        real_read = audit_module.read
        for mutation in ('missing', 'baseline'):
            def altered(path):
                data = real_read(path)
                if path.name == 'result.json':
                    data = deepcopy(data)
                    if mutation == 'missing':
                        data['policies'][0]['cases'].pop()
                    else:
                        data['policies'][0]['cases'][0]['evaluation']['intact']['mean_reward'] = .125
                return data
            with patch.object(audit_module, 'read', side_effect=altered):
                with self.assertRaises(ValueError):
                    audit_module.audit(DATASET)

    def test_changed_fingerprint_or_unfinished_guard_is_rejected(self):
        real_read = audit_module.read
        for mutation in ('fingerprint', 'guard'):
            def altered(path):
                data = real_read(path)
                if mutation == 'fingerprint' and path.name == 'result.json':
                    data = deepcopy(data)
                    data['source']['input_files'][0]['bytes'] += 1
                if mutation == 'guard' and path == DATASET / 'original/thermal_guard/status.json':
                    data = deepcopy(data)
                    data['state'] = 'monitoring'
                return data
            with patch.object(audit_module, 'read', side_effect=altered):
                with self.assertRaises(ValueError):
                    audit_module.audit(DATASET)


if __name__ == '__main__':
    unittest.main()
