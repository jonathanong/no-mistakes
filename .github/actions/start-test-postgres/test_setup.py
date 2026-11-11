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
spec = importlib.util.spec_from_file_location("postgres_setup", HERE / "setup.py")
setup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(setup)


def completed(command, code=0, stdout="", stderr=""):
    return subprocess.CompletedProcess(command, code, stdout, stderr)


class FakeInstallerRunner:
    def __init__(self, *, dns_update=False, dns_install=False, bad_fingerprint=False, install_server=True):
        self.calls = []
        self.dns_update = dns_update
        self.dns_install = dns_install
        self.bad_fingerprint = bad_fingerprint
        self.install_server = install_server
        self.did_fail_update = False
        self.did_fail_install = False
        self.server_root = None

    def __call__(self, command, **options):
        self.calls.append((command, options))
        if command[:2] == ["dpkg", "--print-architecture"]:
            return completed(command, stdout="amd64\n")
        if "--version" in command:
            return completed(command, stdout="postgres (PostgreSQL) 18.6\n")
        if "--show-keys" in command:
            fingerprint = "0" * 40 if self.bad_fingerprint else setup.PGDG_FINGERPRINT
            return completed(command, stdout=f"fpr:::::::::{fingerprint}:\n")
        if "curl" in command:
            shutil.copyfile(FIXTURES / "keys/apt.postgresql.org.asc", Path(command[command.index("-o") + 1]))
            return completed(command)
        if "install" in command and "-m" in command:
            shutil.copyfile(command[-2], command[-1])
            return completed(command)
        if "rm" in command:
            Path(command[-1]).unlink(missing_ok=True)
            return completed(command)
        if "apt-get" in command and "update" in command and self.dns_update and not self.did_fail_update:
            self.did_fail_update = True
            return completed(command, 100, stderr=(FIXTURES / "azure-dns-failure.txt").read_text())
        if "apt-get" in command and "install" in command and self.dns_install and not self.did_fail_install:
            self.did_fail_install = True
            return completed(command, 100, stderr=(FIXTURES / "azure-dns-failure.txt").read_text())
        if "apt-get" in command and "install" in command and self.install_server and self.server_root:
            source = FIXTURES / "preinstalled/usr/lib/postgresql/18"
            destination = self.server_root / "lib/postgresql/18"
            if destination.exists():
                shutil.rmtree(destination)
            shutil.copytree(source, destination)
            shutil.copytree(
                FIXTURES / "preinstalled/usr/share/postgresql/18/extension",
                self.server_root / "share/postgresql/18/extension",
                dirs_exist_ok=True,
            )
        return completed(command)


