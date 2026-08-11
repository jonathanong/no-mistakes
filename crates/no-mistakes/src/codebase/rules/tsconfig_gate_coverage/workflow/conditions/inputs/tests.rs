use super::*;
use crate::codebase::workflow_topology::model::{WorkflowCallInput, WorkflowCallSecret};

#[test]
fn nonboolean_defaults_preserve_scalar_values() {
    assert_eq!(
        default_value(None, WorkflowCallInputType::String),
        StaticValue::String(String::new())
    );
    assert_eq!(
        default_value(None, WorkflowCallInputType::Number),
        StaticValue::Number("0".into())
    );
    assert_eq!(
        default_value(
            Some(&JsonScalar::Number(serde_json::Number::from(2))),
            WorkflowCallInputType::Number
        ),
        StaticValue::Number("2".into())
    );
    assert_eq!(
        default_value(
            Some(&JsonScalar::Text("release".into())),
            WorkflowCallInputType::String
        ),
        StaticValue::String("release".into())
    );
    for (value, input_type, expected) in [
        (
            "${{ false }}",
            WorkflowCallInputType::Boolean,
            StaticValue::Bool(false),
        ),
        (
            "${{ 2 }}",
            WorkflowCallInputType::Number,
            StaticValue::Number("2".into()),
        ),
        (
            "${{ vars.LABEL }}",
            WorkflowCallInputType::String,
            StaticValue::Unknown,
        ),
        (
            "${{ true == false }}",
            WorkflowCallInputType::Boolean,
            StaticValue::Bool(false),
        ),
        (
            "${{ true && false }}",
            WorkflowCallInputType::Boolean,
            StaticValue::Bool(false),
        ),
        (
            "${{ contains('x', 'y') }}",
            WorkflowCallInputType::Boolean,
            StaticValue::Bool(false),
        ),
        (
            "${{ fromJSON('false') }}",
            WorkflowCallInputType::Boolean,
            StaticValue::Bool(false),
        ),
        (
            "${{ fromJSON('0') }}",
            WorkflowCallInputType::Number,
            StaticValue::Number("0".into()),
        ),
        (
            "${{ vars.FLAG == true }}",
            WorkflowCallInputType::Boolean,
            StaticValue::Unknown,
        ),
        (
            "release-${{ github.ref_name }}",
            WorkflowCallInputType::String,
            StaticValue::Unknown,
        ),
    ] {
        assert_eq!(
            default_value(Some(&JsonScalar::Text(value.into())), input_type),
            expected,
            "{value}"
        );
    }
}

#[test]
fn direct_event_inputs_use_declared_type_empty_states() {
    let contract = WorkflowCallContract {
        inputs: BTreeMap::from([
            (
                "enabled".to_string(),
                WorkflowCallInput {
                    input_type: Some(WorkflowCallInputType::Boolean),
                    required: false,
                    default: None,
                    description: None,
                },
            ),
            (
                "attempts".to_string(),
                WorkflowCallInput {
                    input_type: Some(WorkflowCallInputType::Number),
                    required: false,
                    default: None,
                    description: None,
                },
            ),
            (
                "label".to_string(),
                WorkflowCallInput {
                    input_type: Some(WorkflowCallInputType::String),
                    required: false,
                    default: None,
                    description: None,
                },
            ),
        ]),
        ..WorkflowCallContract::default()
    };

    assert_eq!(
        direct_inputs(Some(&contract), "push", None),
        Some(InputState::from([
            ("enabled".to_string(), StaticValue::Bool(false)),
            ("attempts".to_string(), StaticValue::Number("0".to_string())),
            ("label".to_string(), StaticValue::String(String::new())),
            (
                "\0github.event_name".to_string(),
                StaticValue::String("push".to_string()),
            ),
        ]))
    );
}

#[test]
fn complete_literal_bindings_preserve_nonboolean_values() {
    for (value, input_type, expected) in [
        (
            "${{ '' }}",
            WorkflowCallInputType::String,
            StaticValue::String(String::new()),
        ),
        (
            "${{ 'value' }}",
            WorkflowCallInputType::String,
            StaticValue::String("value".into()),
        ),
        (
            "${{ 0 }}",
            WorkflowCallInputType::Number,
            StaticValue::Number("0".into()),
        ),
        (
            "${{ (0) }}",
            WorkflowCallInputType::Number,
            StaticValue::Number("0".into()),
        ),
        (
            "${{ -0x0 }}",
            WorkflowCallInputType::Number,
            StaticValue::Number("-0x0".into()),
        ),
        (
            "${{ 1 }}",
            WorkflowCallInputType::Number,
            StaticValue::Number("1".into()),
        ),
        (
            "${{ inputs.value }}",
            WorkflowCallInputType::String,
            StaticValue::Unknown,
        ),
    ] {
        assert_eq!(
            nonboolean_binding_value(&Value::String(value.into()), &InputState::new(), input_type),
            expected
        );
    }
    assert_eq!(
        nonboolean_binding_value(
            &Value::String("release-${{ github.ref_name }}".into()),
            &InputState::new(),
            WorkflowCallInputType::String,
        ),
        StaticValue::Unknown
    );
}

