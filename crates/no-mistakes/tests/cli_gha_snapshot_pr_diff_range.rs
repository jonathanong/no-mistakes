use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-pr-diff-range";

#[test]
fn github_actions_pr_diff_range_json_matches_snapshot() {
    let cases: Vec<_> = [
        "report",
        "pass",
        "suppression",
        "env-and-heredoc",
        "event-fallbacks",
        "shell-guard-elif",
        "shell-guard-preamble",
        "shell-guard-runner-event",
        "shell-guard-chain",
        "shell-guard-regex",
        "shell-guard-separator",
    ]
    .into_iter()
    .map(|name| {
        (
            name,
            rules_path(RULE, "fixture", name),
            rules_path(RULE, "configs", "report.yml"),
        )
    })
    .collect();
    assert_saved_fixture("github_actions_pr_diff_range", &cases);
}
