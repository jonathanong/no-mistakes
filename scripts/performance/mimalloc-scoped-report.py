"""Recompute supplemental cells from raw files without qualifying adoption cells."""
from collections import defaultdict
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
REPORT = runpy.run_path(str(HERE / 'mimalloc-report.py'))
CONTRACT = runpy.run_path(str(HERE / 'mimalloc-scoped-contract.py'))
SCOPE_FIELDS = ('config_sha256', 'gitignore_sha256', 'inventory_sha256', 'workload_commit', 'workload_tree')


def summarize(rows, scopes):
    groups = defaultdict(list)
    hosts, identities, binaries = set(), set(), defaultdict(set)
    semantics, defaults = defaultdict(set), set()
    for row in rows:
        meta = row['metadata']
        assert meta['scope'] == 'supplemental-no-adoption'
        workload = meta['workload']
        assert workload in CONTRACT['SCOPES']
        assert all(meta[key] == scopes[workload][key] for key in SCOPE_FIELDS)
        assert row['cwd'] == scopes[workload]['root']
        expected = [row['executable'], '--timeout', '0', '--lock-timeout', '0', 'check',
                    '--root', scopes[workload]['root'], '--format', 'json']
        if meta['jobs'] != 'max':
            expected += ['--jobs', meta['jobs']]
        if meta['diagnostics']:
            expected += ['--progress', '--verbose-timings']
        expected += ['--config', scopes[workload]['config']]
        assert row['argv'] == expected
        groups[(workload, meta['diagnostics'], meta['jobs'])].append(row)
        hosts.add(tuple(row[key] for key in REPORT['HOST_FIELDS'] if key != 'cwd'))
        identities.add(tuple(json.dumps(meta[key], sort_keys=True) for key in
                             ('source', 'tree', 'lock', 'rustc', 'cargo', 'build_flags')))
        binaries[meta['variant']].add(meta['artifact_sha256'])
        semantics[workload].add(tuple(row[key] for key in REPORT['SEMANTICS']) + (row['exit'],))
        defaults.add((meta['automatic_rayon_jobs'], meta['num_cpus_default']))
    expected = {(workload, diagnostics, jobs) for workload, jobs_list in CONTRACT['SCOPES'].items()
                for diagnostics in (False, True) for jobs in jobs_list}
    assert set(groups) == expected, 'missing or unexpected supplemental cells'
    assert len(hosts) == len(identities) == len(defaults) == 1
    assert all(len(values) == 1 for values in semantics.values())
    assert set(binaries) == {'system', 'mimalloc'}
    assert all(len(values) == 1 for values in binaries.values())
    cells = [REPORT['summarize_group'](entries) for _, entries in sorted(groups.items())]
    assert all(cell['valid'] for cell in cells), cells
    return {'scope': 'supplemental-no-adoption', 'adopt': False, 'complete': True, 'cells': cells,
            'note': 'Four pairs per cell; scoped observations do not clear the full adoption gate.'}


def from_directory(output):
    output = Path(output)
    scopes = json.loads((output / 'scopes.json').read_text())
    rows = []
    for path in sorted(output.glob('*-pair*-*.json')):
        row = REPORT['read_run'](path)
        for suffix in ('stdout', 'stderr'):
            assert CONTRACT['digest'](path.with_suffix('.' + suffix)) == row[suffix + '_sha256']
        CONTRACT['validate_report'](row, path.with_suffix('.stdout'), path.with_suffix('.stderr'))
        rows.append(row)
    return summarize(rows, scopes)


if __name__ == '__main__':
    import sys
    CONTRACT['save'](Path(sys.argv[1]) / 'scoped-summary.json', from_directory(sys.argv[1]))
