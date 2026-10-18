#!/usr/bin/env python3
"""Regression guards for semantic report comparison and failure rejection."""
import runpy
import unittest
from pathlib import Path

HERE = Path(__file__).parent
FIXTURES = HERE / "fixtures" / "mimalloc-report"
REPORT = runpy.run_path(str(HERE / "mimalloc-report.py"))


class MimallocReportTests(unittest.TestCase):
    def test_nested_progress_named_payload_is_semantic(self):
        first = REPORT["read_run"](FIXTURES / "report-first.json")
        changed = REPORT["read_run"](FIXTURES / "report-nested-change.json")
        self.assertNotEqual(first["normalized_report_sha256"], changed["normalized_report_sha256"])

    def test_rejects_incomplete_and_malformed_json(self):
        for name in ("report-incomplete.json", "report-malformed.json", "report-empty.json"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                REPORT["read_run"](FIXTURES / name)

    def test_rejects_timeout_exit_even_when_both_variants_could_match(self):
        with self.assertRaises(ValueError):
            REPORT["read_run"](FIXTURES / "report-timeout.json")


if __name__ == "__main__":
    unittest.main()
