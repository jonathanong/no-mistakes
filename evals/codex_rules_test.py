#!/usr/bin/env python3
"""Arm isolation and grader tests. These never call a model."""
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import codex_rules
import codex_rules_env
import codex_rules_grade
import power
import summarize

MARKER = "USER_HOME_MARKER_3580"
REPO = Path(__file__).resolve().parents[1]


def _rel_bytes(root: Path) -> dict:
    found = {}
    for path in _files(root):
        if path.is_file() and not path.is_symlink():
            found[path.relative_to(root).as_posix()] = path.read_bytes()
    return found


def _files(root: Path):
    stack = [root]
    while stack:
        current = stack.pop()
        for child in current.iterdir():
            if child.is_symlink():
                yield child
                continue
            if child.is_dir():
                stack.append(child)
            elif child.is_file():
                yield child


def _read(path: Path) -> str:
    return path.read_bytes().decode("utf-8", errors="replace")


class ArmIsolationTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)
        self.user = self.base / "user"
        (self.user / ".codex").mkdir(parents=True)
        (self.user / ".claude").mkdir()
        (self.user / ".codex" / "secret").write_text(MARKER)
        (self.user / ".claude" / "secret").write_text(MARKER)
        (self.user / "AGENTS.md").write_text(MARKER)
        agents = self.user / ".agents" / "plugins"
        agents.mkdir(parents=True)
        (agents / "marketplace.json").write_text('{"plugins":[{"name":"other-plugin"}]}')
        self.fixture = self.base / "fixture"
        (self.fixture / "nested").mkdir(parents=True)
        (self.fixture / "AGENTS.md").write_text("agent instructions")
        (self.fixture / "CLAUDE.md").write_text("claude instructions")
        (self.fixture / "nested" / "AGENTS.md").write_text("nested agent")
        (self.fixture / "leak.md").write_text("see no-mistakes-rules")
        (self.fixture / "skill.md").write_text("load testing-rules")
        (self.fixture / "business.md").write_text("business rules apply")
        (self.fixture / "keep.txt").write_text("hello from the fixture\n")
        (self.fixture / ".notes").write_text("hello hidden\n")
        (self.fixture / "testing-rules.md").write_text("plain\n")
        (self.fixture / "nested" / "rules.txt").write_text("plain\n")
        (self.fixture / "home-link").symlink_to(self.user / "AGENTS.md")

    def _build(self, arm, auth=None):
        root = self.base / arm
        return codex_rules_env.build_arm(
            root,
            arm,
            fixture=self.fixture,
            user_home=self.user,
            repo=REPO,
            auth_file=auth,
        )

    def test_only_plugin_install_differs(self):
        with mock.patch.object(Path, "home", side_effect=AssertionError("read user home")):
            with_arm = self._build("with")
            without = self._build("without")
        prompt = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        with_plan = codex_rules_env.exec_plan(with_arm, prompt["prompt"], prompt["timeout_seconds"])
        without_plan = codex_rules_env.exec_plan(without, prompt["prompt"], prompt["timeout_seconds"])
        for built in (with_arm, without):
            names = sorted(path.relative_to(built["workspace"]).as_posix() for path in _files(built["workspace"]))
            self.assertEqual(names, [".notes", "keep.txt"])
            self.assertEqual(_read(built["workspace"] / "keep.txt"), "hello from the fixture\n")
            self.assertNotIn("RefundError", _read(built["workspace"] / "keep.txt"))
        self.assertEqual(with_plan["prompt"], without_plan["prompt"])
        self.assertIn("There is no repository", with_plan["prompt"])
        self.assertIn("RefundError", with_plan["prompt"])
        self.assertEqual(with_plan["argv"][-1], without_plan["argv"][-1])
        for key in ("model", "reasoning", "sandbox", "approval", "timeout_seconds", "prompt"):
            self.assertEqual(with_plan[key], without_plan[key])
        with_argv = json.dumps(with_plan["argv"]).replace(str(with_arm["root"]), "<attempt>")
        without_argv = json.dumps(without_plan["argv"]).replace(str(without["root"]), "<attempt>")
        self.assertEqual(with_argv, without_argv)
        with_env = {key: value.replace(str(with_arm["root"]), "<attempt>") for key, value in with_plan["env"].items()}
        without_env = {
            key: value.replace(str(without["root"]), "<attempt>") for key, value in without_plan["env"].items()
        }
        self.assertEqual(with_env, without_env)
        self.assertIn("plugins = true\n", without["config"])
        self.assertTrue(without["config"].endswith("hooks = false\n"))
        self.assertTrue(with_arm["config"].startswith(without["config"]))
        extra = with_arm["config"][len(without["config"]) :]
        self.assertIn('[marketplaces.rules-eval]', extra)
        self.assertIn('[plugins."no-mistakes-rules@rules-eval"]', extra)
        self.assertIn("enabled = true", extra)
        self.assertNotIn("[mcp_servers", extra)
        self.assertNotIn("[hooks", extra)
        catalog = json.loads(
            (with_arm["root"] / "marketplace" / ".agents" / "plugins" / "marketplace.json").read_text()
        )
        self.assertEqual([item["name"] for item in catalog["plugins"]], ["no-mistakes-rules"])
        self.assertFalse((without["root"] / "marketplace").exists())
        self.assertFalse((without["codex_home"] / "plugins").exists())
        version = json.loads(
            (codex_rules_env.PLUGIN_SRC / ".codex-plugin" / "plugin.json").read_text()
        )["version"]
        cache = with_arm["codex_home"] / "plugins" / "cache" / "rules-eval" / "no-mistakes-rules" / version
        self.assertTrue((cache / ".codex-plugin" / "plugin.json").is_file())
        self.assertTrue((cache / "skills" / "testing-rules" / "SKILL.md").is_file())
        self.assertEqual(
            sorted(path.name for path in cache.parent.parent.iterdir()),
            ["no-mistakes-rules"],
        )
        for built in (with_arm, without):
            for path in _files(built["root"]):
                self.assertFalse(path.is_symlink(), path)
                if path.is_file():
                    self.assertNotIn(MARKER, _read(path))
                self.assertFalse(str(path.resolve()).startswith(str(self.user)))
            self.assertNotEqual(Path(built["env"]["HOME"]).resolve(), self.user.resolve())
            self.assertNotEqual(Path(built["env"]["CODEX_HOME"]).resolve(), (self.user / ".codex").resolve())
            self.assertTrue(str(built["workspace"]).startswith(str(built["root"])))
        with_files = _rel_bytes(with_arm["root"])
        without_files = _rel_bytes(without["root"])
        self.assertTrue(set(without_files) <= set(with_files))
        for rel, data in without_files.items():
            if rel == "codex-home/config.toml":
                self.assertTrue(with_files[rel].startswith(data))
                self.assertIn(b'[plugins."no-mistakes-rules@rules-eval"]', with_files[rel])
            else:
                self.assertEqual(with_files[rel], data, rel)
        extra = set(with_files) - set(without_files)
        self.assertTrue(extra)
        for rel in extra:
            self.assertTrue(
                rel.startswith("marketplace/") or rel.startswith("codex-home/plugins/"),
                rel,
            )

    def test_auth_file_is_copied_only_when_outside_user_homes(self):
        banned = self.user / ".codex" / "auth.json"
        banned.write_text("{}\n")
        with self.assertRaises(SystemExit):
            self._build("with", auth=banned)
        allowed = self.base / "spare-auth.json"
        allowed.write_text('{"auth_mode":"test"}\n')
        with_arm = self._build("with", auth=allowed)
        without = self._build("without", auth=allowed)
        self.assertEqual(_read(with_arm["codex_home"] / "auth.json"), _read(without["codex_home"] / "auth.json"))
        self.assertNotIn(MARKER, _read(with_arm["codex_home"] / "auth.json"))

    def test_main_refuses_auth_and_output_before_a_model(self):
        home = self.base / "real-home"
        (home / ".codex").mkdir(parents=True)
        banned = home / ".codex" / "auth.json"
        banned.write_text("{}\n")
        out = self.base / "out"
        with mock.patch.object(Path, "home", return_value=home), mock.patch.object(
            codex_rules.subprocess, "run", side_effect=AssertionError("model")
        ):
            with self.assertRaises(SystemExit):
                codex_rules.main(
                    [
                        "--run",
                        "--case",
                        "rules-testing-write-error-text",
                        "--runs",
                        "1",
                        "--auth-file",
                        str(banned),
                        "--output",
                        str(out),
                    ]
                )
        self.assertFalse(out.exists())
        spare = self.base / "spare-auth.json"
        spare.write_text("{}\n")
        inside = REPO / "evals" / "codex-rules-output-should-not-exist"
        self.addCleanup(lambda: inside.is_dir() and __import__("shutil").rmtree(inside))
        with mock.patch.object(codex_rules.subprocess, "run", side_effect=AssertionError("model")):
            with self.assertRaises(SystemExit):
                codex_rules.main(
                    [
                        "--run",
                        "--case",
                        "rules-testing-write-error-text",
                        "--runs",
                        "1",
                        "--auth-file",
                        str(spare),
                        "--output",
                        str(inside),
                    ]
                )
        self.assertFalse(inside.exists())


