use super::*;
use std::sync::OnceLock;

fn repo_root() -> PathBuf {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    );
    root.canonicalize().unwrap_or(root)
}

fn repository_config() -> &'static NoMistakesConfig {
    static CONFIG: OnceLock<NoMistakesConfig> = OnceLock::new();
    // Use the real repository configuration: a copied fixture would miss its regression.
    CONFIG.get_or_init(|| crate::config::v2::load_v2_config(&repo_root(), None).unwrap())
}

fn repository_report(files: &[&str]) -> ImpactedChecksReport {
    let root = repo_root();
    let changed_files: Vec<String> = files
        .iter()
        .map(|file| (*file).to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let missing: BTreeSet<String> = changed_files
        .iter()
        .filter(|file| !root.join(file).exists())
        .cloned()
        .collect();
    let checks =
        dedupe_checks(generic_checks(repository_config(), &changed_files, &missing).unwrap());
    ImpactedChecksReport {
        changed_files,
        checks,
        warnings: Vec::new(),
        fallback_triggered: false,
        empty_result: None,
    }
}

#[test]
fn repository_declarations_select_lint_and_format_without_file_arguments() {
    let files = [
        "packages/no-mistakes/index.d.ts",
        "packages/no-mistakes/scripts/api.test.js",
        "fixtures/node-api/executed-postgres-sql/public.mts",
    ];
    let mut request = args(&files);
    request.root = repo_root();
    request.generic_only = true;
    let report = generate_impacted_checks(&request).unwrap();
    assert_eq!(
        command_strings(&report),
        command_strings(&repository_report(&files))
    );
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
    assert_eq!(
        command_strings(&repository_report(&[
            "crates/no-mistakes/src/impacted_checks.rs"
        ])),
        [
            "cargo test --locked -p no-mistakes --lib architecture",
            "pnpm run lint:duplicates"
        ]
    );
    assert_eq!(
        command_strings(&repository_report(&[
            "packages/no-mistakes/vendor/generated.js"
        ])),
        ["pnpm run lint:duplicates"]
    );
}

#[test]
fn repository_duplicate_gate_excludes_test_helpers_but_keeps_production_analyzers() {
    for file in [
        "crates/no-mistakes/src/test_support.rs",
        "crates/no-mistakes/src/codebase/dependencies/root_dependency_test_helpers.rs",
        "crates/no-mistakes/src/codebase/ts_resolver/scoped_test_support.rs",
        "crates/no-mistakes/src/codebase/dependencies/graph/test_support/graph_files.rs",
        "crates/no-mistakes/src/impacted_checks/tests/repository_checks.rs",
        "crates/no-mistakes/tests/cli.rs",
        "packages/eslint-plugin-no-mistakes/test/helpers.mjs",
    ] {
        assert!(
            !command_strings(&repository_report(&[file]))
                .contains(&"pnpm run lint:duplicates".to_string()),
            "{file}"
        );
    }
    for file in [
        "crates/no-mistakes/src/tests/plan/framework.rs",
        "crates/no-mistakes/src/integration_tests/project_config/parse.rs",
    ] {
        assert_eq!(
            command_strings(&repository_report(&[file])),
            [
                "cargo test --locked -p no-mistakes --lib architecture",
                "pnpm run lint:duplicates"
            ],
            "{file}"
        );
    }
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
        "packages/no-mistakes/index.jsx",
        "packages/no-mistakes/test-timeout-policy/index.mjs",
        "packages/no-mistakes/index.d.cts",
        "packages/no-mistakes/index.d.mts",
        "packages/no-mistakes/index.tsx",
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
    // Root runner config is linted and formatted, but outside the crates/packages duplicate scan.
    assert_eq!(
        command_strings(&repository_report(&["vitest.config.js"])),
        ["pnpm run fmt:js:check", "pnpm run lint:js"]
    );
}
