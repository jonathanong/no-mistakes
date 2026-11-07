use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_literal_comment_names_check_json_snapshot() {
    let rule = "postgres-require-query-annotation";
    assert_saved_fixture(
        "postgres_literal_comment_names",
        &[(
            "report",
            rules_path(rule, "fixture", "finite-closed-names"),
            rules_path(rule, "configs", "literal-comment-names-report.yml"),
        )],
    );
}
