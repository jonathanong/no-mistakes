"""A version-only main carry never relabels retained compiler artifacts."""
import copy
import json
from pathlib import Path
import unittest

D = Path(__file__).resolve().parent


def validate(plan):
    carry = plan['baseline_main_version_carry']
    assert plan['baseline_actual_compiler'] == carry['baseline_actual_compiler'] == '099cfed26a23050aee6e9e8427af4de9b7eab4ef'
    assert plan['baseline_compiled_equivalent_main'] == carry['baseline_compiled_equivalent_main'] == '281673146341e6915c051aedfde6fd05ae6dd055'
    assert plan['baseline_equivalent_main'] == carry['admitted_current_main'] == '29e4439601c3d4a936a7f8a0ad300cb4de8abe7b'
    assert carry['compiled_baseline_version'] == '0.91.2' and carry['current_main_version'] == '0.91.3'
    assert carry['thirdparty_Cargo_lock_and_all_dependency_records_exact'] is True
    assert carry['all_other_paths_modes_types_blobs_exact'] is True
    assert carry['unchanged_git_entries'] == 22880
    assert len(carry['changed_files']) == 10
    assert all(row['transformation'] == 'Exact literal0.91.2→0.91.3 only' for row in carry['changed_files'])


class VersionCarry(unittest.TestCase):
    def test_exact_source_identity_and_negative_relabel(self):
        plan = json.loads((D / 'plan.json').read_text())
        validate(plan)
        for key in ['baseline_actual_compiler', 'baseline_compiled_equivalent_main', 'baseline_equivalent_main']:
            bad = copy.deepcopy(plan)
            bad[key] = 'unreviewed'
            with self.assertRaises(AssertionError):
                validate(bad)
        bad = copy.deepcopy(plan)
        bad['baseline_main_version_carry']['thirdparty_Cargo_lock_and_all_dependency_records_exact'] = False
        with self.assertRaises(AssertionError):
            validate(bad)
