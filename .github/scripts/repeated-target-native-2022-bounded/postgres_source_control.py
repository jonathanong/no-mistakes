import io
import tarfile
"""Untimed2078 real-addon control; never a timing workload or old-binary carry."""
import hashlib
import json
import os
import re
import subprocess

from common import require, write

SCRIPT = 'packages/no-mistakes/scripts/postgres-source-api.test.js'
TARGET = 'compiled CJS and ESM retain legacy nested index identity strings across feature modes'


def validate_pins(spec):
    control = spec['semantic_policy'].get('postgres_source_control')
    assert isinstance(control, dict) and set(control) == {'source_revision', 'rows', 'required_passing_test'}, 'missing PostgreSQL source control pins'
    assert control['source_revision'] == spec['baseline']['revision'], 'PostgreSQL source control revision changed'
    assert control['required_passing_test'] == TARGET, 'PostgreSQL source control required test changed'
    rows = control['rows']
    assert isinstance(rows, list) and rows, 'missing PostgreSQL source control pins'
    paths = []
    scripts = {SCRIPT, 'packages/no-mistakes/scripts/postgres-boolean-stringification-api.test.js',
               'packages/no-mistakes/scripts/postgres-conditional-executors-api.test.js'}
    for row in rows:
        assert isinstance(row, dict) and set(row) == {'path', 'sha256'}, 'missing PostgreSQL source control pins'
        assert row['path'] in scripts or row['path'].startswith(('fixtures/postgres-facts/source/', 'fixtures/postgres-facts/executed-source/')), 'PostgreSQL source control path changed'
        digest = row['sha256']
        assert isinstance(digest, str) and len(digest) == 64 and all(c in '0123456789abcdef' for c in digest), 'missing PostgreSQL source control pins'
        paths.append(row['path'])
    assert len(paths) == len(set(paths)), 'duplicate PostgreSQL source control pins'
    assert scripts | {'fixtures/postgres-facts/source/index-identity-feature-parity.sql',
                      'fixtures/postgres-facts/source/index-identity-feature-parity.expected.json'} <= set(paths), 'missing PostgreSQL source control pins'
    return control


def tap_counts(stdout):
    text = stdout.decode('utf-8')
    counts = {}
    for key in ('tests', 'pass', 'fail', 'skipped'):
        values = re.findall(r'^# ' + key + r' ([0-9]+)$', text, re.MULTILINE)
        require(len(values) == 1, 'postgres_source_test_summary_missing', metric=key)
        counts[key] = int(values[0])
    require(counts['tests'] > 0 and counts['pass'] == counts['tests']
            and counts['fail'] == 0 and counts['skipped'] == 0, 'postgres_source_tests_not_all_real_passes')
    require(re.search(r'^ok [0-9]+ - ' + re.escape(TARGET) + r'$', text, re.MULTILINE) is not None,
            'postgres_source_required_identity_test_missing')
    return counts


def run(root, source, spec, stderr_notices):
    control = validate_pins(spec)
    expected = set(subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', spec['baseline']['revision'], '--', 'fixtures/postgres-facts/source', 'fixtures/postgres-facts/executed-source'], cwd=source, text=True).splitlines())
    expected.update({SCRIPT, 'packages/no-mistakes/scripts/postgres-boolean-stringification-api.test.js', 'packages/no-mistakes/scripts/postgres-conditional-executors-api.test.js'})
    require({row['path'] for row in control['rows']} == expected, 'postgres_source_fixture_pin_inventory_changed')
    blobs={}
    for arm in ('baseline','candidate'):
        raw=subprocess.check_output(['git','archive','--format=tar',spec[arm]['revision'],*sorted(expected)],cwd=source)
        with tarfile.open(fileobj=io.BytesIO(raw),mode='r:')as archive:
            blobs[arm]={member.name:archive.extractfile(member).read()for member in archive if member.isfile()}
        require(set(blobs[arm])==expected,'postgres_source_archive_inventory_changed',arm=arm)
    for row in control['rows']:
        live = (source / row['path']).read_bytes()
        require(hashlib.sha256(live).hexdigest() == row['sha256'], 'postgres_source_live_pin_changed', path=row['path'])
        for arm in ('baseline', 'candidate'):
            blob = blobs[arm][row['path']]
            require(blob == live, 'postgres_source_pair_pin_changed', arm=arm, path=row['path'])
    private = root / 'private/napi-parity'
    private.mkdir(parents=True, exist_ok=True)
    arms = {}
    for arm in ('baseline', 'candidate'):
        addon = root / arm / 'transfer/addon.node'
        require(addon.is_file() and addon.suffix == '.node', 'postgres_source_real_addon_missing', arm=arm)
        env = dict(os.environ, NO_MISTAKES_TEST_NAPI_ADDON_PATH=str(addon))
        proc = subprocess.run(['node', '--test', '--test-reporter=tap', str(source / SCRIPT)],
                              env=env, capture_output=True, timeout=spec['budgets_minutes']['napi_parity'] * 60)
        (private / f'postgres-source-{arm}.stdout').write_bytes(proc.stdout)
        (private / f'postgres-source-{arm}.stderr').write_bytes(proc.stderr)
        (private / f'postgres-source-{arm}.exit').write_text(str(proc.returncode) + '\n')
        require(proc.returncode == 0, 'postgres_source_native_test_failed', arm=arm)
        counts = tap_counts(proc.stdout)
        notices = stderr_notices(proc.stderr)
        arms[arm] = {'counts': counts, 'stdout_sha256': hashlib.sha256(proc.stdout).hexdigest(),
                     'stderr_sha256': hashlib.sha256(proc.stderr).hexdigest(),
                     'adapter_lock_notice_count': notices, 'addon_sha256': hashlib.sha256(addon.read_bytes()).hexdigest()}
    write(root / 'public/postgres-source-parity.json', {'status': 'actual2078 API allPASS/0skip on both fresh addons',
          'pins': control, 'source_pin_sha256': hashlib.sha256(json.dumps(control, sort_keys=True, separators=(',', ':')).encode()).hexdigest(), 'arms': arms, 'timing_workload': False})
    return arms
