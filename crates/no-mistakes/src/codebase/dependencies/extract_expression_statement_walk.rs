fn visit_statement_with_type_binding<'a>(
    collector: &mut ImportCollector,
    statement: &Statement<'a>,
) {
    if matches!(
        statement,
        Statement::ThrowStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
    ) || (collector.current_function_id().is_some() || collector.replacement_region.is_some())
        && !matches!(
            statement,
            Statement::BlockStatement(_)
                | Statement::ExpressionStatement(_)
                | Statement::VariableDeclaration(_)
                | Statement::FunctionDeclaration(_)
                | Statement::EmptyStatement(_)
                | Statement::DebuggerStatement(_)
        )
    {
        collector.invalidate_replacement_execution();
    }
    let direct = match statement {
        Statement::ExpressionStatement(statement) => {
            match crate::codebase::ts_source::unwrap_ts_wrappers(&statement.expression) {
                Expression::CallExpression(call) => Some((call.span.start, call.span.end)),
                _ => None,
            }
        }
        _ => None,
    };
    let previous = std::mem::replace(&mut collector.replacement_statement_call, direct);
    record_statement_type_binding(collector, statement);
    walk::walk_statement(collector, statement);
    collector.replacement_statement_call = previous;
}

fn visit_expression_with_annex_b_initializer<'a>(
    collector: &mut ImportCollector,
    expression: &Expression<'a>,
) {
    if matches!(
        expression,
        Expression::AwaitExpression(_) | Expression::YieldExpression(_)
    ) || matches!(expression, Expression::UnaryExpression(unary) if unary.operator == oxc_syntax::operator::UnaryOperator::Delete)
    {
        collector.invalidate_replacement_execution();
    }
    if let Some((_, outer_conditional)) = collector
        .annex_b_for_in_initializer
        .filter(|(span, _)| !collector.suppress_imports && *span == expression.span())
    {
        let initializer = collector.annex_b_for_in_initializer.take();
        let previous = std::mem::replace(&mut collector.conditional_execution, outer_conditional);
        walk::walk_expression(collector, expression);
        collector.conditional_execution = previous;
        collector.annex_b_for_in_initializer = initializer;
    } else {
        walk::walk_expression(collector, expression);
    }
}
