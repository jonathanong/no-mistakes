use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-no-poll-wait-job";

#[test]
fn github_actions_no_poll_wait_job_json_matches_snapshot() {
    assert_saved_fixture(
        "github_actions_no_poll_wait_job",
        &[
            (
                "report",
                rules_path(RULE, "fixture", "report"),
                rules_path(RULE, "configs", "report.yml"),
            ),
            (
                "pass",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "report.yml"),
            ),
            (
                "unconfigured",
                rules_path(RULE, "fixture", "report"),
                rules_path(RULE, "configs", "unconfigured.yml"),
            ),
            (
                "bounded",
                rules_path(RULE, "fixture", "bounded"),
                rules_path(RULE, "configs", "bounded.yml"),
            ),
            (
                "bounded-default",
                rules_path(RULE, "fixture", "bounded"),
                rules_path(RULE, "configs", "report.yml"),
            ),
            (
                "suppression",
                rules_path(RULE, "fixture", "suppression"),
                rules_path(RULE, "configs", "report.yml"),
            ),
        ],
    );
}
