fn visit_function_with_scope<'a>(
    collector: &mut ImportCollector,
    function: &oxc_ast::ast::Function<'a>,
    flags: oxc_syntax::scope::ScopeFlags,
) {
    let name = function_name(function);
    if collector.current_function().is_some() {
        if let Some(name) = &name {
            collector.add_binding_name(name);
        }
    }
    if name.is_some() {
        collector.push_function_scope(name);
        if let Some(scope) = collector.current_function() {
            collector.callable_scopes.insert(scope.clone());
            if collector.export_depth > 0 && collector.function_stack.len() == 1 {
                collector.exported_functions.insert(scope);
            }
        }
    } else {
        collector.push_anonymous_function_scope();
    }
    collector.add_type_parameter_names(function.type_parameters.as_deref());
    collector.add_formal_parameters(&function.params);
    predeclare_function_body(collector, function);
    walk::walk_function(collector, function, flags);
    collector.pop_function_scope(true);
}

fn visit_arrow_function_with_scope<'a>(
    collector: &mut ImportCollector,
    arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
) {
    collector.push_anonymous_function_scope();
    collector.add_type_parameter_names(arrow.type_parameters.as_deref());
    collector.add_formal_parameters(&arrow.params);
    predeclare_arrow_body(collector, arrow);
    walk::walk_arrow_function_expression(collector, arrow);
    collector.pop_function_scope(true);
}
