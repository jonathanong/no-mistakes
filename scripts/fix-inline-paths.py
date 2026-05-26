#!/usr/bin/env python3
"""Fix inline PathBuf paths that are missing .join("fixture") after the sub-fixture name."""

import re
from pathlib import Path

WORKTREE = Path(__file__).parent.parent


def fix_inline_codebase(path: Path) -> bool:
    """
    Fix inline paths of the form:
        .join("../../test-cases/codebase-analysis")
        .join("FIXTURE_NAME");
    by adding .join("fixture") between them (but only where not already present).
    """
    content = path.read_text()
    original = content

    # Pattern: .join("../../test-cases/codebase-analysis")\n        .join("NAME");
    # or:      .join("../../test-cases/codebase-analysis")\n        .join("NAME")\n        .join(...)
    pattern = r'(\.join\("../../test-cases/codebase-analysis"\)\s*\n\s*\.join\("[^"]+"\))(\s*;|\s*\n)'

    def replace_fn(m):
        inner = m.group(1)
        after = m.group(2)
        # Check if already followed by .join("fixture")
        pos = m.end()
        rest = content[pos:]
        if rest.lstrip().startswith('.join("fixture")'):
            return m.group(0)
        return inner + '\n        .join("fixture")' + after

    content = re.sub(pattern, replace_fn, content)

    if content != original:
        path.write_text(content)
        print(f"Fixed: {path.relative_to(WORKTREE)}")
        return True
    return False


def main():
    crates = WORKTREE / "crates/no-mistakes"

    # Fix execution.rs inline paths
    execution = crates / "src/codebase/dependencies/tests/execution.rs"
    content = execution.read_text()
    fixes = [
        ('.join("simple");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)',
         '.join("simple")\n        .join("fixture");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)'),
        ('.join("format-output");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)',
         '.join("format-output")\n        .join("fixture");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)'),
        ('.join("test-framework");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)',
         '.join("test-framework")\n        .join("fixture");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)'),
        ('.join("filter");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)',
         '.join("filter")\n        .join("fixture");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)'),
        ('.join("symbol-export");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)',
         '.join("symbol-export")\n        .join("fixture");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)'),
        ('.join("folder-suffix");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)',
         '.join("folder-suffix")\n        .join("fixture");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)'),
    ]
    original = content
    for old, new in fixes:
        content = content.replace(old, new)
    if content != original:
        execution.write_text(content)
        print(f"Fixed: {execution.relative_to(WORKTREE)}")

    # Fix extra.rs
    extra = crates / "src/codebase/dependencies/tests/extra.rs"
    content = extra.read_text()
    # Check what patterns exist
    if '.join("../../test-cases/codebase-analysis")' in content:
        # Read and check what follows
        lines = content.split('\n')
        new_lines = []
        i = 0
        while i < len(lines):
            line = lines[i]
            if '.join("../../test-cases/codebase-analysis")' in line:
                new_lines.append(line)
                i += 1
                # Check if next non-whitespace line has .join("...") followed by ;
                if i < len(lines):
                    next_line = lines[i]
                    if '.join("' in next_line and not '.join("fixture")' in next_line:
                        new_lines.append(next_line)
                        i += 1
                        # Check if NEXT next line is NOT .join("fixture")
                        if i < len(lines) and '.join("fixture")' not in lines[i]:
                            # Add fixture join
                            indent = '        '
                            new_lines.append(f'{indent}.join("fixture")')
                    continue
            new_lines.append(line)
            i += 1
        new_content = '\n'.join(new_lines)
        if new_content != content:
            extra.write_text(new_content)
            print(f"Fixed: {extra.relative_to(WORKTREE)}")

    # Fix extra_cases.rs: only specific inline paths (not the corpus root)
    extra_cases = crates / "src/codebase/dependencies/graph/tests/extra_cases.rs"
    content = extra_cases.read_text()
    fixes = [
        # test-framework line 469-473: add fixture between test-framework and src usage
        ('.join("test-framework")\n        .join("src");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)',
         '.join("test-framework")\n        .join("fixture");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)'),
        # codebase-intel line 514-516: add fixture at end
        ('.join("codebase-intel");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)',
         '.join("codebase-intel")\n        .join("fixture");\n    let root = crate::codebase::ts_resolver::normalize_path(&root)'),
    ]
    original = content
    for old, new in fixes:
        content = content.replace(old, new)
    if content != original:
        extra_cases.write_text(content)
        print(f"Fixed: {extra_cases.relative_to(WORKTREE)}")

    # Check if test-framework was split across 3 lines (src on separate line)
    # Re-read to verify
    content = extra_cases.read_text()
    if '.join("test-framework")\n        .join("src");' in content:
        # fixture wasn't inserted — the 3-line form exists
        content = content.replace(
            '.join("test-framework")\n        .join("src");\n',
            '.join("test-framework")\n        .join("fixture");\n    // root.join("src") is now relative to fixture\n',
        )
        extra_cases.write_text(content)
        print(f"Fixed (test-framework): {extra_cases.relative_to(WORKTREE)}")

    # Fix defs_frontend/tests.rs inline path
    defs_frontend = crates / "src/codebase/ts_routes/defs_frontend/tests.rs"
    content = defs_frontend.read_text()
    old = '''\
        .join("../../test-cases/codebase-analysis")
        .join("routes")
        .join("good")
        .join("web")
        .join("app");'''
    new = '''\
        .join("../../test-cases/codebase-analysis")
        .join("routes")
        .join("fixture")
        .join("good")
        .join("web")
        .join("app");'''
    if old in content:
        content = content.replace(old, new)
        defs_frontend.write_text(content)
        print(f"Fixed: {defs_frontend.relative_to(WORKTREE)}")

    print("Done.")


if __name__ == "__main__":
    main()
