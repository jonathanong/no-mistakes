use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-no-required-pr-path-filters";

#[test]
fn github_actions_required_pr_path_filters_json_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    let cases = ["default", "disabled", "invalid", "advisory"].map(|case| {
        (
            case,
            root.clone(),
            rules_path(RULE, "configs", &format!("{case}.yml")),
        )
    });
    assert_saved_fixture("github_actions_required_pr_path_filters", &cases);
}
