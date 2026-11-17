"""Replay/audit integration tests; download the archived pilot before running."""
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import summarize_acquisition as analysis

DATA = Path(__file__).resolve().parent.parent / 'output/visible-acquisition/pilot_20261117T163705Z'


def replace_json(path, value):
    path.unlink()  # Our temporary symlink only; raw dataset remains untouched.
    path.write_text(json.dumps(value) + '\n')


@unittest.skipUnless(DATA.exists(), 'download the archived acquisition pilot first')
class AuditTests(unittest.TestCase):
    def test_complete_fixed_pilot(self):
        result = analysis.summarize(DATA)
        self.assertEqual(len(result['runs']), 3)
        self.assertEqual(result['total_candidate_evaluations'], 4992)
        self.assertTrue(all(len(r['generations']) == 26 for r in result['runs']))

    def test_missing_generation_and_empty_controls_rejected(self):
        for fault in ['missing_generation', 'empty_controls', 'topology']:
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                shadow = Path(temporary) / 'batch'
                shutil.copytree(DATA, shadow, copy_function=os.symlink)
                session = next((shadow / 'runs').glob('session_*'))
                if fault == 'missing_generation':
                    path = session / 'seed_2000/generations.jsonl'
                    lines = path.read_text().splitlines()
                    path.unlink()
                    path.write_text('\n'.join(lines[:2] + lines[3:]) + '\n')
                elif fault == 'empty_controls':
                    path = session / 'controls.json'
                    data = json.loads(path.read_text())
                    data['controls'] = []
                    replace_json(path, data)
                else:
                    path = session / 'seed_2000/final_population.json'
                    data = json.loads(path.read_text())
                    data[0]['genome']['connections'][0]['from'] = 19
                    replace_json(path, data)
                with self.assertRaises(ValueError):
                    analysis.summarize(shadow)

    def test_mismapped_visible_replay_rejected_even_if_summaries_agree(self):
        with tempfile.TemporaryDirectory() as temporary:
            shadow = Path(temporary) / 'batch'
            shutil.copytree(DATA, shadow, copy_function=os.symlink)
            session = next((shadow / 'runs').glob('session_*'))
            path = session / 'seed_2000/summary.json'
            summary = json.loads(path.read_text())
            evaluation = summary['champion_training_trials']
            for mode in ['intact', 'same_network_state_cleared']:
                evaluation[mode]['trials'][0]['trajectory'][0]['inputs'][12] = 0
            replace_json(path, summary)
            path = session / 'summary.json'
            completed = json.loads(path.read_text())
            completed['runs'][0] = summary
            replace_json(path, completed)
            with self.assertRaisesRegex(ValueError, 'visible movement trace'):
                analysis.summarize(shadow)


if __name__ == '__main__':
    unittest.main()
