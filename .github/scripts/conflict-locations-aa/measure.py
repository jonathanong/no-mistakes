"""Three balanced identical-main A/A blocks on one ARM VM, retaining raw resources."""
import json
import os
from pathlib import Path
import subprocess
import sys

siblings = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(siblings / 'conflict-locations-arm'))
from ordering import derive_config, validate_fixture_contract
import common
import worker
from image_contract import image_provenance

common.install_cancel_handler()
assert os.uname().machine == 'aarch64'
jobs, diagnostics, ledger = sys.argv[1:]
assert jobs in ('1', '4', 'default') and diagnostics in ('off', 'on') and ledger == 'off'
output = Path('results')
output.mkdir()
root = Path('fixture').resolve()
physical = common.workload(root)
assert physical == json.loads(Path('prepared/fixture-provenance.json').read_text())
config = Path(os.environ['RUNNER_TEMP']) / 'ordering-only-aa.yml'
config_proof = derive_config(root, config,
    'harness/fixtures/performance/current-verdict-arm/ordering-only.yml')
validate_fixture_contract(
    'harness/fixtures/performance/current-verdict-arm/ordering-only-provenance.json',
    common.digest(Path('prepared/fixture.tar.gz')))
folder = Path('cli/base')
proof = json.loads((folder / 'provenance.json').read_text())
assert proof['source'] == '350ada01a24c33f98696ee98de9b965dfbe6d671'
assert proof['artifactHarnessCommit'] == os.environ['GITHUB_SHA']
for filename, field in [('no-mistakes', 'binarySha256'), ('cpu-probe', 'cpuProbeSha256')]:
    path = folder / filename
    assert common.digest(path) == proof[field]
    path.chmod(0o755)
binary = (folder / 'no-mistakes').resolve()
common.save(output / 'provenance.json', {
    'purpose': 'Intentional identical-binary main A/A noise control, three BCCB blocks',
    'build': proof, 'machine': common.machine(), 'consumer': physical,
    'config': config_proof, 'jobs': jobs, 'diagnostics': diagnostics,
    'defaultJobs': int(subprocess.check_output([str((folder / 'cpu-probe').resolve())])),
    'imageVersions': image_provenance(proof['machine'], common.machine()),
    'harnessCommit': os.environ['GITHUB_SHA'], 'blocks': 3,
    'powerToolLimitation': 'evals/power.py sizes Bernoulli skill outcomes, not runtime resources'})
rows = []
try:
    for block in range(3):
        for arm in ('b', 'c', 'c', 'b'):
            names = set(subprocess.check_output(['ps', '-e', '-o', 'comm='], text=True).split())
            assert not names.intersection({'cargo', 'rustc', 'rustdoc', 'cc1', 'ld.lld'})
            row = worker.sample(binary, root, config, jobs, diagnostics,
                                False, output, arm, len(rows))
            row['block'] = block
            rows.append(row)
            common.save(output / 'samples.json', rows)
            assert row['eligible'], row
            assert row['stdoutSha256'] == rows[0]['stdoutSha256']
            assert row['semanticStderr'] == rows[0]['semanticStderr']
            assert row['exit'] == rows[0]['exit']
            assert 'sqlVerdicts' not in json.loads((output / row['stdout']).read_bytes())
    assert common.workload(root) == physical
    common.save(output / 'status.json', {'status': 'passed', 'arms': len(rows), 'blocks': 3})
finally:
    config.unlink(missing_ok=True)
