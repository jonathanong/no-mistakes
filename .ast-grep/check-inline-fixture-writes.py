#!/usr/bin/env python3
"""Gate the inline fixture-write ban.

ast-grep scan runs with both fixture-write rules turned off so the shrink-only
baseline does not fail the raw scan. This script is the gate:

- findings are the union of the two project rules plus the same calls in test
  files the path globs miss (`#[cfg(test)] mod` targets, child `mod`s of test
  files, and `include!` slices)
- allowlisted paths must still match, must carry a one-line justification, and
  must not also be baselined
- `copy` allowlist entries may only match `create_dir_all`
- remaining finding paths must equal the baseline exactly
- once the baseline exists at --base, the current path set must be a subset
- an empty baseline at --base is final: no paths may be added again
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST = ROOT / ".ast-grep" / "inline-fixture-write-allowlist.txt"
BASELINE = ROOT / ".ast-grep" / "inline-fixture-write-baseline.txt"
CALL_RULE = ".ast-grep/utils/inline-fixture-write-call.yml"
PATH_RULE = "no-inline-test-fixture-write"
CFG_RULE = "no-inline-test-fixture-write-cfg"

CFG_ATTR = re.compile(
    r"#\s*\[\s*cfg\(\s*(?:test\s*\)|(?:all|any)\((?:[^]\"']|\"[^\"]*\")*\btest\s*[,)])"
)
NOT_TEST_ATTR = re.compile(
    r"#\s*\[\s*cfg\(\s*not\(\s*(?:test\b|(?:all|any)\((?:[^]\"']|\"[^\"]*\")*\btest\b)"
)
PATH_ATTR = re.compile(r"#\s*\[\s*path\s*=\s*\"([^\"]+)\"\s*\]")
MOD_DECL = re.compile(
    r"^(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;\s*(?://.*)?$"
)
INCLUDE_TEXT = re.compile(r"""include!\(\s*"([^"]+)"\s*\)""")
INCLUDE_RULE = (
    "id: test-include\nlanguage: rust\nrule:\n  pattern: include!($PATH)\n"
)


def git_env() -> dict[str, str]:
    env = os.environ.copy()
    env.pop("GIT_DIR", None)
    env.pop("GIT_WORK_TREE", None)
    return env


def run(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        env=git_env(),
        text=True,
        capture_output=True,
    )


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def ast_grep(args: list[str]) -> list[dict]:
    proc = run(["ast-grep", *args])
    if proc.returncode not in (0, 1):
        sys.stderr.write(proc.stderr)
        sys.stderr.write(proc.stdout)
        raise SystemExit(proc.returncode or 1)
    rows = []
    for line in proc.stdout.splitlines():
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as error:
            fail(f"ast-grep JSON line failed to parse: {error}\n{line}")
    return rows


def rel_path(path: str) -> str:
    candidate = Path(path)
    if candidate.is_absolute():
        candidate = candidate.relative_to(ROOT)
    text = candidate.as_posix()
    if text.startswith("../") or text == ".." or text.startswith("/"):
        fail(f"ast-grep reported a path outside the repo: {path}")
    return text


def is_sample(rel: str) -> bool:
    return "fixtures" in rel.split("/") or "test-cases" in rel.split("/")


def is_glob_test(rel: str) -> bool:
    if not rel.endswith(".rs") or is_sample(rel):
        return False
    name = rel.rsplit("/", 1)[-1]
    if name == "tests.rs" or name.endswith("_tests.rs"):
        return True
    return "tests" in rel.split("/")[:-1]


def tracked_rust() -> list[str]:
    proc = run(
        [
            "git",
            "ls-files",
            "-z",
            "-c",
            "-o",
            "--exclude-standard",
            "--",
            "*.rs",
        ]
    )
    if proc.returncode != 0:
        fail(proc.stderr.strip() or "git ls-files failed")
    return [part for part in proc.stdout.split("\0") if part.endswith(".rs")]


def resolve_mod(parent: Path, name: str, path_attr: str | None) -> list[Path]:
    if path_attr is not None:
        return [parent.parent / path_attr]
    base = parent.parent if parent.name == "mod.rs" else parent.parent / parent.stem
    return [base / f"{name}.rs", base / name / "mod.rs"]


