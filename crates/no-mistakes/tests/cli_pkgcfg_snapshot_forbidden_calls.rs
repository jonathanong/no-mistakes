use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn cfg(name: &str) -> PathBuf {
    rules_path("forbidden-calls", "configs", name)
}

fn root(kind: &str) -> PathBuf {
    rules_path("forbidden-calls", kind, "fixture")
}

#[test]
fn forbidden_calls_json_matches_snapshot() {
    let coverage = root("coverage");
    let malformed = root("malformed-roots");
    let vitest = root("explicit-vitest-project");
    let playwright_project = root("explicit-playwright-project");
    let collection = root("playwright-collection");
    let same_line = root("same-line-occurrences");
    assert_saved_fixture(
        "forbidden_calls",
        &[
            (
                "traversal_direct",
                coverage.clone(),
                cfg("traversal-direct.yml"),
            ),
            (
                "traversal_transitive",
                coverage.clone(),
                cfg("traversal-transitive.yml"),
            ),
            (
                "traversal_max_depth_1",
                coverage.clone(),
                cfg("traversal-max-depth-1.yml"),
            ),
            (
                "file_and_module_roots",
                coverage.clone(),
                cfg("file-and-module-roots.yml"),
            ),
            (
                "global_set_timeout",
                coverage.clone(),
                cfg("selectors-global-set-timeout.yml"),
            ),
            (
                "exact_timers_set_timeout",
                coverage.clone(),
                cfg("selectors-exact-timers-set-timeout.yml"),
            ),
            (
                "terminal_set_timeout",
                coverage.clone(),
                cfg("selectors-terminal-set-timeout.yml"),
            ),
            (
                "module_export_set_timeout",
                coverage.clone(),
                cfg("selectors-module-export-set-timeout.yml"),
            ),
            (
                "function_repository_target",
                coverage.clone(),
                cfg("selectors-function-repository-target.yml"),
            ),
            (
                "invocations_call",
                coverage.clone(),
                cfg("invocations-call-date.yml"),
            ),
            (
                "invocations_construct",
                coverage.clone(),
                cfg("invocations-construct-date.yml"),
            ),
            (
                "unknown_calls_ignore",
                coverage.clone(),
                cfg("unknown-calls-ignore.yml"),
            ),
            (
                "unknown_calls_finding",
                coverage.clone(),
                cfg("unknown-calls-finding.yml"),
            ),
            (
                "exact_page_wait_for_timeout",
                coverage.clone(),
                cfg("exact-page-wait-for-timeout.yml"),
            ),
            (
                "vitest_named_unit",
                coverage.clone(),
                cfg("vitest-named-unit.yml"),
            ),
            ("vitest_all", coverage.clone(), cfg("vitest-all-roots.yml")),
            (
                "default_date_calls",
                coverage.clone(),
                cfg("default-date-calls.yml"),
            ),
            (
                "include_restricts_transitive",
                coverage.clone(),
                cfg("include-restricts-transitive.yml"),
            ),
            (
                "repeated_applications",
                coverage.clone(),
                cfg("repeated-applications.yml"),
            ),
            (
                "suppressed_source",
                coverage.clone(),
                cfg("suppressed-source.yml"),
            ),
            (
                "glob_unit_tests",
                coverage.clone(),
                cfg("glob-unit-tests.yml"),
            ),
            (
                "glob_unit_and_integration",
                coverage.clone(),
                cfg("glob-unit-and-integration.yml"),
            ),
            (
                "glob_exclude_integration",
                coverage.clone(),
                cfg("glob-exclude-integration.yml"),
            ),
            (
                "terminal_wait_for_timeout",
                coverage.clone(),
                cfg("terminal-wait-for-timeout.yml"),
            ),
            (
                "file_traversal_function_root",
                coverage,
                cfg("file-traversal-function-root.yml"),
            ),
            (
                "valid_entry_root",
                malformed.clone(),
                cfg("valid-entry-root.yml"),
            ),
            (
                "malformed_file_root",
                malformed,
                cfg("malformed-file-root.yml"),
            ),
            (
                "explicit_vitest_named",
                vitest.clone(),
                cfg("explicit-vitest-named-custom.yml"),
            ),
            (
                "explicit_vitest_all",
                vitest,
                cfg("explicit-vitest-all.yml"),
            ),
            (
                "explicit_playwright_named",
                playwright_project.clone(),
                cfg("explicit-playwright-named-custom.yml"),
            ),
            (
                "explicit_playwright_all",
                playwright_project,
                cfg("explicit-playwright-all.yml"),
            ),
            (
                "playwright_true",
                collection.clone(),
                cfg("playwright-true-file-targets.yml"),
            ),
            (
                "playwright_terminal",
                collection.clone(),
                cfg("playwright-terminal-page.yml"),
            ),
            (
                "playwright_named_chromium",
                collection,
                cfg("playwright-named-chromium.yml"),
            ),
            (
                "same_line_occurrences",
                same_line,
                cfg("same-line-occurrences.yml"),
            ),
        ],
    );
}

#[test]
fn forbidden_calls_arguments_json_matches_snapshot() {
    let root = root("argument-predicates");
    assert_saved_fixture(
        "forbidden_calls_arguments",
        &[
            (
                "binding_aware_target",
                root.clone(),
                cfg("argument-predicates.yml"),
            ),
            ("documented", root.clone(), cfg("documented.yml")),
            ("escape_finding", root.clone(), cfg("escape-finding.yml")),
            ("escape_ignore", root.clone(), cfg("escape-ignore.yml")),
            (
                "destructuring_finding",
                root.clone(),
                cfg("destructuring-finding.yml"),
            ),
            (
                "destructuring_ignore",
                root.clone(),
                cfg("destructuring-ignore.yml"),
            ),
            (
                "repeated_finding",
                root.clone(),
                cfg("repeated-finding.yml"),
            ),
            ("repeated_ignore", root.clone(), cfg("repeated-ignore.yml")),
            ("target_only", root.clone(), cfg("target-only.yml")),
            ("null_arguments", root, cfg("null-arguments.yml")),
        ],
    );
}
