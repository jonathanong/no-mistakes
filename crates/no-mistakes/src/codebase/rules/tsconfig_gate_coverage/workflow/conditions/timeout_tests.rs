use super::{
    step_timeout_minutes_enforced, EnvironmentState, InputState, SecretState, StaticValue,
};
use serde_yaml::Value;

#[test]
fn step_timeouts_resolve_from_the_computed_step_environment() {
    let inputs = InputState::new();
    let workflow: Value = serde_yaml::from_str("env: {TIMEOUT: 5}").unwrap();
    let job: Value = serde_yaml::from_str("env: {TIMEOUT: 6}").unwrap();
    let step: Value = serde_yaml::from_str("env: {TIMEOUT: 7}").unwrap();
    let environment = EnvironmentState::from_workflow(&workflow, &SecretState::direct(), &inputs)
        .with_job(&job, &inputs)
        .with_step(&step, &inputs);

    assert!(step_timeout_minutes_enforced(
        Some(&Value::String("${{ fromJSON(env.TIMEOUT) }}".into())),
        &inputs,
        &environment,
    ));
    assert_eq!(
        environment.value("TIMEOUT"),
        Some(StaticValue::String("7".into()))
    );
}
