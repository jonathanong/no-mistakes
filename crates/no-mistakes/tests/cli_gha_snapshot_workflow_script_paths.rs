use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-workflow-script-paths";

#[test]
fn github_actions_workflow_script_paths_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
    let empty = rules_path(RULE, "configs", "empty.yml");
    let pr_only = rules_path(RULE, "configs", "pr-only.yml");
    let report = rules_path(RULE, "configs", "unanalyzable-report.yml");
    let ignore = rules_path(RULE, "configs", "unanalyzable-ignore.yml");
    assert_saved_fixture(
        "github_actions_workflow_script_paths",
        &[
            (
                "reported",
                rules_path(RULE, "fixture", "reported"),
                policy.clone(),
            ),
            (
                "compliant",
                rules_path(RULE, "fixture", "compliant"),
                policy.clone(),
            ),
            ("pr-only", rules_path(RULE, "fixture", "reported"), pr_only),
            (
                "unanalyzable-advise",
                rules_path(RULE, "fixture", "unanalyzable"),
                policy,
            ),
            (
                "empty-opt-in",
                rules_path(RULE, "fixture", "reported"),
                empty,
            ),
            (
                "unanalyzable-report",
                rules_path(RULE, "fixture", "unanalyzable"),
                report,
            ),
            (
                "unanalyzable-ignore",
                rules_path(RULE, "fixture", "unanalyzable"),
                ignore,
            ),
        ],
    );
}
