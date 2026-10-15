"""Exact, reviewed per-arm expectations for the configured consumer control."""
import re

from common import require


def validate_expectations(expectations):
    require(isinstance(expectations, dict) and set(expectations) == {'baseline', 'candidate'},
            'consumer_expectations_not_frozen')
    for arm, trace in (('baseline', 'transaction-only'), ('candidate', 'query')):
        expected = expectations[arm]
        require(isinstance(expected, dict) and set(expected) ==
                {'report_sha256', 'exit', 'rule_count', 'trace'},
                'consumer_expectation_schema_invalid')
        require(isinstance(expected['report_sha256'], str) and
                re.fullmatch('[0-9a-f]{64}', expected['report_sha256']) is not None and
                type(expected['exit']) is int and expected['exit'] == 1 and
                type(expected['rule_count']) is int and expected['rule_count'] == 1 and
                expected['trace'] == trace, 'consumer_expectation_value_invalid')
    return expectations


def validate_consumer(reports, signatures, expectations):
    validate_expectations(expectations)
    actual = {}
    for arm in ('baseline', 'candidate'):
        expected = expectations[arm]
        report = reports[arm]
        rules = report['rules']
        transaction = sum('transaction client only' in rule['message'] for rule in rules)
        query = sum('queryHelper → read' in rule['message'] and
                    'transaction client only' not in rule['message'] for rule in rules)
        sites = sum(rule['rule'] == 'query-reached-per-item' and
                    rule['file'] == 'src/entry.mts' and rule['line'] == 5 for rule in rules)
        observed = dict(signatures[arm], rule_count=len(rules),
                        transaction_trace_count=transaction, query_trace_count=query,
                        expected_site_count=sites)
        # Only hashes, counts and fixed labels enter a public failure artifact.
        # The full normalized reports remain in the private semantic directory.
        require(observed['report_sha256'] == expected['report_sha256'] and
                observed['exit'] == expected['exit'] and
                observed['rule_count'] == expected['rule_count'] and sites == 1 and
                (transaction, query) == ((1, 0) if arm == 'baseline' else (0, 1)),
                'consumer_pinned_report_mismatch', arm=arm,
                actual=observed, expected=expected)
        actual[arm] = observed
    require(set(reports['baseline']) == set(reports['candidate']),
            'unapproved_consumer_other_field_delta')
    for key in set(reports['candidate']) - {'rules'}:
        require(reports['baseline'][key] == reports['candidate'][key],
                'unapproved_consumer_other_field_delta')
    return actual
