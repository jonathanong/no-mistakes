use super::{complete_expression_type, StaticExpressionType};

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
