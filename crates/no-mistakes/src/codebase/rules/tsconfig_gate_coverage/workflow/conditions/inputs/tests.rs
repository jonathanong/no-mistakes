use super::*;
use crate::codebase::workflow_topology::model::{WorkflowCallInput, WorkflowCallSecret};

#[test]
fn nonboolean_default_truthiness_handles_every_scalar_variant() {
    assert_eq!(default_falsy_state(None), StaticBool::False);
    assert_eq!(
        default_falsy_state(Some(&JsonScalar::Bool(false))),
        StaticBool::False
    );
    assert_eq!(
        default_falsy_state(Some(&JsonScalar::Bool(true))),
        StaticBool::TruthyNonBoolean
    );
    assert_eq!(
        default_falsy_state(Some(&JsonScalar::Number(serde_json::Number::from(0)))),
        StaticBool::False
    );
    assert_eq!(
        default_falsy_state(Some(&JsonScalar::Text(String::new()))),
        StaticBool::False
    );
}

#[test]
fn complete_literal_bindings_preserve_nonboolean_truthiness() {
    for (value, expected) in [
        ("${{ '' }}", StaticBool::False),
        ("${{ 'value' }}", StaticBool::TruthyNonBoolean),
        ("${{ 0 }}", StaticBool::False),
        ("${{ (0) }}", StaticBool::False),
        ("${{ -0x0 }}", StaticBool::False),
        ("${{ ('') }}", StaticBool::False),
        ("${{ 1 }}", StaticBool::TruthyNonBoolean),
        ("${{ inputs.value }}", StaticBool::Unknown),
    ] {
        assert_eq!(
            nonboolean_binding_state(&Value::String(value.into())),
            expected
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
            WorkflowCallInputType::Boolean
        ));
    }
    assert!(binding_matches_type(
        &Value::String("${{ needs.detect.outputs.enabled }}".to_string()),
        WorkflowCallInputType::Boolean
    ));
    assert!(binding_matches_type(
        &Value::String("${{ format('{0}', inputs.enabled) }}".to_string()),
        WorkflowCallInputType::Boolean
    ));
    assert!(binding_matches_type(
        &Value::String("${{ format('it''s {0}', inputs.enabled) }}".to_string()),
        WorkflowCallInputType::Boolean
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
            input_type
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
            input_type
        ));
    }
}
