use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn check_json_snapshot_workflow_forbidden_activity_types() {
    let cases: Vec<_> = [
        ("all-events", "activity-types.yml"),
        ("event-filter", "activity-types-filter.yml"),
        ("empty-event-filter", "activity-types-empty-events.yml"),
        ("custom-message", "activity-types-custom.yml"),
        ("disabled", "activity-types-disabled.yml"),
        ("invalid-options", "activity-types-invalid.yml"),
    ]
    .into_iter()
    .map(|(label, config)| {
        (
            label,
            rules_path("workflow-topology-policy", "fixture", "activity-types"),
            rules_path("workflow-topology-policy", "configs", config),
        )
    })
    .collect();
    assert_saved_fixture("workflow_forbidden_activity_types", &cases);
}
