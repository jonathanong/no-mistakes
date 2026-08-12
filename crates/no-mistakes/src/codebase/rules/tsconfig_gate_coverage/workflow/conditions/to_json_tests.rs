use super::{
    condition_values::condition_value, EnvironmentState, InputState, StaticBool, StaticValue,
};

#[test]
fn to_json_serializes_known_scalars_arrays_and_literal_mappings() {
    let inputs = InputState::from([("dynamic".into(), StaticValue::Unknown)]);
    for (expression, expected) in [
        (
            "toJSON('release')",
            Some(StaticValue::String("\"release\"".into())),
        ),
        ("toJSON(true)", Some(StaticValue::String("true".into()))),
        ("toJSON(null)", Some(StaticValue::String("null".into()))),
        ("toJSON(0xff)", Some(StaticValue::String("255".into()))),
        (
            "toJSON(fromJSON('[true, 2, null]'))",
            Some(StaticValue::String("[\n  true,\n  2,\n  null\n]".into())),
        ),
        ("toJSON(inputs.dynamic)", None),
        ("toJSON(fromJSON('not-json'))", Some(StaticValue::Invalid)),
        (
            "toJSON(fromJSON('{\"name\":\"release\"}'))",
            Some(StaticValue::String("{\n  \"name\": \"release\"\n}".into())),
        ),
    ] {
        assert_eq!(
            condition_value(
                expression,
                &inputs,
                &EnvironmentState::default(),
                StaticBool::True
            ),
            expected,
            "{expression}"
        );
    }
}
