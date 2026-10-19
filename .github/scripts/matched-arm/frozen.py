"""Reuse only independently audited immutable ARM CLI artifacts."""
import json
from pathlib import Path
import shutil

from common import digest

RUN = 39185937985
PINS = {
    'base': '82e318de143c852f46f32b16fa60d908fdc32c011418ab266e84fe622f3bfe53',
    'phases': 'fc3173c7b49f4d430bff7ec947873575413eade92e948260d050069f5b8b2151',
}
FIELDS = ('source', 'tree', 'lockSha256', 'rustc', 'cargo', 'features',
          'profile', 'command', 'buildJobs', 'incremental')


def verify_manifest(proof, expected, binary_sha, pinned_sha):
    assert binary_sha == pinned_sha, 'frozen CLI hash differs from audited artifact'
    for field in FIELDS:
        assert proof[field] == expected[field], field
    for field in ('imageOS', 'imageVersion'):
        assert proof['machine'][field] == expected['machine'][field], field
    assert proof['machine']['uname'][-1] == expected['machine']['uname'][-1]


def copy_frozen(role, expected, output):
    folder = Path('frozen')
    proof = json.loads((folder / 'source.json').read_text())
    verify_manifest(proof, expected, digest(folder / 'no-mistakes'), PINS[role])
    shutil.copy2(folder / 'no-mistakes', output / 'no-mistakes')
    shutil.copy2(folder / 'build.log', output / 'build.log')
    proof['frozenCliBuild'] = {
        'run': RUN, 'artifact': 'matched-cli-' + role,
        'harnessCommit': '9699dc1a7e418ec195d4b7f16da5bdd7ee3b4194',
        'binarySha256': PINS[role], 'buildLogSha256': digest(folder / 'build.log'),
        'note': 'Production CLI build succeeded; the separate CPU helper failed. No samples ran.',
    }
    return proof
