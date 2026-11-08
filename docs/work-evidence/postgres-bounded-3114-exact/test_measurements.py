import errno
import json
import os
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

from measurement_support import launch_binaries, validate_sample

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "test-cases/performance/postgres-bounded-3114"


class MeasurementTests(unittest.TestCase):
    def test_sample_validation(self):
        cases = json.loads((FIXTURE / "harness-samples.json").read_text())
        for case in cases:
            with self.subTest(case=case["name"]):
                report = {key: [] for key in (
                    "rules", "advisories", "warnings", "codebase", "integration", "queues", "react"
                )}
                report["rules"] = [{}] * case["findings"]
                if "extra" in case:
                    report[case["extra"]] = [{}]
                args = (report, case["status"], case.get("stderr", "").encode(), case["expected"])
                if case["valid"]:
                    validate_sample(*args)
                else:
                    with self.assertRaisesRegex(ValueError, "unexpected"):
                        validate_sample(*args)

    def test_private_hard_link_and_cross_filesystem_copy(self):
        source = FIXTURE / "harness-executable.sh"
        original_link = os.link
        for cross_filesystem in (False, True):
            with self.subTest(cross_filesystem=cross_filesystem):
                linked = []
                def attempt_link(source, destination):
                    if cross_filesystem:
                        linked.append(False)
                        raise OSError(errno.EXDEV, "cross device")
                    try:
                        original_link(source, destination)
                    except OSError:
                        linked.append(False)
                        raise
                    linked.append(True)
                with patch("measurement_support.os.link", side_effect=attempt_link):
                    directory, launches = launch_binaries({"A": source}, "nm-harness-test-")
                with directory:
                    path = Path(launches["A"])
                    self.assertEqual(path.name, "no-mistakes")
                    self.assertEqual(Path(directory.name).stat().st_mode & 0o777, 0o700)
                    self.assertEqual(path.read_bytes(), source.read_bytes())
                    self.assertEqual(path.stat().st_mode, source.stat().st_mode)
                    self.assertEqual(path.samefile(source), linked == [True])
                    self.assertEqual(subprocess.check_output([path]), b"saved executable\n")

    def test_other_launch_errors_are_not_hidden(self):
        source = FIXTURE / "harness-executable.sh"
        with patch("measurement_support.os.link", side_effect=PermissionError(errno.EACCES, "denied")):
            with self.assertRaises(PermissionError):
                launch_binaries({"A": source}, "nm-harness-test-")

    def test_invalid_modes_fail_before_binary_setup(self):
        for name in ("measure.py", "measure_wide.py"):
            result = subprocess.run(["python3", "-B", str(Path(__file__).with_name(name)), "unsupported"], capture_output=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn(b"invalid choice", result.stderr)


if __name__ == "__main__":
    unittest.main()
