fn import_main_is_false(expression: &Expression<'_>) -> bool {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::StaticMemberExpression(member) => {
            member.property.name == "main"
                && matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&member.object),
                    Expression::ImportMeta(_))
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::StrictEquality => {
            let is_true = |expression: &Expression<'_>| matches!(
                crate::codebase::ts_source::unwrap_ts_wrappers(expression),
                Expression::BooleanLiteral(value) if value.value
            );
            (is_true(&binary.right) && import_main_is_false(&binary.left))
                || (is_true(&binary.left) && import_main_is_false(&binary.right))
        }
        Expression::LogicalExpression(logical)
            if logical.operator == oxc_ast::ast::LogicalOperator::And =>
        {
            import_main_is_false(&logical.left) || import_main_is_false(&logical.right)
        }
        _ => false,
    }
}

fn record_import_inactive_branch(collector: &mut ImportCollector, statement: &oxc_ast::ast::IfStatement<'_>) {
    if collector.current_function_id().is_none() && import_main_is_false(&statement.test) {
        let span = statement.consequent.span();
        collector.import_inactive_call_ranges.push((span.start, span.end));
    }
}

#[cfg(test)]
#[path = "extract/import_execution_tests.rs"]
mod import_execution_tests;
