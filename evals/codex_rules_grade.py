#!/usr/bin/env python3
"""Grade Codex rules transcripts with the existing grader markdown.

`skill-fired` stays a tool check. LLM graders send the rubric body and the
final message to the same judge model as scripts/run-evals.sh. The report
matches the aggregate-result.json shape summarize.py and power.py already read.
"""
import json
import re
import shlex
from pathlib import Path

JUDGE_MODEL = "claude-opus-5-5"
_PASSED = re.compile(r"\{[^{}]*\"passed\"\s*:\s*(true|false)[^{}]*\}")
_SKILL_DIR = re.compile(r"[\w-]+-rules")
# Content readers only. `ls` and `echo` mention a path without reading it.
_READERS = {"cat", "sed", "head", "tail", "less", "more"}
_SHELLS = {"bash", "sh", "zsh", "dash"}
_SHELL_FLAGS = {"-lc", "-c", "-cl"}


def parse_frontmatter(text: str) -> tuple:
    if not text.startswith("---\n"):
        raise ValueError("missing frontmatter")
    end = text.find("\n---\n", 4)
    if end < 0:
        raise ValueError("unclosed frontmatter")
    meta = {}
    lines = text[4:end].splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        if not line.strip():
            index += 1
            continue
        key, sep, value = line.partition(":")
        if not sep:
            raise ValueError(f"bad frontmatter line {line}")
        value = value.strip()
        if value == "|":
            index += 1
            block = []
            while index < len(lines) and (not lines[index] or lines[index][0] == " "):
                block.append(lines[index][2:] if lines[index].startswith("  ") else lines[index])
                index += 1
            meta[key.strip()] = "\n".join(block).strip()
            continue
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
            value = value[1:-1]
        meta[key.strip()] = value
        index += 1
    return meta, text[end + 5 :].strip()


def case_prompt(case_dir: Path) -> dict:
    meta, body = parse_frontmatter((case_dir / "prompt.md").read_text())
    system = meta.get("append_system_prompt", "").strip()
    prompt = f"{system}\n\n{body}".strip() if system else body
    tags = [part.strip() for part in meta.get("tags", "").strip("[]").split(",") if part.strip()]
    return {
        "name": case_dir.name,
        "dir": case_dir,
        "prompt": prompt,
        "runs": int(meta.get("runs", "3")),
        "timeout_seconds": int(meta.get("timeout_seconds", "300")),
        "tags": tags,
    }


def _split_shell(script: str) -> list:
    parts = []
    buf = []
    quote = None
    index = 0
    while index < len(script):
        char = script[index]
        if quote:
            buf.append(char)
            if char == quote:
                quote = None
            index += 1
            continue
        if char in "'\"":
            quote = char
            buf.append(char)
            index += 1
            continue
        if script.startswith("&&", index) or script.startswith("||", index):
            parts.append("".join(buf).strip())
            buf = []
            index += 2
            continue
        if char in "|;\n":
            parts.append("".join(buf).strip())
            buf = []
            index += 1
            continue
        buf.append(char)
        index += 1
    tail = "".join(buf).strip()
    if tail:
        parts.append(tail)
    return [part for part in parts if part]


def _inner_scripts(command: str) -> list:
    try:
        argv = shlex.split(command)
    except ValueError:
        return []
    if len(argv) >= 3 and Path(argv[0]).name in _SHELLS and argv[1] in _SHELL_FLAGS:
        return [argv[2] if len(argv) == 3 else " ".join(argv[2:])]
    return [command]


def _skill_name(token: str, plugin_root: Path) -> str | None:
    if not token or token.startswith("-"):
        return None
    candidate = Path(token)
    if not candidate.is_absolute():
        return None
    root = plugin_root
    try:
        if root.exists():
            root = root.resolve()
            candidate = candidate.resolve()
    except OSError:
        return None
    if not _is_under(candidate, root):
        return None
    if candidate.name != "SKILL.md" or not _SKILL_DIR.fullmatch(candidate.parent.name):
        return None
    return candidate.parent.name


