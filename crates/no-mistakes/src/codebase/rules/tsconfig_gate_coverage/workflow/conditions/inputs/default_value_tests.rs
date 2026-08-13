use super::*;

#[test]
fn nonboolean_defaults_preserve_scalar_values() {
    assert_eq!(
        default_value(None, WorkflowCallInputType::String, &InputState::new()),
        StaticValue::String(String::new())
    );
    assert_eq!(
        default_value(None, WorkflowCallInputType::Number, &InputState::new()),
        StaticValue::Number("0".into())
    );
    assert_eq!(
        default_value(
            Some(&JsonScalar::Number(serde_json::Number::from(2))),
            WorkflowCallInputType::Number,
            &InputState::new(),
        ),
        StaticValue::Number("2".into())
    );
    assert_eq!(
        default_value(
            Some(&JsonScalar::Text("release".into())),
            WorkflowCallInputType::String,
            &InputState::new(),
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
            default_value(
                Some(&JsonScalar::Text(value.into())),
                input_type,
                &InputState::new(),
            ),
            expected,
            "{value}"
        );
    }
}
