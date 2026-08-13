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
