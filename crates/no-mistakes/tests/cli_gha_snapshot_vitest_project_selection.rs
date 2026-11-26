use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-vitest-project-selection";

#[test]
fn github_actions_vitest_project_selection_json_matches_snapshot() {
    assert_saved_fixture(
        "github_actions_vitest_project_selection",
        &["report", "heredocs", "suppressions"].map(|name| {
            (
                name,
                rules_path(RULE, "fixture", name),
                rules_path(RULE, "configs", "report.yml"),
            )
        }),
    );
}
