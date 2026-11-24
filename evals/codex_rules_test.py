#!/usr/bin/env python3
"""Arm isolation and grader tests. These never call a model."""
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import codex_rules
import codex_rules_env
import codex_rules_grade
import codex_rules_store as store
import power
import summarize

MARKER = "USER_HOME_MARKER_3580"
REPO = Path(__file__).resolve().parents[1]


def _read_event(command: str, *, exit_code: int = 0) -> str:
    return json.dumps(
        {
            "type": "item.completed",
            "item": {
                "type": "command_execution",
                "command": command,
                "aggregated_output": "",
                "exit_code": exit_code,
                "status": "completed",
            },
        }
    )


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
        existed = inside.exists()

        def _remove_created_output():
            if not existed and inside.is_dir():
                __import__("shutil").rmtree(inside)

        self.addCleanup(_remove_created_output)
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
        self.assertEqual(len(cases), 21)
        text = codex_rules.format_preview(cases, None)
        self.assertIn("cases: 21\n", text)
        self.assertIn("arms: 2 (with, without)\n", text)
        self.assertIn("attempts: 126 (21 x 2 x 3)\n", text)
        self.assertIn("dollars: unknown (not measured)\n", text)
        self.assertIn("judge model: claude-opus-5-5\n", text)
        self.assertIn(f"agent model: {codex_rules_env.MODEL}\n", text)

    def test_fake_exec_writes_the_aggregate_shape(self):
        plans = []
        case = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        output = Path(tempfile.mkdtemp(prefix="codex-rules-out-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(output, ignore_errors=True))

        def run_exec(plan):
            plans.append(plan)
            root = plan.get("plugin_root")
            if root:
                command = f"cat {Path(root) / 'skills' / 'testing-rules' / 'SKILL.md'}"
            else:
                command = "echo no-plugin"
            if plan["arm"] == "without":
                saved = json.loads((output / "aggregate-result.json").read_text())
                self.assertTrue(saved["partial"])
                self.assertEqual(len(saved["cases"][0]["arms"]["with"]), 1)
                transcript = output / "attempts" / case["name"] / "with" / "1" / "transcript.jsonl"
                self.assertIn("command_execution", transcript.read_text())
            return {
                "returncode": 0,
                "stdout": _read_event(command) + "\n",
                "stderr": "",
                "last_message": "code",
            }

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
                check_plugins=False,
            )
        self.assertEqual(len(plans), 2)
        self.assertEqual(plans[0]["prompt"], plans[1]["prompt"])
        self.assertEqual(plans[0]["model"], plans[1]["model"])
        self.assertNotEqual(plans[0]["codex_home"], plans[1]["codex_home"])
        saved = json.loads((output / "aggregate-result.json").read_text())
        self.assertEqual(saved["suite"]["ablation"], "codex-with-without")
        self.assertFalse(saved["partial"])
        self.assertEqual(summarize._fired(report["cases"][0]["arms"]["with"]), 1)
        self.assertEqual(summarize._fired(report["cases"][0]["arms"]["without"]), 0)
        self.assertIn("with: should-fire 1/1", codex_rules_grade.format_trigger(report))
        self.assertIn("without: should-fire 0/1", codex_rules_grade.format_trigger(report))
        self.assertTrue((output / "attempts" / case["name"] / "without" / "1" / "result.json").is_file())

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
                check_plugins=False,
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

    def test_skill_fired_counts_only_a_real_read(self):
        meta, _ = codex_rules_grade.parse_frontmatter(
            (REPO / "evals" / "rules-testing-write-error-text" / "graders" / "skill-fired.md").read_text()
        )
        off, _ = codex_rules_grade.parse_frontmatter(
            (REPO / "evals" / "rules-offtopic-neg-big-o" / "graders" / "skill-fired.md").read_text()
        )
        root = Path(tempfile.mkdtemp(prefix="codex-rules-plugin-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(root, ignore_errors=True))
        skill = root / "skills" / "testing-rules" / "SKILL.md"
        skill.parent.mkdir(parents=True)
        skill.write_text("skill body\n")
        other = root / "skills" / "queue-rules" / "SKILL.md"
        other.parent.mkdir(parents=True)
        other.write_text("other\n")
        outside = Path(tempfile.mkdtemp(prefix="codex-rules-outside-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(outside, ignore_errors=True))
        stray = outside / "testing-rules" / "SKILL.md"
        stray.parent.mkdir(parents=True)
        stray.write_text("stray\n")
        spoken = json.dumps(
            {
                "type": "item.completed",
                "item": {"type": "agent_message", "text": f"{meta['input_match']} {skill}"},
            }
        )
        self.assertFalse(codex_rules_grade.skill_fired(meta, spoken, root))
        self.assertFalse(codex_rules_grade.skill_fired(meta, _read_event(f"cat {skill}"), None))
        self.assertTrue(codex_rules_grade.skill_fired(meta, _read_event(f"cat {skill}"), root))
        self.assertTrue(codex_rules_grade.skill_fired(meta, _read_event(f"head -n 20 {skill}"), root))
        self.assertTrue(codex_rules_grade.skill_fired(meta, _read_event(f"sed -n '1,20p' {skill}"), root))
        wrapped = "bash -lc " + json.dumps(f"cat {skill}")
        self.assertTrue(codex_rules_grade.skill_fired(meta, _read_event(wrapped), root))
        self.assertFalse(codex_rules_grade.skill_fired(meta, _read_event(f"ls -l {skill}"), root))
        self.assertFalse(codex_rules_grade.skill_fired(meta, _read_event(f"echo {skill}"), root))
        self.assertFalse(codex_rules_grade.skill_fired(meta, _read_event(f"cat {stray}"), root))
        self.assertFalse(codex_rules_grade.skill_fired(meta, _read_event(f"cat {other}"), root))
        self.assertTrue(codex_rules_grade.skill_fired(off, _read_event(f"cat {other}"), root))
        self.assertFalse(codex_rules_grade.skill_fired(off, spoken, root))
        mentioned = json.dumps(
            {
                "type": "item.completed",
                "item": {
                    "type": "command_execution",
                    "command": "ls -l /work",
                    "aggregated_output": str(skill),
                    "exit_code": 0,
                    "status": "completed",
                },
            }
        )
        self.assertFalse(codex_rules_grade.skill_fired(meta, mentioned, root))
        started = json.dumps(
            {
                "type": "item.started",
                "item": {
                    "type": "command_execution",
                    "command": f"cat {skill}",
                    "status": "in_progress",
                    "exit_code": None,
                },
            }
        )
        self.assertFalse(codex_rules_grade.skill_fired(meta, started, root))
        for event_type in ("skill_invocation", "mcp_tool_call"):
            ignored = json.dumps(
                {
                    "type": "item.completed",
                    "item": {
                        "type": event_type,
                        "tool": "Skill",
                        "skill_name": "testing-rules",
                        "arguments": {"skill": "testing-rules"},
                    },
                }
            )
            self.assertFalse(codex_rules_grade.skill_fired(meta, ignored, root))

    def test_skill_fired_counts_marketplace_and_relative_reads(self):
        meta, _ = codex_rules_grade.parse_frontmatter(
            (REPO / "evals" / "rules-testing-write-error-text" / "graders" / "skill-fired.md").read_text()
        )
        base = Path(tempfile.mkdtemp(prefix="codex-rules-reads-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(base, ignore_errors=True))
        attempt = base / "attempt"
        workspace = attempt / "workspace"
        workspace.mkdir(parents=True)
        plugin = attempt / "plugin"
        skill = plugin / "skills" / "testing-rules" / "SKILL.md"
        skill.parent.mkdir(parents=True)
        skill.write_text("skill body\n")
        market = attempt / "marketplace" / "plugins" / "no-mistakes-rules"
        market_skill = market / "skills" / "testing-rules" / "SKILL.md"
        market_skill.parent.mkdir(parents=True)
        market_skill.write_text("market body\n")
        outside = base / "outside"
        outside_skill = outside / "skills" / "testing-rules" / "SKILL.md"
        outside_skill.parent.mkdir(parents=True)
        outside_skill.write_text("outside\n")
        decoy = workspace / "testing-rules" / "SKILL.md"
        decoy.parent.mkdir(parents=True)
        decoy.write_text("decoy\n")

        def fired(command, *, root=plugin, market_root=market, cwd=workspace, exit_code=0):
            return codex_rules_grade.skill_fired(
                meta,
                _read_event(command, exit_code=exit_code),
                root,
                marketplace_root=market_root,
                attempt_root=attempt,
                cwd=cwd,
            )

        self.assertTrue(fired(f"nl -ba {skill} | sed -n '1,20p'"))
        self.assertTrue(fired(f"cd {skill.parent} && cat SKILL.md"))
        self.assertTrue(fired(f"rg -n rule {skill}"))
        self.assertTrue(fired(f"rg SKILL.md {skill}"))
        self.assertTrue(fired(f"awk '/rule/' {skill}"))
        self.assertTrue(fired(f"grep -n rule {skill}"))
        self.assertTrue(fired(f"grep --regexp=rule {skill}"))
        self.assertTrue(fired(f"bat {skill}"))
        self.assertTrue(fired(f"cat < {skill}"))
        self.assertTrue(fired(f"cat {market_skill}"))
        self.assertTrue(fired("python -c \"open('" + str(skill) + "').read()\""))
        self.assertTrue(fired("node -e \"require('fs').readFileSync('" + str(skill) + "')\""))
        wrapped = "bash -lc " + json.dumps(f"cd {skill.parent} && cat SKILL.md")
        self.assertTrue(fired(wrapped))
        self.assertTrue(fired(f"(cd {skill.parent} && cat SKILL.md)"))
        self.assertTrue(fired(f"find {skill.parent} -name SKILL.md -exec cat {{}} +"))
        self.assertTrue(fired(f"find {skill.parent} -name SKILL.md -execdir cat {{}} +"))
        self.assertTrue(fired(f"find {skill} -exec cat {{}} +"))
        self.assertFalse(fired(f"find {skill} -name README.md -exec cat {{}} +"))
        self.assertFalse(fired(f"find {skill} -exec cat /dev/null \\;"))
        self.assertFalse(fired(f"(cd {skill.parent} && true); cat SKILL.md"))
        self.assertFalse(fired(f"cat /dev/null >> {skill}"))
        self.assertFalse(fired(f"ls -l {skill}"))
        self.assertFalse(fired(f"stat {skill}"))
        self.assertFalse(fired(f"find {plugin} -name SKILL.md"))
        self.assertFalse(fired(f"find {plugin} -name SKILL.md -exec cat {{}} +"))
        self.assertFalse(fired(f"find {skill.parent} -name README.md -exec cat {{}} +"))
        self.assertFalse(fired(f"find {skill.parent} -exec ls {{}} +"))
        self.assertFalse(fired(f"echo {skill}"))
        self.assertFalse(fired(f"cat {skill}", exit_code=1))
        self.assertFalse(fired(f"cat {outside_skill}", root=outside, market_root=None))
        self.assertFalse(fired(f"cat {decoy}"))
        self.assertFalse(fired("rg SKILL.md", cwd=skill.parent))
        self.assertFalse(fired(f"cd {skill.parent} | cat SKILL.md"))
        self.assertFalse(fired("python -c \"print('see SKILL.md for details')\""))
        self.assertFalse(fired("python -c \"print('" + str(skill) + "')\""))
        self.assertTrue(
            fired("python -c \"from pathlib import Path; print(Path('" + str(skill) + "').read_text())\"")
        )
        self.assertTrue(fired("python -c \"from pathlib import Path; Path('" + str(skill) + "').read_bytes()\""))
        self.assertFalse(fired("python -c \"from pathlib import Path; print(Path('" + str(skill) + "'))\""))
        home = codex_rules_grade.trigger.Path
        with mock.patch.object(home, "home", return_value=skill.parent):
            self.assertFalse(fired("cd && cat SKILL.md"))
            self.assertFalse(fired("cd - && cat SKILL.md"))

    def test_regrade_recomputes_skill_fired_without_a_model(self):
        case = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        output = Path(tempfile.mkdtemp(prefix="codex-rules-regrade-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(output, ignore_errors=True))
        attempt = Path(tempfile.mkdtemp(prefix="codex-rules-regrade-attempt-"))
        workspace = attempt / "workspace"
        workspace.mkdir(parents=True)
        plugin = attempt / "plugin"
        skill = plugin / "skills" / "testing-rules" / "SKILL.md"
        skill.parent.mkdir(parents=True)
        skill.write_text("skill body\n")
        command = f"cd {skill.parent} && cat SKILL.md"
        slot = store.slot(output, case["name"], "with", 1)
        store.save_exec(
            slot,
            {
                "returncode": 0,
                "stdout": _read_event(command) + "\n",
                "stderr": "",
                "last_message": "code",
                "plugin_root": str(plugin),
                "marketplace_root": str(attempt / "marketplace" / "plugins" / "no-mistakes-rules"),
                "attempt_root": str(attempt),
                "cwd": str(workspace),
            },
        )
        store.save_result(
            slot,
            {
                "graders": [
                    {"name": "skill-fired", "passed": False, "evidence": "code"},
                    {"name": "asserts-error-code-not-text", "error": "judge down", "evidence": "code"},
                ],
                "transcript": store.transcript_rel(case["name"], "with", 1),
            },
        )
        failed = store.slot(output, case["name"], "without", 1)
        store.save_result(failed, {"error": "codex exec exited 1", "graders": []})
        __import__("shutil").rmtree(attempt)
        with mock.patch.object(codex_rules.subprocess, "run", side_effect=AssertionError("model")), mock.patch.object(
            codex_rules.subprocess, "Popen", side_effect=AssertionError("model")
        ):
            self.assertEqual(codex_rules.main(["--regrade", "--run", "--output", str(output)]), 2)
            self.assertEqual(codex_rules.main(["--regrade"]), 2)
            code = codex_rules.main(["--regrade", "--output", str(output), "--case", case["name"]])
        self.assertEqual(code, 0)
        report = json.loads((output / "aggregate-result.json").read_text())
        self.assertFalse(report["partial"])
        (slot / "transcript.jsonl.partial").write_text("{}\n")
        with mock.patch.object(codex_rules.subprocess, "run", side_effect=AssertionError("model")), mock.patch.object(
            codex_rules.subprocess, "Popen", side_effect=AssertionError("model")
        ):
            self.assertEqual(codex_rules.main(["--regrade", "--output", str(output), "--case", case["name"]]), 0)
        self.assertFalse(json.loads((output / "aggregate-result.json").read_text())["partial"])
        (slot / "transcript.jsonl.partial").unlink()
        row = report["cases"][0]["arms"]["with"][0]
        self.assertTrue(any(item["name"] == "skill-fired" and item["passed"] for item in row["graders"]))
        judged = next(item for item in row["graders"] if item["name"] == "asserts-error-code-not-text")
        self.assertEqual(judged["error"], "judge down")
        self.assertNotIn("passed", judged)
        self.assertEqual(report["cases"][0]["arms"]["without"][0]["error"], "codex exec exited 1")
        saved = json.loads((slot / "result.json").read_text())
        self.assertTrue(any(item["name"] == "skill-fired" and item["passed"] for item in saved["graders"]))
        waiting = store.slot(output, case["name"], "with", 2)
        waiting.mkdir(parents=True)
        (waiting / "transcript.jsonl").write_text("{}\n")
        with mock.patch.object(codex_rules.subprocess, "run", side_effect=AssertionError("model")):
            self.assertEqual(codex_rules.main(["--regrade", "--output", str(output), "--case", case["name"]]), 0)
        again = json.loads((output / "aggregate-result.json").read_text())
        self.assertTrue(again["partial"])
        self.assertEqual(again["partialReason"], "codex rules run still in progress")

    def test_regrade_keeps_a_canonical_read_after_the_attempt_is_deleted(self):
        case = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        output = Path(tempfile.mkdtemp(prefix="codex-rules-canon-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(output, ignore_errors=True))
        base = Path(tempfile.mkdtemp(prefix="codex-rules-canon-root-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(base, ignore_errors=True))
        private = base / "private"
        private.mkdir()
        link = base / "var"
        link.symlink_to(private, target_is_directory=True)
        attempt = link / "attempt"
        built = codex_rules_env.build_arm(
            attempt,
            "with",
            user_home=Path("/codex-rules-user-home-unused"),
            repo=REPO,
        )
        plan = codex_rules_env.exec_plan(built, case["prompt"], case["timeout_seconds"])
        unresolved_plugin = str(codex_rules_env.installed_plugin_root(built["codex_home"]))
        self.assertEqual(plan["attempt_root"], os.path.realpath(attempt))
        self.assertEqual(plan["cwd"], os.path.realpath(built["workspace"]))
        self.assertEqual(plan["plugin_root"], os.path.realpath(unresolved_plugin))
        self.assertNotEqual(plan["attempt_root"], os.path.normpath(attempt))
        self.assertNotEqual(plan["plugin_root"], unresolved_plugin)
        skill = Path(plan["plugin_root"]) / "skills" / "testing-rules" / "SKILL.md"
        self.assertTrue(skill.is_file())
        self.assertFalse(str(skill).startswith(os.path.normpath(attempt)))
        slot = store.slot(output, case["name"], "with", 1)
        store.save_exec(
            slot,
            {
                "returncode": 0,
                "stdout": _read_event(f"cat {skill}") + "\n",
                "stderr": "",
                "last_message": "code",
                "plugin_root": unresolved_plugin,
                "marketplace_root": str(built["root"] / "marketplace" / "plugins" / "no-mistakes-rules"),
                "attempt_root": str(built["root"]),
                "cwd": str(built["workspace"]),
            },
        )
        store.save_result(
            slot,
            {
                "graders": [
                    {"name": "skill-fired", "passed": False, "evidence": "code"},
                    {"name": "asserts-error-code-not-text", "error": "judge down", "evidence": "code"},
                ],
                "transcript": store.transcript_rel(case["name"], "with", 1),
            },
        )
        __import__("shutil").rmtree(plan["attempt_root"])
        self.assertFalse(Path(plan["attempt_root"]).exists())
        self.assertTrue(link.is_symlink())
        with mock.patch.object(codex_rules.subprocess, "run", side_effect=AssertionError("model")), mock.patch.object(
            codex_rules.subprocess, "Popen", side_effect=AssertionError("model")
        ):
            code = codex_rules.main(["--regrade", "--output", str(output), "--case", case["name"]])
        self.assertEqual(code, 0)
        saved = json.loads((slot / "result.json").read_text())
        fired = next(item for item in saved["graders"] if item["name"] == "skill-fired")
        self.assertTrue(fired["passed"])
        judged = next(item for item in saved["graders"] if item["name"] == "asserts-error-code-not-text")
        self.assertEqual(judged["error"], "judge down")
        self.assertNotIn("passed", judged)

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

        root = Path(tempfile.mkdtemp(prefix="codex-rules-plugin-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(root, ignore_errors=True))
        skill = root / "skills" / "testing-rules" / "SKILL.md"
        skill.parent.mkdir(parents=True)
        skill.write_text("skill body\n")
        transcript = _read_event(f"cat {skill}")
        run = codex_rules_grade.grade_run(case, transcript, "final answer text", judge, root)
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

    def test_runs_zero_is_not_the_default(self):
        text = codex_rules.format_preview(codex_rules.discover(), 0)
        self.assertIn("runs: 0\n", text)
        self.assertIn("attempts: 0 (21 x 2 x 0)\n", text)
        with mock.patch.object(codex_rules.subprocess, "run", side_effect=AssertionError("model")):
            self.assertEqual(codex_rules.main(["--runs", "0"]), 0)
            self.assertEqual(
                codex_rules.main(["--run", "--all", "--runs", "0", "--output", "/tmp/codex-rules-should-not-write"]),
                2,
            )

    def test_judge_failure_keeps_skill_fired(self):
        case = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        output = Path(tempfile.mkdtemp(prefix="codex-rules-out-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(output, ignore_errors=True))
        calls = []

        def run_exec(plan):
            calls.append(plan["arm"])
            root = plan.get("plugin_root")
            command = f"cat {Path(root) / 'skills' / 'testing-rules' / 'SKILL.md'}" if root else "echo no-plugin"
            return {
                "returncode": 0,
                "stdout": _read_event(command) + "\n",
                "stderr": "",
                "last_message": "code",
            }

        def judge(_prompt):
            raise RuntimeError("judge down")

        report = codex_rules.run_suite(
            [case],
            1,
            output,
            run_exec,
            judge,
            Path("/codex-rules-user-home-unused"),
            None,
            None,
            check_plugins=False,
        )
        row = report["cases"][0]["arms"]["with"][0]
        self.assertNotIn("error", row)
        self.assertTrue(any(item["name"] == "skill-fired" and item["passed"] for item in row["graders"]))
        graded = [item for item in row["graders"] if item["name"] != "skill-fired"]
        self.assertTrue(graded)
        self.assertTrue(all(item.get("error") == "judge down" and "passed" not in item for item in graded))
        self.assertIn("with: should-fire 1/1", codex_rules_grade.format_trigger(report))
        calls.clear()

        def refuse(_plan):
            raise AssertionError("exec")

        again = codex_rules.run_suite(
            [case],
            1,
            output,
            refuse,
            judge,
            Path("/codex-rules-user-home-unused"),
            None,
            None,
            check_plugins=False,
        )
        self.assertEqual(calls, [])
        again_row = again["cases"][0]["arms"]["with"][0]
        self.assertTrue(any(item["name"] == "skill-fired" and item["passed"] for item in again_row["graders"]))
        self.assertNotIn("error", again_row)

    def test_saved_transcript_skips_exec(self):
        case = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        output = Path(tempfile.mkdtemp(prefix="codex-rules-out-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(output, ignore_errors=True))
        root = output / "plugin"
        skill = root / "skills" / "testing-rules" / "SKILL.md"
        skill.parent.mkdir(parents=True)
        skill.write_text("skill body\n")
        store.save_exec(
            store.slot(output, case["name"], "with", 1),
            {
                "returncode": 0,
                "stdout": _read_event(f"cat {skill}") + "\n",
                "stderr": "",
                "last_message": "code",
                "plugin_root": str(root),
            },
        )
        calls = []

        def run_exec(plan):
            calls.append(plan["arm"])
            return {
                "returncode": 0,
                "stdout": _read_event("echo no-plugin") + "\n",
                "stderr": "",
                "last_message": "code",
            }

        report = codex_rules.run_suite(
            [case],
            1,
            output,
            run_exec,
            lambda _prompt: True,
            Path("/codex-rules-user-home-unused"),
            None,
            None,
            check_plugins=False,
        )
        self.assertEqual(calls, ["without"])
        self.assertTrue(report["cases"][0]["arms"]["with"][0]["graders"][0]["passed"])
        self.assertFalse(report["partial"])

    def test_preflight_aborts_before_a_paid_attempt(self):
        case = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        output = Path(tempfile.mkdtemp(prefix="codex-rules-out-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(output, ignore_errors=True))
        calls = []

        def run_exec(_plan):
            calls.append("exec")
            raise AssertionError("exec")

        def empty(_built):
            return {"installed": [], "available": []}

        with self.assertRaises(SystemExit) as raised:
            codex_rules.run_suite(
                [case],
                1,
                output,
                run_exec,
                lambda _prompt: True,
                Path("/codex-rules-user-home-unused"),
                None,
                None,
                check_plugins=True,
                list_fn=empty,
            )
        self.assertIn("no-mistakes-rules", str(raised.exception))
        self.assertEqual(calls, [])

        def disabled(built):
            if built["arm"] == "with":
                return {
                    "installed": [{"name": "no-mistakes-rules", "installed": True, "enabled": False}],
                    "available": [],
                }
            return {"installed": [], "available": []}

        with self.assertRaises(SystemExit):
            codex_rules.run_suite(
                [case],
                1,
                output,
                run_exec,
                lambda _prompt: True,
                Path("/codex-rules-user-home-unused"),
                None,
                None,
                check_plugins=True,
                list_fn=disabled,
            )
        self.assertEqual(calls, [])

    def test_preflight_allows_only_the_enabled_rules_plugin(self):
        case = codex_rules_grade.case_prompt(REPO / "evals" / "rules-testing-write-error-text")
        output = Path(tempfile.mkdtemp(prefix="codex-rules-out-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(output, ignore_errors=True))
        seen = []

        def list_fn(built):
            seen.append(("list", built["arm"]))
            home = Path(built["env"]["CODEX_HOME"])
            self.assertEqual(home, built["codex_home"])
            self.assertFalse(str(home).startswith(str(Path("/codex-rules-user-home-unused"))))
            self.assertEqual(codex_rules.plugin_list_argv(), ["codex", "plugin", "list", "--json"])
            if built["arm"] == "with":
                return {
                    "installed": [{"name": "no-mistakes-rules", "installed": True, "enabled": True}],
                    "available": [],
                }
            self.assertEqual(built["arm"], "without")
            return {"installed": [], "available": []}

        def run_exec(plan):
            seen.append(("exec", plan["arm"]))
            root = plan.get("plugin_root")
            command = f"cat {Path(root) / 'skills' / 'testing-rules' / 'SKILL.md'}" if root else "echo no-plugin"
            return {
                "returncode": 0,
                "stdout": _read_event(command) + "\n",
                "stderr": "",
                "last_message": "code",
            }

        report = codex_rules.run_suite(
            [case],
            1,
            output,
            run_exec,
            lambda _prompt: True,
            Path("/codex-rules-user-home-unused"),
            None,
            None,
            check_plugins=True,
            list_fn=list_fn,
        )
        self.assertEqual(
            seen,
            [("list", "with"), ("list", "without"), ("exec", "with"), ("exec", "without")],
        )
        for arm in ("with", "without"):
            self.assertNotIn("error", report["cases"][0]["arms"][arm][0])

    def test_exec_codex_writes_the_transcript_file(self):
        slot = Path(tempfile.mkdtemp(prefix="codex-rules-exec-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(slot, ignore_errors=True))
        last = slot / "last-message.txt"
        script = (
            "import json, pathlib, sys\n"
            "pathlib.Path(sys.argv[1]).write_text('final\\n')\n"
            "print(json.dumps({'type':'item.completed','item':{'type':'command_execution',"
            "'command':'echo hi','status':'completed','exit_code':0}}))\n"
            "sys.stdout.flush()\n"
        )
        plan = {
            "argv": [sys.executable, "-c", script, str(last)],
            "env": os.environ.copy(),
            "cwd": str(slot),
            "timeout_seconds": 30,
            "last_message": str(last),
            "transcript": str(slot / "transcript.jsonl"),
        }
        result = codex_rules.exec_codex(plan)
        self.assertEqual(result["returncode"], 0)
        self.assertEqual(result["last_message"], "final\n")
        self.assertIn("command_execution", (slot / "transcript.jsonl").read_text())
        self.assertFalse((slot / "transcript.jsonl.partial").exists())

    def test_exec_timeout_keeps_the_transcript(self):
        slot = Path(tempfile.mkdtemp(prefix="codex-rules-exec-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(slot, ignore_errors=True))
        script = "import time\nprint('hello', flush=True)\ntime.sleep(30)\n"
        plan = {
            "argv": [sys.executable, "-c", script],
            "env": os.environ.copy(),
            "cwd": str(slot),
            "timeout_seconds": 1,
            "last_message": str(slot / "missing-last.txt"),
            "transcript": str(slot / "transcript.jsonl"),
        }
        result = codex_rules.exec_codex(plan)
        self.assertEqual(result["returncode"], 124)
        self.assertIn("hello", (slot / "transcript.jsonl").read_text())
        self.assertFalse((slot / "transcript.jsonl.partial").exists())

    def test_exec_decode_error_leaves_the_partial_transcript(self):
        slot = Path(tempfile.mkdtemp(prefix="codex-rules-exec-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(slot, ignore_errors=True))
        script = "import sys\nsys.stdout.buffer.write(b'\\xff\\n')\nsys.stdout.buffer.flush()\n"
        plan = {
            "argv": [sys.executable, "-c", script],
            "env": os.environ.copy(),
            "cwd": str(slot),
            "timeout_seconds": 30,
            "last_message": str(slot / "missing-last.txt"),
            "transcript": str(slot / "transcript.jsonl"),
        }
        with self.assertRaises(UnicodeDecodeError):
            codex_rules.exec_codex(plan)
        self.assertFalse((slot / "transcript.jsonl").exists())
        self.assertTrue((slot / "transcript.jsonl.partial").exists())

    def test_judge_rejects_a_non_boolean_passed(self):
        self.assertTrue(codex_rules_grade.passed_from_judge_output('{"passed": true}'))
        self.assertFalse(codex_rules_grade.passed_from_judge_output('{"passed": false}'))
        with self.assertRaises(ValueError):
            codex_rules_grade.passed_from_judge_output('{"passed": "false"}')
        case = REPO / "evals" / "rules-testing-write-error-text"
        root = Path(tempfile.mkdtemp(prefix="codex-rules-plugin-"))
        self.addCleanup(lambda: __import__("shutil").rmtree(root, ignore_errors=True))
        skill = root / "skills" / "testing-rules" / "SKILL.md"
        skill.parent.mkdir(parents=True)
        skill.write_text("skill body\n")
        run = codex_rules_grade.grade_run(case, _read_event(f"cat {skill}"), "code", lambda _prompt: "false", root)
        judged = [item for item in run["graders"] if item["name"] != "skill-fired"]
        self.assertTrue(judged)
        self.assertTrue(all(item.get("error") == "judge passed must be a boolean" and "passed" not in item for item in judged))
        self.assertTrue(any(item["name"] == "skill-fired" and item["passed"] for item in run["graders"]))


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
