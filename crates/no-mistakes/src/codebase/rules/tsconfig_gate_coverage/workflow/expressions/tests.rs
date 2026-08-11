use super::interpolation::opaque_interpolated_expression_form;
use super::{
    complete_expression_contexts_available, complete_expression_may_produce_mapping,
    complete_expression_type, complete_literal_expression_value,
    condition_expression_contexts_available, condition_expression_valid,
    condition_has_status_function, interpolated_expression_contexts_available,
    interpolated_expression_valid, StaticExpressionType,
};
use serde_yaml::Value;

mod static_types;

#[test]
fn parses_supported_github_expression_shapes() {
    for expression in [
        "${{ needs.setup.outputs.flag }}",
        "${{ !cancelled() && inputs.enabled }}",
        "${{ contains(needs.*.result, 'failure') }}",
        "${{ inputs['enabled'] == true }}",
        "${{ format('{0}}}', github.ref) }}",
        "${{ -2.99e-2 < 0xff }}",
    ] {
        assert!(
            complete_expression_type(expression).is_some(),
            "{expression}"
        );
    }
}

#[test]
fn rejects_incomplete_or_concatenated_expressions() {
    for expression in [
        "${{ true && }}",
        "${{ == true }}",
        "${{ contains( }}",
        "${{ needs.setup.outputs.flag }}}}",
        "${{ true }}${{ false }}",
        "${{ 'unterminated }}",
        "${{ inputs. }}",
        "${{ 1.foo }}",
        "${{ true() }}",
        "${{ 'x'[0] }}",
        "${{ arbitrary() }}",
        "${{ github.ref() }}",
    ] {
        assert_eq!(complete_expression_type(expression), None, "{expression}");
    }
}

#[test]
fn accepts_only_documented_github_expression_functions_case_insensitively() {
    for expression in [
        "${{ CONTAINS('abc', 'a') }}",
        "${{ startsWith(github.ref, 'refs/') }}",
        "${{ endsWith(github.ref, '/main') }}",
        "${{ format('{0}', github.ref) }}",
        "${{ join(matrix.values, ',') }}",
        "${{ toJSON(github) }}",
        "${{ fromJSON(inputs.payload) }}",
        "${{ hashFiles('**/package-lock.json') }}",
        "${{ case(github.ref == 'refs/heads/main', 'production', 'development') }}",
        "${{ success() }}",
        "${{ failure() }}",
        "${{ always() }}",
        "${{ cancelled() }}",
    ] {
        assert!(
            complete_expression_type(expression).is_some(),
            "{expression}"
        );
    }
}

#[test]
fn rejects_documented_functions_with_invalid_arities() {
    for expression in [
        "${{ contains() }}",
        "${{ contains('only-one') }}",
        "${{ startsWith('a', 'b', 'c') }}",
        "${{ endsWith() }}",
        "${{ format('format-without-replacement') }}",
        "${{ join() }}",
        "${{ join(one, two, three) }}",
        "${{ toJSON() }}",
        "${{ fromJSON() }}",
        "${{ hashFiles() }}",
        "${{ case(true, 'matched') }}",
        "${{ case(true, 'matched', false, 'other') }}",
        "${{ success(1) }}",
        "${{ failure(1) }}",
        "${{ always(1) }}",
        "${{ cancelled(1) }}",
    ] {
        assert_eq!(complete_expression_type(expression), None, "{expression}");
    }
}

#[test]
fn classifies_static_literals() {
    assert_eq!(
        complete_expression_type("${{ 'false' }}"),
        Some(StaticExpressionType::String)
    );
    assert_eq!(
        complete_expression_type("${{ (42) }}"),
        Some(StaticExpressionType::Number)
    );
    assert_eq!(
        complete_expression_type("${{ true == false }}"),
        Some(StaticExpressionType::Boolean)
    );
}

#[test]
fn resolves_only_complete_literal_expressions_to_yaml_values() {
    for (expression, expected) in [
        ("${{ true }}", Value::Bool(true)),
        ("${{ (7) }}", Value::Number(7.into())),
        ("${{ 'release' }}", Value::String("release".to_string())),
        ("${{ null }}", Value::Null),
        ("${{ fromJSON('false') }}", Value::Bool(false)),
        ("${{ fromJSON('0') }}", Value::Number(0.into())),
        (
            "${{ fromJSON('{\"enabled\":true}') }}",
            serde_yaml::from_str("{enabled: true}").unwrap(),
        ),
    ] {
        assert_eq!(
            complete_literal_expression_value(expression),
            Some(expected)
        );
    }
    for expression in [
        "${{ inputs.target }}",
        "${{ true || false }}",
        "${{ fromJSON('not-json') }}",
        "${{ contains(fromJSON('not-json'), 'x') }}",
        "${{ (fromJSON('not-json')) }}",
        "${{ }}",
    ] {
        assert_eq!(
            complete_literal_expression_value(expression),
            None,
            "{expression}"
        );
    }
}

#[test]
fn distinguishes_mapping_candidates_from_guaranteed_scalars() {
    for expression in [
        "${{ true }}",
        "${{ 42 }}",
        "${{ 'matrix' }}",
        "${{ null }}",
        "${{ contains('matrix', 'm') }}",
        "${{ startsWith('matrix', 'm') }}",
        "${{ success() }}",
        "${{ toJSON(github) }}",
        "${{ true || false }}",
    ] {
        assert!(
            !complete_expression_may_produce_mapping(expression),
            "{expression}"
        );
    }
    for expression in [
        "${{ fromJSON(needs.setup.outputs.matrix) }}",
        "${{ needs.setup.outputs.matrix }}",
        "${{ case(inputs.enabled, fromJSON(inputs.matrix), needs.setup.outputs.matrix) }}",
    ] {
        assert!(
            complete_expression_may_produce_mapping(expression),
            "{expression}"
        );
    }
}

