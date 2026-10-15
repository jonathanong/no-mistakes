"""Saved real-derived lifetime regressions and strict parser negatives."""
import json
from pathlib import Path
import unittest

from decode import IncompleteCapture, unpack_chunks
from replay import replay

HERE = Path(__file__).resolve().parent


def fixture(name):
    return json.loads((HERE / 'fixtures' / (name + '.json')).read_text())


def encoded(name):
    lines = (HERE / 'fixtures' / (name + '.msgpack.hex')).read_text().splitlines()
    return bytes.fromhex(''.join(line for line in lines if not line.startswith('#')))


def selected(data, *, subset=True, continuous=True, producer=True, history=True):
    events, ledger = data['events'], data['ledger']
    marker = {row['op']: row['addr'] for row in ledger if row['op'] in ('begin', 'end')}
    start = next(i for i, row in enumerate(events)
                 if row['type'] == 'Malloc' and row['addr'] == marker['begin']) + 1
    end = next(i for i, row in enumerate(events)
               if row['type'] == 'Malloc' and row['addr'] == marker['end'])
    addresses = {row['addr'] for row in ledger if row['op'] not in ('begin', 'end')}
    return replay(events, ledger[0]['pid'], start, end,
                  addresses=addresses if subset else None, continuous_capture=continuous, producer_verified=producer,
                  history_verified=history)


class Accounting(unittest.TestCase):
    def test_real_derived_objects_cover_cross_thread_free_reuse_and_in_place_realloc(self):
        result = selected(fixture('real-ledger-lifetimes'))
        self.assertEqual(result['history_problems'], {})
        self.assertTrue(result['retention_complete'])
        self.assertEqual(result['window_allocated_bytes'], 715)
        self.assertEqual(result['window_freed_bytes'], 816)
        self.assertEqual(result['window_allocation_calls'], 9)
        self.assertEqual(result['window_release_calls'], 10)
        self.assertEqual(result['observed_stream_peak_bytes'], 390)
        self.assertEqual(result['retained_objects'], 0)
        # Negative window net is an earlier object's release, never negative retention.
        self.assertEqual(result['window_net_bytes'], -101)
        self.assertEqual(result['known_live_at_begin_bytes'], 101)
        self.assertEqual(result['known_live_at_end_bytes'], 0)

    def test_real_unmatched_teardown_frees_make_whole_process_retention_unknown(self):
        result = selected(fixture('real-ledger-lifetimes'), subset=False)
        self.assertEqual(result['history_problems'], {'unmatched free/realloc object': 4})
        for field in ('retained_objects', 'observed_stream_peak_bytes',
                      'window_freed_bytes', 'window_net_bytes'):
            self.assertIsNone(result[field])

    def test_raw_records_do_not_prove_tracking_was_continuously_enabled(self):
        result = selected(fixture('real-ledger-lifetimes'), continuous=False)
        self.assertFalse(result['retention_complete'])
        self.assertIsNone(result['retained_objects'])

    def test_count_and_continuity_do_not_prove_kernel_or_interposition_coverage(self):
        for kwargs in ({'history': False}, {'producer': False}):
            with self.subTest(kwargs=kwargs):
                result = selected(fixture('real-ledger-lifetimes'), **kwargs)
                self.assertFalse(result['retention_complete'])
                self.assertIsNone(result['retained_objects'])

    def test_saved_missing_duplicate_legacy_and_prewindow_histories_fail_closed(self):
        for name in ('missing-cross-allocation', 'duplicate-live-address',
                     'missing-old-realloc', 'missing-prewindow-history', 'null-realloc-result',
                     'zero-size-realloc', 'prewindow-exit', 'prewindow-fork'):
            with self.subTest(name=name):
                result = selected(fixture(name))
                self.assertFalse(result['retention_complete'])
                self.assertIsNone(result['retained_objects'])
                self.assertIsNone(result['observed_stream_peak_bytes'])
                self.assertIsNone(result['known_live_at_end_bytes'])
                self.assertIsNone(result['known_live_at_begin_bytes'])

    def test_object_boundaries_are_independent_of_input_chunk_boundaries(self):
        data = encoded('valid-event')
        events, report = unpack_chunks((data[i:i + 1] for i in range(len(data))), expected_events=1)
        self.assertEqual(len(events), 1)
        self.assertTrue(report['producer_count_verified'])

    def test_truncated_trailing_object_and_missing_complete_prefix_are_rejected(self):
        data = encoded('valid-event')
        with self.assertRaises(IncompleteCapture):
            unpack_chunks([data[:-1]])
        with self.assertRaises(IncompleteCapture):
            unpack_chunks([data], expected_events=2)

    def test_duplicate_keys_and_oversized_advertised_payloads_are_rejected(self):
        for name in ('duplicate-key', 'oversized-array', 'oversized-bin'):
            with self.subTest(name=name), self.assertRaises((ValueError, IncompleteCapture)):
                unpack_chunks([encoded(name)])

    def test_missing_producer_count_is_not_silently_marked_complete(self):
        _, report = unpack_chunks([encoded('valid-event')])
        self.assertFalse(report['producer_count_verified'])
        self.assertIn('unknown', report['capture_completeness'])


if __name__ == '__main__':
    unittest.main()
