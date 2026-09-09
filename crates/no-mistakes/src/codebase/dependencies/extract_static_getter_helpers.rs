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
        line: import_line_at(&collector.line_starts, member.span.start as usize),
        offset: member.span.start,
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
    record_assignment_target_writes(collector, &assignment.left, assignment.span.start);
    walk_assignment_lhs_without_written_getter(collector, &assignment.left);
    collector.visit_expression(&assignment.right);
}

fn walk_assignment_lhs_without_written_getter<'a>(
    collector: &mut ImportCollector,
    target: &AssignmentTarget<'a>,
) {
    if let AssignmentTarget::StaticMemberExpression(member) = target {
        collector.visit_expression(&member.object);
        return;
    }
    walk::walk_assignment_target(collector, target);
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
