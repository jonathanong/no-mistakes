"""Compile actual allocator registration with and without a unit-test allocator."""
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
LIBRARY = ROOT / 'crates/no-mistakes/src/lib.rs'
FIXTURE = ROOT / 'fixtures/performance/mimalloc/allocator-configuration.rs'


def compile_configuration(prefix, testing, feature):
    with tempfile.TemporaryDirectory(prefix='nm-allocator-configuration-') as directory:
        command = ['rustc', '--edition=2021', '--crate-name', 'allocator_configuration',
                   '-o', str(Path(directory) / 'test'), '-']
        if testing:
            command.append('--test')
        if feature:
            command += ['--cfg', 'feature="mimalloc"']
        return subprocess.run(command, input=prefix + FIXTURE.read_text(),
                              text=True, capture_output=True, timeout=60)


class ConfigurationTests(unittest.TestCase):
    def test_actual_registration_allows_test_instrumentation_and_production(self):
        prefix, marker, _ = LIBRARY.read_text().partition('pub mod ast;')
        self.assertTrue(marker, 'library registration boundary moved')
        for testing, feature in [(True, True), (True, False), (False, True), (False, False)]:
            result = compile_configuration(prefix, testing, feature)
            with self.subTest(testing=testing, feature=feature):
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_saved_collision_control_rejects_two_global_allocators(self):
        # Reproduce the original registration rather than weakening the compiler check.
        prefix = LIBRARY.read_text().partition('pub mod ast;')[0]
        original = prefix.replace('all(feature = "mimalloc", not(test))', 'feature = "mimalloc"')
        result = compile_configuration(original, True, True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('cannot define multiple global allocators', result.stderr)


if __name__ == '__main__':
    unittest.main()
