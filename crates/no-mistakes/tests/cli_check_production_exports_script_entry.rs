use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn production_exports_script_entry_json_matches_snapshot() {
    let entry = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "script-entry",
    );
    let absent = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "script-absent",
    );
    assert_saved_fixture(
        "production_exports_script_entry",
        &[
            ("entry", entry.clone(), entry.join(".no-mistakes.yml")),
            ("absent", absent.clone(), absent.join(".no-mistakes.yml")),
        ],
    );
}

#[test]
fn production_exports_script_entry_command_positions_match_snapshot() {
    let root = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "script-entry-shapes",
    );
    assert_saved_fixture(
        "production_exports_script_entry_command_positions",
        &[(
            "command_positions",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}
