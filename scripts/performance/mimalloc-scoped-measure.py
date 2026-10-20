#!/usr/bin/env python3
"""Fresh-process paired supplemental CLI measurements on one host per platform."""
import argparse
import json
import os
from pathlib import Path
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
MEASURE = runpy.run_path(str(HERE / 'mimalloc-measure.py'))
CONTRACT = runpy.run_path(str(HERE / 'mimalloc-scoped-contract.py'))
REPORT = runpy.run_path(str(HERE / 'mimalloc-report.py'))


def main():
    parser = argparse.ArgumentParser()
    for key in ('system', 'mimalloc', 'consumer', 'output', 'source', 'platform', 'host-id'):
        parser.add_argument('--' + key, required=True)
    args = parser.parse_args()
    assert not os.environ.get('RAYON_NUM_THREADS')
    assert not MEASURE['mimalloc_environment']()
    output = Path(args.output).resolve()
    scopes = CONTRACT['prepare'](args.consumer, output)
    CONTRACT['save'](output / 'scopes.json', scopes)
    artifacts, defaults = {}, {}
    for variant in ('system', 'mimalloc'):
        binary = Path(getattr(args, variant)).resolve()
        binary.chmod(0o755)
        probe = binary.parent / 'rayon-probe'
        probe.chmod(0o755)
        artifacts[variant] = MEASURE['verify_artifact'](binary, variant, args.source, args.platform)
        MEASURE['verify_probe'](probe, artifacts[variant])
        value = json.loads(subprocess.check_output([str(probe)], text=True))
        assert value['expected_mimalloc'] is (variant == 'mimalloc')
        assert value['is_mimalloc_heap'] is (variant == 'mimalloc')
        defaults[variant] = value['default_rayon_jobs']
    MEASURE['verify_pair'](artifacts)
    assert defaults['system'] == defaults['mimalloc'] > 0
    cells = []
    for workload, jobs_list in CONTRACT['SCOPES'].items():
        for diagnostics in (False, True):
            for jobs in jobs_list:
                scope = scopes[workload]
                cell = f'{workload}-diag{int(diagnostics)}-jobs{jobs}'
                rows = []
                for pair in range(4):
                    order = ('system', 'mimalloc') if pair in (0, 3) else ('mimalloc', 'system')
                    for position, variant in enumerate(order):
                        names = set(subprocess.check_output(['ps', '-e', '-o', 'comm='], text=True).split())
                        assert not names.intersection({'cargo', 'rustc', 'rustdoc', 'cc1', 'ld.lld'})
                        artifact = artifacts[variant]
                        metadata = {key: artifact[key] for key in
                                    ('tree', 'source', 'lock', 'rustc', 'cargo', 'target', 'build_flags', 'lto', 'codegenUnits')}
                        metadata.update({key: scope[key] for key in
                                         ('workload_commit', 'workload_tree', 'inventory_sha256', 'config_sha256', 'gitignore_sha256')})
                        metadata.update(variant=variant, workload=workload, platform=args.platform,
                            diagnostics=diagnostics, jobs=jobs, pair=pair, position=position,
                            order='AB' if order[0] == 'system' else 'BA', host_id=args.host_id,
                            measurement_id=args.host_id, expected_exit=1, expected_failure_control=True,
                            artifact_sha256=artifact['no-mistakes']['sha256'],
                            rayon_jobs=defaults[variant] if jobs == 'max' else int(jobs),
                            automatic_rayon_jobs=defaults[variant], num_cpus_default=defaults[variant],
                            rayon_environment=None, mimalloc_environment={}, timeout_seconds=600,
                            scope='supplemental-no-adoption')
                        sample = f'{cell}-pair{pair}-{variant}'
                        argv = MEASURE['command'](str(Path(getattr(args, variant)).resolve()), workload,
                                                  scope['root'], jobs, diagnostics)
                        argv += ['--config', scope['config']]
                        MEASURE['measure'](argv, scope['root'], output, sample, metadata, 600)
                        path = output / (sample + '.json')
                        row = REPORT['read_run'](path)
                        CONTRACT['validate_report'](row, path.with_suffix('.stdout'), path.with_suffix('.stderr'))
                        rows.append(row)
                        print(f'completed {sample}', flush=True)
                summary = REPORT['summarize_group'](rows)
                assert summary['valid'], summary
                cells.append(summary)
                CONTRACT['save'](output / 'scoped-summary.json', {
                    'scope': 'supplemental-no-adoption', 'adopt': False,
                    'complete': False, 'cells': cells})
    assert CONTRACT['digest'](output / '.gitignore') == scopes['consumer-non-annotation']['gitignore_sha256']
    for scope in scopes.values():
        assert CONTRACT['inventory'](scope['root']) == scope['inventory_sha256']
        assert CONTRACT['digest'](scope['config']) == scope['config_sha256']
    scoped = runpy.run_path(str(HERE / 'mimalloc-scoped-report.py'))
    CONTRACT['save'](output / 'scoped-summary.json', scoped['from_directory'](output))


if __name__ == '__main__':
    main()
