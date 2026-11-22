"""Download exact native runtime artifacts within one verified, bounded invocation."""
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


# 177 MB at about 1 MB/s is 177s. The extra time covers a stall abort and a resume.
OVERALL_SECONDS = 300
DOWNLOAD_ATTEMPTS = 3
# Timeouts and connection drops. HTTP failures (22) and range rejection (33) are separate.
TRANSIENT_CURL_EXITS = {6, 7, 18, 28, 35, 52, 55, 56}
HTTP_ERROR = 22
RANGE_UNSUPPORTED = 33


class Deadline:
    def __init__(self):
        self.end = time.monotonic() + OVERALL_SECONDS

    def remaining(self):
        remaining = self.end - time.monotonic()
        if remaining <= 0:
            raise TimeoutError(
                f"native runtime download exceeded {OVERALL_SECONDS} seconds")
        return remaining


def metadata(url, token, deadline):
    result = subprocess.run([
        "curl", "--config", "-", "--fail", "--silent", "--show-error",
        "--ssl-revoke-best-effort",
        "--connect-timeout", "10", "--max-time", "10", url,
    ], input=f"header = {json.dumps('Authorization: Bearer ' + token)}\n"
       'header = "Accept: application/vnd.github+json"\n'
       'header = "X-GitHub-Api-Version: 2022-11-28"\n',
       text=True, stdout=subprocess.PIPE, check=True, timeout=deadline.remaining())
    return json.loads(result.stdout)


def resolve_artifact(repo, run_id, name, token, deadline):
    matches = []
    for page in range(1, 11):
        url = (f"https://api.github.com/repos/{repo}/actions/runs/{run_id}/artifacts"
               f"?per_page=100&page={page}")
        payload = metadata(url, token, deadline)
        artifacts = payload.get("artifacts")
        if not isinstance(artifacts, list):
            raise DownloadError("invalid artifact catalog")
        matches.extend(a for a in artifacts if a.get("name") == name)
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


def download_command(url, archive, resume):
    command = [
        "curl", "--config", "-", "--fail", "--location", "--silent", "--show-error",
        "--ssl-revoke-best-effort",
        "--speed-limit", "1024", "--speed-time", "30",
        "--connect-timeout", "10",
    ]
    # Each attempt is a new process so resume works before curl 8.17.
    if resume:
        command.extend(["--continue-at", "-"])
    command.extend(["--write-out", "%{http_code}", "--output", str(archive), url])
    return command


def download(repo, artifact, token, archive, deadline):
    url = f"https://api.github.com/repos/{repo}/actions/artifacts/{artifact['id']}/zip"
    config = f"header = {json.dumps('Authorization: Bearer ' + token)}\n"
    try:
        transfer(url, archive, config, deadline, resume=True)
    except subprocess.CalledProcessError as error:
        if error.returncode != RANGE_UNSUPPORTED or not archive.is_file() or archive.stat().st_size == 0:
            raise
        archive.unlink()
        transfer(url, archive, config, deadline, resume=False)


def is_transient_transfer_error(error):
    return error.returncode in TRANSIENT_CURL_EXITS or (
        error.returncode == HTTP_ERROR
        and re.fullmatch(r"5[0-9]{2}", error.stdout or "") is not None
    )


def transfer(url, archive, config, deadline, resume):
    for attempt in range(DOWNLOAD_ATTEMPTS):
        try:
            subprocess.run(
                download_command(url, archive, resume),
                input=config, text=True, stdout=subprocess.PIPE, check=True,
                timeout=deadline.remaining())
            return
        except subprocess.CalledProcessError as error:
            if attempt + 1 == DOWNLOAD_ATTEMPTS or not is_transient_transfer_error(error):
                raise
            time.sleep(min(2 ** attempt, deadline.remaining()))


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


def main(artifact_names=()):
    started = time.monotonic()
    timings = {name: 0 for name in ("metadata", "download", "digest", "extraction")}
    stage = "metadata"
    stage_started = started
    success = False
    completed = []
    temporary = None
    try:
        repo = os.environ["GITHUB_REPOSITORY"]
        run_id = os.environ["NATIVE_RUNTIME_RUN_ID"]
        token = os.environ["GH_TOKEN"]
        if not re.fullmatch(r"[\w.-]+/[\w.-]+", repo) or not run_id.isdigit():
            raise DownloadError("invalid repository or run id")
        names = list(artifact_names)
        if not names:
            target = os.environ["NATIVE_RUNTIME_TARGET"]
            if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", target):
                raise DownloadError("invalid native runtime target")
            names = [f"native-runtime-{target}"]
        if (not token or len(set(names)) != len(names)
                or any(not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", name) for name in names)):
            raise DownloadError("invalid or duplicate artifact names or missing token")
        deadline = Deadline()
        for artifact_name in names:
            artifact_timings = {}
            for stage in timings:
                stage_started = time.monotonic()
                if stage == "metadata":
                    artifact = resolve_artifact(repo, run_id, artifact_name, token, deadline)
                    temporary = tempfile.TemporaryDirectory(prefix="native-runtime-")
                    archive = Path(temporary.name) / "runtime.zip"
                elif stage == "download":
                    download(repo, artifact, token, archive, deadline)
                elif stage == "digest":
                    verify(archive, artifact["digest"], deadline)
                else:
                    extract(archive, Path("target/debug"), deadline)
                elapsed = time.monotonic() - stage_started
                timings[stage] += elapsed
                artifact_timings[stage] = elapsed
            temporary.cleanup()
            completed.append({"name": artifact_name, "seconds": artifact_timings})
        success = True
        return 0
    except Exception as error:
        timings[stage] += time.monotonic() - stage_started
        detail = str(error) if isinstance(error, (DownloadError, TimeoutError)) else ""
        if isinstance(error, subprocess.CalledProcessError):
            detail = f"curl exit status {error.returncode}"
        if isinstance(error, subprocess.TimeoutExpired):
            detail = f"overall {OVERALL_SECONDS}-second deadline exhausted"
        print(f"native runtime download failed during {stage}: "
              f"{type(error).__name__}: {detail}", file=sys.stderr)
        return 1
    finally:
        if temporary is not None:
            temporary.cleanup()
        print(json.dumps({"success": success, "artifacts": completed, "seconds": {
            **timings, "total": time.monotonic() - started}}), flush=True)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