#[test]
fn direct_input_bindings_preserve_parent_nonboolean_values() {
    let parent = InputState::from([
        ("empty".to_string(), StaticValue::String(String::new())),
        ("zero".to_string(), StaticValue::Number("0".into())),
        ("full".to_string(), StaticValue::String("release".into())),
        ("dynamic".to_string(), StaticValue::Unknown),
    ]);

    for (binding, expected) in [
        ("${{ inputs.empty }}", StaticValue::String(String::new())),
        ("${{ inputs.zero }}", StaticValue::Number("0".into())),
        ("${{ inputs.full }}", StaticValue::String("release".into())),
        ("${{ inputs.dynamic }}", StaticValue::Unknown),
    ] {
        assert_eq!(
            nonboolean_binding_value(
                &Value::String(binding.into()),
                &parent,
                WorkflowCallInputType::String
            ),
            expected
        );
    }
}

#[test]
fn exact_forwarding_requires_compatible_input_types() {
    let parent = InputState::from([
        ("boolean".into(), StaticValue::Bool(true)),
        ("number".into(), StaticValue::Number("2".into())),
        ("string".into(), StaticValue::String("release".into())),
        ("dynamic".into(), StaticValue::Unknown),
    ]);
    for (name, input_type, expected) in [
        ("boolean", WorkflowCallInputType::Boolean, true),
        ("boolean", WorkflowCallInputType::Number, false),
        ("number", WorkflowCallInputType::Number, true),
        ("number", WorkflowCallInputType::String, false),
        ("string", WorkflowCallInputType::String, true),
        ("string", WorkflowCallInputType::Boolean, false),
        ("dynamic", WorkflowCallInputType::Boolean, true),
    ] {
        assert_eq!(
            binding_matches_type(
                &Value::String(format!("${{{{ inputs.{name} }}}}")),
                input_type,
                &parent,
            ),
            expected,
            "{name:?} forwarded into {input_type:?}"
        );
    }
}

#[test]
fn malformed_call_input_bindings_are_rejected() {
    let missing_type = WorkflowCallContract {
        inputs: BTreeMap::from([(
            "enabled".to_string(),
            WorkflowCallInput {
                input_type: None,
                required: false,
                default: None,
                description: None,
            },
        )]),
        ..WorkflowCallContract::default()
    };
    let call_job: Value = serde_yaml::from_str("with: true").expect("valid test YAML");

    assert!(inputs_from_contract(&missing_type, None, &InputState::new()).is_none());

    let valid = WorkflowCallContract {
        inputs: BTreeMap::from([(
            "enabled".to_string(),
            WorkflowCallInput {
                input_type: Some(WorkflowCallInputType::Boolean),
                required: false,
                default: None,
                description: None,
            },
        )]),
        ..WorkflowCallContract::default()
    };
    assert!(callee_inputs(Some(&valid), &call_job, &InputState::new()).is_none());
}

#[test]
fn malformed_secret_bindings_are_rejected() {
    let contract = WorkflowCallContract {
        secrets: BTreeMap::from([(
            "token".to_string(),
            WorkflowCallSecret {
                required: false,
                description: None,
            },
        )]),
        ..WorkflowCallContract::default()
    };

    for source in [
        "secrets: true",
        "secrets:\n  token: []",
        "secrets:\n  token: first\n  TOKEN: second",
    ] {
        let call_job: Value = serde_yaml::from_str(source).expect("valid test YAML");
        assert!(
            callee_secrets(&contract, &call_job, &SecretState::direct()).is_none(),
            "{source}"
        );
    }
}

#[test]
fn malformed_complete_expressions_do_not_bypass_input_types() {
    for value in [
        "${{ }}",
        "${{ true }}${{ false }}",
        "${{ true }}}",
        "${{{ true }}",
        "${{ true } invalid }}",
        "${{ true && }}",
        "${{ 'unterminated }}",
    ] {
        assert!(!binding_matches_type(
            &Value::String(value.to_string()),
            WorkflowCallInputType::Boolean,
            &InputState::new()
        ));
    }
    assert!(binding_matches_type(
        &Value::String("${{ needs.detect.outputs.enabled }}".to_string()),
        WorkflowCallInputType::Boolean,
        &InputState::new()
    ));
    assert!(!binding_matches_type(
        &Value::String("${{ format('{0}', inputs.enabled) }}".to_string()),
        WorkflowCallInputType::Boolean,
        &InputState::new()
    ));
    assert!(!binding_matches_type(
        &Value::String("${{ format('it''s {0}', inputs.enabled) }}".to_string()),
        WorkflowCallInputType::Boolean,
        &InputState::new()
    ));
}

