"""Reject the actual failed SQL argv closure before invoking any control."""
import copy
import json
from pathlib import Path
import unittest
from input_guard import declared_sql
D = Path(__file__).resolve().parent
class SQLInputs(unittest.TestCase):
    def test_actual_failed_both_arm_sql_paths_and_complete_closure(self):
        files = json.loads((D / 'payload-files.json').read_text())
        required = json.loads((D / 'required-inputs.json').read_text())
        controls = json.loads((D / 'controls.json').read_text())
        declared_sql(files, required, controls)
        old = json.loads((D / 'fixtures/actual-38986734511-SQL-argv.json').read_text())
        with self.assertRaisesRegex(AssertionError, 'Undeclared SQL argv input'):
            declared_sql(files, required, dict(controls, **old))
        for arm in ['A28', 'B28']:
            wrong = dict(controls, **{arm + '-run': old[arm + '-run']})
            with self.assertRaisesRegex(AssertionError, 'Undeclared SQL argv input'):
                declared_sql(files, required, wrong)
            for path in controls[arm + '-run'][1:]:
                with self.assertRaisesRegex(AssertionError, 'Undeclared SQL argv input'):
                    declared_sql([r for r in files if r['path'] != path], required, controls)
                changed = copy.deepcopy(required)
                next(r for r in changed if r['path'] == path)['roles'].remove(arm + ':SQL')
                with self.assertRaisesRegex(AssertionError, 'Unbound SQL argv role'):
                    declared_sql(files, changed, controls)
