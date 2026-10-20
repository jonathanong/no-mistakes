"""Windows checkout line endings must not change the tracked dependency identity."""
import hashlib
import json
from pathlib import Path
import runpy
import unittest

HERE = Path(__file__).parent
FIXTURES = HERE.parents[1] / 'fixtures/mimalloc-lock'
CHECKOUTS = {key: value.encode() for key, value in json.loads((FIXTURES / 'checkouts.json').read_text()).items()}
IDENTITY = runpy.run_path(str(HERE / 'mimalloc-lock.py'))['lock_identity']


class LockTests(unittest.TestCase):
    def test_checkout_line_endings_preserve_tracked_identity_and_raw_hash(self):
        tracked = CHECKOUTS['tracked']
        expected = hashlib.sha256(tracked).hexdigest()
        for name in ('tracked', 'windows'):
            actual = CHECKOUTS[name]
            proof = IDENTITY(actual, tracked)
            self.assertEqual(proof['lockSha256'], expected)
            self.assertEqual(proof['lockFileSha256'], hashlib.sha256(actual).hexdigest())

    def test_dependency_changes_cannot_be_hidden_as_checkout_conversion(self):
        with self.assertRaisesRegex(ValueError, 'tracked Cargo.lock'):
            IDENTITY(CHECKOUTS['changed'], CHECKOUTS['tracked'])


if __name__ == '__main__':
    unittest.main()
