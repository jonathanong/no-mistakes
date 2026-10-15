"""Structured graph hashes retain target and callable identity."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import unittest

sys.path.append(str(Path(__file__).resolve().parents[1] / 'receiver-fact-native'))
from common import ExperimentFailure
from semantics import canonical_graph, verify_graph_hash


FIXTURE = Path(__file__).with_name('test-fixtures') / 'graph-canonical-snapshot.json'


def digest(graph):
    return hashlib.sha256(json.dumps(canonical_graph({'semantic': graph}), sort_keys=True).encode()).hexdigest()


class GraphSemanticsTests(unittest.TestCase):
    def setUp(self):
        self.graph = json.loads(FIXTURE.read_text())

    def test_structured_snapshot_is_order_independent(self):
        reordered = copy.deepcopy(self.graph)
        reordered['nodes'].reverse()
        self.assertEqual(digest(copy.deepcopy(self.graph)), digest(reordered))

    def test_target_name_change_changes_full_hash(self):
        changed = copy.deepcopy(self.graph)
        changed['sites'][0]['known'][0]['name'] = 'differentCallable'
        expected = digest(copy.deepcopy(self.graph))
        with self.assertRaisesRegex(ExperimentFailure, 'full_semantic_graph_changed'):
            verify_graph_hash(canonical_graph({'semantic': changed}), expected, 'receiver-order', 'candidate')

    def test_callable_identity_change_changes_full_hash(self):
        changed = copy.deepcopy(self.graph)
        old = 'Some(CallableId(1684))'
        new = 'Some(CallableId(1685))'
        changed['sites'][0]['known'][0]['node'] = changed['sites'][0]['known'][0]['node'].replace(old, new)
        expected = digest(copy.deepcopy(self.graph))
        with self.assertRaisesRegex(ExperimentFailure, 'full_semantic_graph_changed'):
            verify_graph_hash(canonical_graph({'semantic': changed}), expected, 'receiver-order', 'candidate')


if __name__ == '__main__':
    unittest.main()
