"""Fresh full raw controls, exact per-arm historical golden hashes; no masks."""
import json
import os
from pathlib import Path
from supervise import command
from transport import sha
from child_fact_order import compare_file

D = Path(__file__).resolve().parent


def run(stage, output, deadline):
    spec = json.loads((D / 'controls.json').read_text())
    env = dict(os.environ, RAYON_NUM_THREADS='4')
    env.pop('GH_TOKEN', None)
    records = []
    for arm, role in [('A26', 'baseline'), ('B26', 'candidate')]:
        assert len(spec[arm]) == 32
        for index, expected in enumerate(spec[arm]):
            argv = [str(stage / (role + '-probe')), *expected['argv'][1:]]
            label = f'{arm}-{index}'
            command(argv, label, output, deadline, 120, env)
            raw = output / (label + '.stdout')
            assert not (output / (label + '.stderr')).read_bytes()
            comparison = compare_file(index, expected['label'], raw, expected)
            records.append({'arm': arm, 'label': expected['label'], 'bytes': raw.stat().st_size, 'sha256': sha(raw), 'comparison': comparison})
        argv = [str(stage / (role + '-sql-probe')), *spec[arm + '-run'][1:]]
        label = arm + '-SQL'
        command(argv, label, output, deadline, 120, env)
        raw = output / (label + '.stdout')
        assert raw.stat().st_size == 19221
        assert sha(raw) == '9e4528c15224dd7fb4396c3396b4021e43855f37012c8bf2cc836bde0f1ab3f8'
        assert not (output / (label + '.stderr')).read_bytes()
    signatures = json.loads((D / 'timing-signatures.json').read_text())
    workloads = json.loads((D / 'workloads.json').read_text())
    smoke = []
    for workload in workloads:
        args = [arg.replace('@INPUT@', '/tmp/nm2019-three-arm-evidence/inputs').replace('@COUNT@', '1') for arg in workload['args']]
        for arm, role in [('A26', 'baseline'), ('B26', 'candidate')]:
            label = arm + '-signature-' + workload['id']
            command([str(stage / (role + '-probe')), *args], label, output, deadline, 120, env)
            raw = (output / (label + '.stdout')).read_bytes()
            assert raw == (signatures[workload['id']][arm] + '\n').encode(), label
            assert not (output / (label + '.stderr')).read_bytes()
            smoke.append({'arm': arm, 'workload': workload['id'], 'signature': signatures[workload['id']][arm]})
    assert len(smoke) == 44
    (output / 'strict22-signatures.json').write_text(json.dumps(smoke, indent=2))
    assert len(records) == 64
    (output / 'fresh-full64.json').write_text(json.dumps({'complete': True, 'normalization': 'Raw first; only indices10/26 pinned full-token map/set order fallback; no masks', 'records': records}, indent=2))
