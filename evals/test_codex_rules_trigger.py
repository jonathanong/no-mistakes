"""Offline checks for transcript-based skill and external-path evidence."""
import json
import unittest
from pathlib import Path

import codex_rules_trigger as trigger

FIXTURES = Path(__file__).resolve().parents[1] / "fixtures" / "evals" / "codex-rules"
ATTEMPT = Path("/attempt")


class TranscriptTriggerTest(unittest.TestCase):
    def test_static_read_forms_count_only_the_rules_opened(self):
        transcript = (FIXTURES / "trigger-reads.jsonl").read_text(encoding="utf-8")
        self.assertEqual(
            trigger.skills_in_transcript(
                transcript,
                ATTEMPT / "plugin",
                marketplace_root=ATTEMPT / "cache" / "1.0.0",
                attempt_root=ATTEMPT,
                cwd=ATTEMPT / "workspace",
            ),
            ["testing-rules", "queue-rules", "postgresql-rules"],
        )

    def test_external_paths_report_only_redacted_completed_command_indices(self):
        transcript = (FIXTURES / "external-paths.jsonl").read_text(encoding="utf-8")
        outside_root = "outside-attempt-root"
        self.assertEqual(
            trigger.external_path_findings(transcript, ATTEMPT, cwd=ATTEMPT / "workspace"),
            [
                {"commandIndex": 2, "reason": outside_root},
                {"commandIndex": 3, "reason": outside_root},
                {"commandIndex": 4, "reason": outside_root},
                {"commandIndex": 5, "reason": "outside-working-directory"},
                {"commandIndex": 7, "reason": outside_root},
                {"commandIndex": 8, "reason": outside_root},
                {"commandIndex": 9, "reason": outside_root},
                {"commandIndex": 10, "reason": outside_root},
                {"commandIndex": 11, "reason": "attempt-auth-file"},
                {"commandIndex": 12, "reason": "attempt-auth-file"},
                {"commandIndex": 13, "reason": "attempt-auth-file"},
            ],
        )

    def test_safe_device_redirects_are_exempt_while_outside_redirects_flag(self):
        transcript = (FIXTURES / "safe-device-redirects.jsonl").read_text(encoding="utf-8")
        self.assertEqual(
            trigger.external_path_findings(transcript, ATTEMPT, cwd=ATTEMPT / "workspace"),
            [{"commandIndex": 10, "reason": "outside-attempt-root"}],
        )

    def test_shell_arguments_keep_only_real_outside_paths(self):
        transcript = (FIXTURES / "shell-arguments.jsonl").read_text(encoding="utf-8")
        expected = json.loads((FIXTURES / "shell-arguments.snap").read_text(encoding="utf-8"))
        self.assertEqual(trigger.external_path_findings(transcript, ATTEMPT), expected)

    def test_search_pattern_and_input_redirection_preserve_skill_inputs(self):
        transcript = (FIXTURES / "shell-arguments.jsonl").read_text(encoding="utf-8")
        self.assertEqual(
            trigger.skills_in_transcript(
                transcript,
                ATTEMPT / "plugin",
                attempt_root=ATTEMPT,
                cwd=ATTEMPT / "workspace",
            ),
            ["testing-rules", "security-rules"],
        )
