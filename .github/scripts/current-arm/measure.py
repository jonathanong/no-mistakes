"""Two fresh same-VM jobs1 capacity observations, excluded from gate evidence."""
import json
import os
from pathlib import Path
import subprocess
import sys

from inputs import BUILD_FIELDS, CAP_SECONDS, SOURCES, outcome, validate_parity
import common
import long_frozen
import worker

common.install_cancel_handler()
worker.CAP_SECONDS = CAP_SECONDS
assert os.uname().machine == 'aarch64'
assert os.environ['ImageOS'] == 'ubuntu24-arm64'
diagnostics = sys.argv[1]
assert diagnostics in ('off', 'on')
output = Path('results')
output.mkdir()
root = Path('fixture').resolve()
physical = common.workload(root)
expected = json.loads(Path('prepared/fixture-provenance.json').read_text())
assert physical == expected
origin = long_frozen.artifact_metadata('matched-consumer')
proofs = {}
for role in SOURCES:
    folder = Path('cli') / role
    proof = json.loads((folder / 'provenance.json').read_text())
    assert proof['source'] == SOURCES[role]
    assert proof['artifactHarnessCommit'] == os.environ['GITHUB_SHA']
    assert not proof.get('frozenCliBuild')
    assert proof['machine']['imageOS'] == os.environ['ImageOS']
    assert proof['machine']['imageVersion'] == os.environ['ImageVersion']
    assert proof['machine']['uname'][-1] == os.uname().machine
    for filename, field in [('no-mistakes', 'binarySha256'), ('cpu-probe', 'cpuProbeSha256')]:
        path = folder / filename
        assert common.digest(path) == proof[field]
        path.chmod(0o755)
    proof['effectiveDefaultJobs'] = int(subprocess.check_output([str((folder / 'cpu-probe').resolve())]))
    proofs[role] = proof
assert all(proofs['base'][key] == proofs['phases'][key] for key in BUILD_FIELDS)
assert proofs['base']['binarySha256'] != proofs['phases']['binarySha256']
assert proofs['base']['effectiveDefaultJobs'] == proofs['phases']['effectiveDefaultJobs']
common.save(output / 'provenance.json', {
    **outcome([]), 'builds': proofs, 'workload': physical,
    'consumerArtifact': origin, 'machine': common.machine(),
    'harnessCommit': os.environ['GITHUB_SHA'], 'jobs': '1', 'diagnostics': diagnostics})
rows = []
status = 'failed'
try:
    for ordinal, (arm, role) in enumerate(zip('bc', SOURCES)):
        names = set(subprocess.check_output(['ps', '-e', '-o', 'comm='], text=True).split())
        assert not names.intersection({'cargo', 'rustc', 'rustdoc', 'cc1', 'ld.lld'})
        row = worker.sample((Path('cli') / role / 'no-mistakes').resolve(), root,
                            root / '.no-mistakes.yml', '1', diagnostics, False,
                            output, arm, ordinal)
        rows.append(row)
        common.save(output / 'samples.json', rows)
        common.save(output / 'feasibility.json', outcome(rows, status))
    assert common.workload(root) == physical
    if all(row['eligible'] for row in rows):
        parity = validate_parity(output, rows)
        common.save(output / 'parity.json', parity)
        status = 'passed'
    else:
        raise SystemExit(1)
finally:
    common.save(output / 'feasibility.json', outcome(rows, status))
