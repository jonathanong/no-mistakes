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
                "default_config",
                rules_path(RULE, "fixture", "fail-step"),
                std::path::PathBuf::new(),
            ),
            (
                "missing_step_timeout",
                rules_path(RULE, "fixture", "fail-step"),
                paired.clone(),
            ),
            (
                "suppressions",
                rules_path(RULE, "fixture", "suppressions"),
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

#[test]
fn nested_deadline_json_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "deadline");
    let cases = [
        ("report", "deadline-report.yml"),
        ("advise", "deadline-advise.yml"),
        ("ignore", "deadline-ignore.yml"),
        ("disabled", "deadline-off.yml"),
        ("custom_advisory", "deadline-custom-advisory.yml"),
        ("with_exact_checks_ignore", "deadline-with-exact.yml"),
        ("with_exact_checks_advise", "deadline-with-exact-advise.yml"),
        ("with_exact_checks_report", "deadline-with-exact-report.yml"),
    ]
    .map(|(label, config)| (label, root.clone(), rules_path(RULE, "configs", config)));
    assert_saved_fixture("github_actions_action_timeout_pair_deadline", &cases);
}
