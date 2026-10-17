"""Reject unresolved children with identities and without signaling them."""
import ast
import json
from pathlib import Path
import unittest
from unittest.mock import patch

import subreaper

D = Path(__file__).resolve().parent


class OwnershipDiagnostics(unittest.TestCase):
    def test_saved_actual_adopted_child_is_only_reported(self):
        evidence = json.loads((D / 'fixtures/clock-observed-identities.json').read_text())
        child = next(row['observed_children'][0] for row in evidence if not row['corrected'])
        with patch.object(subreaper, 'children', return_value=[child]), patch.object(subreaper.os, 'kill') as kill, patch.object(subreaper.os, 'waitpid') as wait:
            with self.assertRaises(AssertionError) as raised:
                subreaper.require_empty()
            reason, identities = raised.exception.args[0]
            self.assertIn('unresolved', reason)
            self.assertEqual(identities, [{key: child[key] for key in ['pid', 'ppid', 'sid', 'starttime']}])
            kill.assert_not_called()
            wait.assert_not_called()
