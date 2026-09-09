fn record_object_value_references(
    collector: &mut ImportCollector,
    object_name: &str,
    object: &ObjectExpression<'_>,
) {
    for property in &object.properties {
        if let ObjectPropertyKind::ObjectProperty(property) = property {
            record_object_computed_key_reference(collector, object_name, &property.key);
        }
        let reference = match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                simple_object_reference(&property.value)
            }
            ObjectPropertyKind::SpreadProperty(spread) => simple_object_reference(&spread.argument),
        };
        if let Some(callee) = reference {
            let callee_binding_scope = collector.callee_binding_scope(&callee);
            collector.symbol_references.push(FunctionCall {
                caller: Some(object_name.to_string()),
                caller_id: collector.current_function_id(),
                syntactic_caller: collector.current_syntactic_caller(),
                callee,
                line: 0,
                offset: 0,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity: CallTargetIdentity::Unknown,
                callee_binding_scope,
                static_arg: None,
                static_cwd: None,
            });
        }
    }
}

fn record_object_computed_key_reference(
    collector: &mut ImportCollector,
    object_name: &str,
    key: &oxc_ast::ast::PropertyKey<'_>,
) {
    let Some(callee) = key.as_expression().and_then(simple_callee_name) else {
        return;
    };
    if collector.callee_shadows_import(&callee) {
        return;
    }
    let callee_binding_scope = collector.callee_binding_scope(&callee);
    collector.symbol_references.push(FunctionCall {
        caller: Some(object_name.to_string()),
        caller_id: collector.current_function_id(),
        syntactic_caller: collector.current_syntactic_caller(),
        callee,
        line: 0,
        offset: 0,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: CallTargetIdentity::Unknown,
        callee_binding_scope,
        static_arg: None,
        static_cwd: None,
    });
}

fn simple_object_reference(expr: &Expression<'_>) -> Option<String> {
    match expr {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::StaticMemberExpression(member) => simple_static_member_name(member),
        _ => None,
    }
}