def _is_under(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True


def _skills_read(command: str, plugin_root: Path) -> list:
    found = []
    for script in _inner_scripts(command):
        for part in _split_shell(script):
            try:
                argv = shlex.split(part)
            except ValueError:
                continue
            if not argv or Path(argv[0]).name not in _READERS:
                continue
            index = 1
            while index < len(argv):
                arg = argv[index]
                if arg == "<" and index + 1 < len(argv):
                    skill = _skill_name(argv[index + 1], plugin_root)
                    index += 2
                elif arg.startswith("-"):
                    index += 1
                    continue
                else:
                    skill = _skill_name(arg, plugin_root)
                    index += 1
                if skill:
                    found.append(skill)
    return found


def _completed_commands(transcript: str) -> list:
    commands = []
    for line in transcript.splitlines():
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(event, dict) or event.get("type") != "item.completed":
            continue
        item = event.get("item")
        if not isinstance(item, dict) or item.get("type") != "command_execution":
            continue
        if item.get("status") != "completed" or item.get("exit_code") != 0:
            continue
        command = item.get("command")
        if isinstance(command, str):
            commands.append(command)
    return commands


def skill_fired(meta: dict, transcript: str, plugin_root: Path | None) -> bool:
    """True when a finished command reads an installed `<skill>-rules/SKILL.md`.

    `codex exec --json` reports that read as `item.completed` /
    `command_execution`. Message text, `ls`, and a path that is only mentioned
    do not count. `skill_invocation` is not an exec JSON event.
    """
    if meta.get("type") != "tool_used" or meta.get("tool") != "Skill":
        raise ValueError("skill-fired grader is not a Skill tool check")
    if plugin_root is None:
        return False
    pattern = meta.get("input_match")
    regex = re.compile(pattern) if pattern else None
    minimum = int(meta.get("min", "1"))
    count = 0
    root = Path(plugin_root)
    for command in _completed_commands(transcript):
        for skill in _skills_read(command, root):
            blob = json.dumps({"skill": skill})
            if regex is None or regex.search(blob):
                count += 1
    return count >= minimum


def judge_prompt(rubric: str, last_message: str) -> str:
    return (
        "Grade the final answer against the rubric. "
        'Reply with one JSON object, {"passed": true} or {"passed": false}.\n\n'
        f"Rubric:\n{rubric}\n\nFinal answer:\n{last_message}"
    )


def judge_argv() -> list:
    return [
        "claude",
        "-p",
        "--model",
        JUDGE_MODEL,
        "--output-format",
        "json",
        "--no-session-persistence",
        "--safe-mode",
        "--restricted",
        "--strict-mcp-config",
        "--tools",
        "",
        "--permission-prompts",
        "none",
    ]


def passed_from_judge_output(raw: str) -> bool:
    text = raw
    try:
        data = json.loads(raw)
    except json.JSONDecodeError:
        data = None
    if isinstance(data, dict) and isinstance(data.get("result"), str):
        text = data["result"]
    elif isinstance(data, dict) and "passed" in data:
        return bool(data["passed"])
    match = _PASSED.search(text)
    if not match:
        raise ValueError("judge did not return passed")
    return json.loads(match.group(0))["passed"]


def grade_run(
    case_dir: Path,
    transcript: str,
    last_message: str,
    judge,
    plugin_root: Path | None,
) -> dict:
    graders = []
    for path in sorted((case_dir / "graders").glob("*.md")):
        meta, body = parse_frontmatter(path.read_text())
        if meta.get("type") == "tool_used":
            passed = skill_fired(meta, transcript, plugin_root)
            row = {"name": path.stem, "passed": passed, "evidence": last_message}
        elif meta.get("type") == "llm":
            if meta.get("focus", "last_message") != "last_message":
                raise ValueError(f"{path.name} focus is not last_message")
            try:
                passed = bool(judge(judge_prompt(body, last_message)))
            except Exception as exc:
                row = {"name": path.stem, "error": str(exc), "evidence": last_message}
                if "weight" in meta:
                    row["weight"] = float(meta["weight"])
                graders.append(row)
                continue
            row = {"name": path.stem, "passed": passed, "evidence": last_message}
        else:
            raise ValueError(f"unsupported grader {path.name}")
        if "weight" in meta:
            row["weight"] = float(meta["weight"])
        graders.append(row)
    return {"graders": graders}


def aggregate_report(cases: list) -> dict:
    return {
        "costUsd": 0,
        "partial": False,
        "partialReason": None,
        "suite": {
            "ablation": "codex-with-without",
            "plugins": [{"path": "plugins/no-mistakes-rules"}],
        },
        "cases": cases,
    }


def format_trigger(report: dict) -> str:
    """Primary measure: skill-fired rate on each arm. Pass rate is separate."""
    lines = ["trigger rate (skill-fired) per arm — primary measure"]
    for arm in ("with", "without"):
        fired = runs = off_fired = off_runs = 0
        for case in report["cases"]:
            rows = case["arms"].get(arm, [])
            negative = "-neg-" in case["name"] or case["name"].startswith("neg-hard")
            for row in rows:
                if row.get("error"):
                    continue
                hit = any(
                    grader["name"] == "skill-fired" and grader.get("passed")
                    for grader in row.get("graders", [])
                )
                if negative:
                    off_fired += int(hit)
                    off_runs += 1
                else:
                    fired += int(hit)
                    runs += 1
        lines.append(
            f"  {arm}: should-fire {fired}/{runs}, off-topic {off_fired}/{off_runs}"
        )
    lines.append("pass rate is a regression check (summarize.py exits 1 when with < without)")
    lines.append("costUsd 0 means dollars were not measured")
    return "\n".join(lines)
