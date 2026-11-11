from __future__ import annotations

import contextlib
import importlib.util
import io
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).parent
FIXTURES = HERE / "fixtures"
spec = importlib.util.spec_from_file_location("postgres_setup_dns", HERE / "setup.py")
setup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(setup)
helper_spec = importlib.util.spec_from_file_location("postgres_setup_tests", HERE / "test_setup.py")
helper_tests = importlib.util.module_from_spec(helper_spec)
helper_spec.loader.exec_module(helper_tests)


class DNSRunner:
    def __init__(self, root: Path, apt: Path, probe_code: int, apt_timeout_without_dns: bool = False, kernel_timeout: bool = False):
        self.base = helper_tests.FakeInstallerRunner(install_server=True)
        self.base.server_root = root
        self.apt = apt
        self.probe_code = probe_code
        self.apt_timeout_without_dns = apt_timeout_without_dns
        self.kernel_timeout = kernel_timeout
        self.calls = []
        self.azure_transcript = (FIXTURES / "azure-update-timeout.txt").read_text()
        self.canonical_transcript = (FIXTURES / "azure-recovery-timeout.txt").read_text()

    def __call__(self, command, **options):
        self.calls.append(command)
        if "getent" in command:
            return subprocess.CompletedProcess(command, self.probe_code, "", "")
        if "apt-get" in command and "update" in command:
            mirror = (self.apt / "apt-mirrors.txt").read_text()
            if "mirrors.edge.kernel.org" in mirror:
                if self.kernel_timeout:
                    return subprocess.CompletedProcess(command, 124, "", "")
                return subprocess.CompletedProcess(command, 0, "Hit: apt sources refreshed\n", "")
            if "archive.ubuntu.com" in mirror and "azure.archive.ubuntu.com" not in mirror:
                return subprocess.CompletedProcess(command, 124, "", self.canonical_transcript)
            if self.apt_timeout_without_dns:
                return subprocess.CompletedProcess(command, 124, "", self.azure_transcript)
        return self.base(command, **options)


class DNSProbeTests(unittest.TestCase):
    def prepare(self, temp: str, probe_code: int, apt_timeout: bool = False):
        base = Path(temp)
        root = base / "usr"
        shutil.copytree(FIXTURES / "preinstalled/usr", root)
        shutil.rmtree(root / "lib/postgresql/18")
        shutil.rmtree(root / "share/postgresql/18/extension")
        apt = base / "etc/apt"
        shutil.copytree(FIXTURES / "etc/apt", apt)
        release = base / "os-release"
        shutil.copyfile(FIXTURES / "os-release", release)
        runner = DNSRunner(root, apt, probe_code, apt_timeout)
        installer = setup.Installer(runner=runner, root=root, sources_dir=apt, os_release=release, architecture="amd64")
        return installer, runner, apt

    def run_silently(self, function, *args):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return function(*args)

    def test_saved_apt_timeout_transcript_has_mirror_retries_but_no_dns_diagnostic(self):
        transcript = (FIXTURES / "azure-update-timeout.txt").read_text()
        self.assertIn("Ign:2 http://azure.archive.ubuntu.com", transcript)
        recovery = (FIXTURES / "azure-recovery-timeout.txt").read_text()
        self.assertIn("Ign:2 http://archive.ubuntu.com", recovery)
        self.assertNotRegex(transcript + recovery, r"Could not resolve|Temporary failure resolving")

    def test_unresolved_probe_rewrites_before_single_update_and_install(self):
        with tempfile.TemporaryDirectory() as temp:
            installer, runner, apt = self.prepare(temp, 2)
            try:
                self.run_silently(installer.install)
            finally:
                installer.close()
            self.assertTrue(installer.recovery_used)
            mirror = (apt / "apt-mirrors.txt").read_text()
            self.assertNotIn("azure.archive.ubuntu.com", mirror)
            self.assertNotIn("archive.ubuntu.com", mirror)
            self.assertIn("https://mirrors.edge.kernel.org/ubuntu/", mirror)
            update = next(i for i, command in enumerate(runner.calls) if "apt-get" in command and "update" in command)
            probe = next(i for i, command in enumerate(runner.calls) if "getent" in command)
            self.assertLess(probe, update)
            probe_command = runner.calls[probe]
            self.assertEqual(probe_command[:3], ["timeout", "--kill-after=1s", "5s"])
            self.assertEqual(probe_command[-2:], ["ahosts", "azure.archive.ubuntu.com"])
            self.assertEqual(sum("update" in command for command in runner.calls if "apt-get" in command), 1)
            self.assertEqual(sum("install" in command for command in runner.calls if "apt-get" in command), 1)

    def test_probe_timeout_is_one_dns_recovery_but_apt_timeout_without_probe_is_fail_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            installer, runner, apt = self.prepare(temp, 124, apt_timeout=True)
            try:
                self.run_silently(installer.install)
            finally:
                installer.close()
            self.assertTrue(installer.recovery_used)
            self.assertNotIn("azure.archive.ubuntu.com", (apt / "apt-mirrors.txt").read_text())
        with tempfile.TemporaryDirectory() as temp:
            installer, runner, apt = self.prepare(temp, 0, apt_timeout=True)
            try:
                with self.assertRaises(setup.CommandFailure):
                    self.run_silently(installer.install)
            finally:
                installer.close()
            self.assertFalse(installer.recovery_used)
            self.assertIn("azure.archive.ubuntu.com", (apt / "apt-mirrors.txt").read_text())
            self.assertEqual(sum("update" in command for command in runner.calls if "apt-get" in command), 1)

    def test_failed_recovery_destination_does_not_fallback_to_archive_or_retry(self):
        with tempfile.TemporaryDirectory() as temp:
            installer, runner, apt = self.prepare(temp, 2)
            runner.kernel_timeout = True
            try:
                with self.assertRaises(setup.CommandFailure):
                    self.run_silently(installer.install)
            finally:
                installer.close()
            mirror = (apt / "apt-mirrors.txt").read_text()
            self.assertIn("https://mirrors.edge.kernel.org/ubuntu/", mirror)
            self.assertNotIn("archive.ubuntu.com", mirror)
            self.assertNotIn("azure.archive.ubuntu.com", mirror)
            self.assertEqual(sum("update" in command for command in runner.calls if "apt-get" in command), 1)

    def test_other_probe_exit_fails_closed_before_apt(self):
        with tempfile.TemporaryDirectory() as temp:
            installer, runner, apt = self.prepare(temp, 1)
            try:
                with self.assertRaises(setup.CommandFailure):
                    self.run_silently(installer.install)
            finally:
                installer.close()
            self.assertFalse(installer.recovery_used)
            self.assertFalse(any("apt-get" in command for command in runner.calls))
            self.assertIn("azure.archive.ubuntu.com", (apt / "apt-mirrors.txt").read_text())

    def test_resolved_probe_preserves_mirror_and_does_not_enable_retry(self):
        with tempfile.TemporaryDirectory() as temp:
            installer, runner, apt = self.prepare(temp, 0, apt_timeout=True)
            try:
                with self.assertRaises(setup.CommandFailure):
                    self.run_silently(installer.install)
            finally:
                installer.close()
            self.assertFalse(installer.recovery_used)
            self.assertIn("azure.archive.ubuntu.com", (apt / "apt-mirrors.txt").read_text())
            self.assertEqual(sum("update" in command for command in runner.calls if "apt-get" in command), 1)


if __name__ == "__main__":
    unittest.main()
