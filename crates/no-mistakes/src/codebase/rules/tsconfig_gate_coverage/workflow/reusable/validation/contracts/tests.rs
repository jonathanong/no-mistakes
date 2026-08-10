use super::*;

fn on(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn contract_shape_validates_every_declaration_kind() {
    assert!(workflow_call_shape_valid(Some(&Value::Bool(true))));
    assert!(!workflow_call_trigger_keys_valid(&Value::Bool(true)));
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
