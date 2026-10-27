use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_fk_index_check_json_snapshot() {
    let rule = "postgres-fk-index";
    let migrations = "migrations.yml";
    let cases = [
        ("missing_index", "fail", migrations),
        ("btree_index", "pass", migrations),
        ("repeated_inline", "repeated-inline", migrations),
        ("same_line_directive", "directive", migrations),
        ("allowed_column", "allow", "allowed-columns.yml"),
        ("stale_allowed_column", "stale", "stale-column.yml"),
        ("gin_index", "gin", migrations),
        ("allowed_table", "fail", "allowed-tables.yml"),
        ("stale_allowed_table", "pass", "stale-tables.yml"),
        ("directive_mismatch", "directive", "allow-directive.yml"),
        ("hash_index", "hash", migrations),
        ("case_insensitive_column", "case-insensitive", migrations),
        ("partial_not_null", "partial-not-null", migrations),
        ("partial_other_column", "partial-other", migrations),
        ("leading_column_mismatch", "leading-other", migrations),
        ("schema_qualified_index", "schema-qualified", migrations),
        // A bare `--` comment is not an allow directive.
        ("empty_directive", "empty-directive", migrations),
        (
            "custom_directive",
            "custom-directive",
            "allow-directive.yml",
        ),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_fk_index", &cases);
}