class PreviewAndGradeTest(unittest.TestCase):
    def test_preview_counts_cases_without_a_model(self):
        with mock.patch.object(codex_rules.subprocess, "run", side_effect=AssertionError("model")):
            code = codex_rules.main([])
        self.assertEqual(code, 0)
        cases = codex_rules.discover()
        self.assertEqual(len(cases), 18)
        text = codex_rules.format_preview(cases, None)
        self.assertIn("cases: 18\n", text)
        self.assertIn("arms: 2 (with, without)\n", text)
        self.assertIn("attempts: 108 (18 x 2 x 3)\n", text)
        self.assertIn("dollars: unknown (not measured)\n", text)
        self.assertIn("judge model: claude-opus-5-5\n", text)
        self.assertIn(f"agent model: {codex_rules_env.MODEL}\n", text)

    def test_fake_exec_writes_the_aggregate_shape(self):
        plans = []

        def run_exec(plan):
            plans.append(plan)
            line = json.dumps(
                {
                    "type": "item.completed",
                    "item": {
                        "type": "mcp_tool_call",
                        "tool": "Skill",
                        "arguments": {"skill": "testing-rules"},
                    },
                }
            )
            return {"returncode": 0, "stdout": line, "stderr": "", "last_message": "code"}

        case = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        output = Path(tempfile.mkdtemp(prefix="codex-rules-out-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(output, ignore_errors=True))
        with mock.patch.object(codex_rules.subprocess, "run", side_effect=AssertionError("model")):
            report = codex_rules.run_suite(
                [case],
                1,
                output,
                run_exec,
                lambda _prompt: True,
                Path("/codex-rules-user-home-unused"),
                None,
                None,
                keep_temp=False,
            )
        self.assertEqual(len(plans), 2)
        self.assertEqual(plans[0]["prompt"], plans[1]["prompt"])
        self.assertEqual(plans[0]["model"], plans[1]["model"])
        self.assertNotEqual(plans[0]["codex_home"], plans[1]["codex_home"])
        saved = json.loads((output / "aggregate-result.json").read_text())
        self.assertEqual(saved["suite"]["ablation"], "codex-with-without")
        self.assertEqual(summarize._fired(report["cases"][0]["arms"]["with"]), 1)
        self.assertEqual(summarize._fired(report["cases"][0]["arms"]["without"]), 1)
        self.assertIn("with: should-fire 1/1", codex_rules_grade.format_trigger(report))
        self.assertIn("without: should-fire 1/1", codex_rules_grade.format_trigger(report))

    def test_empty_final_message_is_an_error(self):
        case = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        output = Path(tempfile.mkdtemp(prefix="codex-rules-out-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(output, ignore_errors=True))

        def run_exec(_plan):
            return {"returncode": 0, "stdout": "", "stderr": "", "last_message": "  "}

        def judge(_prompt):
            raise AssertionError("judge")

        with mock.patch.object(codex_rules, "build_arm", return_value={}), mock.patch.object(
            codex_rules, "exec_plan", return_value={}
        ):
            report = codex_rules.run_suite(
                [case],
                1,
                output,
                run_exec,
                judge,
                Path("/codex-rules-user-home-unused"),
                None,
                None,
            )
        self.assertEqual(
            report["cases"][0]["arms"]["with"][0]["error"],
            "codex exec wrote no final message",
        )

    def test_run_refuses_before_calling_a_model(self):
        env = os.environ.copy()
        env.pop("CODEX_API_KEY", None)
        env.pop("CODEX_AUTH_FILE", None)
        with mock.patch.dict(os.environ, env, clear=True), mock.patch.object(
            codex_rules.subprocess, "run", side_effect=AssertionError("model")
        ):
            code = codex_rules.main(["--run", "--all", "--output", "/tmp/codex-rules-should-not-write"])
        self.assertEqual(code, 2)

    def test_skill_fired_ignores_message_text(self):
        meta, _ = codex_rules_grade.parse_frontmatter(
            (REPO / "evals" / "rules-testing-write-error-text" / "graders" / "skill-fired.md").read_text()
        )
        spoken = json.dumps(
            {"type": "item.completed", "item": {"type": "agent_message", "text": meta["input_match"]}}
        )
        self.assertFalse(codex_rules_grade.skill_fired(meta, spoken))
        called = json.dumps(
            {
                "type": "item.completed",
                "item": {
                    "type": "mcp_tool_call",
                    "tool": "Skill",
                    "arguments": {"skill": "testing-rules"},
                },
            }
        )
        self.assertTrue(codex_rules_grade.skill_fired(meta, called))
        invoked = json.dumps(
            {
                "type": "item.completed",
                "item": {
                    "type": "skill_invocation",
                    "skill_name": "testing-rules",
                    "plugin_id": "no-mistakes-rules",
                },
            }
        )
        self.assertTrue(codex_rules_grade.skill_fired(meta, invoked))
        command = json.dumps(
            {
                "type": "item.completed",
                "item": {
                    "type": "command_execution",
                    "command": "cat /tmp/skills/testing-rules/SKILL.md",
                },
            }
        )
        self.assertTrue(codex_rules_grade.skill_fired(meta, command))
        off, _ = codex_rules_grade.parse_frontmatter(
            (REPO / "evals" / "rules-offtopic-neg-big-o" / "graders" / "skill-fired.md").read_text()
        )
        self.assertTrue(codex_rules_grade.skill_fired(off, called))
        self.assertFalse(codex_rules_grade.skill_fired(off, spoken))

    def test_judge_prompt_uses_grader_markdown_and_report_shape(self):
        case = REPO / "evals" / "rules-testing-write-error-text"
        rubric_meta, rubric = codex_rules_grade.parse_frontmatter(
            (case / "graders" / "asserts-error-code-not-text.md").read_text()
        )
        seen = []

        def judge(prompt):
            seen.append(prompt)
            self.assertIn(rubric, prompt)
            self.assertIn("final answer text", prompt)
            self.assertEqual(rubric_meta["type"], "llm")
            return True

        transcript = json.dumps(
            {
                "type": "item.completed",
                "item": {"type": "mcp_tool_call", "tool": "Skill", "arguments": {"skill": "testing-rules"}},
            }
        )
        run = codex_rules_grade.grade_run(case, transcript, "final answer text", judge)
        self.assertEqual(len(seen), 1)
        self.assertIn(JUDGE := codex_rules_grade.JUDGE_MODEL, codex_rules_grade.judge_argv())
        script = (REPO / "scripts" / "run-evals.sh").read_text()
        self.assertIn(f"JUDGE_MODEL={JUDGE}", script)
        report = codex_rules_grade.aggregate_report(
            [{"name": "rules-testing-write-error-text", "arms": {"with": [run], "without": [run]}}]
        )
        self.assertFalse(power._reject(report, "aggregate-result.json"))
        self.assertEqual(summarize._fired(report["cases"][0]["arms"]["with"]), 1)
        self.assertEqual(summarize._mean_score(report["cases"][0]["arms"]["with"]), 1)
        self.assertIn("should-fire 1/1", codex_rules_grade.format_trigger(report))
        self.assertTrue(_summarize_ok(report))


def _summarize_ok(report) -> bool:
    handle = tempfile.NamedTemporaryFile("w", suffix=".json", delete=False)
    json.dump(report, handle)
    handle.close()
    try:
        with mock.patch("sys.stdout"):
            return summarize.summarize(handle.name) is False
    finally:
        os.unlink(handle.name)


if __name__ == "__main__":
    unittest.main()
