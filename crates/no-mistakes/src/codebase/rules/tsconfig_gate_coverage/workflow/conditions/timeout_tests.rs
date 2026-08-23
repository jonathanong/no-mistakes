use super::{
    job_timeout_minutes_validity, step_timeout_minutes_validity, EnvironmentState, InputState,
    SecretState, StaticBool, StaticValue,
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

    assert_eq!(
        step_timeout_minutes_validity(
            Some(&Value::String("${{ fromJSON(env.TIMEOUT) }}".into())),
            &inputs,
            &environment,
        ),
        StaticBool::True
    );
    assert_eq!(
        environment.value("TIMEOUT"),
        Some(StaticValue::String("7".into()))
    );
}

#[test]
fn timeout_validation_rejects_wrong_types_and_out_of_range_numbers() {
    let inputs = InputState::new();
    let environment = EnvironmentState::default();
    for (value, expected) in [
        (Value::Number(360.into()), StaticBool::True),
        (Value::Number(361.into()), StaticBool::False),
        (Value::Bool(true), StaticBool::False),
        (Value::String("${{ 5 }}".into()), StaticBool::True),
        (Value::String("${{ '5' }}".into()), StaticBool::False),
        (Value::String("${{ true }}".into()), StaticBool::False),
    ] {
        assert_eq!(
            step_timeout_minutes_validity(Some(&value), &inputs, &environment),
            expected,
            "{value:?}"
        );
    }
    assert_eq!(
        job_timeout_minutes_validity(Some(&Value::Number(361.into())), &inputs),
        StaticBool::True
    );
}

#[test]
fn timeout_validation_evaluates_negated_static_input_expressions() {
    let inputs = InputState::from([("disabled".into(), StaticValue::Bool(false))]);
    assert_eq!(
        step_timeout_minutes_validity(
            Some(&Value::String("${{ !inputs.disabled }}".into())),
            &inputs,
            &EnvironmentState::default(),
        ),
        StaticBool::False
    );
}
