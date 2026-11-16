use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_identifier_holes_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "fixture",
        "identifier-holes",
    );
    assert_saved_fixture(
        "postgres_identifier_holes",
        &[(
            "unknown_identifier_keeps_the_statement",
            root,
            rules_path(
                "postgres-no-test-state-mutation",
                "configs",
                "identifier-holes.yml",
            ),
        )],
    );
}
