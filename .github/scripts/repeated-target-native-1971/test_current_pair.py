"""Current-main gates use the branch ref, including after the build completes."""
import json
from pathlib import Path
from unittest import TestCase, main
from unittest.mock import patch

import pair
import report
from common import ExperimentFailure
from report import completion_pair, summary


class CurrentPairTests(TestCase):
    def test_driver_report_precedes_shared_helper_report(self):
        self.assertEqual(Path(report.__file__).resolve().parent, Path(pair.__file__).resolve().parent)

    def responses(self, main_revision=None, head=None, merge=None):
        return [
            json.dumps({'object': {'sha': main_revision or pair.SPEC['baseline']['revision']}}),
            json.dumps({'head': {'sha': head or pair.SPEC['candidate']['pr_head']},
                        'merge_commit_sha': merge or pair.SPEC['candidate']['revision'],
                        'mergeable': True,
                        'base': {'sha': 'stale-pull-request-base'}}),
        ]

    def test_actual_branch_ref_allows_a_stale_pr_base(self):
        with patch.object(pair, 'command', side_effect=self.responses()) as command:
            _, revision = pair.current_pair('/unused')
        self.assertEqual(revision, pair.SPEC['baseline']['revision'])
        self.assertIn('/git/ref/heads/main', command.call_args_list[0].args[0][-1])

    def test_changed_actual_main_fails_closed(self):
        with patch.object(pair, 'command', side_effect=self.responses(main_revision='new-main')):
            with self.assertRaisesRegex(ExperimentFailure, 'actual_main_ref_changed'):
                pair.current_pair('/unused')

    def test_changed_head_fails_closed(self):
        with patch.object(pair, 'command', side_effect=self.responses(head='new-head')):
            with self.assertRaisesRegex(ExperimentFailure, 'pr_head_changed'):
                pair.current_pair('/unused')

    def test_changed_synthetic_merge_fails_closed(self):
        with patch.object(pair, 'command', side_effect=self.responses(merge='new-merge')):
            with self.assertRaisesRegex(ExperimentFailure, 'pr_synthetic_merge_changed'):
                pair.current_pair('/unused')

    def test_final_drift_keeps_retained_results_but_marks_gate_ineligible(self):
        with patch.object(pair, 'current_pair', side_effect=ExperimentFailure('actual_main_ref_changed')):
            result = completion_pair('/unused')
        self.assertFalse(result['eligible'])
        spec = {'baseline': {'revision': 'A'}, 'candidate': {'revision': 'B'},
                'diagnostic_parent': 'HEAD', 'workloads': [{'name': 'retained-control'}]}
        assessments = [{'workload': 'retained-control', 'status': 'higher cost observed',
                        'phase': 'long', 'samples': 32}]
        markdown, incomplete = summary(spec, assessments, {'retained-control': 32}, result)
        self.assertTrue(incomplete)
        self.assertIn('historical or unverified; ineligible', markdown)
        self.assertIn('| retained-control | higher cost observed |', markdown)
        self.assertIn('| long / 32 | 32 |', markdown)
        self.assertEqual(assessments[0]['samples'], 32)

    def test_unavailable_final_check_never_exposes_raw_error(self):
        with patch.object(pair, 'current_pair', side_effect=RuntimeError('/private/raw/path')):
            result = completion_pair('/unused')
        self.assertFalse(result['eligible'])
        self.assertEqual(result['error_code'], 'final_pair_check_failed')
        self.assertNotIn('/private', json.dumps(result))


if __name__ == '__main__':
    main()
