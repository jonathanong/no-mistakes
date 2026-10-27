use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-action-timeout-pair";

#[test]
fn github_actions_action_timeout_pair_json_matches_snapshot() {
    let paired = rules_path(RULE, "configs", "paired.yml");
    assert_saved_fixture(
        "github_actions_action_timeout_pair",
        &[
            ("pass", rules_path(RULE, "fixture", "pass"), paired.clone()),
            (
                "missing_step_timeout",
                rules_path(RULE, "fixture", "fail-step"),
                paired.clone(),
            ),
            (
                "quoted_nested_timeout",
                rules_path(RULE, "fixture", "fail-nested"),
                paired.clone(),
            ),
            (
                "nested_composite",
                rules_path(RULE, "fixture", "fail-composite"),
                paired,
            ),
            // `uses: []` returns before the missing step timeout is checked.
            (
                "empty_uses",
                rules_path(RULE, "fixture", "fail-step"),
                rules_path(RULE, "configs", "empty-uses.yml"),
            ),
            // `include: [nope.yml]` matches neither pass file. The old explicit
            // list omitted the action file; this include drops both.
            (
                "include_miss",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "include-miss.yml"),
            ),
            // Project root only: the finding is the missing nested input, not
            // "nests a configured action".
            (
                "wrapper_project_target",
                rules_path(RULE, "fixture", "cases/wrapper-project-target"),
                rules_path(RULE, "configs", "wrapper-project.yml"),
            ),
        ],
    );
}
