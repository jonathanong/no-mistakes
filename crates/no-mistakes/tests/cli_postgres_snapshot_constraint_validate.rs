use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_constraint_validate_check_json_snapshot() {
    let rule = "postgres-constraint-validate";
    // These are the output cases formerly asserted in the rule's unit module.
    // Snapshot the complete findings, retaining counts, messages, and clean cases.
    let cases = [
        (
            "not_valid_without_validate",
            "fail-missing",
            "migration.yml",
        ),
        (
            "implicit_check_name",
            "fail-generated-missing",
            "migration.yml",
        ),
        ("validate_without_not_valid", "fail-orphan", "migration.yml"),
        ("paired", "pass", "migration.yml"),
        (
            "postgres_regression_sql",
            "pass-postgres-regress",
            "migration.yml",
        ),
        ("paired_do_block", "pass-do-block", "migration.yml"),
        (
            "paired_do_block_ignore_recovery",
            "pass-do-block",
            "ignore.yml",
        ),
        (
            "paired_column_specific_set_null",
            "pass-set-null-columns",
            "migration.yml",
        ),
        ("paired_set_null", "pass-set-null", "migration.yml"),
        // Both child_topic_result_fk and child_other_fk must remain findings.
        (
            "unmatched_column_specific_set_null",
            "fail-set-null-columns",
            "migration.yml",
        ),
        (
            "do_block_without_validate",
            "fail-do-missing",
            "migration.yml",
        ),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_constraint_validate", &cases);
}
