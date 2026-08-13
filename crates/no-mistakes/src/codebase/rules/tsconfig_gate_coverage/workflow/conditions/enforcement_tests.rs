use super::*;

#[test]
fn job_enforcement_respects_static_continue_on_error_values() {
    let inputs = InputState::new();
    let tolerated: Value =
        serde_yaml::from_str("if: true\ncontinue-on-error: '${{ true }}'").unwrap();
    let enforcing: Value = serde_yaml::from_str("if: true\ncontinue-on-error: false").unwrap();
    let disabled: Value = serde_yaml::from_str("if: false").unwrap();

    assert!(job_tolerates_failure(&tolerated, &inputs));
    assert!(job_statically_not_enforcing(&tolerated, &inputs));
    assert!(!job_statically_enforcing(&tolerated, &inputs, false));
    assert!(job_statically_enabled(&tolerated, &inputs));
    assert!(job_statically_enforcing(&enforcing, &inputs, false));
    assert!(job_statically_disabled(&disabled, &inputs));
}
