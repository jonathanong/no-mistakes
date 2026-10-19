"""Saved-fixture parity, pin, BCCB, censor, and provenance contract tests."""
import json
from pathlib import Path
import sys
import unittest

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / 'matched-arm'))
from inputs import (ARTIFACT_ID, ARTIFACT_ZIP_SHA256, FIXTURE_TAR_SHA256, CAP_SECONDS, CONFIG_SHA256,
                    CONSUMER, JOBS, SOURCES, DIAGNOSTICS, LEDGER,
                    MEASURE_MINUTES, JOB_MINUTES, EXPECTED_FINDINGS)
from common import assert_same_machine_identity
from ordering import (report_parity, run_bccb, sample_plan, sha256,
                      validate_fixture_contract, validate_repeats)

FIXTURE = ROOT / 'fixtures/performance/matched-arm/feature-delta'


class OrderingOnlyContract(unittest.TestCase):
    def test_workflow_builds_the_pins_and_downloads_its_own_pair(self):
        workflow = (ROOT / '.github/workflows/conflict-locations-ordering-arm.yml').read_text()
        self.assertIn('branches: [eval/2238-conflict-locations-arm]', workflow)
        for source in SOURCES.values():
            self.assertIn('ref: ' + source, workflow)
        self.assertEqual(workflow.count('name: conflict-locations-arm-clis'), 2)
        self.assertIn('scripts/conflict-locations-arm/build.py', workflow)
        self.assertIn('scripts/conflict-locations-arm/measure.py', workflow)

    def test_pins_and_ordering_config_hash_match_provenance(self):
        self.assertEqual(SOURCES, {
            'base': 'fbac9dc5857259ea8eff7820b0b0385ba2dea9ab',
            'candidate': '90d9ec30758770ec503c7f6b76e5c3f91772b9b1'})
        self.assertEqual(CONSUMER, 'f38a048f5eeff0438fc59a15e9eb742def575a36')
        self.assertEqual(ARTIFACT_ID, 12158768554)
        config = ROOT / 'fixtures/performance/current-verdict-arm/ordering-only.yml'
        proof = ROOT / 'fixtures/performance/current-verdict-arm/ordering-only-provenance.json'
        data = json.loads(proof.read_text())
        self.assertEqual(sha256(config), CONFIG_SHA256)
        self.assertEqual(data['orderingConfigSha256'], CONFIG_SHA256)
        self.assertEqual(data['selectedRuleApplications'], 1)
        self.assertEqual(data['originalRuleApplications'], 133)
        self.assertTrue(data['unchangedNonRulePayload'])

    def test_machine_identity_ignores_meminfo_but_rejects_runner_changes(self):
        folder = ROOT / 'fixtures/performance/current-verdict-arm'
        accepted = json.loads((folder / 'machine-identity-meminfo-diff.json').read_text())
        assert_same_machine_identity(accepted['base'], accepted['candidate'])
        for name in ('machine-identity-hostname-change.json',
                     'machine-identity-image-change.json',
                     'machine-identity-arch-change.json',
                     'machine-identity-missing-field.json'):
            row = json.loads((folder / name).read_text())
            with self.subTest(fixture=name), self.assertRaises(AssertionError):
                assert_same_machine_identity(row['base'], row['candidate'])

    def test_all_twelve_matrix_blocks_are_bccb_and_budgeted(self):
        blocks = [sample_plan(jobs, diagnostics, ledger)
                  for jobs in JOBS for diagnostics in DIAGNOSTICS for ledger in LEDGER]
        self.assertEqual(len(blocks), 12)
        self.assertTrue(all([row[2] for row in block] == ['b', 'c', 'c', 'b']
                            for block in blocks))
        self.assertTrue(all(not block[0][1] and not block[3][1] for block in blocks))
        self.assertEqual(CAP_SECONDS, 3600)
        self.assertEqual(MEASURE_MINUTES, 245)
        self.assertEqual(JOB_MINUTES, 260)
        self.assertLessEqual(4 * CAP_SECONDS + 300, MEASURE_MINUTES * 60)
        self.assertLessEqual(MEASURE_MINUTES + 12, JOB_MINUTES)

    def test_saved_reports_pass_real_bccb_parity_path_for_both_ledger_modes(self):
        from report import digest
        modes = [
            (False, ('baseline.json', 'valid.json', 'valid.json', 'baseline.json')),
            (True, ('baseline.json', 'ledger-candidate.json',
                    'ledger-candidate.json', 'baseline.json')),
        ]
        for ledger_on, reports in modes:
            rows = []
            output = ROOT / 'fixtures/performance'
            for index, (arm, report) in enumerate(zip('bccb', reports)):
                path = (ROOT / 'fixtures/performance/current-verdict-arm' / report
                        if report == 'ledger-candidate.json' else FIXTURE / report)
                rows.append({'eligible': True, 'arm': arm, 'exit': 1,
                             'semanticStderr': 'same',
                             'stdout': path.relative_to(output).as_posix(),
                             'stdoutSha256': digest(path.read_bytes())})
            result = validate_repeats(rows, output, ledger_on)
            self.assertTrue(result['rawRepeatHashesMatch'])
            self.assertTrue(result['semanticStderrMatch'])
            self.assertTrue(result['exitStatusMatch'])

    def test_first_pair_rejects_unexpected_delta_before_repeats(self):
        from report import digest
        output = ROOT / 'fixtures/performance'
        reports = {'b': FIXTURE / 'baseline.json',
                   'c': FIXTURE / 'changed-warning.json'}
        launched, saved, rows = [], [], []

        def sample(role, use_ledger, arm, ordinal):
            launched.append(arm)
            path = reports[arm]
            return {'eligible': True, 'arm': arm, 'exit': 1,
                    'semanticStderr': 'same',
                    'stdout': path.relative_to(output).as_posix(),
                    'stdoutSha256': digest(path.read_bytes())}

        with self.assertRaises((AssertionError, ValueError)):
            run_bccb(sample_plan('1', 'off', 'off'), sample, output, False, rows,
                     lambda rows: saved.append(list(rows)))
        self.assertEqual(launched, ['b', 'c'])
        self.assertEqual(len(saved[-1]), 2)
        self.assertEqual([row['arm'] for row in saved[-1]], ['b', 'c'])
        self.assertEqual([row['arm'] for row in rows], ['b', 'c'])

    def test_first_pair_accepts_and_completes_bccb(self):
        from report import digest
        output = ROOT / 'fixtures/performance'
        reports = [FIXTURE / 'baseline.json', FIXTURE / 'valid.json',
                   FIXTURE / 'valid.json', FIXTURE / 'baseline.json']
        launched, saved, rows = [], [], []

        def sample(role, use_ledger, arm, ordinal):
            launched.append(arm)
            path = reports[ordinal]
            return {'eligible': True, 'arm': arm, 'exit': 1,
                    'semanticStderr': 'same',
                    'stdout': path.relative_to(output).as_posix(),
                    'stdoutSha256': digest(path.read_bytes())}

        run_bccb(sample_plan('1', 'off', 'off'), sample, output, False, rows,
                        lambda current: saved.append(list(current)))
        self.assertEqual(launched, ['b', 'c', 'c', 'b'])
        self.assertEqual(len(rows), 4)
        self.assertEqual(len(saved[-1]), 4)
        validate_repeats(rows, output, False)

    def test_unbalanced_or_censored_blocks_fail_closed(self):
        for rows in ([], [{'eligible': True}] * 3,
                     [{'eligible': True}] * 3 + [{'eligible': False}]):
            with self.assertRaises(ValueError):
                validate_repeats(rows, FIXTURE, False)

    def test_flag_off_report_accepts_only_exact_two_findings(self):
        result = report_parity(FIXTURE / 'baseline.json', FIXTURE / 'valid.json', False)
        added = {(row['file'], row['line'], row['rule'], row.get('target'))
                 for row in result['featureDelta']['added_policy_findings']}
        self.assertEqual(added, EXPECTED_FINDINGS)
        for bad in ('changed-warning.json', 'missing-recovery.json',
                    'removed-existing.json', 'unexpected-recovery.json'):
            with self.subTest(bad=bad), self.assertRaises((AssertionError, ValueError)):
                report_parity(FIXTURE / 'baseline.json', FIXTURE / bad, False)

    def test_ledger_path_validates_and_then_compares_the_same_two_findings(self):
        path = ROOT / 'fixtures/performance/current-verdict-arm/ledger-candidate.json'
        result = report_parity(FIXTURE / 'baseline.json', path, True)
        self.assertTrue(result['ledgerValidated'])
        added = {(row['file'], row['line'], row['rule'], row.get('target'))
                 for row in result['featureDelta']['added_policy_findings']}
        self.assertEqual(added, EXPECTED_FINDINGS)
        with self.assertRaises((AssertionError, ValueError)):
            report_parity(FIXTURE / 'baseline.json',
                          ROOT / 'fixtures/performance/current-verdict-arm/ledger-changed-warning.json', True)

    def test_artifact_provenance_contract_rejects_changed_config_demand(self):
        proof = ROOT / 'fixtures/performance/current-verdict-arm/ordering-only-provenance.json'
        validate_fixture_contract(proof, FIXTURE_TAR_SHA256)
        with self.assertRaises(AssertionError):
            validate_fixture_contract(proof, ARTIFACT_ZIP_SHA256)
        with self.assertRaises(AssertionError):
            validate_fixture_contract(proof, '0' * 64)
        with self.assertRaises(AssertionError):
            validate_fixture_contract(
                ROOT / 'fixtures/performance/current-verdict-arm/ordering-only-wrong-demand.json',
                FIXTURE_TAR_SHA256)


if __name__ == '__main__':
    unittest.main()
