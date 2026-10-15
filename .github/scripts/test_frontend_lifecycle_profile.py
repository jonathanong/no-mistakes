import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("profile", Path(__file__).with_name("frontend-lifecycle-profile.py"))
PROFILE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROFILE)


class ProfileIdentityTests(unittest.TestCase):
    def test_saved_sample_identity_cases(self):
        fixture = json.loads((ROOT / "fixtures/performance/frontend-lifecycle/profile-cases.json").read_text())
        for case in fixture["cases"]:
            with self.subTest(case=case["name"]):
                if case["valid"]:
                    # A logical Run ID may repeat; PID/profile/upload index may not.
                    PROFILE.validate_records(case["record"], fixture["previous"])
                else:
                    with self.assertRaises(AssertionError):
                        PROFILE.validate_records(case["record"], fixture["previous"])


if __name__ == "__main__":
    unittest.main()
