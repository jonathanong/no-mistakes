"""The actual failed 6ad closure must reject before any native control starts."""
import copy
import json
from pathlib import Path
import unittest
from input_guard import declared

D = Path(__file__).resolve().parent


class InputTests(unittest.TestCase):
    def test_complete_and_actual_missing_input_closures(self):
        current = json.loads((D / 'payload-files.json').read_text())
        required = json.loads((D / 'required-inputs.json').read_text())
        declared(current, required)
        old = json.loads((D / 'fixtures/6ad-before-extra-controls-payload.json').read_text())
        with self.assertRaisesRegex(AssertionError, 'Undeclared required control input'):
            declared(old, required)
        missing = next(row['path'] for row in required if row['kind'] == 'file' and 'mutable-root-single-fields.mts' in row['path'])
        with self.assertRaisesRegex(AssertionError, 'Undeclared required control input'):
            declared([row for row in current if row['path'] != missing], required)

    def test_directory_inventory_cannot_drop_or_add_inputs(self):
        files = json.loads((D / 'payload-files.json').read_text())
        required = json.loads((D / 'required-inputs.json').read_text())
        for mode in ['drop', 'extra']:
            changed = copy.deepcopy(required)
            directory = next(row for row in changed if row['kind'] == 'directory')
            if mode == 'drop':
                directory['files'].pop()
            else:
                directory['files'].append(directory['path'] + '/undeclared.ts')
            with self.assertRaisesRegex(AssertionError, 'Incomplete required directory inventory'):
                declared(files, changed)
