use super::{complete_expression_type, StaticExpressionType};

#[test]
fn classifies_guaranteed_function_case_and_logical_result_types() {
    for expression in [
        "${{ contains('abc', 'a') }}",
        "${{ startsWith('abc', 'a') }}",
        "${{ endsWith('abc', 'c') }}",
        "${{ success() }}",
        "${{ failure() }}",
        "${{ always() }}",
        "${{ cancelled() }}",
        "${{ !inputs.enabled }}",
        "${{ true && false }}",
    ] {
        assert_eq!(
            complete_expression_type(expression),
            Some(StaticExpressionType::Boolean),
            "{expression}"
        );
    }

    for expression in [
        "${{ format('{0}', github.ref) }}",
        "${{ join(matrix.values, ',') }}",
        "${{ toJSON(github) }}",
        "${{ hashFiles('**/package-lock.json') }}",
        "${{ hashFiles('**/package-lock.json') }}",
        "${{ 'first' || 'second' }}",
    ] {
        assert_eq!(
            complete_expression_type(expression),
            Some(StaticExpressionType::String),
            "{expression}"
        );
    }

    for expression in [
        "${{ fromJSON(inputs.payload) }}",
        "${{ github.ref }}",
        "${{ toJSON(github).value }}",
        "${{ true || 'second' }}",
    ] {
        assert_eq!(
            complete_expression_type(expression),
            Some(StaticExpressionType::Dynamic),
            "{expression}"
        );
    }
}
