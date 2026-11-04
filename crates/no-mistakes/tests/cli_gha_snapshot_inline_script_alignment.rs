use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn github_actions_inline_script_alignment_json_matches_snapshot() {
    let rule = "github-actions-max-inline-script-lines";
    assert_saved_fixture(
        "github_actions_inline_script_alignment",
        &[(
            "alignment",
            rules_path(rule, "fixture", "alignment"),
            rules_path(rule, "configs", "three.yml"),
        )],
    );
}
