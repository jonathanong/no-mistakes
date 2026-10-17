"""Complete declared native control inputs are mandatory before any execution."""
import os
from pathlib import Path
from transport import sha


def declared(files, required):
    paths = {row['path'] for row in files}
    assert len(paths) == len(files), 'Duplicate frozen input path'
    for row in required:
        if row['kind'] == 'file':
            assert row['path'] in paths, ('Undeclared required control input', row['path'])
        else:
            assert row['kind'] == 'directory' and row['files'], 'Empty/invalid directory inventory'
            actual = sorted(path for path in paths if Path(path).is_relative_to(row['path']))
            assert actual == row['files'], ('Incomplete required directory inventory', row['path'])


def discovery_error(error):
    raise error


def verify(files, required):
    declared(files, required)
    for row in files:
        path = Path(row['path'])
        for part in [path, *path.parents]:
            assert not part.is_symlink(), ('Unexpected input symlink', str(part))
        assert path.is_file(), ('Missing frozen input file', row['path'])
        assert path.stat().st_size == row['bytes'] and sha(path) == row['sha256'], ('Frozen input changed', row['path'])
    for row in required:
        if row['kind'] == 'directory':
            root = Path(row['path'])
            assert root.is_dir() and not root.is_symlink(), row['path']
            actual = []
            for directory, dirs, names in os.walk(root, followlinks=False, onerror=discovery_error):
                for name in [*dirs, *names]:
                    assert not (Path(directory) / name).is_symlink(), 'Undeclared input symlink'
                actual.extend(str(Path(directory) / name) for name in names)
            assert sorted(actual) == row['files'], ('Physical directory inventory changed', row['path'])


def declared_sql(files, required, controls):
    """Bind every actual SQL argv input to its transported file and arm role."""
    paths = {row['path'] for row in files}
    declared_roles = {row['path']: row for row in required if row['kind'] == 'file'}
    for arm in ['A28', 'B28']:
        arguments = controls[arm + '-run'][1:]
        assert len(arguments) == 3 and len(set(arguments)) == 3
        for path in arguments:
            assert Path(path).is_absolute() and path in paths, ('Undeclared SQL argv input', arm, path)
            row = declared_roles.get(path)
            assert row and arm + ':SQL' in row['roles'], ('Unbound SQL argv role', arm, path)
