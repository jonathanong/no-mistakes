"""Saved current controls protect exact labels, tokens and actual API matrix."""
import copy
import json
from pathlib import Path
import unittest
from full_control import verify_tokens, verify_raw
from unittest.mock import Mock
from readback import verify_api

D = Path(__file__).resolve().parent


class CurrentControls(unittest.TestCase):
    def test_saved_actual_labels_and_semantic_drift(self):
        goldens = json.loads((D / 'unordered-control-goldens.json').read_text())
        for arm in ['A28', 'B28']:
            for index in [10, 26]:
                raw = (D / 'fixtures' / f'{arm}-{index}.data').read_bytes()
                label = goldens[arm][str(index)]['label']
                verify_tokens(raw, arm, index, label, goldens)
                with self.assertRaisesRegex(AssertionError, 'undeclared unordered control'):
                    verify_tokens(raw, arm, index, 'wrong-label', goldens)
                with self.assertRaisesRegex(AssertionError, 'undeclared unordered control'):
                    verify_tokens(raw, arm, 9, label, goldens)
                with self.assertRaisesRegex(AssertionError, 'full tokens or statistics changed'):
                    verify_tokens(raw.replace(b'TsFileFacts', b'ChangedFacts', 1), arm, index, label, goldens)

    def test_actual_raw_length_is_not_hidden_by_token_equivalence(self):
        controls = json.loads((D / 'controls.json').read_text())
        raw = (D / 'fixtures/A28-10.data').read_bytes()
        path = Mock()
        path.exists.return_value = True
        # Whitespace preserves tokens but changes the required actual byte count.
        path.read_bytes.return_value = raw + b' '
        with self.assertRaisesRegex(AssertionError, 'raw byte length changed'):
            verify_raw(path, 'A28', 10, controls['A28'][10])

    def test_exact_actual_api_matrix(self):
        counts = json.loads((D / 'api-counts.json').read_text())
        records = [dict(arm=arm, script=name,
                        counts=dict(tests=count, **{'pass': count}, fail=0, skipped=0))
                   for arm in ['baseline', 'candidate'] for name, count in counts.items()
                   if not (arm == 'baseline' and name == 'mutable-receiver-api.test.js')]
        assert len(records) == 23
        verify_api(dict(complete=True, records=records), counts)
        for bad in [records[:-1], records + [records[0]]]:
            with self.assertRaises(AssertionError):
                verify_api(dict(complete=True, records=bad), counts)
        bad = copy.deepcopy(records)
        bad[0]['counts']['skipped'] = 1
        with self.assertRaises(AssertionError):
            verify_api(dict(complete=True, records=bad), counts)


if __name__ == '__main__':
    unittest.main()
