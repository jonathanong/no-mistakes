use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "postgres-object-naming";

#[test]
fn postgres_object_naming_check_json_snapshot() {
    // Output cases formerly asserted from check_with_files, plus the saved
    // replacement catalogs whose finding counts came from the same scanner.
    let prepared = naming_root("pass");
    let prepared_config = rules_path(RULE, "configs", "prepared-catalog.yml");
    let denied = rules_path(RULE, "configs", "replacement-denied.yml");
    let position = rules_path(RULE, "configs", "replacement-position.yml");
    assert_saved_fixture(
        "postgres_object_naming",
        &[
            (
                "prepared_catalog",
                prepared.clone(),
                prepared_config.clone(),
            ),
            (
                "custom_message",
                prepared.clone(),
                rules_path(RULE, "configs", "custom-message.yml"),
            ),
            // include: [missing.json] selects no catalog, so the report is empty.
            (
                "include_misses_catalog",
                prepared,
                rules_path(RULE, "configs", "include-missing.yml"),
            ),
            (
                "replacement_pass",
                rules_path(RULE, "fixture", "replacement-pass"),
                denied.clone(),
            ),
            // All seven denied-token findings must remain.
            (
                "replacement_fail",
                rules_path(RULE, "fixture", "replacement-fail"),
                denied,
            ),
            (
                "replacement_position_pass",
                rules_path(RULE, "fixture", "replacement-position-pass"),
                position.clone(),
            ),
            // All three out-of-position findings must remain.
            (
                "replacement_position_fail",
                rules_path(RULE, "fixture", "replacement-position-fail"),
                position,
            ),
        ],
    );
}

fn naming_root(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres/object-naming")
            .join(name),
    )
}
