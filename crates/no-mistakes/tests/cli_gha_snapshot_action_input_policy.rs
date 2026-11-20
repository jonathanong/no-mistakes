use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-action-input-policy";

#[test]
fn github_actions_action_input_policy_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
    assert_saved_fixture(
        "github_actions_action_input_policy",
        &[
            // 8 findings: wrong widget values, a pinned SHA, runtime bans and
            // requirements, and one banned ship input. Unlisted actions stay out.
            (
                "violations",
                rules_path(RULE, "fixture", "violations"),
                policy.clone(),
            ),
            // 0 findings: quoted and bare scalars match, and ban-only inputs may be absent.
            ("pass", rules_path(RULE, "fixture", "pass"), policy.clone()),
            // 0 findings: an empty actions list does not require or ban anything.
            (
                "empty_actions",
                rules_path(RULE, "fixture", "violations"),
                rules_path(RULE, "configs", "empty-actions.yml"),
            ),
            // 0 findings: include drops the workflow before inputs are judged.
            (
                "include_miss",
                rules_path(RULE, "fixture", "violations"),
                rules_path(RULE, "configs", "include-miss.yml"),
            ),
            // 0 findings: invalid YAML is skipped.
            (
                "invalid_yaml",
                rules_path(RULE, "fixture", "invalid"),
                policy.clone(),
            ),
            // 1 finding: the line directive hides keep-days on the first step only.
            (
                "disabled_line",
                rules_path(RULE, "fixture", "disabled-line"),
                policy.clone(),
            ),
            // 0 findings: a file directive hides every input in the workflow.
            (
                "disabled_file",
                rules_path(RULE, "fixture", "disabled-file"),
                policy,
            ),
        ],
    );
}
