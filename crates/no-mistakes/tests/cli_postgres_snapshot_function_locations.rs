use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

#[test]
fn postgres_bounded_function_locations_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/variants/function-locations"),
    );
    assert_saved_fixture(
        "postgres_bounded_function_locations",
        &[(
            "physical_relation_lines",
            root,
            rules_path(
                "postgres-bounded-statements",
                "configs",
                "function-locations.yml",
            ),
        )],
    );
}
