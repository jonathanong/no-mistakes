#!/usr/bin/env python3
"""Decide whether a Codex transcript opened a rules SKILL.md.

A finished command counts when a reader opens `<skill>-rules/SKILL.md` under
the attempt's installed plugin cache or the marketplace copy of that plugin.
Paths resolve from the command cwd and from any `cd` earlier in that command.
"""
import json
import os
import re
import shlex
from pathlib import Path

_READERS = {"cat", "sed", "head", "tail", "less", "more", "nl", "awk", "rg", "grep", "bat"}
_SHELLS = {"bash", "sh", "zsh", "dash"}
_SHELL_FLAGS = {"-lc", "-c", "-cl"}
_SKILL_DIR = re.compile(r"[\w-]+-rules")
# Flags whose next token is an option value, not a file to grade.
_FLAG_ARG = {
    "rg": {
        "-e", "--regexp", "-g", "--glob", "-t", "--type", "-T", "--type-not",
        "-m", "--max-count", "-A", "--after-context", "-B", "--before-context",
        "-C", "--context", "-r", "--replace",
    },
    "grep": {
        "-e", "--regexp", "-f", "--file", "-m", "--max-count",
        "-A", "--after-context", "-B", "--before-context", "-C", "--context",
    },
}
_SCRIPT_FLAGS = {"-c", "-e", "--eval"}


def skills_in_transcript(
    transcript: str,
    plugin_root: Path | None,
    *,
    marketplace_root: Path | None = None,
    attempt_root: Path | None = None,
    cwd: Path | None = None,
) -> list:
    roots = [Path(path) for path in (plugin_root, marketplace_root) if path]
    if not roots:
        return []
    if attempt_root is None:
        attempt_root = roots[0]
    found = []
    for command, item_cwd in _commands(transcript):
        found.extend(
            _skills_in_command(
                command,
                roots,
                Path(attempt_root),
                Path(item_cwd) if item_cwd else (Path(cwd) if cwd else None),
            )
        )
    return found


def _commands(transcript: str) -> list:
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
        if not isinstance(command, str):
            continue
        item_cwd = item.get("cwd") if isinstance(item.get("cwd"), str) else None
        commands.append((command, item_cwd))
    return commands


def _skills_in_command(command: str, roots: list, attempt_root: Path, cwd: Path | None) -> list:
    found = []
    for script in _inner_scripts(command):
        found.extend(_skills_in_script(script, roots, attempt_root, cwd))
    return found


def _inner_scripts(command: str) -> list:
    try:
        argv = shlex.split(command)
    except ValueError:
        return []
    if len(argv) >= 3 and Path(argv[0]).name in _SHELLS and argv[1] in _SHELL_FLAGS:
        return [argv[2] if len(argv) == 3 else " ".join(argv[2:])]
    return [command]


def _skills_in_script(script: str, roots: list, attempt_root: Path, cwd: Path | None) -> list:
    parts = _split_ops(script)
    current = cwd
    found = []
    index = 0
    while index < len(parts):
        stages = [parts[index]]
        while stages[-1][1] == "|" and index + 1 < len(parts):
            index += 1
            stages.append(parts[index])
        stage_cwd = current
        for text, _sep in stages:
            try:
                argv = shlex.split(text)
            except ValueError:
                continue
            if not argv:
                continue
            name = Path(argv[0]).name
            if name == "cd" and len(stages) == 1:
                current = _cd_target(argv, stage_cwd, roots, attempt_root)
                continue
            if _reader_kind(name) is None:
                continue
            for token in _reader_tokens(argv):
                skill = _skill_name(token, stage_cwd, roots, attempt_root)
                if skill:
                    found.append(skill)
        index += 1
    return found


def _split_ops(script: str) -> list:
    parts = []
    buf = []
    quote = None
    index = 0

    def push(sep: str | None) -> None:
        text = "".join(buf).strip()
        buf.clear()
        if text:
            parts.append((text, sep))

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
            push("seq")
            index += 2
            continue
        if char == "|":
            push("|")
            index += 1
            continue
        if char in ";\n":
            push("seq")
            index += 1
            continue
        buf.append(char)
        index += 1
    push(None)
    return parts


