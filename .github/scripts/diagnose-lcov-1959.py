#!/usr/bin/env python3
"""Read-only intersection of LCOV zero-hit lines and changed production Rust lines."""
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import PurePosixPath

BASE = "d36ea2af42d7a2a6d20074e4cd8ac1d0bc89568a"
SOURCE = "0288f460c671d8e4db18dbb12d463234a31fd0ec"
SOURCE_TREE = "cb3ad202468dba164fb064854bfd2539db149f04"
MAX_LCOV_BYTES = 100_000_000
MAX_CHANGED_FILES = 400
MAX_OUTPUT_BYTES = 24_000


def git(*args):
    return subprocess.check_output(["git", *args], text=True, stderr=subprocess.STDOUT).strip()


def production_path(path):
    p = PurePosixPath(path)
    return (p.as_posix().startswith("crates/no-mistakes/src/")
            and p.suffix == ".rs"
            and "tests" not in p.parts
            and p.name != "tests.rs"
            and not p.name.endswith("_tests.rs"))


def canonical_sf(sf):
    # LLVM may emit an absolute runner path; retain only the repository-relative suffix.
    marker = "crates/no-mistakes/src/"
    pos = sf.replace("\\", "/").find(marker)
    return sf.replace("\\", "/")[pos:] if pos >= 0 else sf.replace("\\", "/")


def parse_lcov(path):
    if os.path.getsize(path) > MAX_LCOV_BYTES:
        raise SystemExit("LCOV exceeds 100 MB guard")
    files = {}
    current = None
    with open(path, encoding="utf-8", errors="strict") as stream:
        for raw in stream:
            line = raw.rstrip("\n")
            if line.startswith("SF:"):
                current = canonical_sf(line[3:])
                files.setdefault(current, {})
            elif line.startswith("DA:") and current:
                fields = line[3:].split(",")
                if len(fields) >= 2 and fields[0].isdigit() and fields[1].isdigit():
                    line_number, hits = int(fields[0]), int(fields[1])
                    files[current][line_number] = files[current].get(line_number, 0) + hits
            elif line == "end_of_record":
                current = None
    return files


def added_lines(base, source):
    raw = subprocess.check_output(["git", "diff", "--no-ext-diff", "--no-renames", "--unified=0",
                                   base, source, "--", "crates/no-mistakes/src"], text=True)
    result = {}
    path = None
    for line in raw.splitlines():
        if line.startswith("+++ b/"):
            path = line[6:]
            if not production_path(path):
                path = None
        elif line.startswith("@@") and path:
            m = re.match(r"@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", line)
            if m:
                start = int(m.group(1)); count = int(m.group(2) or "1")
                result.setdefault(path, set()).update(range(start, start + count))
    return result


def ranges(lines):
    out = []
    for n in sorted(lines):
        if out and n == out[-1][1] + 1:
            out[-1][1] = n
        else:
            out.append([n, n])
    return out


def main():
    os.chdir(os.environ["SOURCE_DIR"])
    lcov = sys.argv[1]
    if git("rev-parse", "HEAD") != SOURCE or git("rev-parse", "HEAD^{tree}") != SOURCE_TREE:
        raise SystemExit("checked-out measured source SHA/tree mismatch")
    git("cat-file", "-e", f"{BASE}^{{commit}}")
    changed = added_lines(BASE, SOURCE)
    if len(changed) > MAX_CHANGED_FILES:
        raise SystemExit("changed-file count exceeds guard")
    coverage = parse_lcov(lcov)
    rows = []
    for path, lines in sorted(changed.items()):
        file_coverage = coverage.get(path, {})
        uncovered = {line for line in lines if line in file_coverage and file_coverage[line] == 0}
        if uncovered:
            rows.append({"path": path, "changed_lines": len(lines),
                         "zero_hit_changed_lines": len(uncovered), "lines": ranges(uncovered)})
    digest = hashlib.sha256()
    with open(lcov, "rb") as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b""):
            digest.update(chunk)
    allowlist_bytes = "\n".join(sorted(changed)).encode()
    report = {
        "base_sha": BASE,
        "measured_source_sha": SOURCE,
        "measured_source_tree": SOURCE_TREE,
        "lcov_sha256": digest.hexdigest(),
        "changed_production_files": len(changed),
        "production_file_allowlist_sha256": hashlib.sha256(allowlist_bytes).hexdigest(),
        "files_with_zero_hit_changed_lines": rows,
        "zero_hit_changed_line_count": sum(row["zero_hit_changed_lines"] for row in rows),
        "note": "Line mapping uses LCOV from the compiled synthetic source and added-line hunks from base-to-measured-source git diff; no source or test code was executed."
    }
    encoded = json.dumps(report, sort_keys=True, separators=(",", ":"))
    if len(encoded.encode()) > MAX_OUTPUT_BYTES:
        raise SystemExit("curated report exceeds output-size guard")
    print(encoded)
    summary = ["## Rust changed-line coverage diagnostic", "",
               f"Base: `{BASE}`  ", f"Measured compiled source: `{SOURCE}` (tree `{SOURCE_TREE}`)  ",
               f"Changed production files: {len(changed)}  ",
               f"Changed lines with zero LCOV hits: {report['zero_hit_changed_line_count']}", ""]
    for row in rows:
        summary.append(f"- `{row['path']}`: {row['zero_hit_changed_lines']} lines, ranges `{row['lines']}`")
    summary.append("\nRead-only report: LCOV was parsed and intersected with changed production line hunks; no build, test, source execution, raw LCOV upload, or source mutation.")
    with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as out:
        out.write("\n".join(summary) + "\n")


if __name__ == "__main__":
    main()
