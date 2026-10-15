fn visit_static_member_expression_with_getters<'a>(
    collector: &mut ImportCollector,
    member: &StaticMemberExpression<'a>,
) {
    collector.note_selected_member(&member.object, member.property.name.as_str());
    if let Some(name) = simple_static_member_name(member) {
        record_static_getter_read(collector, member.span.start, &name);
        record_object_getter_read(collector, member, &name);
        collector.push_value_symbol_reference(name);
    }
    walk::walk_static_member_expression(collector, member);
}

fn visit_computed_member_expression_with_getters<'a>(
    collector: &mut ImportCollector,
    member: &ComputedMemberExpression<'a>,
) {
    if let Expression::StringLiteral(property) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&member.expression)
    {
        collector.note_selected_member(&member.object, property.value.as_str());
    }
    if let Some(name) = simple_computed_member_name(member) {
        record_static_getter_read(collector, member.span.start, &name);
        collector.push_value_symbol_reference(name);
    }
    collector.visit_expression(&member.object);
    let conditional = member.optional || has_optional_chain(&member.object);
    with_conditional_execution(collector, conditional, |collector| {
        collector.visit_expression(&member.expression);
    });
}

fn record_static_getter_read(collector: &mut ImportCollector, offset: u32, callee: &str) {
    let Some((binding, property)) = callee.split_once('.') else {
        return;
    };
    let Some(binding_scope) = collector.callee_binding_scope(callee) else {
        return;
    };
    if collector.has_reassigned_callable_at(binding_scope, binding)
        || collector.has_reassigned_callable_at(binding_scope, callee)
    {
        return;
    }
    let Some(class_id) = collector.class_id_for_binding(binding_scope, binding) else {
        return;
    };
    if !collector.has_static_getter_member(class_id, property) {
        return;
    }
    collector.function_calls.push(FunctionCall {
        caller: collector.current_function(),
        caller_id: collector.current_function_id(),
        syntactic_caller: collector.current_syntactic_caller(),
        callee: callee.to_string(),
        line: import_line_at(&collector.line_starts, offset as usize),
        offset,
        is_callback: false,
        invocation: InvocationKind::Get,
        target_identity: CallTargetIdentity::RepositoryFunction,
        callee_binding_scope: Some(binding_scope),
        static_arg: None,
        static_cwd: None,
    });
}

fn is_static_getter(collector: &ImportCollector, callee: &str) -> bool {
    let Some((binding, property)) = callee.split_once('.') else {
        return false;
    };
    let Some(binding_scope) = collector.callee_binding_scope(callee) else {
        return false;
    };
    let Some(class_id) = collector.class_id_for_binding(binding_scope, binding) else {
        return false;
    };
    collector.has_static_getter_member(class_id, property)
}

fn record_static_setter_assignment(
    collector: &mut ImportCollector,
    callee: &str,
    offset: u32,
) -> bool {
    let Some((binding, property)) = callee.split_once('.') else {
        return false;
    };
    let Some(binding_scope) = collector.callee_binding_scope(callee) else {
        return false;
    };
    let Some(class_id) = collector.class_id_for_binding(binding_scope, binding) else {
        return false;
    };
    if !collector.has_static_setter_member(class_id, property) {
        return false;
    }
    collector.function_calls.push(FunctionCall {
        caller: collector.current_function(),
        caller_id: collector.current_function_id(),
        syntactic_caller: collector.current_syntactic_caller(),
        callee: callee.to_string(),
        line: import_line_at(&collector.line_starts, offset as usize),
        offset,
        is_callback: false,
        invocation: InvocationKind::Set,
        target_identity: CallTargetIdentity::RepositoryFunction,
        callee_binding_scope: Some(binding_scope),
        static_arg: None,
        static_cwd: None,
    });
    true
}

fn visit_assignment_expression_with_calls<'a>(
    collector: &mut ImportCollector,
    assignment: &AssignmentExpression<'a>,
) {
    collector.record_replacement_assignment(assignment);
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
    collector.record_mutable_receiver_assignment(assignment);
    collector.finish_destructuring_phases(phases);
    collector.end_phase_receipts(receipts);
}

fn walk_assignment_lhs_without_written_getter<'a>(
    collector: &mut ImportCollector,
    target: &AssignmentTarget<'a>,
) {
    if let AssignmentTarget::StaticMemberExpression(member) = target {
        collector.visit_expression(&member.object);
        return;
    }
    if let AssignmentTarget::ComputedMemberExpression(member) = target {
        collector.visit_expression(&member.object);
        collector.visit_expression(&member.expression);
        return;
    }
    walk::walk_assignment_target(collector, target);
}

fn visit_update_expression_with_calls<'a>(
    collector: &mut ImportCollector,
    update: &oxc_ast::ast::UpdateExpression<'a>,
) {
    collector.record_callable_assignment(update.argument.as_assignment_target(), None, false, update.span.end);
    collector.record_replacement_target(
        update.argument.as_assignment_target(),
        ValueExpression::Unknown,
        update.span.end,
    );
    record_assignment_target_writes(
        collector,
        update.argument.as_assignment_target(),
        update.span.start,
        update.span.end,
    );
    walk::walk_update_expression(collector, update);
}
