"""Saved colon-named fixture and complete 704-row carrier roundtrip."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import subprocess
import unittest
from unittest.mock import patch

from cohort_transport import pack, unpack, ARCHIVE, MANIFEST
from union import verify

D = Path(__file__).resolve().parent
module_spec = importlib.util.spec_from_file_location('pack_evidence', D / 'pack-evidence.py')
packer = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(packer)
CONTEXT = {'GITHUB_REPOSITORY': 'jonathanong/no-mistakes', 'GITHUB_RUN_ID': 'SYNTHETIC-NOT-A-RUN', 'GITHUB_RUN_ATTEMPT': '1', 'GITHUB_SHA': 'SYNTHETIC', 'APPROVED_SHA': 'SYNTHETIC'}


class Carrier(unittest.TestCase):
    def test_final_cohort_uses_real_packaging_entrypoints(self):
        # Exercise the entrypoints: calling pack() alone misses cohort allowlists.
        fixture = D / 'fixtures/synthetic-archive/7'
        semantic_root = '/tmp/nm2022-main281-current-pair-functional/current-pair-semantics'
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {**CONTEXT, 'COHORT': '7', 'RUNNER_TEMP': directory}):
            def resolve_path(value):
                return fixture if str(value) == semantic_root else Path(value)
            with patch.object(packer, 'Path', side_effect=resolve_path):
                packer.main()
            restored = Path(directory) / 'restored'
            unpack(Path(directory) / '2022-evidence-carrier', restored, '7')
            self.assertEqual((restored / 'semantic/semantic-manifest.json').read_bytes(), (fixture / 'semantic-manifest.json').read_bytes())
            result = subprocess.run(['python3', str(D / 'cohort_transport.py'), 'pack', str(fixture), str(Path(directory) / 'cli-carrier'), '7'], capture_output=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr.decode())

    def test_colon_bytes_and_all704_survive_roundtrip(self):
        roots = []
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, CONTEXT):
            for cohort in range(8):
                source = D / 'fixtures/synthetic-archive' / str(cohort)
                destination = Path(directory) / f'carrier-{cohort}'
                staged = Path(directory) / f'staged-{cohort}'
                packer.stage_tree(source, staged)
                manifest = pack(staged, destination, str(cohort))
                self.assertEqual({p.name for p in destination.iterdir()}, {ARCHIVE, MANIFEST})
                if cohort == 0:
                    self.assertIn('private/graphs/cold-graph:calls-baseline.stderr', manifest['files'])
                restored = Path(directory) / f'restored-{cohort}'
                unpack(destination, restored, str(cohort))
                roots.append(restored)
            result = verify(roots, json.loads((D / 'plan.json').read_text()))
            self.assertEqual(result['rows'], 704)
            self.assertEqual(len(result['workloads']), 16)
            self.assertEqual(sum(len(w['metrics']) for w in result['workloads']), 48)

    def test_wrong_run_context_rejected_before_extract(self):
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, CONTEXT):
            destination = Path(directory) / 'carrier'
            pack(D / 'fixtures/synthetic-archive/0', destination, '0')
            with patch.dict(os.environ, {'GITHUB_RUN_ATTEMPT': '2'}), self.assertRaises(AssertionError):
                unpack(destination, Path(directory) / 'wrong', '0')
            self.assertFalse((Path(directory) / 'wrong').exists())
