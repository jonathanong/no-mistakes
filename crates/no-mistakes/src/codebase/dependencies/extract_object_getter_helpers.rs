fn record_object_getter_read(
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
    if collector.has_reassigned_callable_at(binding_scope, callee)
        || collector.has_reassigned_callable_at(binding_scope, binding)
    {
        return;
    }
    let Some(object_id) = collector.callable_binding_at(binding_scope, binding) else {
        return;
    };
    if !collector.has_object_getter_member(object_id, property) {
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

fn is_object_getter(collector: &ImportCollector, callee: &str) -> bool {
    let Some((binding, property)) = callee.split_once('.') else {
        return false;
    };
    let Some(binding_scope) = collector.callee_binding_scope(callee) else {
        return false;
    };
    if collector.has_reassigned_callable_at(binding_scope, binding)
        || collector.has_reassigned_callable_at(binding_scope, callee)
    {
        return false;
    }
    let Some(object_id) = collector.callable_binding_at(binding_scope, binding) else {
        return false;
    };
    collector.has_object_getter_member(object_id, property)
}
