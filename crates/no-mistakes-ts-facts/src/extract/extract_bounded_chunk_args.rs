fn chunk_argument(
    collector: &ImportCollector,
    expression: &Expression<'_>,
    counter: &str,
    array: &str,
    increment: (f64, Option<&str>),
    slices: &mut usize,
) -> bool {
    let (step, step_name) = increment;
    if let Expression::ArrayExpression(args) = expression {
        return args.elements.iter().all(|arg| {
            arg.as_expression().is_some_and(|arg| {
                chunk_argument(collector, arg, counter, array, increment, slices)
            })
        });
    }
    if let Expression::CallExpression(call) = expression {
        if let Expression::StaticMemberExpression(member) = &call.callee {
            if member.property.name == "slice"
                && matches!(&member.object, Expression::Identifier(id) if id.name == array)
                && call.arguments.len() == 2
                && matches!(call.arguments[0].as_expression(), Some(Expression::Identifier(id)) if id.name == counter)
            {
                if let Some(Expression::BinaryExpression(end)) = call.arguments[1].as_expression() {
                    if end.operator == BinaryOperator::Addition
                        && matches!(&end.left, Expression::Identifier(id) if id.name == counter)
                        && collector.bounded_number(&end.right) == Some(step)
                        && match &end.right {
                            Expression::NumericLiteral(_) => true,
                            Expression::Identifier(id) => Some(id.name.as_str()) == step_name,
                            _ => false,
                        }
                    {
                        *slices += 1;
                        return true;
                    }
                }
            }
        }
    }
    !cursor_plain_references(expression, true).contains(counter)
}
