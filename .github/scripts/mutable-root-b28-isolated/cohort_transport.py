"""Portable outer artifact names, lossless authenticated inner public inventory."""
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import tarfile

ARCHIVE = 'cohort-public.tar'
MANIFEST = 'cohort-transport.json'

def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def safe_name(name):
    path = PurePosixPath(name)
    assert name and not path.is_absolute() and '..' not in path.parts
    assert str(path) == name, 'noncanonical archive path'
    return name

def identity(cohort):
    return {'run_id': os.environ['GITHUB_RUN_ID'],
            'diagnostic_commit': os.environ['APPROVED_DIAGNOSTIC_COMMIT'],
            'cohort': cohort}

def pack(public, destination, cohort):
    public, destination = Path(public), Path(destination)
    assert public.is_dir() and not public.is_symlink()
    assert not destination.exists(), 'fresh transport directory required'
    destination.mkdir(parents=True)
    files = {}
    for path in sorted(public.rglob('*')):
        assert not path.is_symlink(), 'public symlink rejected'
        assert path.is_file() or path.is_dir(), 'public special file rejected'
        if path.is_file():
            name = safe_name(path.relative_to(public).as_posix())
            files[name] = {'bytes': path.stat().st_size, 'sha256': digest(path)}
    assert files, 'empty public inventory'
    archive = destination / ARCHIVE
    with tarfile.open(archive, 'w', format=tarfile.PAX_FORMAT, dereference=True) as output:
        for name in files:
            output.add(public / name, arcname=name, recursive=False)
    # Check original inventory remained identical through archive creation.
    actual = {p.relative_to(public).as_posix() for p in public.rglob('*') if p.is_file()}
    assert actual == set(files)
    for name, row in files.items():
        assert (public/name).stat().st_size == row['bytes'] and digest(public/name) == row['sha256']
    manifest = {**identity(cohort), 'archive': ARCHIVE, 'sha256': digest(archive),
                'bytes': archive.stat().st_size, 'files': files}
    (destination / MANIFEST).write_text(json.dumps(manifest, indent=2) + '\n')
    return manifest

def unpack(source, public, cohort):
    source, public = Path(source), Path(public)
    assert source.is_dir() and not source.is_symlink()
    assert {p.name for p in source.iterdir()} == {ARCHIVE, MANIFEST}
    assert all(p.is_file() and not p.is_symlink() for p in source.iterdir())
    manifest = json.loads((source / MANIFEST).read_text())
    assert {key: manifest[key] for key in identity(cohort)} == identity(cohort)
    assert manifest['archive'] == ARCHIVE
    archive = source / ARCHIVE
    assert archive.stat().st_size == manifest['bytes'] and digest(archive) == manifest['sha256']
    assert not public.exists(), 'fresh public directory required'
    with tarfile.open(archive, 'r') as incoming:
        members = incoming.getmembers()
        assert len({m.name for m in members}) == len(members), 'duplicate archive member'
        assert {m.name for m in members} == set(manifest['files']), 'transport inventory differs'
        # Validate ALL members and bytes before writing any destination content.
        for member in members:
            safe_name(member.name)
            assert member.isfile(), 'links/special members rejected'
            row = manifest['files'][member.name]
            assert member.size == row['bytes']
            with incoming.extractfile(member) as stream:
                assert hashlib.file_digest(stream, 'sha256').hexdigest() == row['sha256']
        public.mkdir(parents=True)
        for member in members:
            path = public / member.name
            path.parent.mkdir(parents=True, exist_ok=True)
            with incoming.extractfile(member) as stream, path.open('xb') as output:
                import shutil
                shutil.copyfileobj(stream, output)
            assert digest(path) == manifest['files'][member.name]['sha256']
    return manifest

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('operation', choices=['pack'])
    parser.add_argument('public')
    parser.add_argument('destination')
    parser.add_argument('cohort', choices=['A', 'B'])
    args = parser.parse_args()
    pack(args.public, args.destination, args.cohort)
