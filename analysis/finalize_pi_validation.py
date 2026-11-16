"""Wait for one batch, then write a strict summary using a pinned summarizer.

Usage: python3 finalize_pi_validation.py BATCH_DIR
Copy this file and summarize_validation.py into BATCH/finalization before launch.
No experiment is started and no result is published automatically.
"""
import argparse
import fcntl
import json
from pathlib import Path
import time

from summarize_validation import summarize


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('batch', type=Path)
    args = parser.parse_args()
    batch = args.batch.resolve()
    destination = batch / 'finalization'
    destination.mkdir(exist_ok=True)
    lock = (destination / '.lock').open('a')
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    status = destination / 'status'
    status.write_text('waiting\n')
    while True:
        state = (batch / 'status').read_text().strip()
        if state == 'completed':
            break
        if state == 'failed':
            status.write_text('batch_failed_inspect_before_resume\n')
            raise SystemExit('Batch failed; summary not promoted. Inspect logs and resume as needed.')
        if state != 'running':
            status.write_text('unexpected_batch_status\n')
            raise SystemExit(f'Unknown batch status: {state!r}')
        time.sleep(30)
    try:
        result = summarize(batch)
        temporary = destination / 'summary.json.pending'
        temporary.write_text(json.dumps(result, indent=2) + '\n')
        temporary.replace(destination / 'summary.json')
        status.write_text('summarized\n')
        print(json.dumps(dict(finished=result['finished'], decision=result['decision'])), flush=True)
    except BaseException:
        status.write_text('summary_failed\n')
        raise


if __name__ == '__main__':
    main()
