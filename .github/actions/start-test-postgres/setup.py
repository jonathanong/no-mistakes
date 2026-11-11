#!/usr/bin/env python3
"""Install PostgreSQL 18 for catalog fixtures with bounded APT recovery."""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Callable

APT_SOURCES = Path("/etc/apt")
PGDG_KEY = "https://www.postgresql.org/media/keys/ACCC4CF8.asc"
PGDG_FINGERPRINT = "B97B0AFCAA1A47F044F244A07FCC7D46ACCC4CF8"
PACKAGE_BUDGET_SECONDS = 220
AZURE_DNS_ERROR = re.compile(
    r"(?:Could not resolve|Temporary failure resolving|Name or service not known)[^\n]*"
    r"azure\.archive\.ubuntu\.com",
    re.IGNORECASE,
)
AZURE_HOST = re.compile(r"(?<![A-Za-z0-9.-])azure\.archive\.ubuntu\.com(?![A-Za-z0-9.-])")


class CommandFailure(RuntimeError):
    def __init__(self, command: list[str], result: subprocess.CompletedProcess[str]):
        super().__init__(f"command failed ({result.returncode}): {' '.join(command)}\n{result.stdout}{result.stderr}")
        self.transcript = result.stdout + result.stderr
        self.returncode = result.returncode


def replace_azure_source(text: str, kind: str) -> str:
    """Rewrite only active APT source URLs or Ubuntu mirror-list entries."""
    output = []
    in_uri_field = False
    for line in text.splitlines(keepends=True):
        stripped = line.lstrip()
        if kind == "mirror-list":
            active = not stripped.startswith("#")
        elif kind == "sources":
            if line[:1].isspace():
                active = in_uri_field and not stripped.startswith("#")
            else:
                in_uri_field = stripped.startswith("URIs:")
                active = in_uri_field
        else:
            active = not stripped.startswith("#")
        output.append(AZURE_HOST.sub("archive.ubuntu.com", line) if active else line)
    return "".join(output)


def installed(root: Path = Path("/usr"), run: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run) -> bool:
    pg_bin = root / "lib/postgresql/18/bin"
    extension_dir = root / "share/postgresql/18/extension"
    required = [pg_bin / name for name in ("postgres", "initdb", "pg_ctl")]
    required += [extension_dir / f"{name}.control" for name in ("citext", "pg_stat_statements")]
    if not all(path.is_file() and (not path.name in {"postgres", "initdb", "pg_ctl"} or os.access(path, os.X_OK)) for path in required):
        return False
    result = run([str(pg_bin / "postgres"), "--version"], capture_output=True, text=True, timeout=5)
    return result.returncode == 0 and re.search(r"^postgres \(PostgreSQL\) 18\.", result.stdout, re.MULTILINE) is not None


