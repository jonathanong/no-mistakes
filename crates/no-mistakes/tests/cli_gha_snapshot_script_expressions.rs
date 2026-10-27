use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-script-expressions";

#[test]
fn github_actions_script_expressions_json_matches_snapshot() {
    assert_saved_fixture(
        "github_actions_script_expressions",
        &[
            (
                "pass",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "pass.yml"),
            ),
            (
                "fail",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "fail.yml"),
            ),
            (
                "allowlist",
                rules_path(RULE, "fixture", "allowlist"),
                rules_path(RULE, "configs", "allowlist.yml"),
            ),
            (
                "suppression",
                rules_path(RULE, "fixture", "suppression"),
                rules_path(RULE, "configs", "suppression.yml"),
            ),
            (
                "unanalyzable",
                rules_path(RULE, "fixture", "unanalyzable"),
                rules_path(RULE, "configs", "unanalyzable.yml"),
            ),
        ],
    );
}
