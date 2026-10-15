fn record_eager_require(
    collector: &mut ImportCollector,
    specifier: &str,
    kind: ImportKind,
    computed: bool,
) {
    if kind == ImportKind::Require
        && !computed
        && collector.function_stack.is_empty()
        && collector.current_function_id().is_none()
        && !collector.conditional_execution
    {
        collector
            .eager_require_specifiers
            .insert(specifier.to_string());
    }
}

fn with_conditional_execution(
    collector: &mut ImportCollector,
    conditional: bool,
    visit: impl FnOnce(&mut ImportCollector),
) {
    if conditional {
        collector.invalidate_replacement_execution();
        collector.receiver_conditional_region = None;
    }
    let previous = collector.conditional_execution;
    let previous_receiver = collector.receiver_write_is_conditional;
    collector.conditional_execution |= conditional;
    collector.receiver_write_is_conditional |= conditional;
    collector.receiver_write_conditional_depth += usize::from(conditional);
    visit(collector);
    collector.conditional_execution = previous;
    collector.receiver_write_is_conditional = previous_receiver;
    collector.receiver_write_conditional_depth -= usize::from(conditional);
}

fn visit_binding_default_with_execution<'a>(
    collector: &mut ImportCollector,
    pattern: &oxc_ast::ast::AssignmentPattern<'a>,
) {
    collector.visit_binding_pattern(&pattern.left);
    with_conditional_receiver_write(collector, |collector| {
        collector.visit_expression(&pattern.right)
    });
}

fn with_conditional_receiver_write(
    collector: &mut ImportCollector,
    visit: impl FnOnce(&mut ImportCollector),
) {
    // Keep this certainty local to receiver cutoffs; do not alter other fact demands.
    collector.receiver_conditional_region = None;
    let previous = std::mem::replace(&mut collector.receiver_write_is_conditional, true);
    collector.receiver_write_conditional_depth += 1;
    visit(collector);
    collector.receiver_write_is_conditional = previous;
    collector.receiver_write_conditional_depth -= 1;
}

fn with_repeat_execution<T>(
    collector: &mut ImportCollector,
    statement: &T,
    visit: impl FnOnce(&mut ImportCollector, &T),
) {
    // Repeated child positions retain all enclosing execution guards.
    collector.repeat_depth += 1;
    visit(collector, statement);
    collector.repeat_depth -= 1;
}

// Literal conditions can prove one branch executes. Unknown conditions remain
// conservative; these facts do not evaluate bindings or replace syntactic edges.
fn literal_truthiness(expression: &Expression<'_>) -> Option<bool> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::BooleanLiteral(value) => Some(value.value),
        Expression::NullLiteral(_) => Some(false),
        Expression::NumericLiteral(value) => Some(value.value != 0.0),
        Expression::BigIntLiteral(value) => Some(value.value != "0"),
        Expression::StringLiteral(value) => Some(!value.value.is_empty()),
        _ => None,
    }
}

fn visit_if_with_execution<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::IfStatement<'a>,
) {
    collector.visit_expression(&statement.test);
    let truthy = literal_truthiness(&statement.test);
    with_conditional_execution(collector, truthy != Some(true), |collector| {
        visit_guarded_receiver_arm(collector, &statement.consequent, truthy.is_none());
    });
    if let Some(alternate) = &statement.alternate {
        with_conditional_execution(collector, truthy != Some(false), |collector| {
            visit_guarded_receiver_arm(collector, alternate, truthy.is_none());
        });
    }
}

fn visit_conditional_with_execution<'a>(
    collector: &mut ImportCollector,
    expression: &oxc_ast::ast::ConditionalExpression<'a>,
) {
    collector.visit_expression(&expression.test);
    let truthy = literal_truthiness(&expression.test);
    with_conditional_execution(collector, truthy != Some(true), |collector| {
        collector.visit_expression(&expression.consequent);
    });
    with_conditional_execution(collector, truthy != Some(false), |collector| {
        collector.visit_expression(&expression.alternate);
    });
}

fn visit_logical_with_execution<'a>(
    collector: &mut ImportCollector,
    expression: &oxc_ast::ast::LogicalExpression<'a>,
) {
    if collector.collect_value_flow && expression.operator == oxc_ast::ast::LogicalOperator::Coalesce {
        collector.value_flow.has_nullish_alternatives = true;
    }
    collector.visit_expression(&expression.left);
    let unconditional = match expression.operator {
        oxc_ast::ast::LogicalOperator::And => literal_truthiness(&expression.left) == Some(true),
        oxc_ast::ast::LogicalOperator::Or => literal_truthiness(&expression.left) == Some(false),
        oxc_ast::ast::LogicalOperator::Coalesce => matches!(
            crate::codebase::ts_source::unwrap_ts_wrappers(&expression.left),
            Expression::NullLiteral(_)
        ),
    };
    with_conditional_execution(collector, !unconditional, |collector| {
        collector.visit_expression(&expression.right);
    });
}

fn walk_call_with_execution<'a>(collector: &mut ImportCollector, call: &CallExpression<'a>) {
    // The callee executes even when an optional call skips its arguments.
    let previous = collector.bare_callable_callee;
    collector.bare_callable_callee = match crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee) {
        Expression::Identifier(identifier) => Some(identifier.span.start),
        _ => None,
    };
    collector.visit_expression(&call.callee);
    collector.bare_callable_callee = previous;
    if let Some(type_arguments) = &call.type_arguments {
        collector.visit_ts_type_parameter_instantiation(type_arguments);
    }
    with_conditional_execution(collector, call_has_execution_guard(call), |collector| {
        collector.visit_arguments(&call.arguments);
    });
}

fn call_has_execution_guard(call: &CallExpression<'_>) -> bool {
    call.optional || has_optional_chain(&call.callee)
}

fn has_optional_chain(expression: &Expression<'_>) -> bool {
    let expression = crate::codebase::ts_source::unwrap_ts_wrappers(expression);
    if let Some(member) = expression.as_member_expression() {
        member.optional() || has_optional_chain(member.object())
    } else if let Expression::CallExpression(call) = expression {
        call_has_execution_guard(call)
    } else {
        false
    }
}

fn visit_try_with_receiver_execution<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::TryStatement<'a>,
) {
    with_conditional_receiver_write(collector, |collector| {
        collector.visit_block_statement(&statement.block)
    });
    if let Some(handler) = &statement.handler {
        collector.visit_catch_clause(handler);
    }
    if let Some(finalizer) = &statement.finalizer {
        collector.visit_block_statement(finalizer);
    }
}

fn visit_label_with_receiver_execution<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::LabeledStatement<'a>,
) {
    with_conditional_receiver_write(collector, |collector| {
        collector.visit_statement(&statement.body)
    });
}

fn visit_new_expression_with_imports_and_calls<'a>(
    collector: &mut ImportCollector,
    new: &NewExpression<'a>,
) {
    collector.record_unknown_fresh_call(new.span.end);
    visit_new_expression_with_imports(collector, new);
    walk::walk_new_expression(collector, new);
    collector.record_descriptor_call(new.span.end);
}
