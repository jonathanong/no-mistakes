"""DRAFT B26 hosted boundary; no activation without a closed reviewed package."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import sys
import time
import subreaper
from controls import run as controls
from deadline import install
from eligibility import verify as eligibility
from schedule import verify_rows
from supervise import command
from transport import sha, verify
from retain import compress
from dependency_guard import verify as dependency_guard
from input_guard import verify as input_guard

D = Path(__file__).resolve().parent


def main():
    cohort = sys.argv[1]
    assert cohort in ['A', 'B']
    out = Path(os.environ['RUNNER_TEMP']) / ('mutable-root-B26-' + cohort)
    out.mkdir(exist_ok=False)
    subreaper.enable()
    complete = False
    try:
        end = install(out, 'Child2064 complete workload cohort (' + cohort + ')')
        spec = json.loads((D / 'hosted-spec.json').read_text())
        assert spec['status'] == 'REVIEWED AND FROZEN'
        assert os.environ['APPROVED_DIAGNOSTIC_COMMIT'] == os.environ['GITHUB_SHA']
        assert spec['approved_diagnostic_commit'] is None  # External exact SHA avoids a self-referential file hash.
        assert __import__('subprocess').check_output(['git', 'rev-parse', 'HEAD'], timeout=10).decode().strip() == os.environ['GITHUB_SHA']
        assert not __import__('subprocess').check_output(['git', 'status', '--porcelain', '--untracked-files=no'], timeout=10).strip()
        pins = json.loads((D / 'source-manifest.json').read_text())
        for name, digest in pins.items():
            assert sha(D / name) == digest, name
        assert platform.machine() == 'x86_64'
        assert {0, 1, 2, 3} <= os.sched_getaffinity(0)
        assets = json.loads((D / 'assets.json').read_text())
        stage = Path(os.environ['RUNNER_TEMP']) / 'native-stage'
        proof = verify(stage, assets, os.environ)
        (out / 'verified-transfer.json').write_text(json.dumps(proof, indent=2))
        for row in json.loads((D / 'payload-files.json').read_text()):
            source = D / 'payload' / row['payload']
            assert source.parent == D / 'payload' and not source.is_symlink()
            assert source.stat().st_size == row['bytes'] and sha(source) == row['sha256']
            target = Path(row['path'])
            assert str(target).startswith('/tmp/') and '..' not in target.parts
            assert not target.exists() and not target.is_symlink()
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
        source = out / 'pair'
        manifests = json.loads((D / 'producer-manifests.json').read_text())
        for arm, role in [('A26', 'baseline'), ('B26', 'candidate')]:
            root = source / arm
            root.mkdir(parents=True)
            shutil.copyfile(stage / (role + '-probe'), root / 'probe')
            (root / 'probe').chmod(0o700)
            assert sha(root / 'probe') == manifests[arm]['binary_sha256']
        host = {'cohort': cohort, 'platform': platform.platform(), 'cpuinfo': Path('/proc/cpuinfo').read_text(),
                'meminfo': Path('/proc/meminfo').read_text(), 'affinity': sorted(os.sched_getaffinity(0)),
                'run_id': os.environ['GITHUB_RUN_ID'], 'attempt': os.environ['GITHUB_RUN_ATTEMPT'],
                'workflow_sha': os.environ['GITHUB_SHA'], 'no_local_quiet_lease': True}
        (out / 'host.json').write_text(json.dumps(host, indent=2))
        dependencies = Path(os.environ['RUNNER_TEMP']) / ('B26-dependencies-' + cohort)
        dependency_proof = json.loads((dependencies / 'closure.json').read_text())
        assert sha(Path('candidate-source/pnpm-lock.yaml')) == spec['candidate_lock_sha256']
        dependency_guard(dependency_proof, spec['candidate_head'], spec['candidate_lock_sha256'])
        input_files = json.loads((D / 'payload-files.json').read_text())
        required_inputs = json.loads((D / 'required-inputs.json').read_text())
        input_guard(input_files, required_inputs)
        eligibility(spec, out, end, 'before-controls')
        raw = out / 'full-controls'
        raw.mkdir()
        controls(stage, raw, end)
        compress(raw)
        # API source closure and exact TAP locks are separately declared by the
        # closed package. A missing control implementation never admits timing.
        assert spec['api_controls_complete'] is True
        command([sys.executable, str(D / 'api_controls.py'), str(stage), str(out), str(end)],
                'real-and-shim-api-controls', out, end, 180)
        eligibility(spec, out, end, 'before-sampling')
        assert end - time.monotonic() >= spec['sampling_minimum_remaining_seconds']
        (out / 'START.json').write_text(json.dumps({'status': 'ACTUAL START on isolated VM',
            'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'cohort': cohort,
            'source_spec_sha256': sha(D / 'hosted-spec.json'), 'host_sha256': sha(out / 'host.json')}, indent=2))
        clean = dict(os.environ, RAYON_NUM_THREADS='4')
        clean.pop('GH_TOKEN', None)
        command([sys.executable, str(D / 'sample_worker.py'), str(out), str(source), cohort, str(end)],
                'complete-cohort-sampling', out, end, end - time.monotonic(), clean)
        eligibility(spec, out, end, 'after-sampling')
        rows = json.loads((out / 'A26_B26-long-all-rows.json').read_text())
        verify_rows(rows, cohort, manifests, json.loads((D / 'timing-signatures.json').read_text()))
        for row in assets:
            assert sha(stage / row['name']) == row['sha256']
        for row in json.loads((D / 'payload-files.json').read_text()):
            assert Path(row['path']).stat().st_size == row['bytes'] and sha(row['path']) == row['sha256']
        input_guard(input_files, required_inputs)
        assert time.monotonic() < end
        complete = True
    finally:
        (out / 'closure.json').write_text(json.dumps({'complete': complete, 'cohort': cohort,
            'run_id': os.environ['GITHUB_RUN_ID'], 'attempt': os.environ['GITHUB_RUN_ATTEMPT'],
            'workflow_sha': os.environ['GITHUB_SHA'], 'no_numerical_gate_claim': True}, indent=2))


if __name__ == '__main__':
    main()
