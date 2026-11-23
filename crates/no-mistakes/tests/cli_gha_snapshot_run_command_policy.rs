use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-run-command-policy";

#[test]
fn github_actions_run_command_policy_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
    let empty = rules_path(RULE, "configs", "empty.yml");
    let report = rules_path(RULE, "configs", "unanalyzable-report.yml");
    let ignore = rules_path(RULE, "configs", "unanalyzable-ignore.yml");
    assert_saved_fixture(
        "github_actions_run_command_policy",
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
            (
                "unparsable",
                rules_path(RULE, "fixture", "unparsable"),
                policy,
            ),
            (
                "empty-opt-in",
                rules_path(RULE, "fixture", "reported"),
                empty,
            ),
            (
                "unparsable-report",
                rules_path(RULE, "fixture", "unparsable"),
                report,
            ),
            (
                "unparsable-ignore",
                rules_path(RULE, "fixture", "unparsable"),
                ignore,
            ),
        ],
    );
}
