"""Exercise the actual runtime guard without importing the hosted entrypoint."""
import ast
import copy
import json
from pathlib import Path
import unittest

D = Path(__file__).resolve().parent


def validator():
    source = ast.parse((D / 'hosted.py').read_text())
    functions = [node for node in source.body if isinstance(node, ast.FunctionDef)
                 and node.name == 'verify_transport_inventory']
    assert len(functions) == 1
    namespace = {}
    exec(compile(ast.Module(body=functions, type_ignores=[]), 'actual-inventory-validator', 'exec'), namespace)
    return namespace['verify_transport_inventory']


class InventoryTests(unittest.TestCase):
    def setUp(self):
        self.spec = json.loads((D / 'transport-REVIEWED.json').read_text())
        self.verify = validator()

    def test_actual_complete105_transport(self):
        self.assertEqual(len(self.spec['files']), 105)
        self.verify(self.spec)

    def test_duplicate_path_rejects(self):
        changed = copy.deepcopy(self.spec)
        changed['files'][-1]['path'] = changed['files'][0]['path']
        with self.assertRaises(AssertionError):
            self.verify(changed)

    def test_missing_path_rejects(self):
        changed = copy.deepcopy(self.spec)
        changed['files'].pop()
        with self.assertRaises(AssertionError):
            self.verify(changed)
