"""Saved synthetic fixtures exercise fail-closed reuse, never supply launch pins."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / 'matched-arm'))
from recheck_archive import extract_checked
from recheck_contract import (ARTIFACT_NAME, BUILDER_RUN, BUILDER_SHA, RECHECK_JOB_MINUTES,
                              validate_builds, validate_manifest, validate_metadata, validate_run)

ROOT = HERE.parents[2]
FIXTURE = ROOT / 'fixtures/performance/current-verdict-arm/recheck-contract'


def load(name):
    return json.loads((FIXTURE / name).read_text())


class RecheckContracts(unittest.TestCase):
    def setUp(self):
        self.manifest = load('manifest-synthetic.json')
        self.metadata = load('metadata-synthetic.json')

    def verify_metadata(self, row):
        validate_metadata(row, self.manifest['artifactId'], ARTIFACT_NAME,
                          self.manifest['artifactZipSha256'], BUILDER_RUN, BUILDER_SHA)

    def test_incomplete_launch_manifest_is_fail_closed(self):
        manifest = json.loads((FIXTURE.parent / 'recheck-artifact-pending.json').read_text())
        # The real manifest may be filled after upload; do not hardcode missing metadata forever.
        if manifest['artifactId'] is None:
            with self.assertRaises((AssertionError, TypeError)):
                validate_manifest(manifest)
        else:
            validate_manifest(manifest)
        incomplete = copy.deepcopy(self.manifest)
        incomplete['artifactId'] = None
        incomplete['artifactZipSha256'] = None
        with self.assertRaises((AssertionError, TypeError)):
            validate_manifest(incomplete)

    def test_root_verified_real_manifest_matches_saved_upload_metadata(self):
        manifest = json.loads((FIXTURE.parent / 'recheck-artifact.json').read_text())
        metadata = load('artifact-metadata-real.json')
        validate_manifest(manifest)
        self.assertEqual(manifest['artifactId'], metadata['id'])
        self.assertEqual('sha256:' + manifest['artifactZipSha256'], metadata['digest'])
        self.assertEqual(manifest['builderRun'], metadata['workflow_run']['id'])
        self.assertEqual(manifest['builderCommit'], metadata['workflow_run']['head_sha'])
        self.assertEqual(len(manifest['files']), 12)
        self.assertEqual(manifest['sourceTrees'], {
            'base': '31c5fd596f09b238762546ddf3045bcc98dcc4e3',
            'candidate': 'c9b3626f13eac7013a357b73f893c8e3b442da9e'})

    def test_pinned_manifest_and_actual_builder_proof_contract(self):
        validate_manifest(self.manifest)
        proofs, initial = load('proofs-synthetic.json'), load('initial-synthetic.json')
        validate_builds(self.manifest, proofs, initial)
        for role, field, value in [('base', 'source', '0' * 40),
                                    ('candidate', 'tree', '0' * 40),
                                    ('base', 'artifactHarnessCommit', '0' * 40),
                                    ('candidate', 'features', 'all'),
                                    ('base', 'frozenCliBuild', {'source': 'old'})]:
            changed = copy.deepcopy(proofs)
            changed[role][field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                validate_builds(self.manifest, changed, initial)

    def test_origin_expiry_repository_and_digest_rejections(self):
        self.verify_metadata(self.metadata)
        for key, value in [('id', 2), ('name', 'old-cli'), ('digest', 'sha256:' + '0' * 64),
                           ('expired', True), ('expires_at', '2000-01-01T00:00:00Z')]:
            row = copy.deepcopy(self.metadata)
            row[key] = value
            with self.subTest(field=key), self.assertRaises(AssertionError):
                self.verify_metadata(row)
        for key, value in [('id', 1), ('head_sha', '0' * 40),
                           ('repository_id', 1), ('head_repository_id', 1)]:
            row = copy.deepcopy(self.metadata)
            row['workflow_run'][key] = value
            with self.subTest(origin=key), self.assertRaises(AssertionError):
                self.verify_metadata(row)

    def test_builder_success_is_required_without_waiting_for_entire_matrix(self):
        run, jobs = load('run-synthetic.json'), load('jobs-synthetic.json')
        validate_run(run, jobs)
        for key, value in [('path', '.github/workflows/other.yml'),
                           ('event', 'pull_request'), ('head_branch', 'other')]:
            changed = copy.deepcopy(run)
            changed[key] = value
            with self.subTest(field=key), self.assertRaises(AssertionError):
                validate_run(changed, jobs)
        jobs['jobs'][0]['conclusion'] = 'failure'
        with self.assertRaises(AssertionError):
            validate_run(run, jobs)

    def test_saved_zip_requires_safe_paths_exact_inventory_and_file_hashes(self):
        hashes = load('safe-files.json')
        with tempfile.TemporaryDirectory(prefix='recheck-output-') as directory:
            self.assertEqual(extract_checked(FIXTURE / 'safe.zip', Path(directory) / 'good', hashes), hashes)
            for name in ['traversal.zip', 'symlink.zip']:
                with self.subTest(zip=name), self.assertRaises(AssertionError):
                    extract_checked(FIXTURE / name, Path(directory) / name)
            with self.assertRaises(AssertionError):
                extract_checked(FIXTURE / 'safe.zip', Path(directory) / 'extra', {})
            with self.assertRaises(AssertionError):
                extract_checked(FIXTURE / 'safe.zip', Path(directory) / 'hash', {'base/no-mistakes': '0' * 64})

    def test_actual_zip_digest_is_checked_separately_from_extracted_file(self):
        from unittest.mock import patch
        from common import digest
        from recheck_archive import download
        fixture_zip = FIXTURE / 'safe.zip'
        zip_hash = digest(fixture_zip)
        hashes = load('safe-files.json')
        self.assertNotEqual(zip_hash, hashes['base/no-mistakes'])
        metadata = copy.deepcopy(self.metadata)
        metadata['digest'] = 'sha256:' + zip_hash

        def copy_saved_zip(command, **kwargs):
            kwargs['stdout'].write(fixture_zip.read_bytes())

        with tempfile.TemporaryDirectory(prefix='recheck-download-') as directory:
            with patch.dict('os.environ', {'RUNNER_TEMP': directory, 'GITHUB_TOKEN': 'synthetic'}), \
                    patch('recheck_archive.api', return_value=metadata), \
                    patch('recheck_archive.subprocess.run', side_effect=copy_saved_zip):
                result = download(1, ARTIFACT_NAME, zip_hash, BUILDER_RUN, BUILDER_SHA,
                                  Path(directory) / 'verified', hashes)
                self.assertEqual(result['actualZipSha256'], zip_hash)
                self.assertEqual(result['files'], hashes)
                metadata['digest'] = 'sha256:' + '0' * 64
                with self.assertRaises(AssertionError):
                    download(1, ARTIFACT_NAME, '0' * 64, BUILDER_RUN, BUILDER_SHA,
                             Path(directory) / 'bad', hashes)

    def test_measure_only_workflow_keeps_all_blocks_and_distinct_commits(self):
        import yaml
        path = ROOT / '.github/workflows/current-verdict-ordering-recheck-arm.yml'
        workflow = yaml.safe_load(path.read_text())
        self.assertEqual(set(workflow['jobs']), {'measure-recheck'})
        job = workflow['jobs']['measure-recheck']
        self.assertEqual(job['strategy']['matrix'], {
            'jobs': ['1', '4', 'default'], 'diagnostics': ['off', 'on'], 'ledger': ['off', 'on']})
        self.assertEqual(job['timeout-minutes'], RECHECK_JOB_MINUTES)
        self.assertLessEqual(sum(step['timeout-minutes'] for step in job['steps']), RECHECK_JOB_MINUTES)
        self.assertLess(RECHECK_JOB_MINUTES, 360)
        step = next(step for step in job['steps'] if 'MATCHED_COLLECTION_CAP_SECONDS' in step.get('env', {}))
        self.assertEqual(step['env']['CLI_BUILD_HARNESS_SHA'], BUILDER_SHA)
        self.assertEqual(step['env']['MATCHED_COLLECTION_CAP_SECONDS'], '3600')
        self.assertNotIn('needs', job)
        prepare = next(step for step in job['steps'] if 'recheck_prepare.py' in step.get('run', ''))
        self.assertIn('/recheck-artifact.json', prepare['run'])
        self.assertNotIn('pending', prepare['run'])
        self.assertEqual(workflow['permissions'], {'contents': 'read', 'actions': 'read'})


if __name__ == '__main__':
    unittest.main()
