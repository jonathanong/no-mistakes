"""Offline checks of saved Codex rules reports and resume inputs."""

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import codex_rules
import codex_rules_env as env
import codex_rules_store as store
import summarize
from codex_rules_grade import passed_from_judge_output

FIXTURES = Path(__file__).resolve().parents[1] / "fixtures" / "evals" / "codex-rules"


class JudgeOutputTest(unittest.TestCase):
    def test_accepts_a_single_boolean_verdict_in_the_wrapper(self):
        wrapped = json.dumps({"is_error": False, "result": '{"passed":false}'})
        self.assertFalse(passed_from_judge_output(wrapped))

    def test_rejects_an_ambiguous_or_failed_verdict(self):
        for raw in (
            '{"passed":true,"passed":false}',
            '{"passed":true}\n{"passed":false}',
            'Quote: {"passed":true}',
            '{"is_error":true,"result":"{\\"passed\\":true}"}',
            json.dumps({"result": '{"passed":true}\n{"passed":false}'}),
        ):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                passed_from_judge_output(raw)

    def test_legacy_judge_outage_does_not_look_like_a_content_regression(self):
        rendered = io.StringIO()
        with contextlib.redirect_stdout(rendered):
            regressed = summarize.summarize(FIXTURES / "judge-outage.json")
        self.assertFalse(regressed)
        self.assertIn("judge error: judge unavailable", rendered.getvalue())
        self.assertIn("1 run(s) FAILED", rendered.getvalue())


class ResumeInputsTest(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.output = Path(tmp.name)
        self.case = codex_rules.filter_cases(
            codex_rules.discover(), [], "rules-testing-write-error-text"
        )[0]

    def test_fingerprint_refuses_changed_paid_run_inputs(self):
        store.check_run_inputs(self.output, [self.case], 1, env.PLUGIN_SRC)
        store.check_run_inputs(self.output, [self.case], 1, env.PLUGIN_SRC)
        manifest = json.loads((self.output / "run-inputs.json").read_text())
        self.assertRegex(manifest["sha256"], r"^[a-f0-9]{64}$")
        for name, patch in (
            ("model", mock.patch.object(env, "MODEL", "example-model")),
            ("effort", mock.patch.object(env, "REASONING", "low")),
            ("plugin", mock.patch.object(store, "plugin_files", return_value=iter(()))),
        ):
            with self.subTest(input=name), patch, self.assertRaisesRegex(SystemExit, "run inputs changed"):
                store.check_run_inputs(self.output, [self.case], 1, env.PLUGIN_SRC)
        changed = dict(self.case, prompt="Different example prompt")
        with self.assertRaisesRegex(SystemExit, "run inputs changed"):
            store.check_run_inputs(self.output, [changed], 1, env.PLUGIN_SRC)
        with self.assertRaisesRegex(SystemExit, "run inputs changed"):
            store.check_run_inputs(self.output, [self.case], 2, env.PLUGIN_SRC)

    def test_legacy_output_can_be_regraded_but_not_resumed(self):
        store.write_aggregate(self.output, [], partial=True)
        with self.assertRaisesRegex(SystemExit, "no input fingerprint"):
            store.check_run_inputs(self.output, [self.case], 1, env.PLUGIN_SRC)
        self.assertTrue(codex_rules.regrade_output([self.case], self.output)["partial"])


    def test_audit_flags_both_arms_and_withholds_the_judge(self):
        transcript = (FIXTURES / "outside-read.jsonl").read_text()
        run_exec = mock.Mock(return_value={
            "returncode": 0, "stdout": transcript, "stderr": "", "last_message": "Example answer",
        })
        judge = mock.Mock(side_effect=AssertionError("judge must not receive the answer"))
        report = codex_rules.run_suite(
            [self.case], 1, self.output, run_exec, judge,
            self.output / "user", None, None, check_plugins=False,
        )
        self.assertEqual(run_exec.call_count, 2)
        judge.assert_not_called()
        self.assertEqual(report["isolationFlaggedAttempts"], 2)
        self.assertTrue(report["partial"])
        self.assertNotIn("/example-outside", json.dumps(report))
        self.assertIn("with: should-fire 0/0", codex_rules.format_trigger(report))

        with mock.patch.object(codex_rules, "exec_codex") as exec_model, mock.patch.object(
            codex_rules, "exec_judge"
        ) as judge_model:
            regraded = codex_rules.regrade_output([self.case], self.output)
        exec_model.assert_not_called()
        judge_model.assert_not_called()
        for arm in ("with", "without"):
            row = regraded["cases"][0]["arms"][arm][0]
            self.assertEqual(row["isolationFindings"], [
                {"commandIndex": 1, "reason": "outside-attempt-root"},
            ])
            self.assertTrue(row["judgeWithheld"])