#[test]
fn validates_interpolated_expression_strings() {
    for value in [
        "literal name",
        "build ${{ github.ref }}",
        "${{ github.repository }} checks ${{ github.ref }}",
        "${{ format('{0}}}', github.ref) }}",
        "build github.ref }}",
    ] {
        assert!(interpolated_expression_valid(value), "{value}");
    }
    for value in ["${{ }}", "build ${{ github.ref", "${{ arbitrary() }}"] {
        assert!(!interpolated_expression_valid(value), "{value}");
    }
    assert_eq!(
        opaque_interpolated_expression_form(
            "cache-${{ format('{0}}}', github.ref) }}:/data",
            "<dynamic>",
        ),
        Some("cache-<dynamic>:/data".to_string())
    );
    assert_eq!(
        opaque_interpolated_expression_form("literal <dynamic>", "<dynamic>"),
        None
    );
}

#[test]
fn context_and_status_helpers_reject_malformed_or_unavailable_expressions() {
    assert!(!complete_expression_contexts_available(
        "literal",
        &["github"]
    ));
    assert!(!complete_expression_contexts_available(
        "${{ 'unterminated }}",
        &["github"]
    ));
    assert!(!complete_expression_contexts_available(
        "${{ secrets.TOKEN }}",
        &["github"]
    ));
    assert!(complete_expression_contexts_available(
        "${{ github.ref }}",
        &["github"]
    ));
    assert!(!condition_expression_valid("${{ true"));
    assert!(condition_has_status_function("${{ success() }}"));
    assert!(!condition_has_status_function(
        "contains('success()', 'success')"
    ));
    assert!(!condition_has_status_function("${{ success()"));
    assert!(interpolated_expression_contexts_available(
        "literal ${{ 'it''s valid' }}",
        &["github"]
    ));
    assert!(!interpolated_expression_contexts_available(
        "${{ success() }}",
        &["github"]
    ));
}

#[test]
fn condition_logical_operator_budget_bounds_flat_evaluation() {
    let at_limit = std::iter::repeat_n("true", 256)
        .collect::<Vec<_>>()
        .join(" && ");
    let over_limit = std::iter::repeat_n("true", 257)
        .collect::<Vec<_>>()
        .join(" || ");

    assert!(condition_expression_valid(&at_limit));
    assert!(condition_expression_contexts_available(
        &at_limit,
        &[],
        false
    ));
    assert!(!condition_expression_valid(&over_limit));
    assert!(!condition_expression_contexts_available(
        &over_limit,
        &[],
        false
    ));
    assert!(!condition_has_status_function(&format!(
        "always() && {over_limit}"
    )));
}

#[test]
fn rejects_unterminated_context_strings_and_malformed_numeric_literals() {
    assert!(!super::contexts::root_contexts_available(
        "'unterminated",
        &["github"]
    ));
    assert_eq!(super::lexer::numeric_literal_end(b"12.", 0), None);
    assert_eq!(super::lexer::numeric_literal_end(b"1e+", 0), None);
}

#[test]
fn rejects_unclosed_accessors_and_function_arguments() {
    for expression in [
        "${{ inputs['enabled' }}",
        "${{ contains('value' 'needle') }}",
    ] {
        assert_eq!(complete_expression_type(expression), None, "{expression}");
    }
}

#[test]
fn rejects_expressions_beyond_the_parser_nesting_budget_without_stack_overflow() {
    let deeply_unary = format!("${{{{ {}true }}}}", "!".repeat(10_000));
    let deeply_grouped = format!("${{{{ {}true{} }}}}", "(".repeat(1_000), ")".repeat(1_000));
    let deeply_called = format!(
        "${{{{ {}github.ref, 'x'{} }}}}",
        "contains(".repeat(1_000),
        ")".repeat(1_000)
    );
    let deeply_accessed = format!(
        "${{{{ root[{}value{}] }}}}",
        "root[".repeat(1_000),
        "]".repeat(1_000)
    );

    for expression in [deeply_unary, deeply_grouped, deeply_called, deeply_accessed] {
        assert_eq!(complete_expression_type(&expression), None, "{expression}");
    }
}

#[test]
fn literal_from_json_helpers_handle_escapes_nested_calls_and_invalid_payloads() {
    assert_eq!(
        super::literal_value::literal_from_json_value(r#"fromJSON('"it''''s"')"#),
        Some(Value::String("it''s".to_string()))
    );
    for expression in [
        "${{ fromJSON('not-json') }}",
        "${{ contains(fromJSON('not-json'), 'value') }}",
    ] {
        assert!(
            super::literal_value::invalid_literal_from_json(expression),
            "{expression}"
        );
    }
    for expression in [
        "fromJSON('not-json')",
        "${{ fromJSON(inputs.payload) }}",
        "${{ fromJSON('true', 'false') }}",
        "${{ 'fromJSON(''not-json'')' }}",
    ] {
        assert!(
            !super::literal_value::invalid_literal_from_json(expression),
            "{expression}"
        );
    }
}

#[test]
fn literal_from_json_scanner_handles_spacing_nesting_and_unclosed_calls() {
    let call = super::condition_function_call("fromJSON  ( 'true' )").unwrap();
    assert_eq!(call.arguments, vec!["'true'"]);

    assert!(super::literal_value::invalid_literal_from_json(
        "${{ fromJSON   ('not-json') }}"
    ));
    assert!(!super::literal_value::invalid_literal_from_json(
        "${{ fromJSON(('not-json')) }}"
    ));
    assert!(!super::literal_value::invalid_literal_from_json(
        "${{ fromJSON('not-json' }}"
    ));
}
