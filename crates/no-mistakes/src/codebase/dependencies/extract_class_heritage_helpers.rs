/// Synthetic ancestry marker; never a source call target.
pub(crate) const DYNAMIC_BASE_CALLEE: &str = "<unknown>";

/// Preserve declared ancestry, including a dynamic base with unknown identity.
/// Constructing a derived class also constructs its statically named base.
/// This synthetic transition is never a source call for call-based checks.
fn record_class_base_construction(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    class: &Class<'_>,
) {
    let Some(expression) = class.heritage_expression() else {
        return;
    };
    // A dynamic base still declares ancestry. Do not infer that a factory
    // preserves an Error argument, or pretend its result is a global root.
    let (base, target_identity) = match simple_callee_name(expression) {
        Some(base) => {
            collector.record_class_local_base(class_id, base.clone());
            let identity = collector.call_target_identity(&base);
            (base, identity)
        }
        // Preserve the existing class-expression boundary; this fallback is
        // for declarations whose dynamic ancestry used to disappear entirely.
        None if class.r#type == oxc_ast::ast::ClassType::ClassExpression => return,
        None => (DYNAMIC_BASE_CALLEE.to_string(), CallTargetIdentity::Unknown),
    };
    collector
        .class_declaration_lines
        .push((class_id, import_line_at(&collector.line_starts, class.span.start as usize)));
    record_class_namespace_facts(collector, class_id, class);
    collector.function_calls.push(FunctionCall {
        caller: Some(class_name.to_string()),
        caller_id: Some(class_id),
        syntactic_caller: collector.current_syntactic_caller(),
        callee_binding_scope: collector.callee_binding_scope(&base),
        target_identity,
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
