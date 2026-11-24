#!/usr/bin/env python3
"""Decide whether a Codex transcript opened a rules SKILL.md.

A finished command counts when a reader opens `<skill>-rules/SKILL.md` under
the attempt's installed plugin cache or the marketplace copy of that plugin.
Paths resolve from the command cwd and from any `cd` earlier in that command.
"""
import ast
import json
import os
import re
import shlex
from fnmatch import fnmatchcase
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
_PATH_TOOLS = {
    "cp", "diff", "file", "install", "ln", "ls", "mkdir", "mv", "readlink",
    "realpath", "rm", "rmdir", "source", "stat", "tee", "touch", "wc",
}


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
    attempt_root = Path(attempt_root)
    for command, item_cwd, output, success in _commands(transcript):
        if not success:
            continue
        for argv, directory, heredocs in _script_stages(command, item_cwd or cwd, attempt_root):
            found.extend(_skills_in_stage(argv, directory, heredocs, output, roots, attempt_root))
    return found


def external_path_findings(
    transcript: str, attempt_root: Path | None, *, cwd: Path | None = None
) -> list[dict]:
    """Report redacted hints for explicit paths and cwd in completed command events."""
    if not attempt_root:
        return []
    roots = tuple(_audit_path(base) for base in _forms(attempt_root))
    findings = []
    for index, (command, item_cwd, _output, _success) in enumerate(_commands(transcript), 1):
        directory = item_cwd or cwd
        if directory and not _audit_under(directory, roots):
            findings.append({"commandIndex": index, "reason": "outside-working-directory"})
        reasons = {
            reason
            for argv, stage_cwd, heredocs in _script_stages(command, directory, roots[0])
            for token in _audit_tokens(argv, heredocs)
            if (reason := _audit_reason(token, stage_cwd, roots))
        }
        findings.extend({"commandIndex": index, "reason": reason} for reason in sorted(reasons))
    return findings


def _commands(transcript: str):
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
        if item.get("status") not in ("completed", "failed"):
            continue
        command = item.get("command")
        if not isinstance(command, str):
            continue
        item_cwd = Path(item["cwd"]) if isinstance(item.get("cwd"), str) and item["cwd"] else None
        output = item.get("aggregated_output")
        success = item.get("status") == "completed" and item.get("exit_code") == 0
        yield command, item_cwd, output if isinstance(output, str) else "", success


def _skills_in_stage(argv, cwd, heredocs, output, roots, attempt_root):
    name = Path(argv[0]).name
    if name == "find":
        tokens = _find_exec_tokens(argv)
    elif _reader_kind(name) is not None and not (name == "rg" and "--files" in argv):
        tokens = _reader_tokens(argv)
        tokens.extend(_heredoc_reads(argv, heredocs))
    else:
        return []
    found = [
        skill
        for token in tokens
        if (skill := _skill_name(token, cwd, roots, attempt_root))
    ]
    if name == "rg" and not any(flag in argv for flag in ("--no-filename", "-I")):
        found.extend(
            skill for skill in _rg_output_skills(output, cwd, tokens, roots, attempt_root)
            if skill not in found
        )
    return found


def _script_stages(script: str, cwd: Path | None, attempt_root: Path, depth: int = 0):
    if depth >= 16:
        return
    parts = _split_ops(script)
    current = cwd
    index = 0
    while index < len(parts):
        stages = [parts[index]]
        while stages[-1][1] == "|" and index + 1 < len(parts):
            index += 1
            stages.append(parts[index])
        stage_cwd = current
        for text, _sep, heredocs in stages:
            try:
                lexer = shlex.shlex(text, posix=True, punctuation_chars="<>&")
                lexer.whitespace_split = True
                lexer.commenters = ""
                argv = list(lexer)
            except ValueError:
                continue
            if not argv:
                continue
            inner = _subshell(text)
            if inner is not None:
                yield from _script_stages(inner, stage_cwd, attempt_root, depth + 1)
                continue
            name = Path(argv[0]).name
            yield argv, stage_cwd, heredocs
            if name in _SHELLS and len(argv) >= 3 and argv[1] in _SHELL_FLAGS:
                yield from _script_stages(argv[2], stage_cwd, attempt_root, depth + 1)
            elif name == "cd" and len(stages) == 1:
                current = _cd_target(argv, stage_cwd, attempt_root)
        index += 1


_HEREDOC_START = re.compile(r"""<<(-?)[ \t]*(['"]?)([A-Za-z_][\w-]*)\2(?=[ \t;&|\n]|$)""")


