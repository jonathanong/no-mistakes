#!/usr/bin/env python3
"""
Fix remaining fixture path issues after migration to test-cases/[name]/fixture/ structure.

This script handles several patterns:
1. Helper functions that end with .join(name) missing .join("fixture")
2. inline direct paths that are missing "fixture/" component
3. Special cases like v2_config_fixture and cli_check_rules fixture(category, scenario)
"""

import re
import sys
from pathlib import Path

WORKTREE = Path(__file__).parent.parent

def fix_file(path: Path, fixes: list[tuple[str, str]]) -> bool:
    """Apply string substitutions to a file. Returns True if changed."""
    content = path.read_text()
    original = content
    for old, new in fixes:
        content = content.replace(old, new)
    if content != original:
        path.write_text(content)
        print(f"Fixed: {path.relative_to(WORKTREE)}")
        return True
    return False


def fix_simple_fixture_fn(path: Path, category_path: str, var_name: str = "name") -> bool:
    """
    Fix a helper function that ends with .join(CATEGORY).join(var_name)
    by adding .join("fixture") after .join(var_name).

    Handles both with and without normalize_path wrapping.
    """
    fixes = [
        (
            f'.join("{category_path}")\n            .join({var_name}),\n    )',
            f'.join("{category_path}")\n            .join({var_name})\n            .join("fixture"),\n    )',
        ),
        (
            f'.join("{category_path}")\n        .join({var_name})\n}}',
            f'.join("{category_path}")\n        .join({var_name})\n        .join("fixture")\n}}',
        ),
        (
            f'.join("{category_path}")\n            .join({var_name})\n    }}\n}}',
            f'.join("{category_path}")\n            .join({var_name})\n            .join("fixture")\n    }}\n}}',
        ),
    ]
    return fix_file(path, fixes)


