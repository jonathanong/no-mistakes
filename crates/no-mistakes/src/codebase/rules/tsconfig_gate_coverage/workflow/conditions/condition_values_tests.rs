use super::{
    condition_values::condition_value, functions::static_case_value, static_bool,
    static_value_string, EnvironmentState, InputState, StaticBool, StaticValue,
};
use serde_yaml::Value;

#[test]
fn truthy_nonboolean_values_preserve_expression_semantics() {
    assert_eq!(StaticBool::TruthyNonBoolean.negate(), StaticBool::False);
    for (value, truthiness) in [
        (
            StaticValue::Sequence(Vec::new()),
            StaticBool::TruthyNonBoolean,
        ),
        (StaticValue::NonStringable, StaticBool::Unknown),
    ] {
        assert_eq!(value.function_string(), None);
        assert_eq!(value.clone().truthiness(), truthiness);
        assert_eq!(
            value.less_than_or_equal(&StaticValue::Number("1".into())),
            StaticBool::False,
        );
    }
    assert_eq!(
        StaticValue::NonStringable.equals(&StaticValue::NonStringable),
        StaticBool::Unknown,
    );
}

#[test]
fn static_interpolation_values_follow_github_string_coercion() {
    for (value, expected) in [
        (Value::Bool(true), Some("true".to_string())),
        (Value::Number(42.into()), Some("42".to_string())),
        (Value::String("release".into()), Some("release".into())),
        (Value::Null, Some(String::new())),
        (Value::Sequence(Vec::new()), None),
        (Value::Mapping(Default::default()), None),
    ] {
        assert_eq!(static_value_string(value), expected);
    }
}

#[test]
fn statically_resolves_falsy_condition_literals() {
    let inputs = InputState::new();
    for value in [
        Value::String("".into()),
        Value::String("${{ '' }}".into()),
        Value::String("0".into()),
        Value::String("${{ 0 }}".into()),
        Value::String("0x0".into()),
        Value::String("${{ -0x0 }}".into()),
        Value::String("null".into()),
        Value::String("${{ null }}".into()),
        Value::Number(0.into()),
        Value::Null,
    ] {
        assert_eq!(
            static_bool(Some(&value), &inputs),
            StaticBool::False,
            "{value:?}"
        );
    }
}

#[test]
fn condition_values_preserve_known_status_and_short_circuit_operands() {
    let inputs = InputState::new();
    assert_eq!(
        condition_value(
            "success()",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True
        ),
        Some(StaticValue::Bool(true))
    );
    assert_eq!(
        condition_value(
            "true || github.ref",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True
        ),
        Some(StaticValue::Bool(true))
    );
    assert_eq!(
        condition_value(
            "false && github.ref",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True
        ),
        Some(StaticValue::Bool(false))
    );
}

#[test]
fn condition_values_remain_unknown_for_unknown_status_and_operands() {
    let inputs = InputState::from([("dynamic".into(), StaticValue::Unknown)]);
    assert_eq!(StaticValue::Unknown.function_string(), None);
    assert_eq!(
        StaticValue::Unknown.less_than_or_equal(&StaticValue::Number("1".into())),
        StaticBool::Unknown
    );
    assert_eq!(
        condition_value(
            "success()",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::Unknown
        ),
        None
    );
    assert_eq!(
        condition_value(
            "inputs.dynamic || false",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True
        ),
        None
    );
    assert_eq!(
        condition_value(
            "!github.ref",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True
        ),
        Some(StaticValue::Unknown)
    );
}

#[test]
fn unary_boolean_values_and_case_predicates_cover_scalar_conversion() {
    let inputs = InputState::from([("dynamic".into(), StaticValue::Unknown)]);
    assert_eq!(
        condition_value(
            "fromJSON('null')",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True
        ),
        Some(StaticValue::Null)
    );
    assert_eq!(
        condition_value(
            "fromJSON('[]')",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True
        ),
        Some(StaticValue::Sequence(Vec::new()))
    );
    assert_eq!(
        condition_value(
            "!true",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True
        ),
        Some(StaticValue::Bool(false))
    );
    assert_eq!(
        static_case_value(
            "case(inputs.dynamic, 'selected', 'default')",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True
        ),
        None
    );
}
