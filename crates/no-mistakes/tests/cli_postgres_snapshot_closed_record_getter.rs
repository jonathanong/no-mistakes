use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_closed_record_getter_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "fixture",
        "closed-record-getter",
    );
    assert_saved_fixture(
        "postgres_closed_record_getter",
        &[(
            "nested_freeze_getter_column",
            root,
            rules_path(
                "postgres-no-test-state-mutation",
                "configs",
                "closed-record-getter.yml",
            ),
        )],
    );
}
