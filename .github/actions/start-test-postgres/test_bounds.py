from __future__ import annotations

import importlib.util
import subprocess
import unittest
from pathlib import Path

HERE = Path(__file__).parent
spec = importlib.util.spec_from_file_location("postgres_setup_bounds", HERE / "setup.py")
setup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(setup)


class BoundsTests(unittest.TestCase):
    def test_privileged_commands_bound_root_descendants_and_reserve_kill_grace(self):
        now = [0.0]
        calls = []

        def runner(command, **options):
            calls.append((command, options))
            self.assertEqual(
                command[:4], ["sudo", "timeout", "--signal=TERM", "--kill-after=5s"]
            )
            cap = int(command[4].removesuffix("s"))
            self.assertLessEqual(cap, 100)
            self.assertLessEqual(cap + 5, setup.PACKAGE_BUDGET_SECONDS - now[0])
            self.assertEqual(options["timeout"], cap + 6)
            self.assertEqual(command[5:], ["apt-get", "update"])
            now[0] += cap + 5
            return subprocess.CompletedProcess(command, 0, "", "")

        installer = setup.Installer(runner=runner, clock=lambda: now[0])
        try:
            while now[0] < setup.PACKAGE_BUDGET_SECONDS:
                installer._run(["apt-get", "update"], privileged=True)
            self.assertEqual(now[0], setup.PACKAGE_BUDGET_SECONDS)
            self.assertGreater(len(calls), 1)
            count = len(calls)
            with self.assertRaises(TimeoutError):
                installer._run(["apt-get", "update"], privileged=True)
            self.assertEqual(len(calls), count)
        finally:
            installer.close()

    def test_exhausted_deadline_prevents_starting_a_command(self):
        now = [0.0]
        calls = []
        installer = setup.Installer(
            runner=lambda *args, **kwargs: calls.append(args), clock=lambda: now[0]
        )
        try:
            now[0] = setup.PACKAGE_BUDGET_SECONDS
            with self.assertRaises(TimeoutError):
                installer._run(["apt-get", "update"], privileged=True)
            self.assertEqual(calls, [])
        finally:
            installer.close()

    def test_deadline_without_kill_grace_prevents_starting_a_command(self):
        now = [0.0]
        calls = []
        installer = setup.Installer(
            runner=lambda *args, **kwargs: calls.append(args), clock=lambda: now[0]
        )
        try:
            now[0] = setup.PACKAGE_BUDGET_SECONDS - 4
            with self.assertRaises(TimeoutError):
                installer._run(["apt-get", "update"], privileged=True)
            self.assertEqual(calls, [])
        finally:
            installer.close()

    def test_timeout_exit_fails_without_mirror_recovery_or_retry(self):
        calls = []

        def runner(command, **options):
            calls.append(command)
            return subprocess.CompletedProcess(command, 124, "", "")

        installer = setup.Installer(runner=runner, clock=lambda: 0.0)
        try:
            with self.assertRaises(setup.CommandFailure):
                installer._run_apt(["apt-get", "update"])
            self.assertEqual(len(calls), 1)
            self.assertFalse(installer.recovery_used)
        finally:
            installer.close()

    def test_outer_subprocess_timeout_fails_without_retry(self):
        calls = []

        def runner(command, **options):
            calls.append(command)
            raise subprocess.TimeoutExpired(command, options["timeout"])

        installer = setup.Installer(runner=runner, clock=lambda: 0.0)
        try:
            with self.assertRaises(subprocess.TimeoutExpired):
                installer._run_apt(["apt-get", "update"])
            self.assertEqual(len(calls), 1)
            self.assertFalse(installer.recovery_used)
        finally:
            installer.close()


if __name__ == "__main__":
    unittest.main()
