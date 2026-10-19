"""Saved ELF counterexamples guard the actual-addon TLS contract."""
from pathlib import Path
import runpy
import unittest

MODULE = runpy.run_path(str(Path(__file__).with_name('mimalloc-tls.py')))
FIXTURES = Path(__file__).resolve().parents[2] / 'fixtures/mimalloc-tls'


class TlsTests(unittest.TestCase):
    def test_dynamic_tls_is_loadable_without_static_tls(self):
        MODULE['validate_dynamic']((FIXTURES / 'dynamic.txt').read_text())

    def test_static_tls_and_non_dynamic_output_are_rejected(self):
        for name in ['static.txt', 'not-dynamic.txt']:
            with self.subTest(name=name), self.assertRaises(ValueError):
                MODULE['validate_dynamic']((FIXTURES / name).read_text())


if __name__ == '__main__':
    unittest.main()