class Installer:
    def __init__(
        self,
        runner: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
        clock: Callable[[], float] = time.monotonic,
        root: Path = Path("/usr"),
        sources_dir: Path = APT_SOURCES,
        os_release: Path = Path("/etc/os-release"),
        architecture: str | None = None,
    ):
        self.runner, self.clock = runner, clock
        self.root, self.sources_dir, self.os_release = root, sources_dir, os_release
        self.architecture = architecture
        self.deadline = clock() + PACKAGE_BUDGET_SECONDS
        self.recovery_used = False
        self.temp = tempfile.TemporaryDirectory(prefix="nm-pg-apt-")
        self.apt_config = Path(self.temp.name) / "apt.conf"

    def close(self) -> None:
        self.temp.cleanup()

    def _remaining(self) -> int:
        seconds = int(self.deadline - self.clock()) - 5
        if seconds <= 0:
            raise TimeoutError("PostgreSQL package setup exceeded its 220-second budget")
        return min(seconds, 100)

    def _run(self, args: list[str], *, privileged: bool = False) -> subprocess.CompletedProcess[str]:
        cap = self._remaining()
        command = args
        if privileged:
            command = ["sudo", "timeout", "--signal=TERM", "--kill-after=5s", f"{cap}s", *args]
        print(f"$ {' '.join(command)}", flush=True)
        result = self.runner(command, capture_output=True, text=True, timeout=cap + 6)
        if result.stdout:
            print(result.stdout, end="", flush=True)
        if result.stderr:
            print(result.stderr, end="", file=sys.stderr, flush=True)
        if result.returncode:
            raise CommandFailure(command, result)
        return result

    def _azure_sources(self):
        candidates = [
            self.sources_dir / "sources.list",
            self.sources_dir / "apt-mirrors.txt",
            *self.sources_dir.glob("sources.list.d/*"),
        ]
        for path in candidates:
            if not path.is_file() or path.is_symlink() or path.suffix not in ("", ".list", ".sources", ".txt"):
                continue
            kind = "mirror-list" if path.name == "apt-mirrors.txt" else "sources" if path.suffix == ".sources" else "list"
            original = path.read_text()
            updated = replace_azure_source(original, kind)
            if updated != original:
                yield path, updated

    def _install_recovery_mirror(self) -> int:
        staging = Path(self.temp.name) / "rewritten"
        staging.mkdir(exist_ok=True)
        changed = 0
        for index, (path, updated) in enumerate(self._azure_sources()):
            staged = staging / str(index)
            staged.write_text(updated)
            self._run(["install", "-m", "0644", str(staged), str(path)], privileged=True)
            changed += 1
        return changed

    def _probe_mirror_dns(self) -> None:
        if not any(self._azure_sources()):
            return
        try:
            self._run(["timeout", "--kill-after=1s", "5s", "getent", "ahosts", "azure.archive.ubuntu.com"])
        except CommandFailure as error:
            if error.returncode not in (2, 124):
                raise
            self.recovery_used = True
            print("Azure Ubuntu mirror DNS unavailable; selecting the canonical archive", flush=True)
            if not self._install_recovery_mirror():
                raise

    def _run_apt(self, args: list[str]) -> None:
        command = ["env", f"APT_CONFIG={self.apt_config}", "DEBIAN_FRONTEND=noninteractive", *args]
        try:
            self._run(command, privileged=True)
        except CommandFailure as error:
            if self.recovery_used or not AZURE_DNS_ERROR.search(error.transcript):
                raise
            self.recovery_used = True
            if not self._install_recovery_mirror():
                raise
            if "apt-get" in args and "install" in args:
                self._run(["env", f"APT_CONFIG={self.apt_config}", "DEBIAN_FRONTEND=noninteractive", "apt-get", "update"], privileged=True)
            self._run(command, privileged=True)

    def _configure_pgdg(self) -> None:
        release = self.os_release.read_text()
        codename = re.search(r"^VERSION_CODENAME=(.+)$", release, re.MULTILINE)
        if not codename:
            raise RuntimeError("Unable to determine Ubuntu package codename")
        suite = codename.group(1).strip().strip('"')
        architecture = self.architecture or self._run(["dpkg", "--print-architecture"]).stdout.strip()
        key_dir = self.root / "share/postgresql-common/pgdg"
        key_path = key_dir / "apt.postgresql.org.asc"
        legacy_key = key_dir / "apt.postgresql.org.gpg"
        if not key_path.is_file() and legacy_key.is_file():
            legacy_output = self._run(["gpg", "--show-keys", "--with-colons", str(legacy_key)]).stdout
            if PGDG_FINGERPRINT in re.findall(r"^fpr:::::::::([A-F0-9]+):$", legacy_output, re.MULTILINE):
                key_path = legacy_key
        valid_key = False
        if key_path.is_file():
            existing = self._run(["gpg", "--show-keys", "--with-colons", str(key_path)]).stdout
            valid_key = PGDG_FINGERPRINT in re.findall(r"^fpr:::::::::([A-F0-9]+):$", existing, re.MULTILINE)
        if not valid_key:
            downloaded = Path(self.temp.name) / "apt.postgresql.org.asc"
            self._run(["curl", "--fail", "--location", "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "25", "-o", str(downloaded), PGDG_KEY])
            keys = self._run(["gpg", "--show-keys", "--with-colons", str(downloaded)]).stdout
            fingerprints = re.findall(r"^fpr:::::::::([A-F0-9]+):$", keys, re.MULTILINE)
            if PGDG_FINGERPRINT not in fingerprints:
                raise RuntimeError("Downloaded PGDG key fingerprint did not match the official key")
            key_stage = Path(self.temp.name) / "keyring"
            key_stage.write_bytes(downloaded.read_bytes())
            self._run(["install", "-d", str(key_path.parent)], privileged=True)
            self._run(["install", "-m", "0644", str(key_stage), str(key_path)], privileged=True)
        source = Path(self.temp.name) / "pgdg.sources"
        source.write_text(
            "Types: deb\n"
            "URIs: https://apt.postgresql.org/pub/repos/apt\n"
            f"Suites: {suite}-pgdg\n"
            f"Architectures: {architecture}\n"
            "Components: main\n"
            f"Signed-By: {key_path}\n"
        )
        self._run(["rm", "-f", str(self.sources_dir / "sources.list.d/pgdg.list")], privileged=True)
        self._run(["install", "-m", "0644", str(source), str(self.sources_dir / "sources.list.d/pgdg.sources")], privileged=True)

    def install(self) -> None:
        if installed(self.root, self.runner):
            return
        self.apt_config.write_text(
            'Acquire::Retries "1";\n'
            'Acquire::http::Timeout "10";\n'
            'Acquire::https::Timeout "10";\n'
            'DPkg::Lock::Timeout "10";\n'
            'APT::Update::Error-Mode "any";\n'
        )
        self._configure_pgdg()
        self._probe_mirror_dns()
        self._run_apt(["apt-get", "update"])
        self._run_apt(["apt-get", "install", "-y", "postgresql-18"])
        if not installed(self.root, self.runner):
            raise RuntimeError("PostgreSQL 18 or required citext/pg_stat_statements extensions are missing")


def main() -> int:
    installer = Installer()
    try:
        installer.install()
        return 0
    finally:
        installer.close()


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (CommandFailure, OSError, RuntimeError, TimeoutError) as error:
        print(f"PostgreSQL setup failed: {error}", file=sys.stderr)
        sys.exit(1)
