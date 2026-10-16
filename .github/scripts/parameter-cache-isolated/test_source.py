import ast
import hashlib
import json
import pathlib
import unittest

D = pathlib.Path(__file__).resolve().parent


class SourceTests(unittest.TestCase):
    def test_draft_cannot_activate(self):
        spec = json.loads((D / 'transport-DRAFT.json').read_text())
        self.assertEqual(spec['status'], 'DRAFT')
        self.assertEqual(len(spec['assets']), 4)
        self.assertTrue(all(row['asset_id'] is None for row in spec['assets']))

    def test_exact_original_payload(self):
        spec = json.loads((D / 'transport-DRAFT.json').read_text())
        freeze = json.loads((D / 'payload/original-freeze.json').read_text())
        self.assertEqual(len(spec['files']), 97)
        self.assertEqual([{k: r[k] for k in ['path', 'bytes', 'sha256']} for r in spec['files']], freeze['files'])
        for row in spec['files']:
            if 'payload' in row:
                raw = (D / 'payload' / row['payload']).read_bytes()
                self.assertEqual(len(raw), row['bytes'])
                self.assertEqual(hashlib.sha256(raw).hexdigest(), row['sha256'])

    def test_unchanged_assessor_and_controls(self):
        pins = json.loads((D / 'original-source-hashes.json').read_text())
        for name, digest in pins.items():
            if name != 'measure-once.py':
                self.assertEqual(hashlib.sha256((D / name).read_bytes()).hexdigest(), digest)

    def test_sampler_only_start_label_changes(self):
        source = (D / 'measure-once.py').read_text()
        restored = source.replace('requires a separately approved isolated hosted START.', 'requires an immediate coordinated START.').replace('ACTUAL START on isolated hosted VM after current pair and host controls', 'ACTUAL START after fresh immediate ACKs and host idle')
        pins = json.loads((D / 'original-source-hashes.json').read_text())
        self.assertEqual(hashlib.sha256(restored.encode()).hexdigest(), pins['measure-once.py'])

    def test_no_token_in_recorded_command(self):
        source = (D / 'hosted.py').read_text()
        self.assertNotIn('Bearer ', source)
        self.assertIn("clean.pop('GH_TOKEN', None)", source)
        self.assertNotIn("releases/assets/", source)
        self.assertIn("verify_staged_assets(stage, spec, os.environ)", source)

    def test_python_parses(self):
        for path in D.glob('*.py'):
            ast.parse(path.read_text(), filename=str(path))


if __name__ == '__main__':
    unittest.main()
