use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_require_fk_on_delete_json_matches_snapshot() {
    let config = rules_path(
        "postgres-require-fk-on-delete",
        "configs",
        "migration-001.yml",
    );
    assert_saved_fixture(
        "postgres_require_fk_on_delete",
        &[
            (
                "omitted_on_delete",
                rules_path("postgres-require-fk-on-delete", "fixture", "fail"),
                config.clone(),
            ),
            (
                "no_action",
                rules_path("postgres-require-fk-on-delete", "fixture", "fail-no-action"),
                config.clone(),
            ),
            (
                "alter_table",
                rules_path("postgres-require-fk-on-delete", "fixture", "fail-alter"),
                config.clone(),
            ),
            (
                "do_block",
                rules_path("postgres-require-fk-on-delete", "fixture", "fail-do-block"),
                config.clone(),
            ),
            (
                "explicit_actions_pass",
                rules_path("postgres-require-fk-on-delete", "fixture", "pass"),
                config.clone(),
            ),
            // The regression fixture has a valid FK before its violating FK on line 3.
            (
                "postgres_regression",
                rules_path(
                    "postgres-require-fk-on-delete",
                    "fixture",
                    "pass-postgres-regress",
                ),
                config,
            ),
            (
                "disabled_next_line",
                rules_path("postgres-require-fk-on-delete", "fixture", "fail"),
                rules_path("postgres-require-fk-on-delete", "configs", "disabled.yml"),
            ),
        ],
    );
}
