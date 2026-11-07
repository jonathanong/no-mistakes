use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_literal_tables_check_json_snapshot() {
    let rule = "postgres-require-query-annotation";
    assert_saved_fixture(
        "postgres_literal_tables",
        &[(
            "report",
            rules_path(rule, "fixture", "low-heads"),
            rules_path(rule, "configs", "literal-tables-report.yml"),
        )],
    );
}
