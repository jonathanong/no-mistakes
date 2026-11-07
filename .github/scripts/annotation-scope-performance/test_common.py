import json
import unittest
from pathlib import Path
from common import descendant_usage, distinct, parity, rss_kib, sha, verbose_metrics

SAVED = json.loads((Path(__file__).parent / "fixtures/validation.json").read_text())


class ValidationTests(unittest.TestCase):
    def test_verbose_counters_coexist_with_resource_and_hot_file_lines(self):
        fixtures = Path(__file__).parent / "fixtures"
        self.assertEqual(verbose_metrics((fixtures / "verbose.stderr").read_bytes()),
                         json.loads((fixtures / "verbose.json").read_text()))

    def test_sha_rejects_non_commit_inputs(self):
        self.assertEqual(sha(SAVED["validSha"]), SAVED["validSha"])
        for value in SAVED["invalidShas"]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                sha(value)

    def test_native_rss_units(self):
        for value in SAVED["rss"]:
            self.assertEqual(rss_kib(value["raw"], value["system"]), value["kib"])

    def test_identical_binary_is_not_a_comparison(self):
        distinct(SAVED["sameDigest"], SAVED["otherDigest"])
        with self.assertRaises(ValueError):
            distinct(SAVED["sameDigest"], SAVED["sameDigest"])

    def test_report_and_exit_must_both_match(self):
        parity(SAVED["report"], dict(SAVED["report"]))
        with self.assertRaises(ValueError):
            parity(SAVED["report"], SAVED["differentReport"])
        altered = dict(SAVED["report"], exit=0)
        with self.assertRaises(ValueError):
            parity(SAVED["report"], altered)

    def test_nested_process_usage_is_required_for_native_measurements(self):
        for value in SAVED["descendantUsage"]:
            with self.subTest(value=value):
                if value["valid"]:
                    descendant_usage(value["cpu"], value["rss"])
                else:
                    with self.assertRaises(ValueError):
                        descendant_usage(value["cpu"], value["rss"])


if __name__ == "__main__":
    unittest.main()
