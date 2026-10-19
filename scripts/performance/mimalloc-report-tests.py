#!/usr/bin/env python3
"""Regression guards for semantic report comparison and failure rejection."""
import json
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

    def test_numeric_memory_telemetry_is_not_a_semantic_difference(self):
        first = REPORT["read_run"](FIXTURES / "report-memory-first.json")
        changed = REPORT["read_run"](FIXTURES / "report-memory-changed.json")
        warning = REPORT["read_run"](FIXTURES / "report-memory-warning.json")
        self.assertEqual(first["stderr_semantics_sha256"], changed["stderr_semantics_sha256"])
        self.assertNotEqual(changed["stderr_semantics_sha256"], warning["stderr_semantics_sha256"])

    def test_nested_memory_named_json_payload_remains_semantic(self):
        first = REPORT["read_run"](FIXTURES / "report-nested-memory-first.json")
        changed = REPORT["read_run"](FIXTURES / "report-nested-memory-changed.json")
        self.assertNotEqual(first["normalized_report_sha256"], changed["normalized_report_sha256"])

    def test_rejects_incomplete_and_malformed_json(self):
        for name in ("report-incomplete.json", "report-malformed.json", "report-empty.json"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                REPORT["read_run"](FIXTURES / name)

    def test_censored_run_is_retained_as_ineligible(self):
        row = REPORT["read_run"](FIXTURES / "report-timeout.json")
        self.assertEqual(row["status"], "timeout")
        self.assertFalse(row["resources_eligible"])

    def test_unexpected_failure_cannot_masquerade_as_expected_control(self):
        failed = REPORT["read_run"](FIXTURES / "report-unexpected-exit.json")
        control = REPORT["read_run"](FIXTURES / "report-expected-control.json")
        self.assertEqual(failed["status"], "unexpected_exit")
        self.assertEqual(control["status"], "expected_failure")

    def group(self, name):
        return REPORT["summarize_group"](json.loads((FIXTURES / name).read_text()))

    def test_statistics_and_ratios_respect_each_physical_pair(self):
        group = self.group("paired-success.json")
        self.assertTrue(group["valid"])
        self.assertEqual(group["system"]["resources_eligible_count"], 4)
        self.assertEqual(group["system"]["metrics"]["wall_seconds"],
                         {"n": 4, "median": 60.0, "max": 200, "p95": 200})
        # Ratios must be computed within each host before aggregation. A ratio
        # of cross-host medians would be 50/60 and conceal the paired 0.7.
        self.assertAlmostEqual(group["paired_ratio"]["wall_seconds"]["median"], 0.7)
        self.assertEqual(group["paired_delta"]["wall_seconds"]["median"], -10.0)
        self.assertEqual(group["paired_ratio"]["user_seconds"]["n"], 3)
        self.assertEqual([pair["host_id"] for pair in group["pairs"]],
                         ["runner-0", "runner-1", "runner-2", "runner-3"])
        self.assertEqual(group["pairs"][0]["provenance"]["rayon_jobs"], 1)
        self.assertEqual(group["pairs"][0]["provenance"]["automatic_rayon_jobs"], 4)

    def test_same_unexpected_exit_in_both_variants_blocks_result(self):
        group = self.group("paired-unexpected-exit.json")
        self.assertFalse(group["valid"])
        self.assertEqual(group["mimalloc"]["statuses"]["unexpected_exit"], 1)
        self.assertEqual(group["mimalloc"]["resources_eligible_count"], 4)

    def test_explicit_expected_failure_control_is_classified(self):
        group = self.group("paired-expected-control.json")
        self.assertTrue(group["valid"])
        self.assertEqual(group["mimalloc"]["statuses"]["expected_failure"], 4)
        self.assertEqual(group["mimalloc"]["statuses"]["success"], 0)

    def test_timeouts_and_signals_do_not_pollute_resource_maxima(self):
        group = self.group("paired-censored.json")
        self.assertFalse(group["valid"])
        self.assertEqual(group["system"]["statuses"]["timeout"], 1)
        self.assertEqual(group["mimalloc"]["statuses"]["signal"], 1)
        self.assertEqual(group["system"]["resources_eligible_count"], 3)
        self.assertEqual(group["system"]["metrics"]["peak_rss_bytes"]["max"], 2000)
        self.assertEqual(group["paired_ratio"]["wall_seconds"]["n"], 3)

    def test_mismatched_host_report_command_and_missing_metric_fail_closed(self):
        for name in ("paired-wrong-host.json", "paired-wrong-report.json",
                     "paired-wrong-command.json", "paired-missing-host.json",
                     "paired-missing-metric.json", "paired-wrong-rayon.json",
                     "paired-wrong-lock.json", "paired-wrong-environment.json",
                     "paired-wrong-mimalloc-environment.json",
                     "paired-wrong-workload.json"):
            with self.subTest(name=name):
                self.assertFalse(self.group(name)["valid"])

    def test_duplicate_pair_does_not_replace_another_sample(self):
        with self.assertRaisesRegex(ValueError, "duplicate or missing"):
            self.group("paired-duplicate.json")

    def test_elapsed_deadline_is_censored_even_if_process_exited_zero(self):
        group = self.group("paired-deadline.json")
        self.assertFalse(group["valid"])
        self.assertEqual(group["system"]["statuses"]["timeout"], 1)
        self.assertEqual(group["system"]["resources_eligible_count"], 3)

    def test_all_36_native_cells_are_required(self):
        rows = json.loads((FIXTURES / "matrix-complete.json").read_text())
        summary = REPORT["summarize"](rows)
        self.assertEqual(len(summary), 36)
        self.assertTrue(all(group["valid"] for group in summary))
        with self.assertRaisesRegex(ValueError, "all 36"):
            REPORT["summarize"](json.loads((FIXTURES / "paired-success.json").read_text()))

    def test_empty_sample_set_fails_closed_before_provenance_checks(self):
        rows = json.loads((FIXTURES / "matrix-empty.json").read_text())
        with self.assertRaisesRegex(ValueError, "no measurement samples"):
            REPORT["summarize"](rows)

    def test_artifact_change_across_cells_is_rejected(self):
        rows = json.loads((FIXTURES / "matrix-wrong-binary.json").read_text())
        with self.assertRaisesRegex(ValueError, "identity changed"):
            REPORT["summarize"](rows)

    def test_mimalloc_environment_must_be_empty_and_identical(self):
        mismatched = self.group("paired-wrong-mimalloc-environment.json")
        self.assertFalse(mismatched["valid"])
        self.assertTrue(any("MIMALLOC_" in error for pair in mismatched["pairs"] for error in pair["errors"]))
        rows = json.loads((FIXTURES / "paired-success.json").read_text())
        for row in rows:
            row["mimalloc_environment"] = {}
        self.assertTrue(REPORT["summarize_group"](rows)["valid"])
        for row in rows:
            row["mimalloc_environment"] = {"MIMALLOC_PURGE_DELAY": "0"}
        matched = REPORT["summarize_group"](rows)
        self.assertFalse(matched["valid"])
        self.assertTrue(any("MIMALLOC_" in error for pair in matched["pairs"] for error in pair["errors"]))

    def test_preregistered_adoption_thresholds_have_no_results(self):
        rule = REPORT["ADOPTION_RULE"]
        self.assertEqual(rule["status"], "pre-registered")
        self.assertIsNone(rule["results"])
        self.assertFalse(rule["applied_to_summary"])
        self.assertEqual(rule["median_wall_ratio_max"], 1.00)
        self.assertEqual(rule["pair_wall_ratio_max"], 1.05)
        self.assertEqual(rule["median_peak_rss_ratio_at_jobs_1_max"], 1.00)
        self.assertEqual(rule["median_peak_rss_ratio_at_max_jobs_on_many_core_host_max"], 0.95)
        self.assertEqual(rule["report_parity"], "100%")

    def test_workload_tree_drift_between_hosts_is_rejected(self):
        rows = json.loads((FIXTURES / "matrix-wrong-workload.json").read_text())
        with self.assertRaisesRegex(ValueError, "workload commit or tree"):
            REPORT["summarize"](rows)


if __name__ == "__main__":
    unittest.main()
