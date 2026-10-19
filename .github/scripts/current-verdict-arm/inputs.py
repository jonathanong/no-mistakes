"""Pinned source, consumer, and ordering-only inputs for the temporary #2238 matrix."""
from pathlib import Path

HELPERS = Path(__file__).resolve().parents[1] / 'matched-arm'
SOURCES = {
    'base': 'c009d087f3eb32b27a27d6a200154e26b7be0a5d',
    'candidate': '1f65a505367720f4519e486d300036fad4eb3adc',
}
CONSUMER = 'f38a048f5eeff0438fc59a15e9eb742def575a36'
ARTIFACT_ID = 12158768554
ARTIFACT_SHA256 = '080223598499563b3f21bdd29e450536db72183c5ef65e3d3f221976273c6911'
CONFIG_SHA256 = '14fd858598a1a18607ce0a50442f0ae7b2050f98e1edcd5bd05ba967983d23ab'
ORIGINAL_CONFIG_SHA256 = 'c9fe0bae7cb65b06cbe89b66773bac19b3176a8601f4860e857d42dce169897c'
CAP_SECONDS = 3600
JOB_MINUTES = 260
MEASURE_MINUTES = 245
JOBS = ('1', '4', 'default')
DIAGNOSTICS = ('off', 'on')
LEDGER = ('off', 'on')
BUILD_FIELDS = ('lockSha256', 'rustc', 'cargo', 'features', 'profile',
                'command', 'buildJobs', 'incremental')
EXPECTED_FINDINGS = {
    ('backend/services/post-publication/identity-bridge-cleanup.mts', 18,
     'postgres-conflict-ordering', 'targetless-arbiter'),
    ('backend/services/posts/review-successions/mutations.mts', 94,
     'postgres-conflict-ordering', 'missing-canonical-order'),
}