def _reader_kind(name: str) -> str | None:
    if name in _READERS:
        return "search" if name in _FLAG_ARG else "file"
    if name == "node" or _is_python(name):
        return "script"
    return None


def _is_python(name: str) -> bool:
    if name == "python":
        return True
    if not name.startswith("python"):
        return False
    return name[6:7].isdigit()


def _reader_tokens(argv: list) -> list:
    kind = _reader_kind(Path(argv[0]).name)
    if kind == "script":
        return _script_tokens(argv)
    if kind == "search":
        return _search_tokens(argv)
    return _file_tokens(argv)


def _file_tokens(argv: list) -> list:
    tokens = []
    index = 1
    while index < len(argv):
        arg = argv[index]
        if arg == "<" and index + 1 < len(argv):
            tokens.append(argv[index + 1])
            index += 2
            continue
        if arg.startswith("-"):
            index += 1
            continue
        tokens.append(arg)
        index += 1
    return tokens


def _search_tokens(argv: list) -> list:
    name = Path(argv[0]).name
    tokens = []
    positionals = []
    pattern_flag = False
    index = 1
    while index < len(argv):
        arg = argv[index]
        if arg == "<" and index + 1 < len(argv):
            tokens.append(argv[index + 1])
            index += 2
            continue
        if arg in _FLAG_ARG[name]:
            if index + 1 >= len(argv):
                break
            if name == "grep" and arg in ("-f", "--file"):
                tokens.append(argv[index + 1])
            if arg in ("-e", "--regexp", "-f", "--file"):
                pattern_flag = True
            index += 2
            continue
        if arg.startswith("-"):
            index += 1
            continue
        positionals.append(arg)
        index += 1
    if pattern_flag:
        tokens.extend(positionals)
    elif len(positionals) >= 2:
        tokens.extend(positionals[1:])
    return tokens


def _script_tokens(argv: list) -> list:
    tokens = []
    index = 1
    while index < len(argv):
        arg = argv[index]
        if arg == "<" and index + 1 < len(argv):
            tokens.append(argv[index + 1])
            index += 2
            continue
        if arg in _SCRIPT_FLAGS and index + 1 < len(argv):
            tokens.extend(_quoted(argv[index + 1]))
            index += 2
            continue
        if arg.startswith("-"):
            index += 1
            continue
        tokens.append(arg)
        index += 1
    return tokens


def _quoted(text: str) -> list:
    found = []
    quote = None
    buf = []
    for char in text:
        if quote:
            if char == quote:
                if buf:
                    found.append("".join(buf))
                buf = []
                quote = None
            else:
                buf.append(char)
            continue
        if char in "'\"":
            quote = char
    return found


def _cd_target(argv: list, cwd: Path | None, roots: list, attempt_root: Path) -> Path | None:
    index = 1
    while index < len(argv):
        arg = argv[index]
        if arg == "-":
            return cwd
        if arg.startswith("-"):
            index += 1
            continue
        return _locate(arg, cwd, roots, attempt_root) or cwd
    return cwd


def _skill_name(token: str, cwd: Path | None, roots: list, attempt_root: Path) -> str | None:
    candidate = _locate(token, cwd, roots, attempt_root)
    if candidate is None or not _is_under(candidate, _anchor(attempt_root, roots, attempt_root)):
        return None
    if candidate.name != "SKILL.md" or not _SKILL_DIR.fullmatch(candidate.parent.name):
        return None
    for root in roots:
        if _is_under(candidate, _anchor(root, roots, attempt_root)):
            return candidate.parent.name
    return None


def _locate(token: str, cwd: Path | None, roots: list, attempt_root: Path) -> Path | None:
    if not token or token.startswith("-"):
        return None
    path = Path(token)
    if not path.is_absolute():
        if cwd is None:
            return None
        path = Path(cwd) / path
    return _anchor(path, roots, attempt_root)


def _anchor(path: Path, roots: list, attempt_root: Path) -> Path:
    live = any(item.exists() for item in [*roots, attempt_root])
    if live:
        try:
            return path.resolve()
        except OSError:
            return Path(os.path.normpath(path))
    return Path(os.path.normpath(path))


def _is_under(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True
