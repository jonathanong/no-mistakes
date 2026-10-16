"""Fail-closed clocks and source-level ownership controls; no builds or sampling."""
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from common import ExperimentFailure
import deadline


class DeadlineTests(unittest.TestCase):
    def test_remaining_reserves_upload_and_rejects_foreign_clocks(self):
        with tempfile.TemporaryDirectory()as name,patch.dict(os.environ,{'GITHUB_RUN_ID':'12','GITHUB_RUN_ATTEMPT':'1','DIAG_JOB_NAME':'measurement (0)'}):
            root=Path(name);(root/'public').mkdir()
            row={'run_id':'12','attempt':'1','job_name':'measurement (0)','job_timeout_seconds':1800,'deadline_monotonic':2000}
            path=root/'public/job-clock.json';path.write_text(json.dumps(row))
            with patch.object(deadline.time,'monotonic',return_value=300):
                self.assertEqual(deadline.remaining(root),1520)
                self.assertEqual(deadline.sampling_deadline(root),1800)
                with self.assertRaises(ExperimentFailure):deadline.remaining(root,179)
            with patch.object(deadline.time,'monotonic',return_value=321):
                with self.assertRaises(ExperimentFailure):deadline.sampling_deadline(root)
            with patch.object(deadline.time,'monotonic',return_value=1821):
                with self.assertRaises(ExperimentFailure):deadline.remaining(root)
            for key,value in [('run_id','other'),('attempt','2'),('job_name','measurement (1)'),('job_timeout_seconds',3600)]:
                bad=dict(row);bad[key]=value;path.write_text(json.dumps(bad))
                with self.assertRaises(ExperimentFailure):deadline.clock(root)

    def test_clock_cannot_reset_even_with_valid_runtime_environment(self):
        with tempfile.TemporaryDirectory()as name:
            root=Path(name);(root/'public').mkdir();(root/'public/job-clock.json').write_text('{}')
            with patch.object(deadline.subprocess,'Popen')as spawn,self.assertRaises(ExperimentFailure):
                deadline.start(root,'measurement (0)')
            spawn.assert_not_called()

    def test_no_legacy_unbounded_build_entrypoints_remain(self):
        here=Path(__file__).resolve().parent
        for filename in ('stage.py','build_arm.py','layout_arm.py','report.py'):
            self.assertFalse((here/filename).exists())
