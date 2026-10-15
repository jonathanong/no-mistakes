"""Exact-source local reports exercise policy; ARM release preflight repeats them."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import unittest

sys.path.append(str(Path(__file__).resolve().parents[1] / 'receiver-fact-native'))

from common import ExperimentFailure
from consumer_policy import validate_consumer, validate_expectations

FIXTURES = Path(__file__).with_name('test-fixtures')


class ConsumerPolicyTests(unittest.TestCase):
    def setUp(self):
        self.reports = {arm: json.loads((FIXTURES / f'consumer-{arm}.json').read_text())
                        for arm in ('baseline', 'candidate')}
        self.expected = json.loads((FIXTURES / 'consumer-expectations.json').read_text())
        self.signatures = {arm: {'report_sha256': self.digest(report), 'exit': 1}
                           for arm, report in self.reports.items()}

    @staticmethod
    def digest(report):
        return hashlib.sha256(json.dumps(report, sort_keys=True).encode()).hexdigest()

    def reject(self, code):
        with self.assertRaises(ExperimentFailure) as raised:
            validate_consumer(self.reports, self.signatures, self.expected)
        self.assertEqual(raised.exception.code, code)
        return raised.exception

    def test_transaction_only_to_query_is_approved(self):
        actual = validate_consumer(self.reports, self.signatures, self.expected)
        self.assertEqual(actual['baseline']['transaction_trace_count'], 1)
        self.assertEqual(actual['candidate']['query_trace_count'], 1)

    def test_old_zero_findings_assumption_is_rejected(self):
        self.reports['baseline']['rules'] = []
        self.signatures['baseline']['report_sha256'] = self.digest(self.reports['baseline'])
        self.expected['baseline']['report_sha256'] = self.signatures['baseline']['report_sha256']
        self.reject('consumer_pinned_report_mismatch')

    def test_exact_hash_is_required(self):
        self.signatures['candidate']['report_sha256'] = '0' * 64
        error = self.reject('consumer_pinned_report_mismatch')
        self.assertNotIn('message', json.dumps(error.details))
        self.assertNotIn('src/entry', json.dumps(error.details))

    def test_exact_trace_is_required_even_with_matching_hash(self):
        self.reports['candidate']['rules'][0]['message'] = 'different trace'
        digest = self.digest(self.reports['candidate'])
        self.signatures['candidate']['report_sha256'] = digest
        self.expected['candidate']['report_sha256'] = digest
        self.reject('consumer_pinned_report_mismatch')

    def test_success_exit_is_rejected(self):
        self.signatures['candidate']['exit'] = 0
        self.reject('consumer_pinned_report_mismatch')

    def test_non_rule_delta_is_rejected(self):
        self.reports['candidate']['warnings'] = ['unexpected warning']
        digest = self.digest(self.reports['candidate'])
        self.signatures['candidate']['report_sha256'] = digest
        self.expected['candidate']['report_sha256'] = digest
        self.reject('unapproved_consumer_other_field_delta')

    def test_missing_non_rule_key_is_rejected(self):
        del self.reports['candidate']['warnings']
        digest = self.digest(self.reports['candidate'])
        self.signatures['candidate']['report_sha256'] = digest
        self.expected['candidate']['report_sha256'] = digest
        self.reject('unapproved_consumer_other_field_delta')

    def test_unfrozen_expectations_fail_before_build(self):
        with self.assertRaisesRegex(ExperimentFailure, 'consumer_expectations_not_frozen'):
            validate_expectations(None)

    def test_boolean_exit_and_count_are_not_numeric_expectations(self):
        for key in ('exit', 'rule_count'):
            with self.subTest(key=key):
                expected = copy.deepcopy(self.expected)
                expected['baseline'][key] = True
                with self.assertRaisesRegex(ExperimentFailure, 'consumer_expectation_value_invalid'):
                    validate_expectations(expected)


if __name__ == '__main__':
    unittest.main()
