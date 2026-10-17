"""Pinned pair eligibility survives an identical-tree parent merge only."""
import copy
import json
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch

import eligibility

HERE = Path(__file__).resolve().parent


class PairEligibility(unittest.TestCase):
    def check_case(self, name):
        fixture = json.loads((HERE / 'fixtures/pair-eligibility.json').read_text())
        rows = copy.deepcopy(fixture['responses'])
        rows.update(fixture['cases'][name])

        def command(argv, label, output, deadline, seconds):
            endpoint = argv[-1].split('repos/jonathanong/no-mistakes/')[1]
            (output / (label + '.stdout')).write_text(json.dumps(rows[endpoint]))

        with tempfile.TemporaryDirectory() as directory, patch.object(eligibility, 'command', command):
            eligibility.verify(fixture['spec'], Path(directory), time.monotonic() + 30, 'fixture')

    def test_open_parent(self):
        self.check_case('open')

    def test_identical_tree_merged_parent(self):
        self.check_case('merged')

    def test_changed_parent_head_rejected(self):
        with self.assertRaises(AssertionError):
            self.check_case('changed_head')

    def test_changed_merge_tree_rejected(self):
        with self.assertRaises(AssertionError):
            self.check_case('changed_merge_tree')

    def test_closed_unmerged_parent_rejected(self):
        with self.assertRaises(AssertionError):
            self.check_case('closed_unmerged')
