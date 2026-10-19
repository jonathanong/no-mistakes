"""Exact fresh-build sources and feasibility-only capacity limits."""
from pathlib import Path
import sys

HELPERS = Path(__file__).resolve().parents[1] / 'matched-arm'
sys.path.insert(0, str(HELPERS))
SOURCES = {
    'base': 'e377c2ba31c3f1a59f39054b67d3eafa7c2a850a',
    'phases': 'b2701498b16bd63cd08993283151c4f169275dbe',
}
CAP_SECONDS = 10000
JOB_MINUTES = 355
MEASURE_MINUTES = 338
BUILD_FIELDS = ('lockSha256', 'rustc', 'cargo', 'features', 'profile',
                'command', 'buildJobs', 'incremental')


def outcome(rows, status='incomplete'):
    return {'purpose': 'feasibility-only', 'eligibleForGate': False,
            'order': 'bc', 'collectionCapSeconds': CAP_SECONDS,
            'status': status, 'parityVerified': status == 'passed',
            'completedArms': sum(row['eligible'] for row in rows),
            'sameMachine': True,
            'note': 'Capacity observations only; no ratio or regression-gate conclusion.'}


def validate_parity(output, rows):
    from report import compare_reports
    assert len(rows) == 2 and all(row['eligible'] for row in rows)
    assert rows[0]['exit'] == rows[1]['exit']
    assert rows[0]['semanticStderr'] == rows[1]['semanticStderr']
    return compare_reports(output / rows[0]['stdout'], output / rows[1]['stdout'],
                           with_ledger=False)
