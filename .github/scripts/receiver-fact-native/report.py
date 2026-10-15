"""Readable, text-only evidence. Inconclusive is never reported as no overhead."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

from common import write, failure_record, ExperimentFailure
from summary import assess, stable, validate


def report(root):
    public = root / 'public'
    public.mkdir(parents=True, exist_ok=True)
    incomplete = False
    text = ['# Historical imported-receiver native experiment', '',
            'Refs #2012 and #1972. Historical 57c927d to 8d81aa1; not current main.',
            'A hosted VM is not a claim of dedicated physical CPUs or PMU availability.',
            'No startup subtraction, outlier removal, or selective reruns.',
            'Full build/cache logs, libraries and executables are withheld.',
            'PMU preflight checks capability only; these sample results are native timing, not hardware-counter attribution.', '',
            '| Profile | Result |', '| --- | --- |']
    for profile in ('opt3', 'opt0'):
        path = public / f'{profile}-assessment.json'
        failures = sorted(public.glob('*-failure.json'))
        missing = failure_record(ExperimentFailure('profile_stage_not_completed'), 'report', profile=profile, arm=None)
        for failure_path in failures:
            failure = json.loads(failure_path.read_text())
            if failure.get('profile') == profile or failure.get('stage') == 'prepare':
                missing = failure
        assessment = json.loads(path.read_text()) if path.exists() else missing
        try:
            if assessment['status'] == 'incomplete':
                raise ExperimentFailure('existing_incomplete_stage')
            for tag, blocks in (('original100', 4), ('pilot', 1), ('long', 4)):
                rows_path = public / f'{profile}-{tag}-rows.json'
                if tag != 'long' and not rows_path.exists():
                    raise ValueError('required sample stage missing')
                if rows_path.exists():
                    rows = json.loads(rows_path.read_text())
                    validate(rows, blocks)
                    if tag == 'original100' and any(r['collections'] != 100 for r in rows):
                        raise ValueError('original batch size mismatch')
            long_path = public / f'{profile}-long-rows.json'
            if long_path.exists():
                assert assess(json.loads(long_path.read_text())) == assessment
            else:
                pilot = json.loads((public / f'{profile}-pilot-rows.json').read_text())
                if assessment['status'] != 'inconclusive' or stable(pilot):
                    raise ValueError('missing full rows without unstable pilot')
        except ExperimentFailure:
            pass  # Retain the original actionable stage/arm/profile failure record.
        except (ValueError, AssertionError, KeyError, FileNotFoundError):
            assessment = failure_record(ExperimentFailure('sample_report_integrity_mismatch'),
                                        'report', profile=profile, arm=None)
        incomplete |= assessment['status'] == 'incomplete'
        write(public / f'{profile}-assessment.json', assessment)
        text.append(f"| {profile} | {assessment['status']} |")
        if assessment['status'] == 'incomplete':
            text += ['', f"Incomplete: stage={assessment.get('stage')}, arm={assessment.get('arm')}, "
                     f"profile={profile}, invariant={assessment.get('error_code')}; {assessment.get('details', {})}."]
        for demand, result in assessment.get('demands', {}).items():
            text += ['', f"{profile}/{demand}: {result['status']}; paired CPU {result['paired_cpu_percent']:+.3f}%; "
                     f"descriptive 95% interval {result['descriptive_bootstrap_95_percent']}; "
                     f"ABBA/BAAB {result['orders_percent']}; arm CPU CV {result['arm_cpu_cv']}."]
    text += ['', 'Intervals resample the eight balanced block log-ratios (10,000 draws, seed 2012).',
             'They describe observed variation, not guaranteed stationarity or tail latency.',
             'Positive cost needs stable samples, matching order directions and an interval above zero;',
             'material regression requires the lower bound to reach 3%. Other results remain inconclusive.',
             'A confirmed regression requires attribution before a runtime fix; #1972 is not closed here.',
             'The original cloud-host opt0 slowdown remains an independent preserved observation.', '']
    (public / 'summary.md').write_text('\n'.join(text))
    files = []
    for path in sorted(public.iterdir()):
        if path.name == 'checksums.json':
            continue
        assert path.suffix in ('.json', '.md') and path.is_file()
        data = path.read_bytes()
        content = data.decode('utf-8')
        # Curated records contain no machine-specific absolute workspace paths.
        assert str(root) not in content
        files.append({'path': path.name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
    write(public / 'checksums.json', {'scope': 'exact public UTF-8 files; manifest excludes itself', 'files': files})
    return 1 if incomplete else 0


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    sys.exit(report(parser.parse_args().root.resolve()))