def parse_mod_decls(text: str) -> list[tuple[list[str], str, str | None]]:
    decls: list[tuple[list[str], str, str | None]] = []
    pending: list[str] = []

    def take(attrs: list[str], body: str) -> None:
        match = MOD_DECL.match(body)
        if not match:
            return
        path_attr = None
        for attr in attrs:
            found = PATH_ATTR.search(attr)
            if found:
                path_attr = found.group(1)
        decls.append((attrs, match.group(1), path_attr))

    for raw in text.splitlines():
        stripped = raw.strip()
        if not stripped or stripped.startswith("//"):
            continue
        same = re.match(r"^((?:#\s*\[[^\]]*\]\s*)+)(\S.*)$", stripped)
        if same and MOD_DECL.match(same.group(2).strip()):
            attrs = pending + re.findall(r"#\s*\[[^\]]*\]", same.group(1))
            take(attrs, same.group(2).strip())
            pending = []
            continue
        if stripped.startswith("#[") or stripped.startswith("#!["):
            pending.append(stripped)
            continue
        take(pending, stripped)
        pending = []
    return decls


def existing_mod_files(rel: str, name: str, path_attr: str | None) -> list[str]:
    found: list[str] = []
    for candidate in resolve_mod(ROOT / rel, name, path_attr):
        if not candidate.is_file():
            continue
        try:
            relative = candidate.resolve().relative_to(ROOT.resolve()).as_posix()
        except ValueError:
            continue
        if not is_sample(relative):
            found.append(relative)
    return found


def cfg_mod_targets(rel: str) -> list[str]:
    text = (ROOT / rel).read_text(encoding="utf-8", errors="replace")
    targets: list[str] = []
    for attrs, name, path_attr in parse_mod_decls(text):
        if any(CFG_ATTR.search(attr) for attr in attrs):
            targets.extend(existing_mod_files(rel, name, path_attr))
    if re.search(
        r"#!\s*\[\s*cfg\(\s*(?:test\s*\)|(?:all|any)\((?:[^]\"']|\"[^\"]*\")*\btest\s*[,)])",
        text,
    ):
        # File-level `#![cfg(test)]` makes the whole module test-only.
        targets.append(rel)
    return targets


def module_dir(parent: Path) -> Path:
    if parent.name == "mod.rs":
        return parent.parent
    return parent.parent / parent.stem


def inherited_mod_targets(rel: str) -> list[str]:
    """Child modules of a test file. Paths that leave its module directory stay out.

    Integration tests sometimes `#[path]` back at production modules. Those files
    are not test slices. `#[cfg(test)]` modules are collected separately.
    """
    text = (ROOT / rel).read_text(encoding="utf-8", errors="replace")
    directory = module_dir(ROOT / rel).resolve()
    targets: list[str] = []
    for attrs, name, path_attr in parse_mod_decls(text):
        if any(NOT_TEST_ATTR.search(attr) for attr in attrs):
            continue
        for relative in existing_mod_files(rel, name, path_attr):
            try:
                (ROOT / relative).resolve().relative_to(directory)
            except ValueError:
                continue
            targets.append(relative)
    return targets


def include_targets(files: list[str]) -> list[tuple[str, str]]:
    interesting = [path for path in files if "include!(" in (ROOT / path).read_text(encoding="utf-8", errors="replace")]
    found: list[tuple[str, str]] = []
    for start in range(0, len(interesting), 100):
        batch = interesting[start : start + 100]
        rows = ast_grep(
            [
                "scan",
                "--json=stream",
                "--error",
                "--inline-rules",
                INCLUDE_RULE,
                *batch,
            ]
        )
        for row in rows:
            match = INCLUDE_TEXT.search(row.get("text", ""))
            if match:
                found.append((rel_path(row["file"]), match.group(1)))
    return found


def close_test_files(seeds: set[str]) -> set[str]:
    seen = set(seeds)
    queue = list(seeds)
    while queue:
        batch = queue[:100]
        queue = queue[100:]
        additions: list[str] = []
        for parent, include in include_targets(batch):
            candidate = (ROOT / parent).parent / include
            if not candidate.is_file():
                continue
            try:
                relative = candidate.resolve().relative_to(ROOT.resolve()).as_posix()
            except ValueError:
                continue
            if relative.endswith(".rs") and not is_sample(relative):
                additions.append(relative)
        for parent in batch:
            additions.extend(inherited_mod_targets(parent))
        for relative in additions:
            if relative in seen:
                continue
            seen.add(relative)
            queue.append(relative)
    return seen


def call_class(text: str) -> str:
    first = text.splitlines()[0] if text else ""
    if "create_dir_all" in first:
        return "dir"
    return "write"


