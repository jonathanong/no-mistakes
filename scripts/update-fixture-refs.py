#!/usr/bin/env python3
"""
Update all code references from fixtures/ to test-cases/

Transformation rules:
1. "../../fixtures/{cat}/{sub}[/rest]" -> "../../test-cases/{cat}/{sub}/fixture[/rest]"
2. "../../fixtures/{cat}" (no sub) -> "../../test-cases/{cat}"  (helper fns need .join("fixture") added separately)
3. "../../fixtures" -> "../../test-cases"  (two-arg helpers)
4. "fixtures/{cat}/{sub}[/rest]" -> "test-cases/{cat}/{sub}/fixture[/rest]"  (relative CLI args)
5. config file "fixtures/" -> "test-cases/"

Run with --dry-run to see changes without writing.
"""

import os
import re
import sys

DRY_RUN = "--dry-run" in sys.argv

KNOWN_CATS = {
    "ast-snippets", "check-discovery", "check-runner", "codebase-analysis",
    "config-v2", "eslint-plugin", "eslint-snippets", "integration-tests",
    "next-to-fetch-routes", "nextjs-coverage", "nextjs-fetches", "nextjs-html-ids",
    "nextjs-rewrites", "nextjs-routes", "nextjs-selectors", "nextjs-test-ids",
    "no-mistakes-core-imports", "no-mistakes-proxy", "playwright-configs",
    "playwright-tests", "queue-ast-hop", "react-traits-analyze", "react-traits-components",
    "react-traits-config", "react-traits-fetch", "react-traits-glob", "rules",
    "scan-config", "server-ast-routes",
}

# Pattern: fixtures/ path with relative prefix (../../) or without
# Group 1: optional relative prefix ("../../" or "")
# Group 2: category name
# Group 3: optional "/sub[/rest]" after category
CAT_RE = re.compile(
    r'((?:\.\./\.\./)?)'
    r'fixtures/'
    r'(' + '|'.join(re.escape(c) for c in KNOWN_CATS) + r')'
    r'((?:/[^\s"\')\n]*)?)'
)

# Match "../../fixtures" alone (no category - two-arg helpers)
BARE_FIXTURES_RE = re.compile(r'(../../)fixtures(?!/)(?=["\'])')


def transform_match(m):
    prefix = m.group(1)   # "../../" or ""
    cat = m.group(2)
    after = m.group(3)    # "/sub[/rest]" or ""

    if not prefix and not after:
        # "fixtures/{cat}" with no prefix and no sub in a config file context
        # e.g., .no-mistakes.yml "fixtures/" exclude entries
        return f"test-cases/{cat}"

    if not after:
        # "../../fixtures/{cat}" (no sub) → "../../test-cases/{cat}"
        # Helper functions that add .join(name).join("fixture") separately
        return f"{prefix}test-cases/{cat}"

    # after starts with "/" - split into sub + maybe rest
    after_stripped = after.lstrip("/")
    slash_idx = after_stripped.find("/")

    if slash_idx == -1:
        # "../../fixtures/{cat}/{sub}" → "../../test-cases/{cat}/{sub}/fixture"
        sub = after_stripped
        return f"{prefix}test-cases/{cat}/{sub}/fixture"
    else:
        # "../../fixtures/{cat}/{sub}/{rest}" → "../../test-cases/{cat}/{sub}/fixture/{rest}"
        sub = after_stripped[:slash_idx]
        rest = after_stripped[slash_idx + 1:]
        return f"{prefix}test-cases/{cat}/{sub}/fixture/{rest}"


def transform_line(line):
    # Replace ../../fixtures (bare, for two-arg helpers like .join("../../fixtures"))
    line = BARE_FIXTURES_RE.sub(r'\1test-cases', line)
    # Replace fixtures/{cat}[/sub[/rest]] references
    line = CAT_RE.sub(transform_match, line)
    return line


def process_file(path, special=False):
    with open(path) as f:
        original = f.read()

    lines = original.split("\n")
    new_lines = []
    changed = False

    for line in lines:
        new_line = transform_line(line)
        if new_line != line:
            changed = True
        new_lines.append(new_line)

    if changed:
        new_content = "\n".join(new_lines)
        if DRY_RUN:
            print(f"\n=== {path} ===")
            for i, (old, new) in enumerate(zip(lines, new_lines)):
                if old != new:
                    print(f"  - {old.strip()}")
                    print(f"  + {new.strip()}")
        else:
            with open(path, "w") as f:
                f.write(new_content)
            print(f"Updated: {path}")
    return changed


# Files to process
RUST_FILES = []
for dirpath, dirnames, filenames in os.walk("crates/no-mistakes"):
    dirnames[:] = [d for d in dirnames if d not in ("target",)]
    for fn in filenames:
        if fn.endswith(".rs"):
            RUST_FILES.append(os.path.join(dirpath, fn))

JS_FILES = []
for dirpath, dirnames, filenames in os.walk("packages"):
    dirnames[:] = [d for d in dirnames if d not in ("node_modules",)]
    for fn in filenames:
        if fn.endswith((".mjs", ".js", ".ts")):
            JS_FILES.append(os.path.join(dirpath, fn))

CONFIG_FILES = [
    ".no-mistakes.yml",
    ".codacy.yml",
    ".oxlintrc.json",
    "cspell.config.yaml",
    ".github/workflows/ci.yml",
]

all_files = RUST_FILES + JS_FILES + CONFIG_FILES
total_changed = 0
for path in sorted(all_files):
    if os.path.exists(path):
        if process_file(path):
            total_changed += 1

print(f"\n{'[DRY RUN] ' if DRY_RUN else ''}Updated {total_changed} files.")
