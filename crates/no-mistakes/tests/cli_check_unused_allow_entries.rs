use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

#[test]
fn unused_allow_entries_check_json_snapshot() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/unused-allow-entries");
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut cases = vec![
        ("query", root, PathBuf::new()),
        (
            "other_lists",
            repository.join("fixtures/rules/unused-allow-other"),
            PathBuf::new(),
        ),
        (
            "forbidden_dependencies",
            repository.join("fixtures/rules/unused-allow-forbidden"),
            PathBuf::new(),
        ),
    ];
    for rule in [
        "csharp-no-async-void-delegate",
        "swift-no-raw-print",
        "swift-viewmodel-main-actor",
        "no-raw-ephemeral-port",
        "no-test-git-sha",
        "no-git-identity-mutation",
        "package-json-required-fields",
    ] {
        cases.push((
            rule,
            repository.join(format!("test-cases/rules/{rule}/fixture/allow-audit")),
            PathBuf::new(),
        ));
    }
    cases.push((
        "test-email-domain-policy",
        repository.join("test-cases/rules/test-email-domain-policy/unit-fixture/allow-audit"),
        PathBuf::new(),
    ));
    assert_saved_fixture("unused_allow_entries", &cases);
}

#[test]
fn unused_allow_string_locations_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/unused-allow-string-locations");
    assert_saved_fixture(
        "unused_allow_string_locations",
        &[("long_config", root, PathBuf::new())],
    );
}

#[test]
fn unused_allow_policy_check_json_snapshot() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/unused-allow-entries");
    let cases: Vec<_> = [
        "advise",
        "report",
        "ignore",
        "advise-violation",
        "report-violation",
        "ignore-violation",
        "report-narrowed",
        "report-advisory",
    ]
    .into_iter()
    .map(|policy| (policy, root.clone(), root.join(format!("{policy}.yml"))))
    .collect();
    assert_saved_fixture("unused_allow_policy", &cases);
}

#[test]
fn unused_allow_empty_files_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-empty-or-comments-only-files/fixture/allow-audit");
    let cases: Vec<_> = [
        ".no-mistakes.yml",
        "partial.yml",
        "failed.yml",
        "overlap.yml",
    ]
    .into_iter()
    .map(|name| (name, root.clone(), root.join(name)))
    .collect();
    assert_saved_fixture("unused_allow_empty_files", &cases);
}

#[test]
fn unused_allow_postgres_patterns_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/postgres-pattern-exceptions");
    let cases: Vec<_> = [
        "report.yml",
        "excluded.yml",
        "partial.yml",
        "failed.yml",
        "no-columns.yml",
        "column-selection.yml",
    ]
    .into_iter()
    .map(|name| (name, root.clone(), root.join(name)))
    .collect();
    assert_saved_fixture("unused_allow_postgres_patterns", &cases);
}

#[test]
fn unused_allow_postgres_names_and_sweeps_check_json_snapshot() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut cases = Vec::new();
    for (rule, scenarios) in [
        (
            "postgres-object-naming",
            &["used", "stale", "overlap", "partial", "disabled", "invalid"][..],
        ),
        (
            "postgres-sql-shape-policy",
            &[
                "used",
                "stale",
                "overlap",
                "parse-failure",
                "disabled",
                "unrecoverable",
                "embedded-used",
                "embedded-stale",
                "embedded-parse-failure",
                "typescript-parse-failure",
                "fragment-unrecoverable",
                "filtered",
                "empty",
                "low-effort",
            ][..],
        ),
    ] {
        for scenario in scenarios {
            let prefix = if rule == "postgres-object-naming" {
                "ignore-patterns"
            } else {
                "ignore-tables"
            };
            let root = repository.join(format!(
                "test-cases/rules/{rule}/fixture/{prefix}-{scenario}"
            ));
            cases.push((format!("{rule}/{scenario}"), root, PathBuf::new()));
        }
    }
    let cases: Vec<_> = cases
        .iter()
        .map(|(label, root, config)| (label.as_str(), root.clone(), config.clone()))
        .collect();
    assert_saved_fixture("unused_allow_postgres_names_and_sweeps", &cases);
}

#[test]
fn unused_allow_missing_files_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check/unused-allow-missing/fixture");
    let cases: Vec<_> = [
        ".no-mistakes.yml",
        "ignore.yml",
        "narrowed.yml",
        "advise.yml",
        "migrations.yml",
        "storybook.yml",
        "storybook-alias.yml",
        "steps.yml",
        "disabled.yml",
        "trimmed.yml",
        "hash-selectors.yml",
        "windows.yml",
        "gates.yml",
        "options-narrowed.yml",
        "indices.yml",
        "storybook-dotted.yml",
        "implicit-root.yml",
        "empty-scope.yml",
        "empty-ignore-invalid.yml",
        "empty-report-invalid.yml",
    ]
    .into_iter()
    .map(|name| (name, root.clone(), root.join(name)))
    .collect();
    assert_saved_fixture("unused_allow_missing_files", &cases);
}
