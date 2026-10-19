"""Bounded single-snapshot artifact downloads; no remote-state polling."""
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import urllib.request
import zipfile

from common import digest
from recheck_contract import REPOSITORY, validate_metadata

MAX_ZIP_BYTES = 2_000_000_000
MAX_EXPANDED_BYTES = 4_000_000_000


def api(endpoint):
    request = urllib.request.Request(
        f'https://api.github.com/repos/{REPOSITORY}/{endpoint}',
        headers={'Authorization': 'Bearer ' + os.environ['GITHUB_TOKEN'],
                 'Accept': 'application/vnd.github+json'})
    with urllib.request.urlopen(request, timeout=20) as response:
        return json.load(response)


def extract_checked(archive, destination, expected=None):
    destination = Path(destination)
    assert not destination.exists()
    with zipfile.ZipFile(archive) as zipped:
        entries = zipped.infolist()
        names = [item.filename for item in entries if not item.is_dir()]
        assert len(names) == len(set(names))
        assert len(entries) <= 1000
        assert sum(item.file_size for item in entries) <= MAX_EXPANDED_BYTES
        for item in entries:
            path = PurePosixPath(item.filename)
            assert not path.is_absolute() and '..' not in path.parts
            assert '\\' not in item.filename and item.filename
            assert not stat.S_ISLNK(item.external_attr >> 16)
        if expected is not None:
            assert set(names) == set(expected)
        destination.mkdir()
        actual = {}
        for item in entries:
            path = destination / item.filename
            if item.is_dir():
                path.mkdir(parents=True, exist_ok=True)
                continue
            path.parent.mkdir(parents=True, exist_ok=True)
            with zipped.open(item) as source, path.open('xb') as target:
                shutil.copyfileobj(source, target, length=1024 * 1024)
            actual[item.filename] = digest(path)
            if expected is not None:
                assert actual[item.filename] == expected[item.filename]
        return actual


def download(artifact_id, name, archive_sha, run, head, destination, files=None):
    metadata = api(f'actions/artifacts/{artifact_id}')
    validate_metadata(metadata, artifact_id, name, archive_sha, run, head)
    assert 0 < metadata['size_in_bytes'] <= MAX_ZIP_BYTES
    archive = Path(os.environ['RUNNER_TEMP']) / f'recheck-{artifact_id}.zip'
    assert not archive.exists()
    env = dict(os.environ, GH_TOKEN=os.environ['GITHUB_TOKEN'])
    try:
        # gh handles the authenticated API redirect without forwarding credentials to blob storage.
        with archive.open('xb') as output:
            subprocess.run(['gh', 'api', f'repos/{REPOSITORY}/actions/artifacts/{artifact_id}/zip'],
                           env=env, stdout=output, check=True, timeout=180)
        assert archive.stat().st_size <= MAX_ZIP_BYTES
        assert digest(archive) == archive_sha
        hashes = extract_checked(archive, destination, files)
        return {'metadata': metadata, 'actualZipSha256': digest(archive), 'files': hashes}
    finally:
        archive.unlink(missing_ok=True)
