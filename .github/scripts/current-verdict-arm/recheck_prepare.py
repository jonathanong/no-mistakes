"""Validate and restore a fresh-source CLI artifact before measurement only."""
import json
from pathlib import Path
import sys

from inputs import ARTIFACT_ID, ARTIFACT_ZIP_SHA256, FIXTURE_TAR_SHA256, HELPERS
sys.path.insert(0, str(HELPERS))
import common
import long_frozen
from recheck_archive import api, download
from recheck_contract import (ARTIFACT_NAME, BUILDER_RUN, BUILDER_SHA,
                              validate_builds, validate_manifest, validate_run)


def main(manifest_path):
    common.install_cancel_handler()
    output = Path('results')
    output.mkdir(exist_ok=True)
    common.save(output / 'recheck-preparation.json', {'status': 'incomplete', 'eligibleForGate': False})
    try:
        manifest = json.loads(Path(manifest_path).read_text())
        validate_manifest(manifest)
        validate_run(api(f'actions/runs/{BUILDER_RUN}'),
                     api(f'actions/runs/{BUILDER_RUN}/jobs?per_page=100'))
        cli = download(manifest['artifactId'], ARTIFACT_NAME, manifest['artifactZipSha256'],
                       BUILDER_RUN, BUILDER_SHA, 'cli', manifest['files'])
        proofs = {role: json.loads((Path('cli') / role / 'provenance.json').read_text())
                  for role in manifest['sources']}
        initial = {role: json.loads((Path('cli') / role / 'source.json').read_text())
                   for role in manifest['sources']}
        validate_builds(manifest, proofs, initial)
        consumer = download(ARTIFACT_ID, 'matched-consumer', ARTIFACT_ZIP_SHA256,
                            long_frozen.RUN, long_frozen.HEAD, 'prepared')
        assert set(consumer['files']) == {'fixture.tar.gz', 'fixture-provenance.json'}
        assert common.digest('prepared/fixture.tar.gz') == FIXTURE_TAR_SHA256
        common.save(output / 'recheck-preparation.json', {
            'status': 'passed', 'eligibleForGate': False, 'manifest': manifest,
            'manifestSha256': common.digest(manifest_path), 'cliArtifact': cli,
            'consumerArtifact': consumer, 'fixtureTarSha256': FIXTURE_TAR_SHA256})
    except BaseException as error:
        common.save(output / 'recheck-preparation.json', {
            'status': 'failed', 'eligibleForGate': False,
            'reason': f'{type(error).__name__}: {error}'})
        raise


if __name__ == '__main__':
    main(sys.argv[1])
