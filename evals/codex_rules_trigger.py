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
            inner = _subshell(text)
            if inner is not None:
                found.extend(_skills_in_script(inner, roots, attempt_root, stage_cwd))
                continue
            name = Path(argv[0]).name
            if name == "find":
                for token in _find_exec_tokens(argv):
                    skill = _skill_name(token, stage_cwd, roots, attempt_root)
                    if skill:
                        found.append(skill)
                continue
            if name == "cd" and len(stages) == 1:
                current = _cd_target(argv, stage_cwd)
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
    depth = 0
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
        if char == "(":
            depth += 1
            buf.append(char)
            index += 1
            continue
        if char == ")" and depth:
            depth -= 1
            buf.append(char)
            index += 1
            continue
        if depth:
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


_OUTPUT_REDIR = {">", ">>", ">&", ">|", "&>", "&>>"}


def _take_redirect(argv: list, index: int, tokens: list) -> int | None:
    arg = argv[index]
    if arg == "<" and index + 1 < len(argv):
        tokens.append(argv[index + 1])
        return index + 2
    if arg in _OUTPUT_REDIR and index + 1 < len(argv):
        return index + 2
    return None


def _file_tokens(argv: list) -> list:
    tokens = []
    index = 1
    while index < len(argv):
        taken = _take_redirect(argv, index, tokens)
        if taken is not None:
            index = taken
            continue
        if argv[index].startswith("-"):
            index += 1
            continue
        tokens.append(argv[index])
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
        taken = _take_redirect(argv, index, tokens)
        if taken is not None:
            index = taken
            continue
        flag, sep, attached = arg.partition("=")
        if sep and flag in _FLAG_ARG[name]:
            if name == "grep" and flag in ("-f", "--file"):
                tokens.append(attached)
            if flag in ("-e", "--regexp", "-f", "--file"):
                pattern_flag = True
            index += 1
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
        taken = _take_redirect(argv, index, tokens)
        if taken is not None:
            index = taken
            continue
        if arg in _SCRIPT_FLAGS and index + 1 < len(argv):
            tokens.extend(_script_reads(argv[index + 1]))
            index += 2
            continue
        if arg.startswith("-"):
            index += 1
            continue
        tokens.append(arg)
        index += 1
    return tokens


_SCRIPT_READ = re.compile(
    r"""(?:\b(?:open|readFileSync|readFile)\(\s*(['"])(.*?)\1|\bPath\(\s*(['"])(.*?)\3\)\s*\.read_(?:text|bytes)\()"""
)


def _script_reads(text: str) -> list:
    found = []
    for match in _SCRIPT_READ.finditer(text):
        found.append(match.group(2) or match.group(4))
    return found


def _subshell(text: str) -> str | None:
    body = text.strip()
    if len(body) >= 2 and body[0] == "(" and body[-1] == ")":
        return body[1:-1].strip()
    return None


_FIND_ARG = {"-name", "-iname", "-path", "-regex", "-maxdepth", "-mindepth", "-type", "-size"}


def _find_exec_tokens(argv: list) -> list:
    reader = False
    name_filter = None
    paths = []
    index = 1
    while index < len(argv):
        arg = argv[index]
        if arg in ("-exec", "-execdir") and index + 1 < len(argv):
            command = Path(argv[index + 1]).name
            index += 2
            operands = []
            while index < len(argv) and argv[index] not in ("+", ";"):
                operands.append(argv[index])
                index += 1
            index += 1
            if _reader_kind(command) is not None and "{}" in operands:
                reader = True
            continue
        if arg in _FIND_ARG and index + 1 < len(argv):
            if arg in ("-name", "-iname"):
                name_filter = argv[index + 1]
            index += 2
            continue
        if arg.startswith("-"):
            index += 1
            continue
        paths.append(arg)
        index += 1
    if not reader:
        return []
    kept = []
    for path in paths:
        name = Path(path).name
        if name_filter not in (None, "SKILL.md") and name != name_filter:
            continue
        kept.append(path)
        if name_filter in (None, "SKILL.md") and _SKILL_DIR.fullmatch(name):
            kept.append(str(Path(path) / "SKILL.md"))
    return kept


def _cd_target(argv: list, cwd: Path | None) -> Path | None:
    index = 1
    while index < len(argv):
        arg = argv[index]
        if arg == "-":
            return cwd
        if arg.startswith("-"):
            index += 1
            continue
        return _locate(arg, cwd) or cwd
    return cwd


def _skill_name(token: str, cwd: Path | None, roots: list, attempt_root: Path) -> str | None:
    candidate = _locate(token, cwd)
    if candidate is None or not _anchor(candidate, attempt_root):
        return None
    if candidate.name != "SKILL.md" or not _SKILL_DIR.fullmatch(candidate.parent.name):
        return None
    for root in roots:
        if _anchor(candidate, root):
            return candidate.parent.name
    return None


def _locate(token: str, cwd: Path | None) -> Path | None:
    if not token or token.startswith("-"):
        return None
    path = Path(token)
    if not path.is_absolute():
        if cwd is None:
            return None
        path = Path(cwd) / path
    return Path(os.path.normpath(path))


def _forms(path: Path) -> tuple:
    saved = Path(os.path.normpath(path))
    try:
        canonical = Path(os.path.realpath(path))
    except OSError:
        return (saved,)
    if canonical == saved:
        return (saved,)
    return (saved, canonical)


def _anchor(path: Path, root: Path) -> bool:
    """True when `path` is under `root` as saved or as canonicalized.

    Codex records `/private/tmp` and `/private/var`. A plan may store `/tmp`
    or `/var`. After the attempt directory is deleted, either form matches.
    """
    for candidate in _forms(path):
        for base in _forms(root):
            if _is_under(candidate, base):
                return True
    return False


def _is_under(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True
