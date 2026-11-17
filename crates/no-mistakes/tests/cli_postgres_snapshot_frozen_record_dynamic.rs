use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_frozen_record_dynamic_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "fixture",
        "frozen-record-dynamic",
    );
    assert_saved_fixture(
        "postgres_frozen_record_dynamic",
        &[(
            "dynamic_key_enumerates_imported_records",
            root,
            rules_path(
                "postgres-no-test-state-mutation",
                "configs",
                "frozen-record-dynamic.yml",
            ),
        )],
    );
}
