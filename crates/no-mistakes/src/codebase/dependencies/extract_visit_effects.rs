fn visit_await_expression_with_capture<'a>(
    collector: &mut ImportCollector,
    expression: &oxc_ast::ast::AwaitExpression<'a>,
) {
    collector.invalidate_replacement_expression();
    collector.record_unknown_fresh_call(expression.span.end);
    walk::walk_await_expression(collector, expression);
    // Operand evaluation can create the first capture; completion must
    // invalidate it even when the pre-operand path was still cold.
    collector.record_descriptor_call(expression.span.end);
}

fn visit_yield_expression_with_capture<'a>(
    collector: &mut ImportCollector,
    expression: &oxc_ast::ast::YieldExpression<'a>,
) {
    collector.invalidate_replacement_expression();
    collector.record_unknown_fresh_call(expression.span.end);
    walk::walk_yield_expression(collector, expression);
    collector.record_descriptor_call(expression.span.end);
}

fn visit_unary_expression_with_delete<'a>(
    collector: &mut ImportCollector,
    expression: &oxc_ast::ast::UnaryExpression<'a>,
) {
    if expression.operator == oxc_syntax::operator::UnaryOperator::Delete {
        collector.invalidate_replacement_expression();
        if collector.collect_value_flow {
            if let Some((name, path, start)) =
                extract_value_flow::write_targets::from_expression(&expression.argument)
            {
                collector.record_reassigned_callable_alias_path(
                    &name,
                    start,
                    Some(&path),
                    expression.span.end,
                );
            }
        }
    }
    walk::walk_unary_expression(collector, expression);
}

fn visit_new_expression_with_calls<'a>(collector: &mut ImportCollector, new: &NewExpression<'a>) {
    visit_new_expression_with_imports_and_calls(collector, new);
}

fn visit_tagged_template_with_receiver<'a>(
    collector: &mut ImportCollector,
    tagged: &TaggedTemplateExpression<'a>,
) {
    collector.visit_receiver_tagged(tagged);
}
