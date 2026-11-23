use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn check_json_snapshot_workflow_policy_bans_events() {
    let cases: Vec<_> = [
        ("banned", "forbidden-events.yml"),
        ("empty", "forbidden-events-empty.yml"),
        ("custom-message", "forbidden-events-custom.yml"),
        ("disabled", "forbidden-events-disabled.yml"),
        ("invalid-options", "forbidden-events-invalid.yml"),
    ]
    .into_iter()
    .map(|(label, config)| {
        (
            label,
            rules_path("workflow-topology-policy", "fixture", "forbidden-events"),
            rules_path("workflow-topology-policy", "configs", config),
        )
    })
    .collect();
    assert_saved_fixture("workflow_policy_bans_events", &cases);
}

#[test]
fn check_json_snapshot_workflow_policy_bans_secrets() {
    let cases: Vec<_> = [
        ("banned", "secrets-inherit.yml"),
        ("disabled-false", "secrets-inherit-false.yml"),
        ("custom-message", "secrets-inherit-custom.yml"),
        ("disabled", "secrets-inherit-disabled.yml"),
        ("invalid-options", "secrets-inherit-invalid.yml"),
    ]
    .into_iter()
    .map(|(label, config)| {
        (
            label,
            rules_path("workflow-topology-policy", "fixture", "secrets-inherit"),
            rules_path("workflow-topology-policy", "configs", config),
        )
    })
    .collect();
    assert_saved_fixture("workflow_policy_bans_secrets", &cases);
}

#[test]
fn check_json_snapshot_workflow_policy_bans_reusable_concurrency() {
    let cases: Vec<_> = [
        ("banned", "reusable-concurrency.yml"),
        ("disabled-false", "reusable-concurrency-false.yml"),
        ("custom-message", "reusable-concurrency-custom.yml"),
        ("disabled", "reusable-concurrency-disabled.yml"),
        ("invalid-options", "reusable-concurrency-invalid.yml"),
    ]
    .into_iter()
    .map(|(label, config)| {
        (
            label,
            rules_path(
                "workflow-topology-policy",
                "fixture",
                "reusable-concurrency",
            ),
            rules_path("workflow-topology-policy", "configs", config),
        )
    })
    .collect();
    assert_saved_fixture("workflow_policy_bans_reusable_concurrency", &cases);
}
