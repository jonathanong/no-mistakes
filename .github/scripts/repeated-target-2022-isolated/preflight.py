"""Fresh complete functional reports on this VM, authentic compiled producer Git IDs."""
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import shutil
import subprocess
import sys

D = Path(__file__).resolve().parent
from contracts import layouts
layout_count = layouts(json.loads((D / 'plan.json').read_text()))
ROOT = Path('/tmp/nm2022-main281-current-pair-functional/current-pair-semantics')
GIT = Path('/tmp/no-mistakes-2022-assignment-target-opt')
# The reviewed current-pair adapter declares the authentic overlay baseline
# and performs public-facade lookups at its source-equivalent main revision.
runpy.run_path('/tmp/nm2022-main281-current-pair-functional/collect-current-pair-semantics.py', run_name='__main__')
m = json.loads((ROOT / 'semantic-manifest.json').read_text())
assert not m['failures'] and len(m['raw_files']) == 182
assert sum(len(v) for v in m['report_hashes'].values()) == 27
pins = []

def pin(path):
    path = Path(path)
    h = hashlib.file_digest(path.open('rb'), 'sha256').hexdigest()
    pins.append({'path': str(path), 'sha256': h})
    return h

for path, expected in m['raw_files'].items():
    assert pin(ROOT / path) == expected
public = ROOT / 'public'
public.mkdir()
for arm in ['baseline', 'candidate']:
    rows = [row for row in json.loads((D / 'assets.json').read_text()) if row['arm'] == arm]
    assert len(rows) == 4 and len({row['file'] for row in rows}) == 4
    files = {}
    for row in rows:
        path = ROOT / arm / 'transfer' / row['file']
        assert path.stat().st_size == row['size'] and pin(path) == row['sha256']
        files[row['file']] = {'sha256': row['sha256'], 'size': row['size'], 'asset_id': row['id']}
    transfer = ROOT / arm / 'transfer/transfer-manifest.json'
    transfer.write_text(json.dumps({'arm': arm, 'producer': m['pair'][arm], 'files': files}, indent=2) + '\n')
    pin(transfer)
    addon = ROOT / arm / 'transfer/addon.node'
    for script, count in [('postgres-conditional-executors-api.test.js', 1), ('postgres-source-api.test.js', 34)]:
        env = dict(os.environ, NO_MISTAKES_TEST_NAPI_ADDON_PATH=str(addon))
        prefix = public / f'{arm}-{script}'
        with Path(str(prefix) + '.stdout').open('wb') as out, Path(str(prefix) + '.stderr').open('wb') as err:
            result = subprocess.run(['node', '--test', '--test-reporter=tap', str(GIT / 'packages/no-mistakes/scripts' / script)], env=env, stdout=out, stderr=err, timeout=120)
        Path(str(prefix) + '.exit').write_text(str(result.returncode))
        stdout = Path(str(prefix) + '.stdout').read_text()
        assert result.returncode == 0 and not Path(str(prefix) + '.stderr').read_bytes()
        def total(label):
            found = re.findall(r'^# ' + label + r' (\d+)$', stdout, re.M)
            assert len(found) == 1
            return int(found[0])
        assert total('tests') == total('pass') == count
        assert total('fail') == total('skipped') == total('cancelled') == 0
        if script == 'postgres-source-api.test.js':
            assert 'compiled CJS and ESM retain legacy nested index identity strings across feature modes' in stdout
        for suffix in ['.stdout', '.stderr', '.exit']:
            pin(str(prefix) + suffix)
sys.path.insert(0, str(D / 'protocol'))
from common import identity
spec = json.loads((D / 'protocol/pair.json').read_text())
signatures = {w['name']: {arm: ({'report_sha256': m['report_hashes']['cli'][w['name']][arm]['sha256'], 'exit': m['report_hashes']['cli'][w['name']][arm]['exit']} if w['kind'] == 'cli' else m['report_hashes']['probe'][w['name']][arm]) for arm in ['baseline', 'candidate']} for w in spec['workloads']}
sha = lambda p: hashlib.file_digest(Path(p).open('rb'), 'sha256').hexdigest()
(public / 'semantic-preflight.json').write_text(json.dumps({'signatures': signatures, 'graphs': m['report_hashes']['graph'], 'consumer_overlay_sha256': sha(ROOT / 'inputs' / spec['inputs']['configured-consumer']['path'] / '.no-mistakes.yml'), 'plain_cold_overlay_sha256': sha(ROOT / 'inputs' / spec['inputs']['default-cold']['path'] / '.no-mistakes.yml')}))
(public / 'preflight.json').write_text(json.dumps(identity(ROOT)))
for path in [public / 'semantic-preflight.json', public / 'preflight.json', ROOT / 'semantic-manifest.json', ROOT / 'reviewed-expected-parity.json']:
    pin(path)
(public / 'hosted-preflight.json').write_text(json.dumps({'status': 'PASS actual fresh VM controls', 'semantic_raw_files': 182, 'semantic_groups_per_arm': 27, 'sql_conditional_passes_per_arm': 1, 'sql_source_passes_per_arm': 34, 'skipped': 0, 'layout_records': layout_count, 'run_id': os.environ['GITHUB_RUN_ID'], 'run_attempt': os.environ['GITHUB_RUN_ATTEMPT'], 'workflow_sha': os.environ['GITHUB_SHA'], 'pins': pins}, indent=2))
