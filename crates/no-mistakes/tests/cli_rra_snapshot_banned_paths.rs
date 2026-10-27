use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "banned-paths";

#[test]
fn banned_paths_json_matches_snapshot() {
    assert_saved_fixture(
        "banned_paths",
        &[
            (
                "configured_globs",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "fail-globs.yml"),
            ),
            (
                "include_and_suppression",
                rules_path(RULE, "fixture", "suppressed"),
                rules_path(RULE, "configs", "suppressed-pages.yml"),
            ),
        ],
    );
}
