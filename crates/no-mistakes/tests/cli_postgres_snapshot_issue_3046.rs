use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_namespace_issue_3046_check_json_snapshot() {
    let rule = "postgres-no-offset";
    let cases: Vec<_> = [
        ("repro", "issue-3046.yml"),
        ("import-type", "issue-3046.yml"),
        ("recall", "issue-3046-executors.yml"),
        ("nested", "issue-3046.yml"),
        ("declare", "issue-3046.yml"),
        ("scope-isolation", "issue-3046-executors.yml"),
    ]
    .into_iter()
    .map(|(label, config)| {
        (
            label,
            rules_path(rule, "fixture", &format!("issue-3046/{label}")),
            rules_path(rule, "configs", config),
        )
    })
    .collect();
    assert_saved_fixture("postgres_namespace_issue_3046", &cases);
}