def _read_heredoc(script: str, offset: int, delimiter: str, strip_tabs: bool) -> tuple:
    start = offset
    while offset < len(script):
        end = script.find("\n", offset)
        if end == -1:
            end = len(script)
        line = script[offset:end]
        if (line.lstrip("\t") if strip_tabs else line) == delimiter:
            body = script[start:offset]
            if strip_tabs:
                body = "\n".join(line.lstrip("\t") for line in body.split("\n"))
            return body, end + 1
        offset = end + 1
    return "", offset


def _split_ops(script: str) -> list:
    parts = []
    buf = []
    quote = None
    depth = 0
    index = 0
    pending_heredocs = []

    def push(sep: str | None) -> None:
        text = "".join(buf).strip()
        buf.clear()
        if text:
            parts.append((text, sep, []))

    while index < len(script):
        char = script[index]
        if char == "\\" and quote != "'" and index + 1 < len(script):
            if script[index + 1] != "\n":
                buf.extend(script[index:index + 2])
            index += 2
            continue
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
        if char == "<" and (index == 0 or script[index - 1] != "<"):
            match = _HEREDOC_START.match(script, index)
            if match:
                strip_tabs, _quote, delimiter = match.groups()
                pending_heredocs.append((len(parts), delimiter, bool(strip_tabs)))
                buf.extend(f"<< {delimiter}")
                index = match.end()
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
            if char == "\n":
                for part_index, delimiter, strip_tabs in pending_heredocs:
                    body, index = _read_heredoc(script, index, delimiter, strip_tabs)
                    parts[part_index][2].append(body)
                pending_heredocs.clear()
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
    if arg in ("<<", "<<<", "<&") and index + 1 < len(argv):
        return index + 2
    return None


def _audit_tokens(argv: list, heredocs: list) -> list:
    name = Path(argv[0]).name
    tokens = []
    index = 1
    while index < len(argv) - 1:
        arg = argv[index]
        if arg in _OUTPUT_REDIR and arg != ">&":
            tokens.append(argv[index + 1])
        taken = _take_redirect(argv, index, tokens)
        index = taken if taken is not None else index + 1
    if name == "find":
        paths, _filter, _reader = _find_operands(argv)
        tokens.extend(paths)
    elif name == "rg" and "--files" in argv:
        tokens.extend(_search_tokens(argv, has_pattern=False))
    elif _reader_kind(name) is not None:
        tokens.extend(_reader_tokens(argv))
        tokens.extend(_heredoc_reads(argv, heredocs, include_writes=True))
    elif name in _PATH_TOOLS or name == "cd":
        tokens.extend(_file_tokens(argv))
    return tokens


def _audit_path(path: Path | str) -> Path:
    normalized = Path(os.path.normpath(path))
    if normalized.parts[:3] in (("/", "private", "tmp"), ("/", "private", "var")):
        return Path("/", *normalized.parts[2:])
    return normalized


def _audit_under(path: Path | str, roots: tuple[Path, ...]) -> bool:
    candidate = _audit_path(path)
    return any(_is_under(candidate, root) for root in roots)


def _audit_reason(token: str, cwd: Path | None, roots: tuple[Path, ...]) -> str | None:
    path = _locate(token, cwd, roots[0])
    if path is None:
        return None
    path = _audit_path(path)
    if any(path == root / "codex-home" / "auth.json" for root in roots):
        return "attempt-auth-file"
    if not any(_is_under(path, root) for root in roots):
        return "outside-attempt-root"
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


def _search_tokens(argv: list, *, has_pattern: bool = True) -> list:
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
    if pattern_flag or not has_pattern:
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


def _heredoc_reads(argv: list, heredocs: list, *, include_writes: bool = False) -> list:
    if not heredocs or not _is_python(Path(argv[0]).name):
        return []
    for token in argv[1:]:
        if token == "-" or token == "<<":
            break
        if token not in {"-B", "-E", "-I", "-O", "-OO", "-P", "-q", "-s", "-S", "-u"}:
            return []
    # The last standard-input heredoc is the one Python receives.
    try:
        tree = ast.parse(heredocs[-1])
    except (SyntaxError, ValueError, RecursionError):
        return []
    found = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        func = node.func
        if isinstance(func, ast.Name) and func.id == "open" and node.args:
            mode = next(
                (kw.value for kw in node.keywords if kw.arg == "mode"),
                node.args[1] if len(node.args) > 1 else None,
            )
            if include_writes or mode is None or (
                isinstance(mode, ast.Constant)
                and isinstance(mode.value, str)
                and ("r" in mode.value or "+" in mode.value)
            ):
                found.extend(_literal_path(node.args[0]))
        if isinstance(func, ast.Attribute) and func.attr in ("read_text", "read_bytes"):
            receiver = func.value
            if (
                isinstance(receiver, ast.Call)
                and isinstance(receiver.func, ast.Name)
                and receiver.func.id == "Path"
                and receiver.args
            ):
                found.extend(_literal_path(receiver.args[0]))
    return found


