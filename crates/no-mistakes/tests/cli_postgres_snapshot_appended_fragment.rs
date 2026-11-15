use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_appended_fragment_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "fixture",
        "appended-fragment",
    );
    assert_saved_fixture(
        "postgres_appended_fragment",
        &[(
            "appended_helper_fragment",
            root,
            rules_path(
                "postgres-no-test-state-mutation",
                "configs",
                "appended-fragment.yml",
            ),
        )],
    );
}
