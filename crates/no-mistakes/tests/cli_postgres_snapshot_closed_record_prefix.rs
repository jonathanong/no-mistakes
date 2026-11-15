use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_closed_record_prefix_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "fixture",
        "closed-record-prefix",
    );
    assert_saved_fixture(
        "postgres_closed_record_prefix",
        &[(
            "nested_freeze_and_lock_prefix",
            root,
            rules_path(
                "postgres-no-test-state-mutation",
                "configs",
                "closed-record-prefix.yml",
            ),
        )],
    );
}
