"""Fresh sequential matched builds on one VM; never reuse historical CLIs."""
import json
import os
from pathlib import Path
import runpy
import sys

from inputs import BUILD_FIELDS, HELPERS, SOURCES
import common
import frozen

common.SOURCES = SOURCES
frozen.PINS = {}  # Both production executables must be built from the pinned source.
assert os.uname().machine == 'aarch64'
assert os.environ['ImageOS'] == 'ubuntu24-arm64'
assert not Path('source').exists()
try:
    for role in SOURCES:
        Path('source').symlink_to(Path('source-' + role).resolve(), target_is_directory=True)
        try:
            sys.argv = [str(HELPERS / 'build.py'), role]
            runpy.run_path(str(HELPERS / 'build.py'), run_name='__main__')
        finally:
            Path('source').unlink()
    proofs = {role: json.loads((Path('out') / role / 'provenance.json').read_text())
              for role in SOURCES}
    for role, proof in proofs.items():
        assert proof['source'] == SOURCES[role]
        assert proof['tree'] == common.run('git', 'rev-parse', 'HEAD^{tree}',
                                            cwd='source-' + role)
        assert not proof.get('frozenCliBuild')
    assert proofs['base']['binarySha256'] != proofs['phases']['binarySha256']
    assert all(proofs['base'][key] == proofs['phases'][key] for key in BUILD_FIELDS)
finally:
    assert not Path('source').exists()
