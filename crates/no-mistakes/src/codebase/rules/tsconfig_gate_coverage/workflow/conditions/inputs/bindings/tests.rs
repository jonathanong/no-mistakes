use super::*;

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

#[test]
fn statically_structured_call_inputs_do_not_bind_to_scalar_contracts() {
    for value in [
        "${{ fromJSON('[true]') }}",
        "${{ fromJSON('{\"enabled\":true}') }}",
    ] {
        for input_type in [
            WorkflowCallInputType::Boolean,
            WorkflowCallInputType::Number,
            WorkflowCallInputType::String,
        ] {
            assert!(
                !binding_matches_type(
                    &Value::String(value.to_string()),
                    input_type,
                    &InputState::new()
                ),
                "{value} must not bind to {input_type:?}"
            );
        }
    }
}

#[test]
fn normalized_bindings_reject_duplicate_names_and_non_strings() {
    let mapping: serde_yaml::Mapping = serde_yaml::from_str("Name: a\nname: b\n").unwrap();
    assert!(normalized_bindings(&mapping).is_none());
    let mapping: serde_yaml::Mapping = serde_yaml::from_str("1: a\n").unwrap();
    assert!(normalized_bindings(&mapping).is_none());
}

#[test]
fn interpolated_string_bindings_only_match_string_inputs() {
    let value = Value::String("${{ format('{0}', inputs.name) }}".into());
    assert!(binding_matches_type(
        &value,
        WorkflowCallInputType::String,
        &InputState::new()
    ));
    assert!(!binding_matches_type(
        &value,
        WorkflowCallInputType::Boolean,
        &InputState::new()
    ));
}

#[test]
fn binding_bool_covers_literals_and_unknown_forwarded_values() {
    assert_eq!(
        binding_bool(&Value::Bool(true), &InputState::new()),
        StaticValue::Bool(true)
    );
    assert_eq!(
        binding_bool(&Value::String("not-a-bool".into()), &InputState::new()),
        StaticValue::Unknown
    );
}
