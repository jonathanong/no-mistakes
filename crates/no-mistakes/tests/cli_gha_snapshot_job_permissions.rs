use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-job-permissions";

#[test]
fn github_actions_job_permissions_json_matches_snapshot() {
    let default = rules_path(RULE, "configs", "default.yml");
    let mut cases: Vec<_> = [
        "pass",
        "fail",
        "flow",
        "suppression",
        "mixed",
        "unresolved",
        "edge",
        "unreadable",
        "permission-suppression",
    ]
    .into_iter()
    .map(|scenario| {
        (
            scenario,
            rules_path(RULE, "fixture", scenario),
            default.clone(),
        )
    })
    .collect();
    for (label, scenario, policy) in [
        ("unresolved-advise", "unresolved", "advise"),
        ("unresolved-ignore", "unresolved", "ignore"),
        ("mixed-advise", "mixed", "advise"),
        ("mixed-ignore", "mixed", "ignore"),
    ] {
        cases.push((
            label,
            rules_path(RULE, "fixture", scenario),
            rules_path(RULE, "configs", &format!("{policy}.yml")),
        ));
    }
    let scope = rules_path(RULE, "fixture", "scope");
    cases.push(("scope", scope.clone(), scope.join(".no-mistakes.yml")));
    let custom = rules_path(RULE, "fixture", "custom");
    cases.push((
        "custom-message",
        custom.clone(),
        custom.join(".no-mistakes.yml"),
    ));
    assert_saved_fixture("github_actions_job_permissions", &cases);
}
