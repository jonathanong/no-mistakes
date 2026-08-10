use super::StaticBool;
use super::{static_bool, InputState};
use serde_yaml::Value;

#[test]
fn truthy_nonboolean_values_preserve_expression_semantics() {
    assert_eq!(StaticBool::TruthyNonBoolean.negate(), StaticBool::False);
    assert_eq!(
        StaticBool::TruthyNonBoolean.equals(true),
        StaticBool::Unknown
    );
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
