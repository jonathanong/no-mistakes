use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-local-action-checkout";

#[test]
fn github_actions_local_action_checkout_json_matches_snapshot() {
    let cases = [
        "unit",
        "pass",
        "suppressed",
        "locations",
        "diagnostics",
        "boundaries",
        "named-job",
    ];
    let cases: Vec<_> = cases
        .iter()
        .map(|case| {
            (
                *case,
                rules_path(RULE, "fixture", case),
                rules_path(RULE, "configs", "default.yml"),
            )
        })
        .collect();
    assert_saved_fixture("github_actions_local_action_checkout", &cases);
}
