use super::{
    condition_values::condition_value, static_json::to_json_static_value, EnvironmentState,
    InputState, StaticBool, StaticValue,
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

#[test]
fn to_json_propagates_invalid_and_unresolved_nested_values() {
    assert_eq!(
        to_json_static_value(StaticValue::Sequence(vec![
            StaticValue::Bool(true),
            StaticValue::String("release".into()),
            StaticValue::Number("2".into()),
            StaticValue::Null,
        ])),
        Some(StaticValue::String(
            "[\n  true,\n  \"release\",\n  2,\n  null\n]".into()
        ))
    );

    for value in [
        StaticValue::Mapping,
        StaticValue::MatrixMapping("matrix.axis".into()),
        StaticValue::NonStringable,
        StaticValue::Unknown,
        StaticValue::Sequence(vec![StaticValue::Unknown]),
    ] {
        assert_eq!(to_json_static_value(value), None);
    }

    for value in [
        StaticValue::Invalid,
        StaticValue::Number("not-a-number".into()),
        StaticValue::Sequence(vec![StaticValue::Invalid]),
    ] {
        assert_eq!(to_json_static_value(value), Some(StaticValue::Invalid));
    }
}
