"""Untimed async CJS/ESM standalone, prepared and CLI parity for #2022."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

from common import require, write
from pair import HERE, ROOT, SPEC
from postgres_source_control import run as run_postgres_source_control


def normalized(data, fixture):
    return json.loads(data.decode().replace(str(fixture), '<fixture>'))


def sha(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


LOCK_NOTICE = re.compile(r'waiting for lock held by (?:pid [0-9]+|another no-mistakes invocation) for 0s')


def adapter_lock_notice_count(stderr):
    """Allow only the exact benign same-process lock wait line, retaining bytes."""
    try:
        lines = stderr.decode('utf-8').splitlines()
    except UnicodeDecodeError:
        require(False, 'native_adapter_unexpected_stderr')
    require(all(LOCK_NOTICE.fullmatch(line) for line in lines), 'native_adapter_unexpected_stderr')
    return len(lines)


def facade_identity():
    expected = SPEC['semantic_policy']['napi_facade_sha256']
    require(set(expected) == {'index.js', 'index.mjs'}, 'napi_facade_pin_set_changed')
    for filename, digest in expected.items():
        path = 'packages/no-mistakes/' + filename
        live = (ROOT / path).read_bytes()
        require(hashlib.sha256(live).hexdigest() == digest, 'napi_facade_live_changed', path=path)
        for arm in ('baseline', 'candidate'):
            blob = subprocess.check_output(['git', 'show', SPEC[arm]['revision'] + ':' + path], cwd=ROOT)
            require(blob == live, 'napi_facade_pair_changed', arm=arm, path=path)


def run_conditional_executor_control(root):
    """Untimed saved2075 regression on each real new-arm addon, never a workload."""
    control = SPEC['semantic_policy']['conditional_executor_control']
    for key in ('fixture', 'script'):
        row = control[key]
        live = (ROOT / row['path']).read_bytes()
        require(hashlib.sha256(live).hexdigest() == row['sha256'], 'conditional_executor_live_pin_changed', kind=key)
        for arm in ('baseline', 'candidate'):
            blob = subprocess.check_output(['git', 'show', SPEC[arm]['revision'] + ':' + row['path']], cwd=ROOT)
            require(blob == live, 'conditional_executor_pair_pin_changed', arm=arm, kind=key)
    private = root / 'private/napi-parity'
    private.mkdir(parents=True, exist_ok=True)
    results = {}
    for arm in ('baseline', 'candidate'):
        addon = root / arm / 'transfer/addon.node'
        require(addon.is_file() and addon.suffix == '.node', 'conditional_executor_real_addon_missing', arm=arm)
        env = dict(os.environ, NO_MISTAKES_TEST_NAPI_ADDON_PATH=str(addon))
        proc = subprocess.run(['node', '--test', '--test-reporter=tap', str(ROOT / control['script']['path'])],
                              env=env, capture_output=True, timeout=120)
        (private / f'conditional-executors-{arm}.stdout').write_bytes(proc.stdout)
        (private / f'conditional-executors-{arm}.stderr').write_bytes(proc.stderr)
        require(proc.returncode == 0, 'conditional_executor_native_test_failed', arm=arm)
        counts = conditional_executor_tap_counts(proc.stdout)
        notices = adapter_lock_notice_count(proc.stderr)
        results[arm] = {'tests': counts['tests'], 'passed': counts['pass'], 'failed': counts['fail'],
                        'skipped': counts['skipped'], 'stdout_sha256': hashlib.sha256(proc.stdout).hexdigest(),
                        'stderr_sha256': hashlib.sha256(proc.stderr).hexdigest(), 'adapter_lock_notice_count': notices,
                        'addon_sha256': hashlib.sha256(addon.read_bytes()).hexdigest()}
    write(root / 'public/conditional-executor-parity.json', {'status': 'real saved conditional-executor CJS/ESM control passed on both new addons', 'pins': control, 'arms': results, 'timing_workload': False})
    return results


def conditional_executor_tap_counts(stdout):
    text = stdout.decode('utf-8')
    counts = {}
    for key in ('tests', 'pass', 'fail', 'skipped'):
        values = re.findall(r'^# ' + key + r' ([0-9]+)$', text, re.MULTILINE)
        require(len(values) == 1, 'conditional_executor_test_summary_missing', metric=key)
        counts[key] = int(values[0])
    require(counts == {'tests': 1, 'pass': 1, 'fail': 0, 'skipped': 0}, 'conditional_executor_test_not_one_real_pass')
    return counts


def run(root):
    facade_identity()
    private = root / 'private/napi-parity'
    private.mkdir(parents=True, exist_ok=True)
    cases = SPEC['semantic_policy']['napi_cases']
    expected = SPEC['semantic_policy']['report_hashes']['napi']
    require({case['prefix'] for case in cases} == set(expected), 'native_case_hash_coverage_incomplete')
    results = {}
    for case in cases:
        prefix = case['prefix']
        fixture = root / 'inputs' / SPEC['inputs'][case['input']]['path']
        entries = case['entries']
        require(entries and len(entries) == len(set(entries)), 'native_entry_catalog_changed', fixture=prefix)
        reports = [{'type': 'effects', 'id': f'{prefix}-{i}', 'kind': case['kind'], 'entry': entry}
                   for i, entry in enumerate(entries)]
        reports.append({'type': 'check', 'id': f'{prefix}-check'})
        results[prefix] = {}
        for arm in ('baseline', 'candidate'):
            addon = root / arm / 'transfer/addon.node'
            env = dict(os.environ, NO_MISTAKES_TEST_NAPI_ADDON_PATH=str(addon))
            proc = subprocess.run(['node', str(HERE / 'napi-parity.mjs'), str(ROOT / 'packages/no-mistakes'),
                                   str(fixture), json.dumps(reports)], env=env, capture_output=True,
                                  timeout=SPEC['budgets_minutes']['napi_parity'] * 60)
            (private / f'{prefix}-{arm}.stdout').write_bytes(proc.stdout)
            (private / f'{prefix}-{arm}.stderr').write_bytes(proc.stderr)
            require(proc.returncode == 0, 'native_cjs_esm_execution_failed', arm=arm, fixture=prefix)
            lock_notice_count = adapter_lock_notice_count(proc.stderr)
            report = normalized(proc.stdout, fixture)
            require(len(report['reports']) == len(reports), 'native_report_count_changed', arm=arm, fixture=prefix)
            entries_hashes = {}
            for i, entry in enumerate(entries):
                row = report['reports'][i]
                require(row['id'] == f'{prefix}-{i}', 'native_report_order_changed', arm=arm, fixture=prefix)
                cmd = [str(root / arm / 'transfer/cli'), 'effects', case['kind'], '--entry', entry,
                       '--root', str(fixture), '--format', 'json']
                cli = subprocess.run(cmd, capture_output=True, timeout=60)
                (private / f'{prefix}-{arm}-{i}.stdout').write_bytes(cli.stdout)
                (private / f'{prefix}-{arm}-{i}.stderr').write_bytes(cli.stderr)
                require(cli.returncode == 0 and not cli.stderr, 'native_effects_cli_failed', arm=arm, entry=entry)
                parsed = normalized(cli.stdout, fixture)
                require(row['result'] == parsed, 'native_effects_cli_report_changed', arm=arm, entry=entry)
                entries_hashes[entry] = sha(parsed)
            require(report['reports'][-1]['id'] == f'{prefix}-check', 'native_check_order_changed', arm=arm, fixture=prefix)
            check = subprocess.run([str(root / arm / 'transfer/cli'), 'check', '--root', str(fixture),
                                    '--format', 'json'], capture_output=True, timeout=60)
            (private / f'{prefix}-{arm}-check.stdout').write_bytes(check.stdout)
            (private / f'{prefix}-{arm}-check.stderr').write_bytes(check.stderr)
            require(check.returncode in (0, 1) and not check.stderr, 'native_check_cli_failed', arm=arm, fixture=prefix)
            parsed = normalized(check.stdout, fixture)
            require(report['reports'][-1]['result'] == parsed, 'native_check_cli_report_changed', arm=arm, fixture=prefix)
            require(not parsed['warnings'], 'native_check_warnings', arm=arm, fixture=prefix)
            digest = sha(report)
            require(digest == expected[prefix][arm], 'native_full_report_hash_changed', arm=arm, actual=digest, fixture=prefix)
            results[prefix][arm] = {'full_report_sha256': digest, 'entries': entries_hashes,
                                    'check_sha256': sha(parsed), 'adapter_lock_notice_count': lock_notice_count}
    run_conditional_executor_control(root)
    run_postgres_source_control(root, ROOT, SPEC, adapter_lock_notice_count)
    write(root / 'public/napi-parity.json', results)
    return results
