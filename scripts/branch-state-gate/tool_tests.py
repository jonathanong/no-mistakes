"""Saved guards for native tool identity and directory-toolchain shadowing."""
import json
import os
from pathlib import Path
import unittest
from unittest.mock import patch

from tool_build import pinned_environment
from tool_verify import validate


class ToolProvisionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        fixture = Path(__file__).resolve().parents[2] / 'fixtures/performance/branch-state-gate/tool-provisioning-cases.json'
        cls.cases = json.loads(fixture.read_text())

    def validate(self, metadata):
        valid = self.cases['valid']
        validate(metadata, valid['binary_sha256'], valid['source'], metadata['version_output'])

    def test_accepts_exact_native_default_feature_tool(self):
        self.validate(self.cases['valid'])

    def test_rejects_incomplete_tampered_or_wrong_toolchain_artifacts(self):
        for change in self.cases['rejected']:
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.validate(self.cases['valid'] | change)

    def test_true_toolchain_pin_overrides_directory_stable_without_mutating_parent(self):
        inherited = self.cases['shadowed_environment']
        with patch.dict(os.environ, inherited):
            environment = pinned_environment()
            self.assertEqual(environment['RUSTUP_TOOLCHAIN'], '1.99.1')
            self.assertNotIn('RUSTFLAGS', environment)
            self.assertNotIn('CARGO_ENCODED_RUSTFLAGS', environment)
            self.assertEqual(os.environ['RUSTUP_TOOLCHAIN'], inherited['RUSTUP_TOOLCHAIN'])
