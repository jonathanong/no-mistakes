"""Saved known-rollout and incompatible-host counterexamples."""
import json
from pathlib import Path
import unittest

from image_contract import image_provenance

FIXTURES = Path(__file__).resolve().parents[3] / 'fixtures/performance/matched-arm/build-size'


class ImageContractTests(unittest.TestCase):
    def test_discloses_known_rollout_between_separate_build_and_measurement_stages(self):
        row = json.loads((FIXTURES / 'image-rollout.json').read_text())
        proof = image_provenance(row['builder'], row['measurement'])
        self.assertFalse(proof['versionsMatch'])
        self.assertEqual(proof['builder'], '20261012.149.1')
        self.assertEqual(proof['measurement'], '20261018.154.1')
        self.assertTrue(image_provenance(row['builder'], row['builder'])['versionsMatch'])

    def test_rejects_unknown_images_and_incompatible_architecture(self):
        for name in ['image-unrecognized.json', 'image-wrong-arch.json']:
            row = json.loads((FIXTURES / name).read_text())
            with self.subTest(fixture=name), self.assertRaises(AssertionError):
                image_provenance(row['builder'], row['measurement'])


if __name__ == '__main__':
    unittest.main()
