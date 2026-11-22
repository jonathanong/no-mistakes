use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn vitest_default_excludes_json_matches_snapshot() {
    let root = rules_path("vitest-default-excludes", "fixture", "report");
    let literal_controls = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/vitest-default-excludes");
    assert_saved_fixture(
        "vitest_default_excludes",
        &[
            ("report", root.clone(), root.join(".no-mistakes.yml")),
            (
                "literal-controls",
                literal_controls.clone(),
                literal_controls.join(".no-mistakes.yml"),
            ),
            (
                "custom",
                literal_controls.clone(),
                literal_controls.join("custom.yml"),
            ),
        ],
    );
}
