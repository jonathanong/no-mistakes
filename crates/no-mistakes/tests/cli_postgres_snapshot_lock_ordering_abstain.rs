use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_lock_ordering_abstain_check_json_snapshot() {
    let rule = "postgres-lock-ordering";
    assert_saved_fixture(
        "postgres_lock_ordering_abstain",
        &[
            (
                "closed_table_identifier",
                rules_path(rule, "fixture", "closed-table-identifier"),
                rules_path(rule, "configs", "catalog.yml"),
            ),
            (
                "appended_scope_fragment",
                rules_path(rule, "fixture", "appended-scope-fragment"),
                rules_path(rule, "configs", "ignore-unanalyzable.yml"),
            ),
        ],
    );
}
