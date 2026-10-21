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
    let previous_completion = collector
        .receiver_escapes
        .parameter_completion
        .replace((offset, completed_at));
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
    collector.receiver_escapes.parameter_completion = previous_completion;
}

fn visit_assignment_expression_with_receiver_effects<'a>(
    collector: &mut ImportCollector,
    assignment: &AssignmentExpression<'a>,
) {
    #[cfg(feature = "test-instrumentation")]
    extract_value_flow::ordinary_demand_metrics::site(5);
    let rhs_frame = collector.begin_assignment_rhs(assignment);
    collector.prepare_mutable_pattern_roots(&assignment.left);
    let receipts = collector.begin_phase_receipts(&assignment.left);
    let mut phases =
        collector.begin_reference_destructuring_phases(&assignment.left, &assignment.right);
    let fresh = receiver_replacement_is_fresh(assignment);
    let previous = std::mem::replace(&mut collector.receiver_replacement_is_fresh, fresh);
    with_conditional_execution(collector, assignment.operator.is_logical(), |collector| {
        record_assignment_target_writes(
            collector,
            &assignment.left,
            assignment.span.start,
            assignment.span.end,
        );
    });
    collector.receiver_replacement_is_fresh = previous;
    if phases.is_none() {
        phases = collector.begin_destructuring_phases(&assignment.left, &assignment.right);
    }
    let receipts = if phases.is_some() {
        receipts
    } else {
        collector.end_phase_receipts(receipts);
        None
    };
    let previous_fresh_evaluation = collector.fresh_receiver_effects.active;
    collector.fresh_receiver_effects.active |= fresh && collector.collect_value_flow;
    walk_assignment_lhs_without_written_getter(collector, &assignment.left);
    collector.end_destructuring_lhs(&phases);
    with_conditional_execution(collector, assignment.operator.is_logical(), |collector| {
        collector.visit_expression(&assignment.right);
    });
    collector.fresh_receiver_effects.active = previous_fresh_evaluation;
    collector.finish_assignment_rhs(assignment, rhs_frame);
    collector.record_mutable_receiver_assignment(assignment);
    if let Some(phases) = phases {
        collector.finish_destructuring_phases(phases);
    }
    collector.end_phase_receipts(receipts);
}

fn with_conditional_receiver_write(
    collector: &mut ImportCollector,
    visit: impl FnOnce(&mut ImportCollector),
) {
    if !collector.collect_value_flow {
        visit(collector);
        return;
    }
    #[cfg(feature = "test-instrumentation")]
    extract_value_flow::ordinary_demand_metrics::site(2);
    // Keep this certainty local to receiver cutoffs; do not alter other fact demands.
    collector.receiver_conditional_region = None;
    let previous = std::mem::replace(&mut collector.receiver_write_is_conditional, true);
    collector.receiver_write_conditional_depth += 1;
    visit(collector);
    collector.receiver_write_is_conditional = previous;
    collector.receiver_write_conditional_depth -= 1;
}

fn with_callable_receiver_execution(
    collector: &mut ImportCollector,
    visit: impl FnOnce(&mut ImportCollector),
) {
    if !collector.collect_value_flow {
        visit(collector);
        return;
    }
    #[cfg(feature = "test-instrumentation")]
    extract_value_flow::ordinary_demand_metrics::site(3);
    // A deferred body has its own execution conditions; surrounding guards only
    // decide whether the callable is created or invoked.
    collector.receiver_conditional_region = None;
    let fresh = std::mem::replace(&mut collector.fresh_receiver_effects.active, false);
    let descriptor_call = std::mem::take(&mut collector.fresh_receiver_effects.descriptor_call);
    let conditional = std::mem::replace(&mut collector.receiver_write_is_conditional, false);
    let conditional_depth = std::mem::take(&mut collector.receiver_write_conditional_depth);
    visit(collector);
    collector.receiver_write_is_conditional = conditional;
    collector.receiver_write_conditional_depth = conditional_depth;
    collector.fresh_receiver_effects.active = fresh;
    collector.fresh_receiver_effects.descriptor_call = descriptor_call;
}