def _literal_path(node: ast.expr) -> list[str]:
    return [node.value] if isinstance(node, ast.Constant) and isinstance(node.value, str) else []


def _subshell(text: str) -> str | None:
    body = text.strip()
    if len(body) >= 2 and body[0] == "(" and body[-1] == ")":
        return body[1:-1].strip()
    return None


_FIND_ARG = {"-name", "-iname", "-path", "-regex", "-maxdepth", "-mindepth", "-type", "-size"}


def _find_exec_tokens(argv: list) -> list:
    paths, name_filter, reader = _find_operands(argv)
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


def _find_operands(argv: list) -> tuple:
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
    return paths, name_filter, reader


def _cd_target(argv: list, cwd: Path | None, attempt_root: Path) -> Path | None:
    index = 1
    while index < len(argv):
        arg = argv[index]
        if arg == "-":
            return cwd
        if arg.startswith("-"):
            index += 1
            continue
        return _locate(arg, cwd, attempt_root) or cwd
    return cwd


def _skill_name(token: str, cwd: Path | None, roots: list, attempt_root: Path) -> str | None:
    candidate = _locate(token, cwd, attempt_root)
    if candidate is None:
        return None
    if any(char in token for char in "*?["):
        return _glob_skill(candidate, roots, attempt_root)
    if not _anchor(candidate, attempt_root):
        return None
    if candidate.name != "SKILL.md" or not _SKILL_DIR.fullmatch(candidate.parent.name):
        return None
    for root in roots:
        if _anchor(candidate, root):
            return candidate.parent.name
    return None


def _glob_skill(pattern: Path, roots: list, attempt_root: Path) -> str | None:
    for skill in (part for part in pattern.parts if _SKILL_DIR.fullmatch(part)):
        for root in roots:
            target = root / "skills" / skill / "SKILL.md"
            if not _anchor(target, attempt_root):
                continue
            for form in _forms(target):
                if len(form.parts) == len(pattern.parts) and all(
                    fnmatchcase(value, glob) for value, glob in zip(form.parts, pattern.parts)
                ):
                    return skill
    return None


_RG_RESULT_PATH = re.compile(r"^(.+?SKILL\.md)(?=:|-\d+-|\Z)")


def _rg_output_skills(output: str, cwd: Path | None, tokens: list, roots: list, attempt_root: Path):
    search_paths = [_locate(token, cwd, attempt_root) for token in tokens]
    seen = set()
    for line in output.splitlines():
        match = _RG_RESULT_PATH.match(line)
        if not match:
            continue
        path = _locate(match.group(1), cwd, attempt_root)
        if path is None or path.name != "SKILL.md" or not any(
            base is not None
            and base != path
            and _is_under(_audit_path(path), _audit_path(base))
            for base in search_paths
        ):
            continue
        skill = _skill_name(str(path), cwd, roots, attempt_root)
        if skill and skill not in seen:
            seen.add(skill)
            yield skill


def _locate(token: str, cwd: Path | None, attempt_root: Path | None = None) -> Path | None:
    if not token or token.startswith("-") or "\x00" in token:
        return None
    if attempt_root:
        for alias, directory in (
            ("~", "home"), ("$HOME", "home"), ("${HOME}", "home"),
            ("$CODEX_HOME", "codex-home"), ("${CODEX_HOME}", "codex-home"),
        ):
            if token == alias or token.startswith(alias + "/"):
                token = str(attempt_root / directory) + token[len(alias):]
                break
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
    """Compare a transcript path with the saved and canonical attempt roots.

    Codex records `/private/tmp` and `/private/var`. A plan may store `/tmp`
    or `/var`. Only the saved root is canonicalized.
    """
    candidate = _audit_path(path)
    return any(_is_under(candidate, _audit_path(base)) for base in _forms(root))


def _is_under(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True
