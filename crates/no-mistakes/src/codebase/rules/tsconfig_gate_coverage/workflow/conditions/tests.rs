use super::{continues_after_skipped_need, static_bool, InputState, StaticBool, StaticValue};
use serde_yaml::Value;

#[test]
fn truthy_nonboolean_values_preserve_expression_semantics() {
    assert_eq!(StaticBool::TruthyNonBoolean.negate(), StaticBool::False);
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
        ("disabled".into(), StaticValue::Bool(false)),
        ("enabled".into(), StaticValue::Bool(true)),
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
    let inputs = InputState::from([("enabled".into(), StaticValue::Bool(true))]);
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

#[test]
fn input_comparisons_preserve_static_scalar_values() {
    let inputs = InputState::from([
        ("label".into(), StaticValue::String("Release".into())),
        ("count".into(), StaticValue::Number("2".into())),
        ("dynamic".into(), StaticValue::Unknown),
    ]);
    for (expression, expected) in [
        ("inputs.label == 'release'", StaticBool::True),
        ("'RELEASE' != inputs.label", StaticBool::False),
        ("inputs.count == 2", StaticBool::True),
        ("inputs.count != 1e2", StaticBool::True),
        ("0 != inputs.count", StaticBool::True),
        ("inputs.count == '2'", StaticBool::True),
        ("inputs.label == 0", StaticBool::False),
        ("inputs.dynamic == 'release'", StaticBool::Unknown),
        ("inputs.count == NaN", StaticBool::Unknown),
        ("inputs.count == inf", StaticBool::Unknown),
        ("inputs.label == 'release == candidate'", StaticBool::False),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn status_conditions_model_the_successful_gate_path() {
    let inputs = InputState::new();
    for expression in [
        "failure()",
        "${{ cancelled() }}",
        "!success()",
        "!!failure()",
        "!(success())",
        "failure() == true",
        "cancelled() || false",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::False,
            "{expression}"
        );
    }
    for expression in [
        "success()",
        "${{ always() }}",
        "!cancelled()",
        "!!success()",
        "!(failure())",
        "failure() != true",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::True,
            "{expression}"
        );
    }
}

#[test]
fn skipped_needs_continue_only_for_statically_true_status_conditions() {
    let inputs = InputState::new();
    let job = |expression: &str| {
        Value::Mapping(serde_yaml::Mapping::from_iter([(
            Value::String("if".into()),
            Value::String(format!("${{{{ {expression} }}}}")),
        )]))
    };
    for expression in [
        "always() && true",
        "true && always()",
        "!cancelled() && true",
    ] {
        assert!(
            continues_after_skipped_need(&job(expression), &inputs),
            "{expression}"
        );
    }
    for expression in [
        "always() && success()",
        "!cancelled() && success()",
        "always() && false",
        "contains('always()', 'always')",
        "true",
    ] {
        assert!(
            !continues_after_skipped_need(&job(expression), &inputs),
            "{expression}"
        );
    }
}

#[test]
fn comparisons_cover_null_coercion_unicode_and_bracketed_inputs() {
    let inputs = InputState::from([
        ("enabled".into(), StaticValue::Bool(true)),
        ("empty".into(), StaticValue::String(String::new())),
        ("count".into(), StaticValue::Number("not-a-number".into())),
        ("café".into(), StaticValue::String("café".into())),
    ]);

    for (expression, expected) in [
        ("inputs['enabled'] == 1", StaticBool::True),
        ("inputs.empty == null", StaticBool::True),
        ("inputs.enabled == '1'", StaticBool::True),
        ("inputs.count == 1", StaticBool::Unknown),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
    assert_eq!(
        StaticValue::String("café".into()).equals(&StaticValue::String("café".into())),
        StaticBool::True
    );
    assert_eq!(
        StaticValue::String("café".into()).equals(&StaticValue::String("CAFÉ".into())),
        StaticBool::Unknown
    );
    assert_eq!(
        StaticValue::Null.equals(&StaticValue::Null),
        StaticBool::True
    );
}

#[test]
fn malformed_logical_and_literal_expressions_remain_unknown() {
    let inputs = InputState::new();
    for expression in [
        "true == false == true",
        "(true) && false)",
        "'unterminated",
        "0x",
        "0xnothex",
        "github.ref == 'main'",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::Unknown,
            "{expression}"
        );
    }
    assert_eq!(
        static_bool(Some(&Value::String("'nonempty'".into())), &inputs),
        StaticBool::TruthyNonBoolean
    );
}

#[test]
fn literal_and_parenthesized_condition_helpers_cover_static_paths() {
    let inputs = InputState::new();
    assert_eq!(StaticBool::TruthyNonBoolean.truthiness(), StaticBool::True);
    assert_eq!(StaticValue::Null.truthiness(), StaticBool::False);
    assert_eq!(
        super::hexadecimal_bool("0x1"),
        Some(StaticBool::TruthyNonBoolean)
    );
    assert_eq!(super::number_bool(Some(2.0)), StaticBool::TruthyNonBoolean);
    assert_eq!(super::number_bool(None), StaticBool::Unknown);
    for expression in ["(true)", "((true))", "(true"] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            if expression.ends_with(')') {
                StaticBool::True
            } else {
                StaticBool::Unknown
            },
            "{expression}"
        );
    }
}
