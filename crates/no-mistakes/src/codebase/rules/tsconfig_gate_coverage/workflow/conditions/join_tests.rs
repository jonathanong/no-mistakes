use super::{
    evaluation::static_bool, functions::static_join_value, EnvironmentState, InputState,
    StaticBool, StaticValue,
};
use serde_yaml::Value;

#[test]
fn static_join_value_resolves_known_scalars_and_sequences() {
    let inputs = InputState::from([
        ("label".into(), StaticValue::String("release".into())),
        ("dynamic".into(), StaticValue::Unknown),
    ]);
    for (expression, expected) in [
        (
            "join(fromJSON('[\"release\", \"candidate\"]'), '-')",
            Some(StaticValue::String("release-candidate".into())),
        ),
        (
            "join(fromJSON('[\"release\", \"candidate\"]'))",
            Some(StaticValue::String("release,candidate".into())),
        ),
        (
            "join(fromJSON('[true, null, 2]'), ':')",
            Some(StaticValue::String("true::2".into())),
        ),
        (
            "join(inputs.label, '-')",
            Some(StaticValue::String("release".into())),
        ),
        (
            "join(fromJSON('[1.2345678901234567]'))",
            Some(StaticValue::String("1.23456789012346".into())),
        ),
        (
            "join(fromJSON('[\"a\", \"b\"]'), 1.2345678901234567)",
            Some(StaticValue::String("a1.23456789012346b".into())),
        ),
        ("join(fromJSON('[\"release\"]'), inputs.dynamic)", None),
        ("join(fromJSON('[]'), inputs.dynamic)", None),
        (
            "join(fromJSON('[]'), fromJSON('not-json'))",
            Some(StaticValue::Invalid),
        ),
        (
            "join(fromJSON('[\"release\"]'), fromJSON('not-json'))",
            Some(StaticValue::Invalid),
        ),
    ] {
        assert_eq!(
            static_join_value(
                expression,
                &inputs,
                &EnvironmentState::default(),
                StaticBool::True,
            ),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn join_conditions_preserve_unknown_for_dynamic_or_non_scalar_values() {
    let inputs = InputState::from([
        ("dynamic".into(), StaticValue::Unknown),
        ("mapping".into(), StaticValue::Mapping),
    ]);
    for expression in [
        "join(inputs.dynamic, ',') == 'release'",
        "join(fromJSON('[\"release\", \"candidate\"]'), inputs.dynamic) == 'release'",
        "join(fromJSON('[{\"name\": \"release\"}]'), ',') == 'release'",
        "join(inputs.mapping, ',') == 'release'",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::Unknown,
            "{expression}"
        );
    }
}
