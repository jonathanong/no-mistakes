import importlib.util
import json
from pathlib import Path
import re
import shlex
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("profile", Path(__file__).with_name("frontend-lifecycle-profile.py"))
PROFILE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROFILE)


class ProfileIdentityTests(unittest.TestCase):
    def test_config_relative_commands_and_fixture_paths(self):
        fixture = json.loads((ROOT / "fixtures/performance/frontend-lifecycle/config-paths.json").read_text())
        config_directory = ROOT / fixture["configDirectory"]
        for case in fixture["cases"]:
            with self.subTest(case=case["name"]):
                # Runner5.4 resolves ConfigTargets against the config directory,
                # including its default when working-directory is omitted.
                directory = (config_directory / case["options"].get("working-directory", ".")).resolve()
                self.assertEqual(directory, (ROOT / case["expectedDirectory"]).resolve())
                self.assertEqual((directory / fixture["script"]).is_file(), case["pathsExist"])
                self.assertEqual((directory / fixture["fixture"]).is_dir(), case["pathsExist"])

        config = (ROOT / ".github/frontend-lifecycle-smoke.yml").read_text()
        # The flow mapping is JSON-compatible YAML so this test needs no parser
        # dependency. Keep this assertion tied to the real selected config.
        declarations = re.findall(r"^options: (\{.*\})$", config, re.MULTILINE)
        self.assertEqual(len(declarations), 1)
        options = json.loads(declarations[0])
        directory = (config_directory / options["working-directory"]).resolve()
        self.assertEqual(directory, ROOT)
        commands = re.findall(r"^\s+entrypoint: (.+)$", config, re.MULTILINE)
        self.assertEqual(len(commands), 4)
        for index, command in enumerate(commands):
            arguments = shlex.split(command)
            self.assertEqual(arguments[0], "bash")
            self.assertTrue((directory / arguments[1]).is_file())
            self.assertEqual(arguments[2:], [str(index + 1), fixture["orderedModes"][index]])
        self.assertTrue((directory / fixture["fixture"]).is_dir())

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
