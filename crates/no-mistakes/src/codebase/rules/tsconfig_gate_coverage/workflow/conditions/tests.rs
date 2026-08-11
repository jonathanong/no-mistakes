use super::{
    continues_after_skipped_need,
    evaluation::static_bool,
    expression_bool,
    literals::{hexadecimal_bool, number_bool},
    step_timeout_minutes_enforced, InputState, StaticBool, StaticValue,
};
use serde_yaml::Value;

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
fn literal_comparisons_resolve_without_context_values() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("1 == 2", StaticBool::False),
        ("'production' == 'staging'", StaticBool::False),
        ("1 == 1", StaticBool::True),
        ("'production' == 'production'", StaticBool::True),
        ("'it''s' == 'it''s'", StaticBool::True),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn comparisons_resolve_known_compound_unary_and_function_operands() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("(false || false) == true", StaticBool::False),
        ("!(false) == true", StaticBool::True),
        ("contains('release', 'LEASE') == true", StaticBool::True),
        ("(false || false) != false", StaticBool::False),
        ("(false || 'release') == true", StaticBool::False),
        ("(true && 'release') == true", StaticBool::False),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn case_functions_select_known_predicate_branches_and_defaults() {
    let inputs = InputState::new();
    for (expression, expected) in [
        (
            "case(true, 'release', 'nightly') == 'release'",
            StaticBool::True,
        ),
        (
            "case(false, 'nightly', true, 'release', 'other') == 'release'",
            StaticBool::True,
        ),
        (
            "case(false, 'nightly', 'release') == true",
            StaticBool::False,
        ),
        (
            "case(github.ref == 'refs/heads/main', true, false) == true",
            StaticBool::Unknown,
        ),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn literal_from_json_conditions_preserve_scalar_truthiness_and_comparisons() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("fromJSON('false')", StaticBool::False),
        ("${{ fromJSON('false') }}", StaticBool::False),
        ("fromJSON('0')", StaticBool::False),
        ("fromJSON('true')", StaticBool::True),
        ("fromJSON('1')", StaticBool::True),
        ("fromJSON('\"release\"')", StaticBool::True),
        ("fromJSON('false') == false", StaticBool::True),
        ("fromJSON('0') == 0", StaticBool::True),
        ("fromJSON('true') == false", StaticBool::False),
        ("fromJSON('not-json')", StaticBool::Unknown),
        ("fromJSON('{}')", StaticBool::Unknown),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn comparisons_resolve_static_subexpressions_without_crediting_dynamic_values() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("${{ (false || false) == true }}", StaticBool::False),
        ("${{ (!false) == true }}", StaticBool::True),
        (
            "${{ startsWith('release', 'rel') == true }}",
            StaticBool::True,
        ),
        ("${{ (github.ref || false) == true }}", StaticBool::Unknown),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn deterministic_string_functions_resolve_static_arguments() {
    let inputs = InputState::from([("label".into(), StaticValue::String("Release".into()))]);
    for (expression, expected) in [
        ("contains('Hello world', 'LLO')", StaticBool::True),
        ("contains('Hello world', 'nope')", StaticBool::False),
        ("startsWith(inputs.label, 're')", StaticBool::True),
        ("startsWith(inputs.label, 'lease')", StaticBool::False),
        ("endsWith('release.TS', '.ts')", StaticBool::True),
        ("endsWith('release.TS', '.js')", StaticBool::False),
        ("contains(true, 'RUE')", StaticBool::True),
        ("startsWith(0x10, '16')", StaticBool::True),
        ("endsWith(null, '')", StaticBool::True),
        ("contains('it''s, nested', 'S, N')", StaticBool::True),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn deterministic_string_functions_fail_open_for_dynamic_or_unmodeled_arguments() {
    let inputs = InputState::new();
    for expression in [
        "contains(github.ref, 'main')",
        "contains(format('a,{0}', 'b'), 'a,b')",
        "startsWith('M\u{00f6}na', 'm')",
        "endsWith('release', github.ref)",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::Unknown,
            "{expression}"
        );
    }
}

#[test]
fn missing_inputs_coerce_to_empty_strings_only_for_string_functions() {
    let inputs = InputState::from([("present".into(), StaticValue::String("release".into()))]);
    for (expression, expected) in [
        ("contains(inputs.missing, 'false')", StaticBool::False),
        ("startsWith(inputs.missing, 'f')", StaticBool::False),
        ("endsWith(inputs.missing, 'e')", StaticBool::False),
        ("contains(inputs.missing, '')", StaticBool::True),
        ("startsWith(inputs.missing, '')", StaticBool::True),
        ("endsWith(inputs.missing, '')", StaticBool::True),
        ("contains(inputs.present, 'LEASE')", StaticBool::True),
        ("startsWith(inputs.present, 'RE')", StaticBool::True),
        ("endsWith(inputs.present, 'ASE')", StaticBool::True),
        ("inputs.missing == false", StaticBool::True),
        ("inputs.missing", StaticBool::False),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn literal_relational_comparisons_resolve_without_context_values() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("1 < 0", StaticBool::False),
        ("1 <= 0", StaticBool::False),
        ("0 > 1", StaticBool::False),
        ("0 >= 1", StaticBool::False),
        ("0 < 1", StaticBool::True),
        ("0 <= 0", StaticBool::True),
        ("1 > 0", StaticBool::True),
        ("1 >= 1", StaticBool::True),
        ("'release' < 1", StaticBool::False),
        ("'release' >= 1", StaticBool::False),
        ("github.run_number > 0", StaticBool::Unknown),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn negated_parenthesized_inputs_resolve_at_any_supported_depth() {
    let inputs = InputState::from([("enabled".into(), StaticValue::Bool(true))]);
    for expression in ["!(inputs.enabled)", "!((inputs.enabled))"] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::False,
            "{expression}"
        );
    }
}

#[test]
fn timeout_validation_rejects_non_numeric_values_and_negates_dynamic_values() {
    let inputs = InputState::new();
    assert!(!step_timeout_minutes_enforced(
        Some(&Value::Bool(true)),
        &inputs,
    ));
    assert!(!step_timeout_minutes_enforced(
        Some(&Value::Number(361.into())),
        &inputs,
    ));
    assert_eq!(expression_bool("!github.ref", &inputs), StaticBool::Unknown);
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
fn bracketed_matrix_properties_match_dot_access() {
    let inputs = InputState::from([
        ("enabled".into(), StaticValue::Bool(true)),
        ("\0matrix.enabled".into(), StaticValue::Bool(true)),
    ]);
    for expression in [
        "matrix.enabled",
        "matrix.ENABLED",
        "matrix['enabled']",
        "matrix [ 'ENABLED' ]",
        "MATRIX . ENABLED",
        "MATRIX [ 'ENABLED' ]",
        "INPUTS . ENABLED",
        "INPUTS [ 'ENABLED' ]",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::True,
            "{expression}"
        );
    }
    for expression in [
        "matrix[\"enabled\"]",
        "matrix['enabled'].nested",
        "matrix['enabled']['nested']",
        "matrix['not valid']",
        "matrix[enabled]",
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            StaticBool::Unknown,
            "{expression}"
        );
    }
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
    assert_eq!(hexadecimal_bool("0x1"), Some(StaticBool::TruthyNonBoolean));
    assert_eq!(number_bool(Some(2.0)), StaticBool::TruthyNonBoolean);
    assert_eq!(number_bool(None), StaticBool::Unknown);
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