def collect_findings() -> dict[str, set[str]]:
    rows = ast_grep(
        [
            "scan",
            "--json=stream",
            "--filter",
            r"^no-inline-test-fixture-write(-cfg)?$",
        ]
    )
    findings: dict[str, set[str]] = {}
    for row in rows:
        rule_id = row.get("ruleId")
        if rule_id not in (PATH_RULE, CFG_RULE):
            continue
        path = rel_path(row["file"])
        if is_sample(path):
            fail(f"fixture sample matched {rule_id}: {path}")
        findings.setdefault(path, set()).add(call_class(row.get("text", "")))

    rust = [path for path in tracked_rust() if not is_sample(path)]
    seeds = {path for path in rust if is_glob_test(path)}
    for path in rust:
        seeds.update(cfg_mod_targets(path))
    extra = sorted(close_test_files(seeds) - {path for path in rust if is_glob_test(path)})
    for start in range(0, len(extra), 100):
        batch = extra[start : start + 100]
        for row in ast_grep(
            [
                "scan",
                "--json=stream",
                "--error",
                "--rule",
                CALL_RULE,
                *batch,
            ]
        ):
            path = rel_path(row["file"])
            findings.setdefault(path, set()).add(call_class(row.get("text", "")))
    return findings


def parse_list(text: str) -> list[str]:
    paths = []
    for line_number, raw in enumerate(text.splitlines(), start=1):
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if "\t" in stripped:
            fail(f"baseline path contains a tab at line {line_number}")
        if stripped.startswith("/") or "\\" in stripped or stripped.startswith("../"):
            fail(f"baseline path must be repo-relative: {stripped}")
        paths.append(stripped)
    if len(paths) != len(set(paths)):
        fail("baseline lists a path more than once")
    return paths


def parse_allowlist(text: str) -> dict[str, tuple[str, str]]:
    entries: dict[str, tuple[str, str]] = {}
    for line_number, raw in enumerate(text.splitlines(), start=1):
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        parts = raw.split("\t")
        if len(parts) != 3:
            fail(
                f"allowlist line {line_number} must be "
                "path<TAB>copy|runtime<TAB>justification"
            )
        path, kind, justification = (part.strip() for part in parts)
        if not justification or "\n" in justification:
            fail(f"allowlist entry needs a one-line justification: {path or line_number}")
        if kind not in ("copy", "runtime"):
            fail(f"allowlist kind for {path} must be copy or runtime")
        if path.startswith("/") or "\\" in path or path.startswith("../"):
            fail(f"allowlist path must be repo-relative: {path}")
        if path in entries:
            fail(f"allowlist lists {path} more than once")
        entries[path] = (kind, justification)
    return entries


def shrink_errors(current: set[str], previous: set[str] | None, base: str) -> list[str]:
    if previous is None:
        return []
    if not previous:
        return [f"baseline migration is complete at {base}; must remain empty: {path}" for path in sorted(current)]
    return [f"baseline grew relative to {base}: {path}" for path in sorted(current - previous)]


def baseline_at(base: str) -> set[str] | None:
    verify = run(["git", "rev-parse", "--verify", "--quiet", f"{base}^{{commit}}"])
    if verify.returncode != 0:
        fail(f"base revision does not exist: {base}")
    show = run(["git", "show", f"{base}:.ast-grep/inline-fixture-write-baseline.txt"])
    if show.returncode != 0:
        stderr = show.stderr
        if "does not exist" in stderr or "exists on disk, but not" in stderr:
            return None
        fail(stderr.strip() or f"failed to read the baseline at {base}")
    return set(parse_list(show.stdout))


def check(base: str, print_findings: bool) -> None:
    findings = collect_findings()
    if print_findings:
        for path in sorted(findings):
            print(f"{path}\t{','.join(sorted(findings[path]))}")
        return
    if not ALLOWLIST.is_file() or not BASELINE.is_file():
        fail("allowlist and baseline files must both exist")
    allow = parse_allowlist(ALLOWLIST.read_text(encoding="utf-8"))
    baseline = set(parse_list(BASELINE.read_text(encoding="utf-8")))
    errors: list[str] = []
    for path, (kind, _justification) in sorted(allow.items()):
        classes = findings.get(path)
        if not classes:
            errors.append(f"stale allowlist entry: {path}")
            continue
        if path in baseline:
            errors.append(f"allowlist entry is also baselined: {path}")
        if kind == "copy" and classes != {"dir"}:
            errors.append(f"allowlisted copy helper has a content write: {path}")
    expected = set(findings) - set(allow)
    for path in sorted(expected - baseline):
        errors.append(f"new inline fixture write: {path}")
    for path in sorted(baseline - expected):
        errors.append(f"stale baseline entry: {path}")
    errors.extend(shrink_errors(baseline, baseline_at(base), base))
    if errors:
        fail("inline fixture-write check failed:\n" + "\n".join(errors))


