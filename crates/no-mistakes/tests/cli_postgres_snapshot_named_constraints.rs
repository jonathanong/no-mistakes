use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "postgres-require-named-constraints";

#[test]
fn postgres_require_named_constraints_reports_and_suppressions() {
    // Match the former check_with_files inventory exactly: fail-fk also holds
    // disabled.sql, which belongs only to the dedicated suppression case.
    let migration = rules_path(RULE, "configs", "migration.yml");
    assert_saved_fixture(
        "postgres_require_named_constraints",
        &[
            (
                "foreign_key",
                rules_path(RULE, "fixture", "fail-fk"),
                migration.clone(),
            ),
            (
                "check",
                rules_path(RULE, "fixture", "fail-check"),
                migration.clone(),
            ),
            (
                "do_block",
                rules_path(RULE, "fixture", "fail-do-block"),
                migration.clone(),
            ),
            (
                "named_and_create_table",
                rules_path(RULE, "fixture", "pass"),
                migration.clone(),
            ),
            (
                "postgres_regression_sql",
                rules_path(RULE, "fixture", "pass-postgres-regress"),
                migration,
            ),
            (
                "disabled_next_line",
                rules_path(RULE, "fixture", "fail-fk"),
                rules_path(RULE, "configs", "disabled.yml"),
            ),
        ],
    );
}
