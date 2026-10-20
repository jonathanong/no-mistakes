"""Three balanced BCCB blocks on the unchanged consumer without annotation rules."""
import math
import os
from pathlib import Path
import statistics
import subprocess
import sys

import inputs
import common
import retry
import worker
from cohort import derive_config
from report import compare_reports


def summary(rows):
    metrics = ('wallSeconds', 'cpuSeconds', 'peakRssKiB')
    result = {'samplesPerArm': 6, 'balancedBlocks': 3, 'variants': {}, 'adjacentRatios': {}}
    for arm in ('base', 'phases'):
        values = [row for row in rows if row['arm'] == arm]
        result['variants'][arm] = {}
        for metric in metrics:
            samples = [row[metric] for row in values]
            result['variants'][arm][metric] = {
                'samples': samples, 'mean': statistics.fmean(samples),
                'median': statistics.median(samples), 'max': max(samples),
                'empiricalP95': sorted(samples)[math.ceil(.95 * len(samples)) - 1]}
    pairs = [(rows[offset + 1], rows[offset]) for offset in range(0, 12, 4)]
    pairs += [(rows[offset + 2], rows[offset + 3]) for offset in range(0, 12, 4)]
    for metric in metrics:
        ratios = [candidate[metric] / baseline[metric] for candidate, baseline in pairs]
        result['adjacentRatios'][metric] = {
            'samples': ratios, 'median': statistics.median(ratios),
            'mean': statistics.fmean(ratios), 'max': max(ratios)}
    result['tailNote'] = 'Six observations: empirical p95 is the observed maximum, not a tail guarantee.'
    return result


def main():
    common.install_cancel_handler()
    jobs, diagnostics = sys.argv[1:]
    assert jobs in ('1', '4', 'default') and diagnostics in ('off', 'on')
    output = Path('results')
    output.mkdir(exist_ok=True)
    provenance = retry.verify()
    root = Path('fixture').resolve()
    config = output / 'cohort.yml'
    cohort = derive_config(root, config)
    provenance.update(purpose='non-annotation balanced measurement', eligibleForGate=False,
                      collectionCapSeconds=600, jobs=jobs, diagnostics=diagnostics,
                      cohort=cohort, harnessCommit=os.environ['GITHUB_SHA'])
    common.save(output / 'provenance.json', provenance)
    rows = []
    worker.CAP_SECONDS = 600
    for ordinal, role in enumerate(('base', 'phases', 'phases', 'base') * 3):
        names = set(subprocess.check_output(['ps', '-e', '-o', 'comm='], text=True).split())
        assert not names.intersection({'cargo', 'rustc', 'rustdoc', 'cc1', 'ld.lld'})
        assert common.workload(root) == provenance['workload']
        row = worker.sample((Path('cli') / role / 'no-mistakes').resolve(), root,
                            config.resolve(), jobs, diagnostics, False,
                            output, role, ordinal, heartbeat=True)
        row['cpuSeconds'] = (row['userSeconds'] + row['systemSeconds']) if row['eligible'] else None
        rows.append(row)
        common.save(output / 'samples.json', rows)
        assert row['eligible'], 'censored sample is not performance evidence'
        if diagnostics == 'on':
            stderr = (output / f'{ordinal}-{role}.stderr').read_text()
            assert 'postgres.annotation.project' not in stderr, 'annotation entered the cohort'
        if ordinal:
            assert row['exit'] == rows[0]['exit']
            assert row['semanticStderr'] == rows[0]['semanticStderr']
            compare_reports(output / rows[0]['stdout'], output / row['stdout'], with_ledger=False)
    assert common.workload(root) == provenance['workload']
    common.save(output / 'summary.json', summary(rows))
    common.save(output / 'parity.json', {'allReportsByteIdentical': True,
        'allExitsEqual': True, 'semanticStderrEqual': True,
        'reportSha256': rows[0]['stdoutSha256'], 'annotationPhaseAbsent': diagnostics == 'on'})


if __name__ == '__main__':
    main()
