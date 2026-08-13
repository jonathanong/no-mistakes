use super::*;

#[test]
fn trigger_contracts_reject_non_trigger_values_and_unknown_configs() {
    assert!(!workflow_trigger_configs_valid(&Value::Bool(true)));
    assert!(!workflow_trigger_configs_valid(
        &serde_yaml::from_str("unknown: {}").unwrap()
    ));
    assert!(workflow_trigger_configs_valid(
        &serde_yaml::from_str("pull_request:\n  branches: [main]").unwrap()
    ));
}
