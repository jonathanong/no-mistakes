use super::{
    complete_expression_static_value_with_environment, evaluation::static_bool, EnvironmentState,
    InputState, StaticBool, StaticValue,
};
use serde_yaml::Value;

#[test]
fn ref_comparisons_apply_known_exclusions_and_reference_kinds() {
    let excluded = InputState::from([(
        super::inputs::REF_EXCLUSIONS_KEY.to_string(),
        StaticValue::Sequence(vec![StaticValue::String("refs/heads/main".into())]),
    )]);
    assert_eq!(
        static_bool(
            Some(&Value::String("github.ref != 'refs/heads/main'".into())),
            &excluded,
        ),
        StaticBool::True
    );

    let tag = InputState::from([(
        super::inputs::REF_SHAPE_KEY.to_string(),
        StaticValue::String("tag".into()),
    )]);
    assert_eq!(
        static_bool(
            Some(&Value::String("github.ref != 'refs/heads/main'".into())),
            &tag,
        ),
        StaticBool::True
    );

    let malformed = InputState::from([(
        super::inputs::REF_SHAPE_KEY.to_string(),
        StaticValue::Bool(true),
    )]);
    assert_eq!(
        static_bool(
            Some(&Value::String("github.ref == 'refs/heads/main'".into())),
            &malformed,
        ),
        StaticBool::Unknown
    );
}

#[test]
fn ref_prefix_functions_use_every_known_reference_shape() {
    for (shape, prefix, expected) in [
        ("branch", "refs/heads/", StaticBool::True),
        ("tag", "refs/tags/", StaticBool::True),
        ("pull-request-merge", "refs/pull/", StaticBool::True),
        ("branch", "REFS/HEADS/", StaticBool::True),
        ("unsupported", "refs/heads/", StaticBool::Unknown),
    ] {
        let inputs = InputState::from([(
            super::inputs::REF_SHAPE_KEY.to_string(),
            StaticValue::String(shape.into()),
        )]);
        let expression = Value::String(format!("startsWith(github.ref, '{prefix}')"));
        assert_eq!(static_bool(Some(&expression), &inputs), expected, "{shape}");
    }

    assert_eq!(
        static_bool(
            Some(&Value::String(
                "startsWith(github.ref, fromJSON('not-json'))".into(),
            )),
            &InputState::new(),
        ),
        StaticBool::Invalid,
    );
}

#[test]
fn non_condition_functions_remain_unknown_in_condition_evaluation() {
    assert_eq!(
        static_bool(
            Some(&Value::String(
                "hashFiles('Cargo.lock', 'package-lock.json')".into()
            )),
            &InputState::new(),
        ),
        StaticBool::Unknown,
    );
}

#[test]
fn complete_expression_values_resolve_event_properties_and_conservative_json_inputs() {
    let inputs = InputState::from([
        (
            "\0github.event_name".into(),
            StaticValue::String("push".into()),
        ),
        (
            "\0github.event.action".into(),
            StaticValue::String("synchronize".into()),
        ),
        (
            "\0github.ref.kind".into(),
            StaticValue::String("branch".into()),
        ),
        (
            "\0github.base_ref".into(),
            StaticValue::String("main".into()),
        ),
        (
            "\0github.head_ref".into(),
            StaticValue::String("feature".into()),
        ),
        ("dynamic".into(), StaticValue::Unknown),
        (
            "array".into(),
            StaticValue::Sequence(vec![StaticValue::String("value".into())]),
        ),
    ]);
    let environment = EnvironmentState::default();
    for (expression, expected) in [
        (
            "${{ github.event_name }}",
            Some(StaticValue::String("push".into())),
        ),
        (
            "${{ github.event.action }}",
            Some(StaticValue::String("synchronize".into())),
        ),
        (
            "${{ github.ref_type }}",
            Some(StaticValue::String("branch".into())),
        ),
        (
            "${{ github.base_ref }}",
            Some(StaticValue::String("main".into())),
        ),
        (
            "${{ github.event.pull_request.base.ref }}",
            Some(StaticValue::String("main".into())),
        ),
        (
            "${{ github.head_ref }}",
            Some(StaticValue::String("feature".into())),
        ),
        (
            "${{ toJSON(github.event_name) }}",
            Some(StaticValue::String("\"push\"".into())),
        ),
        ("${{ fromJSON(inputs.dynamic) }}", None),
        (
            "${{ fromJSON(inputs.array) }}",
            Some(StaticValue::NonStringable),
        ),
    ] {
        assert_eq!(
            complete_expression_static_value_with_environment(expression, &inputs, &environment),
            expected,
            "{expression}"
        );
    }
}
