#!/usr/bin/env python3
"""
Migrate fixtures/ → test-cases/

Rules:
- Most categories (Type A): each sub-dir becomes test-cases/{cat}/{sub}/fixture/
- eslint-snippets (Type B): flat files → test-cases/eslint-snippets/fixture/
- codebase-analysis has a top-level README.md → kept as test-cases/codebase-analysis/README.md

Usage:
    python3 scripts/migrate-fixtures.py [--dry-run]
"""

import os
import subprocess
import sys

DRY_RUN = "--dry-run" in sys.argv

TYPE_B_CATEGORIES = {"eslint-snippets"}


def run(cmd):
    if DRY_RUN:
        print("  git mv " + " ".join(str(c) for c in cmd[2:]))
        return
    result = subprocess.run(cmd, check=True, capture_output=True, text=True)
    if result.stderr:
        print(result.stderr, file=sys.stderr)


def mkdir(path):
    os.makedirs(path, exist_ok=True)


def touch(path):
    parent = os.path.dirname(path)
    os.makedirs(parent, exist_ok=True)
    if not os.path.exists(path):
        open(path, "w").close()


def git_mv_children(src_dir, dest_dir):
    """git mv every child of src_dir into dest_dir."""
    mkdir(dest_dir)
    for child in sorted(os.listdir(src_dir)):
        src = os.path.join(src_dir, child)
        dest = os.path.join(dest_dir, child)
        run(["git", "mv", src, dest])


def migrate():
    fixtures_dir = "fixtures"
    test_cases_dir = "test-cases"

    if not DRY_RUN:
        mkdir(test_cases_dir)

    categories = sorted(
        c for c in os.listdir(fixtures_dir)
        if os.path.isdir(os.path.join(fixtures_dir, c))
    )

    total_moves = 0

    for cat in categories:
        cat_src = os.path.join(fixtures_dir, cat)
        entries = os.listdir(cat_src)
        top_files = [e for e in entries if os.path.isfile(os.path.join(cat_src, e))]
        subdirs = sorted(e for e in entries if os.path.isdir(os.path.join(cat_src, e)))

        if DRY_RUN:
            print(f"\n[{cat}] {len(subdirs)} subdirs, {len(top_files)} top-level files")

        if cat in TYPE_B_CATEGORIES:
            # Entire category is one test-case
            dest_fixture = os.path.join(test_cases_dir, cat, "fixture")
            if DRY_RUN:
                count = sum(
                    1 for f in top_files + subdirs
                )
                print(f"  → test-cases/{cat}/fixture/ ({count} items)")
                touch(os.path.join(test_cases_dir, cat, "README.md"))
            else:
                git_mv_children(cat_src, dest_fixture)
                mkdir(os.path.join(test_cases_dir, cat, "snapshots"))
                touch(os.path.join(test_cases_dir, cat, "README.md"))
            total_moves += len(entries)
        else:
            # Type A: each sub-dir is a test-case
            # Handle top-level README.md (only in codebase-analysis)
            for f in top_files:
                if f == "README.md":
                    dest = os.path.join(test_cases_dir, cat, "README.md")
                    if DRY_RUN:
                        print(f"  top-level README.md → test-cases/{cat}/README.md")
                    else:
                        mkdir(os.path.join(test_cases_dir, cat))
                        run(["git", "mv", os.path.join(cat_src, f), dest])
                    total_moves += 1

            for sub in subdirs:
                sub_src = os.path.join(cat_src, sub)
                dest_fixture = os.path.join(test_cases_dir, cat, sub, "fixture")
                if DRY_RUN:
                    n = len(os.listdir(sub_src))
                    print(f"  {sub}/ ({n} items) → test-cases/{cat}/{sub}/fixture/")
                else:
                    git_mv_children(sub_src, dest_fixture)
                    mkdir(os.path.join(test_cases_dir, cat, sub, "snapshots"))
                    touch(os.path.join(test_cases_dir, cat, sub, "README.md"))
                total_moves += 1  # count sub-fixtures, not individual files

    if DRY_RUN:
        print(f"\n=== Total: {total_moves} sub-fixture moves across {len(categories)} categories ===")
        return

    # Remove now-empty fixture subdirs and root
    result = subprocess.run(
        ["find", fixtures_dir, "-depth", "-type", "d"],
        capture_output=True, text=True
    )
    for d in result.stdout.strip().split("\n"):
        if d and os.path.isdir(d) and not os.listdir(d):
            os.rmdir(d)
    if os.path.exists(fixtures_dir) and not os.listdir(fixtures_dir):
        os.rmdir(fixtures_dir)

    print("Migration complete.")


if __name__ == "__main__":
    if DRY_RUN:
        print("=== DRY RUN ===")
    migrate()
