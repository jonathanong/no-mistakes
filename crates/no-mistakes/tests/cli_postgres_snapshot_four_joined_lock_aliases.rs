use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_four_joined_lock_aliases_check_json_snapshot() {
    assert_saved_fixture(
        "postgres_four_joined_lock_aliases",
        &[(
            "four_alias_write_sets",
            rules_path(
                "postgres-lock-written-rows-only",
                "fixture",
                "four-joined-aliases",
            ),
            rules_path("postgres-lock-written-rows-only", "configs", "fail.yml"),
        )],
    );
}
