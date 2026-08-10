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

#[test]
fn compound_conditions_short_circuit_known_input_truthiness() {
    let inputs = InputState::from([
        ("disabled".into(), StaticBool::False),
        ("enabled".into(), StaticBool::True),
    ]);
    for expression in [
        "inputs.disabled && github.ref == 'refs/heads/main'",
        "github.ref == 'refs/heads/main' && inputs.disabled",
        "(inputs.disabled && github.ref == 'refs/heads/main')",
        "(inputs.enabled || false) && inputs.disabled",
        "github.ref == 'literal && text' && inputs.disabled",
        "contains('literal || text', 'literal') && inputs.disabled",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::False,
            "{expression}"
        );
    }
    for expression in [
        "inputs.enabled || github.ref == 'refs/heads/main'",
        "github.ref == 'refs/heads/main' || inputs.enabled",
        "inputs.enabled || false && inputs.disabled",
        "inputs['ENABLED'] || github.ref == 'refs/heads/main'",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::True,
            "{expression}"
        );
    }
    for expression in [
        "inputs.disabled || github.ref == 'refs/heads/main'",
        "inputs.enabled && github.ref == 'refs/heads/main'",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::Unknown,
            "{expression}"
        );
    }
}

#[test]
fn boolean_input_comparisons_accept_case_insensitive_literals() {
    let inputs = InputState::from([("enabled".into(), StaticBool::True)]);
    for (expression, expected) in [
        ("inputs.enabled == FALSE", StaticBool::False),
        ("TRUE == inputs.enabled", StaticBool::True),
        ("inputs.enabled != TRUE", StaticBool::False),
        ("FALSE != inputs.enabled", StaticBool::True),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}