class SetupTests(unittest.TestCase):
    def silence(self, function, *args, **kwargs):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return function(*args, **kwargs)

    def tree(self, temp, *, installed=False):
        base = Path(temp)
        root = base / "usr"
        shutil.copytree(FIXTURES / "preinstalled/usr", root)
        if not installed:
            shutil.rmtree(root / "lib/postgresql/18")
            shutil.rmtree(root / "share/postgresql/18/extension")
        apt = base / "etc/apt"
        shutil.copytree(FIXTURES / "etc/apt", apt)
        release = base / "os-release"
        shutil.copyfile(FIXTURES / "os-release", release)
        return root, apt, release

    def installer(self, temp, runner, *, installed=False):
        root, apt, release = self.tree(temp, installed=installed)
        runner.server_root = root
        return setup.Installer(runner=runner, root=root, sources_dir=apt, os_release=release, architecture="amd64")

    def test_preflight_requires_executables_extensions_and_exact_server_major(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "usr"
            shutil.copytree(FIXTURES / "preinstalled/usr", root)
            self.assertTrue(setup.installed(root, lambda *a, **k: completed(a[0], stdout="postgres (PostgreSQL) 18.6\n")))
            self.assertFalse(setup.installed(root, lambda *a, **k: completed(a[0], stdout="postgres (PostgreSQL) 17.18.1\n")))
            (root / "share/postgresql/18/extension/citext.control").unlink()
            self.assertFalse(setup.installed(root, lambda *a, **k: completed(a[0], stdout="postgres (PostgreSQL) 18.6\n")))

    def test_mirror_rewrite_handles_runner_mirror_list_sources_and_comments(self):
        mirror = (FIXTURES / "apt-mirrors.txt").read_text()
        sources = (FIXTURES / "ubuntu.sources").read_text()
        listing = (FIXTURES / "ubuntu.list").read_text()
        rewritten_mirror = setup.replace_azure_source(mirror, "mirror-list")
        self.assertIn("https://mirrors.edge.kernel.org/ubuntu/ priority:1", rewritten_mirror)
        self.assertIn("http://archive.ubuntu.com/ubuntu/ priority:2", rewritten_mirror)
        rewritten = setup.replace_azure_source(sources, "sources")
        self.assertIn("URIs: https://mirrors.edge.kernel.org/ubuntu/", rewritten)
        self.assertIn("URIs: mirror+file:/etc/apt/apt-mirrors.txt", rewritten)
        self.assertNotIn("azure.archive.ubuntu.com", rewritten)
        all_mirrors = setup.replace_azure_source(mirror, "mirror-list", include_canonical=True)
        self.assertNotIn("archive.ubuntu.com", all_mirrors)
        self.assertEqual(all_mirrors.count("https://mirrors.edge.kernel.org/ubuntu/"), 2)
        self.assertEqual(setup.replace_azure_source((FIXTURES / "third-party.sources").read_text(), "sources", include_canonical=True), (FIXTURES / "third-party.sources").read_text())
        rewritten_list = setup.replace_azure_source(listing, "list")
        self.assertIn("# deb http://azure.archive.ubuntu.com/ubuntu noble main", rewritten_list)
        self.assertIn("deb https://mirrors.edge.kernel.org/ubuntu noble-updates main", rewritten_list)

    def test_preinstalled_server_skips_all_apt_work(self):
        with tempfile.TemporaryDirectory() as temp:
            runner = FakeInstallerRunner()
            installer = self.installer(temp, runner, installed=True)
            try:
                self.silence(installer.install)
            finally:
                installer.close()
            self.assertEqual(len(runner.calls), 1)
            self.assertIn("--version", runner.calls[0][0])

    def test_cold_install_uses_strict_apt_config_and_fails_closed_if_extensions_missing(self):
        with tempfile.TemporaryDirectory() as temp:
            runner = FakeInstallerRunner(install_server=False)
            installer = self.installer(temp, runner)
            apt_configs = []
            original = runner.__call__

            def inspect(command, **options):
                if "APT_CONFIG=" in " ".join(command):
                    config = next(part.split("=", 1)[1] for part in command if part.startswith("APT_CONFIG="))
                    apt_configs.append(Path(config).read_text())
                return original(command, **options)

            installer.runner = inspect
            try:
                with self.assertRaisesRegex(RuntimeError, "required citext/pg_stat_statements"):
                    self.silence(installer.install)
            finally:
                installer.close()
            apt_calls = [command for command, _ in runner.calls if "apt-get" in command]
            self.assertEqual(sum("update" in command for command in apt_calls), 1)
            self.assertEqual(sum("install" in command for command in apt_calls), 1)
            self.assertTrue(all('APT::Update::Error-Mode "any"' in config for config in apt_configs))
            self.assertTrue(all('Acquire::http::Timeout "10"' in config for config in apt_configs))

    def test_update_dns_failure_rewrites_sources_then_retries_update_and_install(self):
        with tempfile.TemporaryDirectory() as temp:
            runner = FakeInstallerRunner(dns_update=True)
            installer = self.installer(temp, runner)
            try:
                self.silence(installer.install)
            finally:
                installer.close()
            apt_calls = [command for command, _ in runner.calls if "apt-get" in command]
            self.assertEqual(["update" if "update" in command else command[-1] for command in apt_calls], ["update", "update", "postgresql-18"])
            apt = Path(temp) / "etc/apt"
            self.assertNotIn("azure.archive.ubuntu.com", (apt / "apt-mirrors.txt").read_text())
            self.assertNotIn("azure.archive.ubuntu.com", (apt / "sources.list.d/ubuntu.sources").read_text())

    def test_install_dns_failure_refreshes_indexes_before_one_install_retry(self):
        with tempfile.TemporaryDirectory() as temp:
            runner = FakeInstallerRunner(dns_install=True)
            installer = self.installer(temp, runner)
            try:
                self.silence(installer.install)
            finally:
                installer.close()
            apt_calls = [command for command, _ in runner.calls if "apt-get" in command]
            self.assertEqual(["update" if "update" in command else command[-1] for command in apt_calls], ["update", "postgresql-18", "update", "postgresql-18"])

    def test_non_azure_failure_and_second_dns_failure_fail_closed(self):
        for error_text, expected_calls in [
            ("Temporary failure resolving apt.postgresql.org", 1),
            ((FIXTURES / "azure-dns-failure.txt").read_text(), 2),
        ]:
            with self.subTest(error=error_text[:20]), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                apt = root / "etc/apt"
                shutil.copytree(FIXTURES / "etc/apt", apt)
                calls = []

                def runner(command, **options):
                    calls.append(command)
                    if "install" in command and "-m" in command:
                        shutil.copyfile(command[-2], command[-1])
                        return completed(command)
                    if "apt-get" in command:
                        return completed(command, 100, stderr=error_text)
                    return completed(command)

                installer = setup.Installer(runner=runner, root=root / "usr", sources_dir=apt)
                try:
                    with self.assertRaises(setup.CommandFailure):
                        self.silence(installer._run_apt, ["apt-get", "update"])
                finally:
                    installer.close()
                self.assertEqual(sum("apt-get" in command for command in calls), expected_calls)

    def test_pdg_source_uses_verified_official_key_suite_architecture_and_bounded_curl(self):
        with tempfile.TemporaryDirectory() as temp:
            installer = self.installer(temp, FakeInstallerRunner())
            root = Path(temp) / "usr"
            (root / "share/postgresql-common/pgdg/apt.postgresql.org.asc").unlink()
            installer.runner = FakeInstallerRunner()
            installer.runner.server_root = root
            calls = []
            base_runner = installer.runner

            def runner(command, **options):
                calls.append((command, options))
                return base_runner(command, **options)

            installer.runner = runner
            try:
                self.silence(installer._configure_pgdg)
            finally:
                installer.close()
            source = (Path(temp) / "etc/apt/sources.list.d/pgdg.sources").read_text()
            self.assertIn("URIs: https://apt.postgresql.org/pub/repos/apt", source)
            self.assertIn("Suites: noble-pgdg", source)
            self.assertIn("Architectures: amd64", source)
            self.assertIn("Signed-By:", source)
            curl = next(command for command, _ in calls if "curl" in command)
            self.assertEqual(curl[curl.index("--max-time") + 1], "25")
            self.assertLessEqual(next(options["timeout"] for command, options in calls if "curl" in command), 106)

    def test_bad_downloaded_key_fingerprint_fails_before_repository_install(self):
        with tempfile.TemporaryDirectory() as temp:
            installer = self.installer(temp, FakeInstallerRunner(bad_fingerprint=True))
            root = Path(temp) / "usr"
            (root / "share/postgresql-common/pgdg/apt.postgresql.org.asc").unlink()
            calls = []
            original = installer.runner

            def runner(command, **options):
                calls.append(command)
                return original(command, **options)

            installer.runner = runner
            try:
                with self.assertRaisesRegex(RuntimeError, "fingerprint did not match"):
                    self.silence(installer._configure_pgdg)
            finally:
                installer.close()
            self.assertTrue(any("curl" in command for command in calls))
            self.assertFalse(any("pgdg.sources" in command for command in calls))



if __name__ == "__main__":
    unittest.main()
