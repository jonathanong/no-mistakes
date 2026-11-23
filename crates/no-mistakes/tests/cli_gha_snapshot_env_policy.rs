use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-env-policy";

#[test]
fn github_actions_env_policy_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
    assert_saved_fixture(
        "github_actions_env_policy",
        &[
            // 5 findings: workflow, job, step and composite scopes plus an expression value.
            (
                "violations",
                rules_path(RULE, "fixture", "violations"),
                policy.clone(),
            ),
            // 0 findings: /dev/null, unconfigured names, and names inside run text.
            ("pass", rules_path(RULE, "fixture", "pass"), policy.clone()),
            // 0 findings: no options means nothing is required or forbidden.
            (
                "empty",
                rules_path(RULE, "fixture", "violations"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            // 1 finding: line and next-line directives hide two of three.
            ("disabled", rules_path(RULE, "fixture", "disabled"), policy),
        ],
    );
}