def self_test() -> None:
    assert CFG_ATTR.search("#[cfg(test)]")
    assert CFG_ATTR.search("#[cfg(all(test, unix))]")
    assert CFG_ATTR.search('#[cfg(all(feature = "test-instrumentation", test))]')
    assert not CFG_ATTR.search("#[cfg(not(test))]")
    assert NOT_TEST_ATTR.search("#[cfg(not(test))]")
    assert not NOT_TEST_ATTR.search("#[cfg(test)]")
    decls = parse_mod_decls(
        "\n".join(
            [
                '#[path = "flow_query_tests/basics.rs"]',
                "mod basics;",
                "#[cfg(not(test))]",
                "mod hidden;",
                "#[cfg(test)]",
                "mod tests;",
            ]
        )
    )
    assert [name for _attrs, name, _path in decls] == ["basics", "hidden", "tests"]
    assert decls[0][2] == "flow_query_tests/basics.rs"
    assert module_dir(Path("/tmp/crate/src/flow_query_tests.rs")) == Path(
        "/tmp/crate/src/flow_query_tests"
    )
    inside = Path("/tmp/crate/src/flow_query_tests/basics.rs").resolve()
    outside = Path("/tmp/crate/src/invocation.rs").resolve()
    directory = module_dir(Path("/tmp/crate/src/flow_query_tests.rs")).resolve()
    inside.relative_to(directory)
    try:
        outside.relative_to(directory)
        raise AssertionError("production path should escape the test module directory")
    except ValueError:
        pass
    assert any(NOT_TEST_ATTR.search(attr) for attr in decls[1][0])
    assert not any(NOT_TEST_ATTR.search(attr) for attr in decls[0][0])
    assert not CFG_ATTR.search('#[cfg(all(feature = "test-instrumentation"))]')
    parent = Path("/tmp/crate/src/lock.rs")
    assert resolve_mod(parent, "tests", "lock/tests.rs") == [
        Path("/tmp/crate/src/lock/tests.rs")
    ]
    assert resolve_mod(parent, "tests2", None) == [
        Path("/tmp/crate/src/lock/tests2.rs"),
        Path("/tmp/crate/src/lock/tests2/mod.rs"),
    ]
    mod_parent = Path("/tmp/crate/src/foo/mod.rs")
    assert resolve_mod(mod_parent, "tests", None)[0] == Path("/tmp/crate/src/foo/tests.rs")
    assert is_glob_test("crates/no-mistakes/src/foo/tests.rs")
    assert is_glob_test("crates/no-mistakes/src/foo/tests/app.rs")
    assert is_glob_test("crates/no-mistakes/src/foo_tests.rs")
    assert not is_glob_test("crates/no-mistakes/src/foo/tests_planning/changed_files.rs")
    assert not is_glob_test("fixtures/demo/tests/app.rs")
    assert shrink_errors({"a.rs"}, None, "base") == []
    assert shrink_errors(set(), {"a.rs"}, "base") == []
    assert shrink_errors(set(), set(), "base") == []
    assert shrink_errors({"a.rs"}, set(), "base") == [
        "baseline migration is complete at base; must remain empty: a.rs"
    ]
    assert shrink_errors({"a.rs"}, {"a.rs", "b.rs"}, "base") == []
    assert shrink_errors({"b.rs"}, {"a.rs"}, "base") == [
        "baseline grew relative to base: b.rs"
    ]
    assert call_class("std::fs::create_dir_all(&target)") == "dir"
    assert call_class("std::fs::write(path, data)") == "write"
    assert parse_list("# header\n\na.rs\nb.rs\n") == ["a.rs", "b.rs"]
    assert parse_list("") == []
    allow = parse_allowlist("a.rs\tcopy\tCopies a saved fixture.\n")
    assert allow["a.rs"][0] == "copy"


def main() -> None:
    self_test()
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", required=True)
    parser.add_argument("--print-findings", action="store_true")
    args = parser.parse_args()
    check(args.base, args.print_findings)


if __name__ == "__main__":
    main()
