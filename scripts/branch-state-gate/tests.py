#!/usr/bin/env python3
"""Saved-fixture tests for the hosted measurement acceptance guard."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from tool_tests import ToolProvisionTests
from build import compile_cpu_probe
from summarize import nearest_rank, validate_cells, validate_pairs, validate_sample


class AcceptanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        fixture = Path(__file__).resolve().parents[2] / 'fixtures/performance/branch-state-gate/acceptance-cases.json'
        cls.cases = json.loads(fixture.read_text())

    def test_accepts_only_complete_exact_report_with_expected_status(self):
        validate_sample(self.cases['valid'], ('1', 'off'), 1)

    def test_rejects_timeout_and_report_drift(self):
        for name in ('censored', 'wrong_report'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                validate_sample(self.cases[name], ('1', 'off'), 1)

    def test_nearest_rank_p95_for_small_samples_is_the_observed_maximum(self):
        self.assertEqual(nearest_rank(self.cases['rank_values'], 95), 6)

    def test_rejects_missing_cell_or_pair_evidence(self):
        cells = {tuple(cell) for cell in self.cases['present_cells']}
        with self.assertRaises(ValueError):
            validate_cells(cells)
        pairs = [{'pair': number} for number in self.cases['missing_pairs']]
        with self.assertRaises(ValueError):
            validate_pairs(pairs)

    def test_cpu_probe_links_lto_dependency_with_matching_fat_lto(self):
        fixture = Path(__file__).resolve().parents[2] / 'fixtures/performance/branch-state-gate/cpu-probe'
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            dependency = output / 'libnum_cpus.rlib'
            subprocess.run(['rustc', '--crate-name', 'num_cpus', '--crate-type', 'rlib',
                            '-Clinker-plugin-lto', str(fixture / 'num_cpus.rs'), '-o', str(dependency)],
                           check=True)
            common = ['rustc', str(fixture / 'main.rs'), '--edition=2021', '-L',
                      'dependency='+temporary, '--extern', 'num_cpus='+str(dependency)]
            # Rust's x86 default linker accepts bitcode. Use the hosted ARM
            # runner's GNU linker behavior so this regression fails on either CPU.
            linker = ['-Clinker=cc', '-Clink-arg=-fuse-ld=bfd']
            real_run = subprocess.run
            failed = real_run(common + linker + ['-o', str(output / 'vanilla-probe')],
                              capture_output=True, text=True)
            self.assertNotEqual(failed.returncode, 0, failed.stderr)
            self.assertIn('file format not recognized', failed.stderr)
            probe = output / 'cpu-probe'
            with patch('build.subprocess.run',
                       side_effect=lambda command, **kw: real_run(command + linker, **kw)):
                compile_cpu_probe(fixture / 'main.rs', dependency, output, probe)
            self.assertEqual(subprocess.check_output([str(probe)], text=True).strip(), '7')


if __name__ == '__main__':
    unittest.main()
