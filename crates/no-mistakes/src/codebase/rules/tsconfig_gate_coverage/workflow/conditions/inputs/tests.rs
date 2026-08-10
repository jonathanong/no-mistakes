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
        assert!(!callee_secrets_valid(&contract, &call_job), "{source}");
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
    assert!(binding_matches_type(
        &Value::String("${{ format('{0}', inputs.enabled) }}".to_string()),
        WorkflowCallInputType::Boolean,
        &InputState::new()
    ));
    assert!(binding_matches_type(
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
        "${{ matrix.node }}",
        "${{ inputs.enabled }}",
        "${{ vars.TYPECHECK_MODE }}",
    ] {
        assert!(binding_matches_type(
            &Value::String(value.to_string()),
            WorkflowCallInputType::Boolean,
            &InputState::new()
        ));
    }
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
fn statically_typed_expression_bindings_must_match_declared_inputs() {
    for (value, input_type) in [
        ("${{ 'false' }}", WorkflowCallInputType::Boolean),
        ("${{ 1 }}", WorkflowCallInputType::Boolean),
        ("${{ false }}", WorkflowCallInputType::String),
        ("${{ '1' }}", WorkflowCallInputType::Number),
    ] {
        assert!(!binding_matches_type(
            &Value::String(value.to_string()),
            input_type,
            &InputState::new()
        ));
    }
    for (value, input_type) in [
        ("${{ false }}", WorkflowCallInputType::Boolean),
        ("${{ 1 }}", WorkflowCallInputType::Number),
        ("${{ 'value' }}", WorkflowCallInputType::String),
        (
            "${{ needs.setup.outputs.value }}",
            WorkflowCallInputType::Boolean,
        ),
    ] {
        assert!(binding_matches_type(
            &Value::String(value.to_string()),
            input_type,
            &InputState::new()
        ));
    }
}
