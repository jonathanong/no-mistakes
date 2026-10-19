"""Saved regression for the candidate build failure in ARM run39323462603."""
import json
from pathlib import Path
import unittest

from build_size import parse_size

FIXTURES = Path(__file__).resolve().parents[3] / 'fixtures/performance/matched-arm/build-size'


class BuildSizeTests(unittest.TestCase):
    def saved(self, name):
        row = json.loads((FIXTURES / (name + '.json')).read_text())
        return parse_size(row['code'], row['stdout'], row['stderr'], '/owned/target')

    def test_disappearing_compiler_archive_keeps_the_valid_byte_count(self):
        # GNU du still reports the owned-root total after a child disappears.
        self.assertEqual(self.saved('removed-temp'), 4096)

    def test_stable_scan_reports_bytes(self):
        self.assertEqual(self.saved('stable'), 4096)

    def test_other_scan_failures_and_invalid_output_remain_fatal(self):
        for name in ['permission', 'malformed', 'failed']:
            with self.subTest(fixture=name), self.assertRaises(ValueError):
                self.saved(name)


if __name__ == '__main__':
    unittest.main()
