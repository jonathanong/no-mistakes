fn receiver_replacement_is_fresh(assignment: &AssignmentExpression<'_>) -> bool {
    (assignment.operator == oxc_syntax::operator::AssignmentOperator::Assign
        || assignment.operator.is_logical())
        && matches!(
            &assignment.left,
            AssignmentTarget::StaticMemberExpression(_)
                | AssignmentTarget::ComputedMemberExpression(_)
        )
        && matches!(
            crate::codebase::ts_source::unwrap_ts_wrappers(&assignment.right),
            Expression::ObjectExpression(_)
                | Expression::ArrayExpression(_)
                | Expression::ArrowFunctionExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::NullLiteral(_)
                | Expression::BooleanLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::BigIntLiteral(_)
                | Expression::StringLiteral(_)
        )
}

fn visit_target_default_with_execution<'a>(
    collector: &mut ImportCollector,
    target: &oxc_ast::ast::AssignmentTargetWithDefault<'a>,
) {
    collector.visit_assignment_target(&target.binding);
    with_conditional_receiver_write(collector, |collector| {
        collector.visit_expression(&target.init)
    });
}

fn visit_parameter_default_with_execution<'a>(
    collector: &mut ImportCollector,
    parameter: &oxc_ast::ast::FormalParameter<'a>,
) {
    collector.visit_decorators(&parameter.decorators);
    collector.visit_binding_pattern(&parameter.pattern);
    if let Some(annotation) = &parameter.type_annotation {
        collector.visit_ts_type_annotation(annotation);
    }
    if let Some(initializer) = &parameter.initializer {
        with_conditional_receiver_write(collector, |collector| {
            collector.visit_expression(initializer)
        });
    }
}

fn record_assignment_target_writes(
    collector: &mut ImportCollector,
    target: &AssignmentTarget<'_>,
    offset: u32,
    completed_at: u32,
) {
    for (name, path, start) in extract_value_flow::write_targets::targets(target) {
        let previous = collector.assignment_target_start.replace(start);
        if !record_static_setter_assignment(collector, &name, offset)
            && !record_object_setter_assignment(collector, &name, offset)
        {
            if path.is_empty() {
                collector.record_reassigned_callable_alias(&name, offset);
            } else {
                collector.record_reassigned_callable_alias_path(
                    &name,
                    offset,
                    Some(&path),
                    completed_at,
                );
            }
        }
        collector.assignment_target_start = previous;
    }
}

fn with_callable_receiver_execution(
    collector: &mut ImportCollector,
    visit: impl FnOnce(&mut ImportCollector),
) {
    // A deferred body has its own execution conditions; surrounding guards only
    // decide whether the callable is created or invoked.
    collector.receiver_conditional_region = None;
    let fresh = std::mem::replace(&mut collector.fresh_receiver_effects.active, false);
    let descriptor_call = std::mem::take(&mut collector.fresh_receiver_effects.descriptor_call);
    let conditional = std::mem::replace(&mut collector.receiver_write_is_conditional, false);
    visit(collector);
    collector.receiver_write_is_conditional = conditional;
    collector.fresh_receiver_effects.active = fresh;
    collector.fresh_receiver_effects.descriptor_call = descriptor_call;
}
