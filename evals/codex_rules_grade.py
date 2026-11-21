#!/usr/bin/env python3
"""Grade Codex rules transcripts with the existing grader markdown.

`skill-fired` stays a tool check. LLM graders send the rubric body and the
final message to the same judge model as scripts/run-evals.sh. The report
matches the aggregate-result.json shape summarize.py and power.py already read.
"""
import json
import re
from pathlib import Path

JUDGE_MODEL = "claude-opus-5-5"
_SKILL_PATH = re.compile(r"([\w-]+-rules)/SKILL\.md")
_PASSED = re.compile(r"\{[^{}]*\"passed\"\s*:\s*(true|false)[^{}]*\}")
_TOOL_TYPES = {
    "mcp_tool_call",
    "McpToolCall",
    "function_call",
    "custom_tool_call",
    "skill_invocation",
    "SkillInvocation",
}


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


def _add_skill(found: list, name, plugin) -> None:
    if not isinstance(name, str) or not name or name == "Skill":
        return
    value = f"{plugin}:{name}" if isinstance(plugin, str) and plugin else name
    found.append(json.dumps({"skill": value}))


def skill_inputs(event) -> list:
    """Tool inputs a `skill-fired` regex can match. Message text is ignored."""
    found = []

    def walk(node) -> None:
        if isinstance(node, list):
            for item in node:
                walk(item)
            return
        if not isinstance(node, dict):
            return
        typ = node.get("type")
        tool = node.get("tool") if isinstance(node.get("tool"), str) else ""
        name = node.get("name") if isinstance(node.get("name"), str) else ""
        skill_call = tool == "Skill" or name == "Skill" or typ in ("skill_invocation", "SkillInvocation")
        if typ in _TOOL_TYPES or skill_call:
            raw = node.get("arguments", node.get("input", node.get("args")))
            if isinstance(raw, str):
                found.append(raw)
            elif isinstance(raw, (dict, list)):
                found.append(json.dumps(raw))
            if skill_call or node.get("skill_name") or node.get("skillName"):
                _add_skill(
                    found,
                    node.get("skill_name") or node.get("skillName") or name,
                    node.get("plugin_id") or node.get("pluginId"),
                )
        command = node.get("command")
        if isinstance(command, str):
            for match in _SKILL_PATH.finditer(command):
                _add_skill(found, match.group(1), None)
        for value in node.values():
            walk(value)

    walk(event)
    return found


def skill_fired(meta: dict, transcript: str) -> bool:
    if meta.get("type") != "tool_used" or meta.get("tool") != "Skill":
        raise ValueError("skill-fired grader is not a Skill tool check")
    pattern = meta.get("input_match")
    regex = re.compile(pattern) if pattern else None
    minimum = int(meta.get("min", "1"))
    count = 0
    for line in transcript.splitlines():
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        for blob in skill_inputs(event):
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


def grade_run(case_dir: Path, transcript: str, last_message: str, judge) -> dict:
    graders = []
    for path in sorted((case_dir / "graders").glob("*.md")):
        meta, body = parse_frontmatter(path.read_text())
        if meta.get("type") == "tool_used":
            passed = skill_fired(meta, transcript)
        elif meta.get("type") == "llm":
            if meta.get("focus", "last_message") != "last_message":
                raise ValueError(f"{path.name} focus is not last_message")
            passed = bool(judge(judge_prompt(body, last_message)))
        else:
            raise ValueError(f"unsupported grader {path.name}")
        row = {"name": path.stem, "passed": passed, "evidence": last_message}
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
