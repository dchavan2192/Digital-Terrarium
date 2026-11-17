"""Dataset integration and rejection checks; skips if the archived data is absent."""
import csv
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import summarize_memory_growth as analysis

DATA = Path(__file__).resolve().parent.parent / 'output/memory-growth/overnight_20261117T040010Z'

@unittest.skipUnless(DATA.exists(), 'download the archived overnight dataset first')
class AuditTests(unittest.TestCase):
    def test_complete_batch_and_export_crosschecks(self):
        result = analysis.summarize(DATA)
        self.assertEqual(len(result['runs']), 8)
        self.assertTrue(all(r['csv_rows'] == 400 and r['late_rows'] == 101 for r in result['runs']))
        self.assertEqual(result['guard']['pauses'], result['guard']['resumes'])

    def test_bad_sampling_and_nonfinite_metrics_are_rejected(self):
        original = next(DATA.glob('rate_0/seed_3000/session_*/stats*.csv'))
        with original.open() as stream:
            reader = csv.DictReader(stream)
            columns, rows = reader.fieldnames, list(reader)
        for fault in ['missing', 'duplicate', 'nan']:
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                shadow = Path(temporary) / 'batch'
                shutil.copytree(DATA, shadow, copy_function=os.symlink)
                target = shadow / original.relative_to(DATA)
                target.unlink()  # Only our temporary symlink, never the raw artifact.
                edited = [dict(row) for row in rows]
                if fault == 'missing':
                    edited.pop(3)
                elif fault == 'duplicate':
                    edited[3]['tick'] = edited[2]['tick']
                else:
                    edited[3]['mean_connections'] = 'nan'
                with target.open('w') as stream:
                    writer = csv.DictWriter(stream, fieldnames=columns)
                    writer.writeheader()
                    writer.writerows(edited)
                with self.assertRaises(ValueError):
                    analysis.summarize(shadow)
        self.assertEqual(analysis.sha(original), analysis.sha(DATA / original.relative_to(DATA)))

    def test_incomplete_status_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            shadow = Path(temporary)
            (shadow / 'status').write_text('running\n')
            with self.assertRaisesRegex(ValueError, 'not completed'):
                analysis.summarize(shadow)

if __name__ == '__main__':
    unittest.main()
