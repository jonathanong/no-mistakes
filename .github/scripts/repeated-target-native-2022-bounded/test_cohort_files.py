"""Partial/foreign/tampered public artifacts cannot be promoted into a union."""
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from cohort_files import close,verify


class CohortFileTests(unittest.TestCase):
    def test_foreign_or_mutated_public_artifact_rejects(self):
        for mutation in ('run','commit','cohort','bytes','extra','missing','symlink'):
            with tempfile.TemporaryDirectory()as name,patch.dict(os.environ,{'GITHUB_RUN_ID':'12','APPROVED_DIAGNOSTIC_COMMIT':'a'*40}):
                root=Path(name);public=root/'public';public.mkdir();(public/'rows.json').write_text('[]')
                close(root,'0');self.assertEqual(verify(public,'0')['cohort'],'0')
                manifest=public/'cohort-artifact-manifest.json';row=json.loads(manifest.read_text())
                if mutation=='run':row['run_id']='99'
                elif mutation=='commit':row['diagnostic_commit']='b'*40
                elif mutation=='cohort':row['cohort']='1'
                elif mutation=='bytes':(public/'rows.json').write_text('[1]')
                elif mutation=='extra':(public/'extra').write_text('extra')
                elif mutation=='missing':(public/'rows.json').unlink()
                else:(public/'link').symlink_to(public,target_is_directory=True)
                manifest.write_text(json.dumps(row))
                with self.assertRaises(AssertionError):verify(public,'0')