#[test]
fn reusable_call_input_bindings_allow_only_available_contexts() {
    // Numeric literals are not contexts, including the `x1` suffix of hex.
    assert!(binding_matches_type(
        &Value::String("${{ 0x1 }}".to_string()),
        WorkflowCallInputType::Number,
        &InputState::new()
    ));
    for value in [
        "${{ github.ref }}",
        "${{ needs.setup.outputs.enabled }}",
        "${{ strategy.job-index }}",
        "${{ inputs.enabled }}",
        "${{ vars.TYPECHECK_MODE }}",
    ] {
        assert!(binding_matches_type(
            &Value::String(value.to_string()),
            WorkflowCallInputType::Boolean,
            &InputState::new()
        ));
    }
    // A syntactically valid but unavailable matrix property resolves to an
    // empty string, so it cannot bind to a boolean input.
    assert!(!binding_matches_type(
        &Value::String("${{ matrix.node }}".to_string()),
        WorkflowCallInputType::Boolean,
        &InputState::new()
    ));
    assert!(binding_matches_type(
        &Value::String("${{ matrix.node }}".to_string()),
        WorkflowCallInputType::String,
        &InputState::new()
    ));
    for value in [
        "${{ secrets.TOKEN }}",
        "${{ secrets.TOKEN == 'enabled' }}",
        "${{ env.TYPECHECK_MODE }}",
        "${{ job.status }}",
        "${{ jobs.typecheck.outputs.enabled }}",
        "${{ runner.os }}",
        "${{ steps.setup.outputs.enabled }}",
        "${{ hashFiles('**/pnpm-lock.yaml') }}",
        "${{ success() }}",
    ] {
        assert!(
            !binding_matches_type(
                &Value::String(value.to_string()),
                WorkflowCallInputType::Boolean,
                &InputState::new()
            ),
            "{value}"
        );
    }
    assert!(binding_matches_type(
        &Value::String("release-${{ github.ref_name }}".into()),
        WorkflowCallInputType::String,
        &InputState::new()
    ));
    assert!(!binding_matches_type(
        &Value::String("release-${{ secrets.TOKEN }}".into()),
        WorkflowCallInputType::String,
        &InputState::new()
    ));
    assert!(!binding_matches_type(
        &Value::String("release-${{ hashFiles('**/pnpm-lock.yaml') }}".into()),
        WorkflowCallInputType::String,
        &InputState::new()
    ));
}

#[test]
fn static_values_cover_event_matrix_and_untyped_scalar_paths() {
    let parent = InputState::from([
        (
            "\0github.event_name".into(),
            StaticValue::String("push".into()),
        ),
        ("\0matrix.enabled".into(), StaticValue::Bool(true)),
    ]);

    assert_eq!(
        super::values::forwarded_input_value(
            &Value::String("${{ github.event_name }}".into()),
            &parent,
        ),
        Some(StaticValue::String("push".into()))
    );
    for expression in [
        "${{ github['event_name'] }}",
        "${{ GiThUb [ 'EVENT_NAME' ] }}",
    ] {
        assert_eq!(
            super::values::forwarded_input_value(&Value::String(expression.into()), &parent),
            Some(StaticValue::String("push".into())),
            "{expression}"
        );
    }
    assert_eq!(
        super::values::forwarded_input_value(
            &Value::String("${{ github['event_name' }}".into()),
            &parent,
        ),
        None
    );
    assert_eq!(
        super::values::forwarded_input_value(
            &Value::String("${{ matrix.enabled }}".into()),
            &parent,
        ),
        Some(StaticValue::Bool(true))
    );
    for (value, expected) in [
        (Value::Bool(true), Some(StaticValue::Bool(true))),
        (
            Value::Number(2.into()),
            Some(StaticValue::Number("2".into())),
        ),
        (Value::Null, Some(StaticValue::Null)),
        (
            Value::String("${{ github.ref }}".into()),
            Some(StaticValue::Unknown),
        ),
        (Value::Sequence(Vec::new()), None),
    ] {
        assert_eq!(
            super::values::matrix_axis_value(&value),
            expected,
            "{value:?}"
        );
    }
    assert_eq!(
        nonboolean_binding_value(&Value::Bool(true), &parent, WorkflowCallInputType::String,),
        StaticValue::Unknown
    );
    assert_eq!(
        default_value(Some(&JsonScalar::Bool(true)), WorkflowCallInputType::String,),
        StaticValue::Unknown
    );
}

#[test]
fn boolean_bindings_reject_forwarded_nonbooleans_and_evaluate_conditions() {
    let parent = InputState::from([
        ("label".into(), StaticValue::String("release".into())),
        ("enabled".into(), StaticValue::Bool(true)),
        ("dynamic".into(), StaticValue::Unknown),
    ]);
    assert_eq!(
        super::bindings::binding_bool(&Value::String("${{ inputs.label }}".into()), &parent,),
        StaticValue::Unknown
    );
    assert_eq!(
        super::bindings::binding_bool(&Value::String("${{ inputs.dynamic }}".into()), &parent,),
        StaticValue::Unknown
    );
    assert_eq!(
        super::bindings::binding_bool(
            &Value::String("${{ inputs.enabled && true }}".into()),
            &parent,
        ),
        StaticValue::Bool(true)
    );
    assert_eq!(
        super::bindings::binding_bool(&Value::Number(1.into()), &parent),
        StaticValue::Bool(false)
    );
}
