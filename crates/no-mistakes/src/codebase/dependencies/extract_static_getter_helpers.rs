fn record_static_getter_read(
    collector: &mut ImportCollector,
    member: &StaticMemberExpression<'_>,
    callee: &str,
) {
    let Some((binding, property)) = callee.split_once('.') else {
        return;
    };
    let Some(binding_scope) = collector.callee_binding_scope(callee) else {
        return;
    };
    if collector
        .reassigned_callable_binding_ids
        .contains(&(binding_scope, binding.to_string()))
        || collector
            .reassigned_callable_binding_ids
            .contains(&(binding_scope, callee.to_string()))
    {
        return;
    }
    let Some(class_id) = collector.class_id_for_binding(binding_scope, binding) else {
        return;
    };
    if !collector
        .static_getter_member_ids
        .contains(&(class_id, property.to_string()))
    {
        return;
    }
    collector.function_calls.push(FunctionCall {
        caller: collector.current_function(),
        caller_id: collector.current_function_id(),
        syntactic_caller: collector.current_syntactic_caller(),
        callee: callee.to_string(),
        line: import_line_at(&collector.line_starts, member.span.start as usize),
        offset: member.span.start,
        is_callback: false,
        invocation: InvocationKind::Call,
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
    collector
        .static_getter_member_ids
        .contains(&(class_id, property.to_string()))
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
    if !collector
        .static_setter_member_ids
        .contains(&(class_id, property.to_string()))
    {
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
        invocation: InvocationKind::Call,
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
    record_assignment_target_writes(collector, &assignment.left, assignment.span.start);
    walk::walk_assignment_expression(collector, assignment);
}

fn visit_update_expression_with_calls<'a>(
    collector: &mut ImportCollector,
    update: &oxc_ast::ast::UpdateExpression<'a>,
) {
    record_assignment_target_writes(
        collector,
        update.argument.as_assignment_target(),
        update.span.start,
    );
    walk::walk_update_expression(collector, update);
}

fn record_assignment_target_writes(
    collector: &mut ImportCollector,
    target: &AssignmentTarget<'_>,
    offset: u32,
) {
    for name in assignment_target_names(target) {
        if !record_static_setter_assignment(collector, &name, offset) {
            collector.record_reassigned_callable_alias(&name, offset);
        }
    }
}
