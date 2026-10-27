use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn embedded() -> PathBuf {
    rules_path(
        "postgres-require-query-annotation",
        "../../../fixtures/postgres-facts",
        "embedded",
    )
}

#[test]
fn postgres_undeclared_executors_check_json_snapshot() {
    let rule = "postgres-require-query-annotation";
    let root = embedded();
    let config = |name: &str| rules_path(rule, "configs", name);
    // include is only the file the unit check passed. The annotation rule can
    // add findings beside undeclared-executor messages, so an isolated
    // findings.len() is not rules.length. SQL verdict ledger fields are absent
    // from check --format json.
    assert_saved_fixture(
        "postgres_undeclared_executors",
        &[
            // Nine undeclared-executor messages must remain in rules: PoolClient,
            // cannot be resolved, has no declared executor type, and Executor.
            (
                "missing_types_report",
                root.clone(),
                config("missing-types-report.yml"),
            ),
            // Those undeclared-executor messages must not be in rules.
            (
                "missing_types_advise",
                root.clone(),
                config("missing-types-advise.yml"),
            ),
            (
                "missing_types_ignore",
                root.clone(),
                config("missing-types-ignore.yml"),
            ),
            // "undeclared executor type `PoolClient`" remains.
            // "has no declared executor type" is absent from that finding.
            (
                "declared_imported_types",
                root.clone(),
                config("constructor-alias.yml"),
            ),
            ("excluded_calls", root.clone(), config("excluded-calls.yml")),
            // No importSpecifier key. Null importSpecifier stays in the unit test.
            (
                "absent_import_specifier",
                root.clone(),
                config("absent-specifier.yml"),
            ),
            (
                "empty_import_specifier",
                root,
                config("empty-specifier.yml"),
            ),
        ],
    );
}
