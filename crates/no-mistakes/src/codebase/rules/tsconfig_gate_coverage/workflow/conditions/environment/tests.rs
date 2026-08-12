use super::*;

#[test]
fn literal_environment_values_use_github_string_coercion() {
    for (value, expected) in [
        (Value::Bool(true), StaticValue::String("true".into())),
        (
            Value::Number(serde_yaml::Number::from(42)),
            StaticValue::String("42".into()),
        ),
        (
            Value::String("release".into()),
            StaticValue::String("release".into()),
        ),
        (Value::Null, StaticValue::String(String::new())),
        (Value::Sequence(Vec::new()), StaticValue::Invalid),
        (
            Value::Mapping(serde_yaml::Mapping::new()),
            StaticValue::Invalid,
        ),
    ] {
        assert_eq!(string_value(value), expected);
    }
}

#[test]
fn expressions_resolve_current_inputs_and_matrix_values_with_string_coercion() {
    let inputs = BTreeMap::from([
        ("enabled".to_string(), StaticValue::Bool(true)),
        ("retries".to_string(), StaticValue::Number("2".to_string())),
        ("empty".to_string(), StaticValue::Null),
        (
            "\0matrix.target".to_string(),
            StaticValue::String("linux".to_string()),
        ),
        ("\0matrix.cfg".to_string(), StaticValue::Mapping),
        ("\0matrix.dynamic".to_string(), StaticValue::Unknown),
    ]);

    for (expression, expected) in [
        ("${{ inputs.enabled }}", StaticValue::String("true".into())),
        ("${{ inputs.retries }}", StaticValue::String("2".into())),
        ("${{ inputs.empty }}", StaticValue::String(String::new())),
        ("${{ inputs.missing }}", StaticValue::String(String::new())),
        ("${{ matrix.target }}", StaticValue::String("linux".into())),
        ("${{ matrix.cfg }}", StaticValue::Mapping),
        ("${{ matrix.dynamic_target }}", StaticValue::Unknown),
        ("prefix-${{ inputs.enabled }}", StaticValue::Unknown),
    ] {
        assert_eq!(
            environment_value(
                &Value::String(expression.into()),
                &SecretState::direct(),
                &inputs,
                &EnvironmentState::default(),
            ),
            expected
        );
    }
}

#[test]
fn known_non_stringable_input_values_remain_invalid_environment_values() {
    let inputs = BTreeMap::from([
        ("sequence".to_string(), StaticValue::Sequence(Vec::new())),
        ("mapping".to_string(), StaticValue::Mapping),
        ("non-stringable".to_string(), StaticValue::NonStringable),
        ("invalid".to_string(), StaticValue::Invalid),
    ]);

    for (name, expected) in [
        ("sequence", StaticValue::Sequence(Vec::new())),
        ("mapping", StaticValue::Mapping),
        ("non-stringable", StaticValue::NonStringable),
        ("invalid", StaticValue::Invalid),
    ] {
        assert_eq!(
            environment_value(
                &Value::String(format!("${{{{ inputs.{name} }}}}")),
                &SecretState::direct(),
                &inputs,
                &EnvironmentState::default(),
            ),
            expected,
            "{name}",
        );
    }
}

#[test]
fn inner_environment_scopes_resolve_outer_values_without_self_references() {
    let inputs = BTreeMap::new();
    let workflow: Value = serde_yaml::from_str("env: {OUTER: true}").unwrap();
    let job: Value = serde_yaml::from_str("env: {FROM_WORKFLOW: '${{ env.OUTER }}'}").unwrap();
    let step: Value = serde_yaml::from_str("env: {FROM_JOB: '${{ env.FROM_WORKFLOW }}'}").unwrap();

    let environment = EnvironmentState::from_workflow(&workflow, &SecretState::direct(), &inputs)
        .with_job(&job, &inputs)
        .with_step(&step, &inputs);

    assert_eq!(
        environment.value("from_job"),
        Some(StaticValue::String("true".into()))
    );
}

#[test]
fn inner_environment_scopes_override_outer_scopes_for_the_active_inputs() {
    let inputs = BTreeMap::from([("enabled".to_string(), StaticValue::Bool(false))]);
    let workflow: Value = serde_yaml::from_str("env: {ENABLED: '${{ inputs.enabled }}'}").unwrap();
    let job: Value = serde_yaml::from_str("env: {ENABLED: job}").unwrap();
    let step: Value = serde_yaml::from_str("env: {ENABLED: '${{ inputs.enabled }}'}").unwrap();

    let environment = EnvironmentState::from_workflow(&workflow, &SecretState::direct(), &inputs)
        .with_job(&job, &inputs)
        .with_step(&step, &inputs);

    assert_eq!(
        environment.value("enabled"),
        Some(StaticValue::String("false".into()))
    );
}
