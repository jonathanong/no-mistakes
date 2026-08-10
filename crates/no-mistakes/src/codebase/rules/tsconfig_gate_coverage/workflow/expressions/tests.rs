use super::{
    complete_expression_contexts_available, complete_expression_type, condition_expression_valid,
    condition_has_status_function, interpolated_expression_contexts_available,
    interpolated_expression_valid, StaticExpressionType,
};

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
fn validates_interpolated_expression_strings() {
    for value in [
        "literal name",
        "build ${{ github.ref }}",
        "${{ github.repository }} checks ${{ github.ref }}",
        "${{ format('{0}}}', github.ref) }}",
    ] {
        assert!(interpolated_expression_valid(value), "{value}");
    }
    for value in [
        "${{ }}",
        "build ${{ github.ref",
        "build github.ref }}",
        "${{ arbitrary() }}",
    ] {
        assert!(!interpolated_expression_valid(value), "{value}");
    }
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