def main():
    crates = WORKTREE / "crates/no-mistakes"

    # ── tests/ files ──────────────────────────────────────────────────────────

    # tests/cli.rs: fixture(name), react_fixture(category, name),
    #               queue_fixture(name), server_fixture(name)
    cli = crates / "tests/cli.rs"
    fix_file(cli, [
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
        (
            '.join("../../test-cases")\n            .join(category)\n            .join(name),',
            '.join("../../test-cases")\n            .join(category)\n            .join(name)\n            .join("fixture"),',
        ),
        (
            '.join("../../test-cases/queue-ast-hop")\n            .join(name),',
            '.join("../../test-cases/queue-ast-hop")\n            .join(name)\n            .join("fixture"),',
        ),
        (
            '.join("../../test-cases/server-ast-routes")\n            .join(name),',
            '.join("../../test-cases/server-ast-routes")\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # tests/cli_format.rs
    clf = crates / "tests/cli_format.rs"
    fix_file(clf, [
        (
            '.join("../../test-cases/queue-ast-hop")\n            .join(name),',
            '.join("../../test-cases/queue-ast-hop")\n            .join(name)\n            .join("fixture"),',
        ),
        (
            '.join("../../test-cases/server-ast-routes")\n            .join(name),',
            '.join("../../test-cases/server-ast-routes")\n            .join(name)\n            .join("fixture"),',
        ),
        (
            '.join("../../test-cases")\n            .join(category)\n            .join(name),',
            '.join("../../test-cases")\n            .join(category)\n            .join(name)\n            .join("fixture"),',
        ),
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # tests/cli_format2.rs (same patterns as cli_format.rs)
    clf2 = crates / "tests/cli_format2.rs"
    fix_file(clf2, [
        (
            '.join("../../test-cases/queue-ast-hop")\n            .join(name),',
            '.join("../../test-cases/queue-ast-hop")\n            .join(name)\n            .join("fixture"),',
        ),
        (
            '.join("../../test-cases/server-ast-routes")\n            .join(name),',
            '.join("../../test-cases/server-ast-routes")\n            .join(name)\n            .join("fixture"),',
        ),
        (
            '.join("../../test-cases")\n            .join(category)\n            .join(name),',
            '.join("../../test-cases")\n            .join(category)\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # tests/cli_extra.rs: fixture(category, name)
    cle = crates / "tests/cli_extra.rs"
    fix_file(cle, [
        (
            '.join("../../test-cases")\n            .join(category)\n            .join(name),',
            '.join("../../test-cases")\n            .join(category)\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # tests/cli_extra2.rs: fixture(category, name) with potential sub-path in name
    # Change to split-once approach to handle "ts-process-spawn/project"
    cle2 = crates / "tests/cli_extra2.rs"
    content = cle2.read_text()
    old_fn = '''\
fn fixture(category: &str, name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases")
            .join(category)
            .join(name),
    )
}'''
    new_fn = '''\
fn fixture(category: &str, name: &str) -> PathBuf {
    let (sub, rest) = name.split_once('/').unwrap_or((name, ""));
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(sub)
        .join("fixture");
    if !rest.is_empty() {
        path = path.join(rest);
    }
    no_mistakes::codebase::ts_resolver::normalize_path(&path)
}'''
    if old_fn in content:
        content = content.replace(old_fn, new_fn)
        cle2.write_text(content)
        print(f"Fixed: {cle2.relative_to(WORKTREE)}")

    # tests/shared_facts.rs: codebase_fixture(name), queue_fixture(name)
    sf = crates / "tests/shared_facts.rs"
    fix_file(sf, [
        (
            '.join("../../test-cases/codebase-analysis")\n        .join(name)\n}',
            '.join("../../test-cases/codebase-analysis")\n        .join(name)\n        .join("fixture")\n}',
        ),
        (
            '.join("../../test-cases/queue-ast-hop")\n        .join(name)\n}',
            '.join("../../test-cases/queue-ast-hop")\n        .join(name)\n        .join("fixture")\n}',
        ),
    ])

    # tests/common/mod.rs: pub fn fixture(name)
    cm = crates / "tests/common/mod.rs"
    fix_file(cm, [
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # tests/cli_codebase_acceptance/common.rs: pub fn fixture(name)
    cca = crates / "tests/cli_codebase_acceptance/common.rs"
    fix_file(cca, [
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # tests/cli_unique_exports.rs: fn fixture(name)
    cue = crates / "tests/cli_unique_exports.rs"
    fix_file(cue, [
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # tests/cli_check_rules.rs: fixture(category, scenario) and codebase_fixture(scenario)
    # fixture: rules/{category}/fixture/{scenario}
    # codebase_fixture: codebase-analysis/{scenario}/fixture
    ccr = crates / "tests/cli_check_rules.rs"
    fix_file(ccr, [
        (
            '.join("../../test-cases/rules")\n            .join(category)\n            .join(scenario),',
            '.join("../../test-cases/rules")\n            .join(category)\n            .join("fixture")\n            .join(scenario),',
        ),
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(scenario),',
            '.join("../../test-cases/codebase-analysis")\n            .join(scenario)\n            .join("fixture"),',
        ),
    ])

    # tests/cli_tests_impact.rs: fn fixture(name)
    cti = crates / "tests/cli_tests_impact.rs"
    fix_file(cti, [
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # ── src/ files ────────────────────────────────────────────────────────────

    # src/integration_tests/tests/*.rs — fn fixture(name)
    for p in [
        "src/integration_tests/tests/config_parsers.rs",
        "src/integration_tests/tests.rs",
        "src/integration_tests/tests_resolution.rs",
        "src/integration_tests/tests_errors.rs",
    ]:
        fix_file(crates / p, [
            (
                '.join("../../test-cases/integration-tests")\n            .join(name),',
                '.join("../../test-cases/integration-tests")\n            .join(name)\n            .join("fixture"),',
            ),
        ])

    # src/config/v2/tests.rs — fn fixture(sub)
    cv2 = crates / "src/config/v2/tests.rs"
    fix_file(cv2, [
        (
            '.join("test-cases/config-v2")\n        .join(sub)\n}',
            '.join("test-cases/config-v2")\n        .join(sub)\n        .join("fixture")\n}',
        ),
    ])

    # src/codebase/config/tests.rs — v2_config_fixture inserts "fixture" before ".no-mistakes.yml"
    cct = crates / "src/codebase/config/tests.rs"
    fix_file(cct, [
        (
            '.join("../../test-cases/config-v2")\n            .join(name)\n            .join(".no-mistakes.yml"),',
            '.join("../../test-cases/config-v2")\n            .join(name)\n            .join("fixture")\n            .join(".no-mistakes.yml"),',
        ),
    ])

    # src/imports/tests.rs
    it = crates / "src/imports/tests.rs"
    fix_file(it, [
        (
            '.join("../../test-cases")\n        .join(category)\n        .join(name)\n}',
            '.join("../../test-cases")\n        .join(category)\n        .join(name)\n        .join("fixture")\n}',
        ),
    ])

    # src/fetches/tests/ files
    for p in [
        "src/fetches/tests/run_with_base_root_tests.rs",
        "src/fetches/tests/target_tests.rs",
        "src/fetches/tests/run_args_tests.rs",
    ]:
        fix_file(crates / p, [
            (
                '.join("../../test-cases")\n        .join(category)\n        .join(name)\n}',
                '.join("../../test-cases")\n        .join(category)\n        .join(name)\n        .join("fixture")\n}',
            ),
        ])

    # src/fetches/tests/metadata_context_tests.rs
    fmc = crates / "src/fetches/tests/metadata_context_tests.rs"
    fix_file(fmc, [
        (
            '.join("../../test-cases/nextjs-fetches")\n        .join(name)\n}',
            '.join("../../test-cases/nextjs-fetches")\n        .join(name)\n        .join("fixture")\n}',
        ),
    ])

    # src/react_traits/ test files
    for p in [
        "src/react_traits/pipeline/run_with_facts/tests.rs",
        "src/react_traits/pipeline/run/tests.rs",
    ]:
        fix_file(crates / p, [
            (
                '.join("../../test-cases/react-traits-components")\n        .join(name)\n}',
                '.join("../../test-cases/react-traits-components")\n        .join(name)\n        .join("fixture")\n}',
            ),
        ])

    for p in [
        "src/react_traits/traits/fetch/tests.rs",
        "src/react_traits/analyze/file/tests.rs",
    ]:
        fix_file(crates / p, [
            (
                '.join("../../test-cases")\n        .join(category)\n        .join(name)\n}',
                '.join("../../test-cases")\n        .join(category)\n        .join(name)\n        .join("fixture")\n}',
            ),
        ])

    # src/napi_api/tests.rs — two patterns
    nat = crates / "src/napi_api/tests.rs"
    fix_file(nat, [
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
        (
            '.join("../../test-cases")\n            .join(category)\n            .join(name),',
            '.join("../../test-cases")\n            .join(category)\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # src/queue/tests.rs
    qt = crates / "src/queue/tests.rs"
    fix_file(qt, [
        (
            '.join("../../test-cases/queue-ast-hop")\n        .join(name)\n}',
            '.join("../../test-cases/queue-ast-hop")\n        .join(name)\n        .join("fixture")\n}',
        ),
    ])

    # src/codebase/unique_exports/tests.rs
    cuet = crates / "src/codebase/unique_exports/tests.rs"
    fix_file(cuet, [
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # src/codebase/ts_routes/defs_frontend/tests.rs
    dtf = crates / "src/codebase/ts_routes/defs_frontend/tests.rs"
    fix_file(dtf, [
        (
            '.join("../../test-cases/codebase-analysis")\n        .join(name)\n}',
            '.join("../../test-cases/codebase-analysis")\n        .join(name)\n        .join("fixture")\n}',
        ),
    ])

    # src/codebase/ts_routes/defs_backend/tests.rs — inline path
    dbt = crates / "src/codebase/ts_routes/defs_backend/tests.rs"
    fix_file(dbt, [
        (
            'let source = root.join("ts-routes/backend-walk-all.ts");',
            'let source = root.join("ts-routes/fixture/backend-walk-all.ts");',
        ),
        (
            'let unmatched = root.join("server-routes/default-function.ts");',
            'let unmatched = root.join("server-routes/fixture/default-function.ts");',
        ),
    ])

    # src/codebase/dependencies/graph/tests/core.rs — multiple inline paths
    dgc = crates / "src/codebase/dependencies/graph/tests/core.rs"
    content = dgc.read_text()
    # These are all PathBuf ending with .join("../../test-cases/codebase-analysis").join(name) forms
    # The function is fn fixture(name: &str) -> PathBuf
    fixed = content.replace(
        '.join("../../test-cases/codebase-analysis")\n        .join(name)\n}',
        '.join("../../test-cases/codebase-analysis")\n        .join(name)\n        .join("fixture")\n}',
    )
    if fixed != content:
        dgc.write_text(fixed)
        print(f"Fixed: {dgc.relative_to(WORKTREE)}")

    # src/codebase/dependencies/graph/tests/extra_cases.rs
    dge = crates / "src/codebase/dependencies/graph/tests/extra_cases.rs"
    content = dge.read_text()
    # First check for helper function pattern
    fixed = content.replace(
        '.join("../../test-cases/codebase-analysis")\n        .join(name)\n}',
        '.join("../../test-cases/codebase-analysis")\n        .join(name)\n        .join("fixture")\n}',
    )
    if fixed != content:
        dge.write_text(fixed)
        print(f"Fixed: {dge.relative_to(WORKTREE)}")

    # src/codebase/dependencies/tests/execution.rs
    dte = crates / "src/codebase/dependencies/tests/execution.rs"
    content = dte.read_text()
    fixed = content.replace(
        '.join("../../test-cases/codebase-analysis")\n        .join(name)\n}',
        '.join("../../test-cases/codebase-analysis")\n        .join(name)\n        .join("fixture")\n}',
    )
    if fixed != content:
        dte.write_text(fixed)
        print(f"Fixed: {dte.relative_to(WORKTREE)}")

    # src/codebase/dependencies/tests/extra.rs
    dtx = crates / "src/codebase/dependencies/tests/extra.rs"
    content = dtx.read_text()
    fixed = content.replace(
        '.join("../../test-cases/codebase-analysis")\n        .join(name)\n}',
        '.join("../../test-cases/codebase-analysis")\n        .join(name)\n        .join("fixture")\n}',
    )
    if fixed != content:
        dtx.write_text(fixed)
        print(f"Fixed: {dtx.relative_to(WORKTREE)}")

    # src/codebase/dependencies/tests/args.rs
    dta = crates / "src/codebase/dependencies/tests/args.rs"
    fix_file(dta, [
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # src/codebase/playwright_coverage/tests.rs
    pct = crates / "src/codebase/playwright_coverage/tests.rs"
    fix_file(pct, [
        (
            '.join("../../test-cases/codebase-analysis")\n            .join(name),',
            '.join("../../test-cases/codebase-analysis")\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # src/codebase/rules/tests.rs
    crt = crates / "src/codebase/rules/tests.rs"
    fix_file(crt, [
        (
            '.join("../../test-cases")\n            .join(category)\n            .join(name),',
            '.join("../../test-cases")\n            .join(category)\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # src/codebase/rules/strict_package_layout/tests.rs
    splt = crates / "src/codebase/rules/strict_package_layout/tests.rs"
    fix_file(splt, [
        (
            '.join("../../test-cases")\n            .join(category)\n            .join(name),',
            '.join("../../test-cases")\n            .join(category)\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # src/codebase/rules/required_local_docs/tests.rs
    rldt = crates / "src/codebase/rules/required_local_docs/tests.rs"
    fix_file(rldt, [
        (
            '.join("../../test-cases")\n            .join(category)\n            .join(name),',
            '.join("../../test-cases")\n            .join(category)\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # src/codebase/rules/test_no_unmocked_dynamic_imports/config/tests/rule_targets.rs
    tnudt = crates / "src/codebase/rules/test_no_unmocked_dynamic_imports/config/tests/rule_targets.rs"
    fix_file(tnudt, [
        (
            '.join("../../test-cases")\n            .join(category)\n            .join(name),',
            '.join("../../test-cases")\n            .join(category)\n            .join(name)\n            .join("fixture"),',
        ),
    ])

    # src/codebase/rules/forbidden_dependencies/tests.rs
    fdt = crates / "src/codebase/rules/forbidden_dependencies/tests.rs"
    fix_file(fdt, [
        (
            '.join("../../test-cases/codebase-analysis")\n        .join(name)\n}',
            '.join("../../test-cases/codebase-analysis")\n        .join(name)\n        .join("fixture")\n}',
        ),
    ])

    # src/codebase/ts_source/tests.rs
    tst = crates / "src/codebase/ts_source/tests.rs"
    fix_file(tst, [
        (
            '.join("../../test-cases")\n        .join(category)\n        .join(name)\n}',
            '.join("../../test-cases")\n        .join(category)\n        .join(name)\n        .join("fixture")\n}',
        ),
    ])

    # src/server_routes/tests.rs and tests/extra.rs
    for p in [
        "src/server_routes/tests.rs",
        "src/server_routes/tests/extra.rs",
    ]:
        fix_file(crates / p, [
            (
                '.join("../../test-cases/server-ast-routes")\n        .join(name)\n}',
                '.join("../../test-cases/server-ast-routes")\n        .join(name)\n        .join("fixture")\n}',
            ),
        ])

    # ── playwright/test_support.rs ──────────────────────────────────────────
    pts = crates / "src/playwright/test_support.rs"
    content = pts.read_text()
    old_fn = '''\
pub fn fixture_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.extend(["..", "..", "test-cases"]);
    path.extend(parts);
    path
}'''
    new_fn = '''\
pub fn fixture_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.extend(["..", "..", "test-cases"]);
    if parts.len() >= 2 {
        path.push(parts[0]);
        path.push(parts[1]);
        path.push("fixture");
        path.extend(&parts[2..]);
    } else {
        path.extend(parts);
    }
    path
}'''
    if old_fn in content:
        content = content.replace(old_fn, new_fn)
        pts.write_text(content)
        print(f"Fixed: {pts.relative_to(WORKTREE)}")
    else:
        print(f"WARNING: playwright/test_support.rs fixture_path not found in expected form")
        print("Current content around fixture_path:")
        for i, line in enumerate(content.split('\n')):
            if 'fixture_path' in line or 'test-cases' in line:
                print(f"  {i+1}: {line}")

    print("\nDone.")


if __name__ == "__main__":
    main()
