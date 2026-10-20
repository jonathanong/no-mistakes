"""Saved real-heartbeat regression: elapsed clocks must not postpone a stall."""
from pathlib import Path
import unittest
from unittest.mock import patch

import inputs
from progress_clock import ProgressClock, fingerprint

FIXTURES = Path(__file__).resolve().parents[3] / 'fixtures/performance/current-arm/retry'


class ProgressTests(unittest.TestCase):
    def test_only_semantic_progress_resets_the_cutoff(self):
        with patch('progress_clock.time.monotonic', return_value=0):
            clock = ProgressClock()
            clock.observe(FIXTURES / 'progress-stalled.jsonl')
        with patch('progress_clock.time.monotonic', return_value=901):
            clock.observe(FIXTURES / 'progress-clock-only.jsonl')
            self.assertEqual(clock.stalled_seconds(), 901)
            clock.observe(FIXTURES / 'progress-reordered.jsonl')
            self.assertEqual(clock.stalled_seconds(), 901)
            clock.observe(FIXTURES / 'progress-advanced.jsonl')
            self.assertEqual(clock.stalled_seconds(), 0)

    def test_incomplete_or_unrelated_diagnostics_do_not_mean_progress(self):
        for line in ['[progress] {', '[progress] {}', '[timing] waiting']:
            self.assertIsNone(fingerprint(line))
        self.assertEqual(fingerprint('[work] parsed: 4'), '[work] parsed: 4')

    def test_work_updates_before_unchanged_heartbeats_are_retained(self):
        with patch('progress_clock.time.monotonic', return_value=0):
            clock = ProgressClock()
            clock.observe(FIXTURES / 'progress-stalled.jsonl')
        with patch('progress_clock.time.monotonic', return_value=600):
            clock.observe(FIXTURES / 'progress-interleaved-work.jsonl')
            self.assertEqual(clock.stalled_seconds(), 0)
            self.assertEqual(clock.work, {'annotation.files': 4, 'annotation.calls': 10})
        with patch('progress_clock.time.monotonic', return_value=1501):
            clock.observe(FIXTURES / 'progress-interleaved-work.jsonl')
            self.assertEqual(clock.stalled_seconds(), 901)


if __name__ == '__main__':
    unittest.main()
