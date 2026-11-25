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

#[test]
fn standalone_required_pr_check_preserves_workflow_targets() {
    use no_mistakes::codebase::rules::github_actions_no_required_pr_path_filters::check;
    use no_mistakes::config::v2::discover::load_v2_config;

    let root = rules_path(RULE, "fixture", "");
    let config = rules_path(RULE, "configs", "default.yml");
    let config = load_v2_config(&root, Some(&config)).expect("load saved rule configuration");
    let findings = check(&root, &config).expect("check saved workflows");
    let mut targets: Vec<_> = findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect();
    targets.sort_unstable();
    assert_eq!(
        targets,
        [
            "on.pull_request.paths",
            "on.pull_request_target.paths-ignore",
            "unanalyzable",
        ]
    );
}
