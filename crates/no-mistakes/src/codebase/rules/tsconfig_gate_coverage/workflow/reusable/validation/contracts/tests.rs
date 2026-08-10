use super::*;

fn on(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn contract_shape_validates_every_declaration_kind() {
    assert!(!workflow_call_shape_valid(Some(&Value::Bool(true))));
    assert!(!workflow_call_trigger_keys_valid(&Value::Bool(true)));
    assert!(workflow_call_shape_valid(Some(&on("push"))));
    assert!(workflow_call_shape_valid(Some(&on("image_version"))));
    assert!(!workflow_call_shape_valid(Some(&on("pussh"))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "[push, workflow_dispath]"
    ))));
    for webhook_only in [
        "project",
        "project_card",
        "project_column",
        "repository",
        "repository_import",
        "repository_vulnerability_alert",
        "secret_scanning_alert",
        "team_add",
        "workflow_job",
    ] {
        assert!(
            !workflow_call_shape_valid(Some(&on(&format!("push:\n{webhook_only}:")))),
            "{webhook_only}"
        );
    }
    assert!(!workflow_call_shape_valid(Some(&on(
        "push:\nworkflow_dispath:"
    ))));
    for malformed in ["push: []", "push: true", "push: invalid"] {
        assert!(
            !workflow_call_shape_valid(Some(&on(malformed))),
            "{malformed}"
        );
    }
    assert!(workflow_call_shape_valid(Some(&on(
        "push:\nschedule:\n  - cron: '0 0 * * *'"
    ))));
    for malformed in [
        "schedule:",
        "schedule: []",
        "schedule:\n  - {}",
        "schedule:\n  - cron: true",
        "schedule:\n  - cron: ''",
        "schedule:\n  - cron: '0 0 * * *'\n    unknown: true",
    ] {
        assert!(
            !workflow_call_shape_valid(Some(&on(malformed))),
            "{malformed}"
        );
    }
    assert!(workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  outputs:\n    result:\n      value: '${{ jobs.build.outputs.result }}'\n      description: result"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  outputs:\n    result:\n      value: [invalid]"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  secrets:\n    token:\n      required: yes"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  inputs:\n    enabled:\n      type: boolean\n      unknown: true"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  inputs:\n    enabled:\n      description: missing type"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  inputs:\n    enabled:\n      type: invalid"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  outputs:\n    result:\n      description: missing value"
    ))));
}
