"""Download one native runtime with bounded I/O and verified archive integrity."""
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import sys
import tempfile
import time
import zipfile


class DownloadError(RuntimeError):
    pass


class Deadline:
    def __init__(self):
        self.end = time.monotonic() + 110

    def remaining(self):
        remaining = self.end - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("native runtime download exceeded 110 seconds")
        return remaining


def metadata(url, token, deadline):
    result = subprocess.run([
        "curl", "--config", "-", "--fail", "--silent", "--show-error",
        "--connect-timeout", "10", "--max-time", "10", url,
    ], input=f"header = {json.dumps('Authorization: Bearer ' + token)}\n"
       'header = "Accept: application/vnd.github+json"\n'
       'header = "X-GitHub-Api-Version: 2022-11-28"\n',
       text=True, stdout=subprocess.PIPE, check=True, timeout=deadline.remaining())
    return json.loads(result.stdout)


def resolve_artifact(repo, run_id, target, token, deadline):
    matches = []
    for page in range(1, 11):
        url = (f"https://api.github.com/repos/{repo}/actions/runs/{run_id}/artifacts"
               f"?per_page=100&page={page}")
        payload = metadata(url, token, deadline)
        artifacts = payload.get("artifacts")
        if not isinstance(artifacts, list):
            raise DownloadError("invalid artifact catalog")
        matches.extend(a for a in artifacts if a.get("name") == f"native-runtime-{target}")
        if len(matches) > 1:
            raise DownloadError("ambiguous native runtime artifact")
        if len(artifacts) < 100:
            break
    else:
        raise DownloadError("artifact catalog exceeds 10 pages")
    if len(matches) != 1 or matches[0].get("expired"):
        raise DownloadError("native runtime artifact missing or expired")
    artifact = matches[0]
    if not re.fullmatch(r"sha256:[0-9a-fA-F]{64}", artifact.get("digest") or ""):
        raise DownloadError("artifact requires a SHA256 digest")
    if not isinstance(artifact.get("id"), int):
        raise DownloadError("artifact requires a numeric id")
    return artifact


def download(repo, artifact, token, archive, deadline):
    url = f"https://api.github.com/repos/{repo}/actions/artifacts/{artifact['id']}/zip"
    config = f"header = {json.dumps('Authorization: Bearer ' + token)}\n"
    subprocess.run([
        "curl", "--config", "-", "--fail", "--location", "--silent", "--show-error",
        "--retry", "2", "--retry-max-time", "90",
        "--connect-timeout", "10", "--max-time", "25", "--output", str(archive), url,
    ], input=config, text=True, check=True, timeout=deadline.remaining())


def verify(archive, expected, deadline):
    digest = hashlib.sha256()
    with archive.open("rb") as source:
        while chunk := source.read(1024 * 1024):
            deadline.remaining()
            digest.update(chunk)
    if digest.hexdigest() != expected.split(":", 1)[1].lower():
        raise DownloadError("native runtime SHA256 mismatch")


def extract(archive, destination, deadline):
    destination = destination.resolve()
    with zipfile.ZipFile(archive) as source:
        entries = source.infolist()
        for entry in entries:
            name = entry.filename
            path = PurePosixPath(name)
            if ("\\" in name or path.is_absolute() or ".." in path.parts
                    or ":" in name or stat.S_ISLNK(entry.external_attr >> 16)):
                raise DownloadError("unsafe native runtime archive path")
            if not (destination / name).resolve().is_relative_to(destination):
                raise DownloadError("native runtime archive escapes destination")
        for entry in entries:
            deadline.remaining()
            output = destination / entry.filename
            if entry.is_dir():
                output.mkdir(parents=True, exist_ok=True)
                continue
            output.parent.mkdir(parents=True, exist_ok=True)
            with source.open(entry) as reader, output.open("wb") as writer:
                while chunk := reader.read(1024 * 1024):
                    deadline.remaining()
                    writer.write(chunk)
            mode = (entry.external_attr >> 16) & 0o777
            if mode:
                output.chmod(mode)


def main():
    started = time.monotonic()
    timings = {name: 0 for name in ("metadata", "download", "digest", "extraction")}
    stage = "metadata"
    stage_started = started
    success = False
    try:
        repo = os.environ["GITHUB_REPOSITORY"]
        run_id = os.environ["NATIVE_RUNTIME_RUN_ID"]
        target = os.environ["NATIVE_RUNTIME_TARGET"]
        token = os.environ["GH_TOKEN"]
        if not re.fullmatch(r"[\w.-]+/[\w.-]+", repo) or not run_id.isdigit():
            raise DownloadError("invalid repository or run id")
        if not re.fullmatch(r"[\w.-]+", target) or not token:
            raise DownloadError("invalid target or missing token")
        deadline = Deadline()
        artifact = resolve_artifact(repo, run_id, target, token, deadline)
        timings[stage] = time.monotonic() - stage_started
        with tempfile.TemporaryDirectory(prefix="native-runtime-") as temporary:
            archive = Path(temporary) / "runtime.zip"
            stage = "download"
            stage_started = time.monotonic()
            download(repo, artifact, token, archive, deadline)
            timings[stage] = time.monotonic() - stage_started
            stage = "digest"
            stage_started = time.monotonic()
            verify(archive, artifact["digest"], deadline)
            timings[stage] = time.monotonic() - stage_started
            stage = "extraction"
            stage_started = time.monotonic()
            extract(archive, Path("target/debug"), deadline)
            timings[stage] = time.monotonic() - stage_started
        success = True
        return 0
    except Exception as error:
        timings[stage] = time.monotonic() - stage_started
        detail = str(error) if isinstance(error, (DownloadError, TimeoutError)) else ""
        if isinstance(error, subprocess.CalledProcessError):
            detail = f"curl exit status {error.returncode}"
        if isinstance(error, subprocess.TimeoutExpired):
            detail = "overall 110-second deadline exhausted"
        print(f"native runtime download failed during {stage}: "
              f"{type(error).__name__}: {detail}", file=sys.stderr)
        return 1
    finally:
        print(json.dumps({"success": success, "seconds": {
            **timings, "total": time.monotonic() - started}}), flush=True)


if __name__ == "__main__":
    sys.exit(main())
