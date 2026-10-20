"""Saved cohort selection and balanced resource-statistic contracts."""
import json
from pathlib import Path
import tempfile
import unittest

from cohort import RULE, derive_config, select_rules
from cohort_measure import summary

FIXTURES = Path(__file__).resolve().parents[3] / 'fixtures/performance/current-arm/non-annotation'


class CohortTests(unittest.TestCase):
    def test_only_annotation_applications_are_removed(self):
        fixture = json.loads((FIXTURES / 'selection.json').read_text())
        selected, removed = select_rules(fixture['original'])
        self.assertEqual(selected, fixture['expected'])
        self.assertEqual([row['rule'] for row in removed], [RULE, RULE])

    def test_saved_full_cohort_preserves_the_declared_counts_and_hash(self):
        import hashlib
        import yaml
        contract = json.loads((FIXTURES / 'contract.json').read_text())
        source = (FIXTURES / 'config.yml').read_bytes()
        self.assertEqual(hashlib.sha256(source).hexdigest(), contract['cohortConfigSha256'])
        config = yaml.safe_load(source)
        self.assertEqual(len(config['rules']), 131)
        self.assertTrue(all(row['rule'] != RULE for row in config['rules']))
        self.assertEqual(contract['originalRuleApplications'], 133)
        self.assertEqual(hashlib.sha256((FIXTURES / 'consumer.gitignore').read_bytes()).hexdigest(),
                         contract['siblingGitignoreSha256'])
        self.assertTrue(contract['configParentGitignorePreserved'])

    def test_balanced_pairs_include_both_directions_and_all_resources(self):
        rows = json.loads((FIXTURES / 'samples.json').read_text())
        result = summary(rows)
        self.assertEqual(result['samplesPerArm'], 6)
        for metric in ('wallSeconds', 'cpuSeconds', 'peakRssKiB'):
            self.assertEqual(result['adjacentRatios'][metric]['samples'], [0.5] * 6)
            for arm in ('base', 'phases'):
                measured = result['variants'][arm][metric]
                self.assertEqual(measured['empiricalP95'], measured['max'])

    def test_external_config_keeps_its_original_gitignore_lookup(self):
        consumer = FIXTURES / 'consumer'
        with tempfile.TemporaryDirectory() as output:
            target = Path(output) / 'cohort.yml'
            derive_config(consumer, target)
            self.assertEqual((target.parent / '.gitignore').read_bytes(),
                             (consumer / '.gitignore').read_bytes())
            self.assertEqual(target.read_bytes(), (FIXTURES / 'config.yml').read_bytes())


if __name__ == '__main__':
    unittest.main()
