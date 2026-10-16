fn replacement_for_of_execution(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForOfStatement<'_>,
) -> (bool, bool, Option<u32>) {
    if !collector.collect_value_flow {
        return (false, false, None);
    }
    #[cfg(feature = "test-instrumentation")]
    {
        collector.value_flow.replacement_loop_classifications += 1;
    }
    let simple_left = matches!(&statement.left, ForStatementLeft::VariableDeclaration(declaration)
        if declaration.declarations.len() == 1
            && matches!(&declaration.declarations[0].id, BindingPattern::BindingIdentifier(_)));
    let may_repeat = !matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&statement.right),
        Expression::ArrayExpression(array) if array.elements.len() <= 1
            && array.elements.iter().all(|element| !matches!(element, oxc_ast::ast::ArrayExpressionElement::SpreadElement(_))));
    let repeated = simple_left
        && !statement.r#await
        && matches!(
            crate::codebase::ts_source::unwrap_ts_wrappers(&statement.right),
            Expression::ArrayExpression(array) if array.elements.len() >= 2
                && array.elements.iter().all(|element| !matches!(element, oxc_ast::ast::ArrayExpressionElement::SpreadElement(_)))
        );
    // A straight-line assignment executes on the first iteration of this
    // literal, nonempty loop. The receiver replacement region remains unsafe
    // for other purposes because assigning a mutable local invalidates it.
    let guaranteed_assignment = if repeated
        && !collector.conditional_execution
        && collector.current_function_id().is_none()
        && matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&statement.right),
        Expression::ArrayExpression(array) if array.elements.iter().all(|element| {
            matches!(element.as_expression(),
                Some(Expression::NumericLiteral(_)
                    | Expression::StringLiteral(_)
                    | Expression::BooleanLiteral(_)))
        })) {
        match &statement.body {
            Statement::BlockStatement(block) => match block.body.as_slice() {
                [Statement::ExpressionStatement(expression)] => {
                    match crate::codebase::ts_source::unwrap_ts_wrappers(&expression.expression) {
                        Expression::AssignmentExpression(assignment)
                            if assignment.operator
                                == oxc_syntax::operator::AssignmentOperator::Assign
                                && matches!(
                                    assignment.left,
                                    AssignmentTarget::AssignmentTargetIdentifier(_)
                                )
                                && matches!(
                                    crate::codebase::ts_source::unwrap_ts_wrappers(
                                        &assignment.right
                                    ),
                                    Expression::FunctionExpression(_)
                                        | Expression::ArrowFunctionExpression(_)
                                ) =>
                        {
                            Some(assignment.span.end)
                        }
                        _ => None,
                    }
                }
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    };
    (repeated, may_repeat, guaranteed_assignment)
}
