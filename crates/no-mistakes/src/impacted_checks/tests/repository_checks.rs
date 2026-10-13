use super::*;

fn repository_report(files: &[&str]) -> ImpactedChecksReport {
    let mut request = args(files);
    // Use the real repository configuration: a copied fixture would miss its regression.
    request.root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    );
    request.generic_only = true;
    generate_impacted_checks(&request).unwrap()
}

#[test]
fn repository_declarations_select_lint_and_format_without_file_arguments() {
    let report = repository_report(&[
        "packages/no-mistakes/index.d.ts",
        "packages/no-mistakes/scripts/api.test.js",
        "fixtures/node-api/executed-postgres-sql/public.mts",
    ]);
    assert_eq!(
        command_strings(&report),
        [
            "pnpm run fmt:js:check",
            "pnpm run lint:duplicates",
            "pnpm run lint:js"
        ]
    );
    assert!(report.warnings.is_empty());
    assert!(!report.fallback_triggered);
}

#[test]
fn repository_quality_configuration_changes_select_their_checks() {
    for (file, expected) in [
        (".jscpd.json", vec!["pnpm run lint:duplicates"]),
        (".oxlintrc.json", vec!["pnpm run lint:js"]),
        (".oxfmtrc.json", vec!["pnpm run fmt:js:check"]),
        ("cspell.config.yaml", vec!["pnpm run lint:spell"]),
        (
            "package.json",
            vec![
                "pnpm run fmt:js:check",
                "pnpm run lint:duplicates",
                "pnpm run lint:js",
                "pnpm run lint:spell",
            ],
        ),
        (
            "pnpm-lock.yaml",
            vec![
                "pnpm run fmt:js:check",
                "pnpm run lint:duplicates",
                "pnpm run lint:js",
                "pnpm run lint:spell",
            ],
        ),
    ] {
        assert_eq!(
            command_strings(&repository_report(&[file])),
            expected,
            "{file}"
        );
    }
}

#[test]
fn repository_docs_select_spelling_with_package_formatting_when_applicable() {
    for (file, expected) in [
        ("docs/cli/impacted-checks.md", vec!["pnpm run lint:spell"]),
        ("README.md", vec!["pnpm run lint:spell"]),
        (
            "packages/no-mistakes/README.md",
            vec!["pnpm run fmt:js:check", "pnpm run lint:spell"],
        ),
    ] {
        assert_eq!(
            command_strings(&repository_report(&[file])),
            expected,
            "{file}"
        );
    }
}

#[test]
fn repository_quality_checks_stay_scoped_and_skip_empty_changes() {
    for files in [
        vec![],
        vec!["fixtures/node-api/executed-postgres-sql/public.mts"],
        vec!["packages/no-mistakes/node_modules/dependency/index.js"],
        vec!["packages/no-mistakes/coverage/index.html"],
    ] {
        assert!(repository_report(&files).checks.is_empty(), "{files:?}");
    }
    for file in [
        "crates/no-mistakes/src/impacted_checks.rs",
        "packages/no-mistakes/vendor/generated.js",
    ] {
        assert_eq!(
            command_strings(&repository_report(&[file])),
            ["pnpm run lint:duplicates"],
            "{file}"
        );
    }
}

#[test]
fn repository_duplicate_gate_excludes_test_helpers_but_keeps_integration_analyzers() {
    for file in [
        "crates/no-mistakes/src/test_support.rs",
        "crates/no-mistakes/src/codebase/dependencies/graph/test_support/graph_files.rs",
        "packages/eslint-plugin-no-mistakes/test/helpers.mjs",
    ] {
        assert!(
            !command_strings(&repository_report(&[file]))
                .contains(&"pnpm run lint:duplicates".to_string()),
            "{file}"
        );
    }
    assert_eq!(
        command_strings(&repository_report(&[
            "crates/no-mistakes/src/integration_tests/project_config/parse.rs",
        ])),
        ["pnpm run lint:duplicates"]
    );
}

#[test]
fn repository_deleted_declaration_still_selects_whole_scope_validation() {
    // A missing changed path models a deletion; do not create this source file.
    let report = repository_report(&["packages/no-mistakes/deleted.d.ts"]);
    assert_eq!(
        command_strings(&report),
        [
            "pnpm run fmt:js:check",
            "pnpm run lint:duplicates",
            "pnpm run lint:js"
        ]
    );
}

#[test]
fn repository_package_module_variants_and_runner_config_select_lint_and_format() {
    for file in [
        "packages/no-mistakes/index.js",
        "packages/no-mistakes/test-timeout-policy/index.mjs",
        "packages/no-mistakes/index.d.cts",
        "packages/no-mistakes/index.d.mts",
        "vitest.config.js",
    ] {
        assert_eq!(
            command_strings(&repository_report(&[file])),
            [
                "pnpm run fmt:js:check",
                "pnpm run lint:duplicates",
                "pnpm run lint:js"
            ],
            "{file}"
        );
    }
}
