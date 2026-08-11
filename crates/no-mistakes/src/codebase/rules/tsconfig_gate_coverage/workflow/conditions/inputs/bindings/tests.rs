use super::*;

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
