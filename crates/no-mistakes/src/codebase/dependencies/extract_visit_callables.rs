fn visit_function_with_value_flow_scope<'a>(
    collector: &mut ImportCollector,
    function: &Function<'a>,
) {
    let enclosing_unreachable_binding_after = collector
        .collect_value_flow
        .then(|| collector.unreachable_value_binding_after.take());
    let name = function_name(function);
    let callable_id = name
        .as_deref()
        .and_then(|name| collector.callable_binding_id(name))
        .unwrap_or(CallableId(function.span.start));
    let pushed_syntactic_caller = collector.push_syntactic_caller(name.clone());
    if collector.current_function().is_some() {
        if let Some(name) = &name {
            collector.add_binding_name(name);
        }
    }
    if name.is_some() {
        if let Some(name) = &name {
            collector.record_callable_binding_id(name, callable_id);
        }
        collector.push_function_scope(name, callable_id);
        if let Some(scope) = collector.current_function() {
            collector.callable_scopes.insert(scope.clone());
            if collector.export_depth > 0 && collector.function_stack.len() == 1 {
                collector.exported_functions.insert(scope.clone());
                collector.record_local_export_binding(scope.clone(), scope);
            }
        }
    } else {
        collector.push_anonymous_function_scope(callable_id);
    }
    collector.add_type_parameter_names(function.type_parameters.as_deref());
    collector.add_formal_parameters(&function.params);
    walk_function_with_body_bindings(collector, function, true);
    collector.pop_function_scope(true);
    collector.pop_syntactic_caller(pushed_syntactic_caller);
    if let Some(previous) = enclosing_unreachable_binding_after {
        collector.unreachable_value_binding_after = previous;
    }
}

fn visit_arrow_function_with_value_flow_scope(
    collector: &mut ImportCollector,
    arrow: &ArrowFunctionExpression<'_>,
) {
    let enclosing_unreachable_binding_after = collector
        .collect_value_flow
        .then(|| collector.unreachable_value_binding_after.take());
    collector.push_anonymous_function_scope(CallableId(arrow.span.start));
    collector.add_type_parameter_names(arrow.type_parameters.as_deref());
    collector.add_formal_parameters(&arrow.params);
    walk_arrow_function_with_body_bindings(collector, arrow, true);
    collector.pop_function_scope(true);
    if let Some(previous) = enclosing_unreachable_binding_after {
        collector.unreachable_value_binding_after = previous;
    }
}
