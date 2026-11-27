use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn github_actions_invalid_yaml_follows_each_rules_policy() {
    const GROUP: &str = "github-actions-invalid-yaml";
    let cases = ["report", "advise", "ignore", "download-low"].map(|mode| {
        (
            mode,
            rules_path(GROUP, "fixture", ""),
            rules_path(GROUP, "configs", &format!("{mode}.yml")),
        )
    });
    assert_saved_fixture("github_actions_invalid_yaml", &cases);
}
