"""Saved actual prerequisite failure and pure fail-closed admission states."""
import copy
import json
from pathlib import Path
import unittest
from dependency_guard import verify

D=Path(__file__).resolve().parent


class DependencyTests(unittest.TestCase):
    def test_missing_dependency_is_the_observed_failure(self):
        red=(D/'fixtures/declaration-prerequisite-MISSING.txt').read_text()
        green=(D/'fixtures/declaration-prerequisite-GREEN.txt').read_text()
        self.assertIn("Cannot find module '@storybook/react'",red)
        self.assertIn('# pass 40\n',red);self.assertIn('# fail 2\n',red)
        self.assertIn('# pass 42\n',green);self.assertIn('# fail 0\n',green)
        self.assertIn('# skipped 0\n',green)

    def test_failed_or_unbound_prerequisite_rejected(self):
        proof=json.loads((D/'fixtures/dependency-complete.json').read_text())
        verify(proof,'saved-unit-head','saved-unit-lock')
        for key,bad in [('complete',False),('candidate','other'),('lock_sha256','other'),
                        ('no_lifecycle_scripts',False),('no_native_build',False),
                        ('node_version','v24'),('pnpm_version','other')]:
            changed=copy.deepcopy(proof);changed[key]=bad
            with self.assertRaises(AssertionError):verify(changed,'saved-unit-head','saved-unit-lock')
        with self.assertRaises(AssertionError):verify({},'saved-unit-head','saved-unit-lock')
