/// Constructing a derived class also constructs its statically named base.
/// Keep this as a synthetic reachability transition: it is not a source call
/// site and must not be reported by call-based checks.
fn record_class_base_construction(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    class: &Class<'_>,
) {
    let Some(base) = class.heritage_expression().and_then(simple_callee_name) else {
        return;
    };
    collector.record_class_local_base(class_id, base.clone());
    collector.function_calls.push(FunctionCall {
        caller: Some(class_name.to_string()),
        caller_id: Some(class_id),
        syntactic_caller: collector.current_syntactic_caller(),
        callee_binding_scope: collector.callee_binding_scope(&base),
        target_identity: collector.call_target_identity(&base),
        callee: base,
        line: 0,
        offset: 0,
        is_callback: true,
        invocation: InvocationKind::Construct,
        static_arg: None,
        static_cwd: None,
    });
}

fn record_class_base_symbol_reference(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    class: &Class<'_>,
) {
    let Some(base) = class.heritage_expression().and_then(simple_callee_name) else {
        return;
    };
    if collector.callee_shadows_import(&base) {
        return;
    }
    collector.symbol_references.push(FunctionCall {
        caller: Some(class_name.to_string()),
        caller_id: Some(class_id),
        syntactic_caller: collector.current_syntactic_caller(),
        callee_binding_scope: collector.callee_binding_scope(&base),
        callee: base,
        line: 0,
        offset: 0,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: CallTargetIdentity::Unknown,
        static_arg: None,
        static_cwd: None,
    });
}
