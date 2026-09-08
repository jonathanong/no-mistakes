fn visit_method_definition_with_scope<'a>(
    collector: &mut ImportCollector,
    method: &MethodDefinition<'a>,
) {
    let name = crate::codebase::ts_source::static_property_key_name(&method.key);
    let keep_class_scope = collector.current_function().is_some_and(|scope| {
        collector.class_scopes.contains(&scope) && collector.exported_functions.contains(&scope)
    });
    let saved_function_stack =
        (!keep_class_scope).then(|| std::mem::take(&mut collector.function_stack));
    walk::walk_decorators(collector, &method.decorators);
    walk::walk_property_key(collector, &method.key);
    if let Some(saved_function_stack) = saved_function_stack {
        collector.function_stack = saved_function_stack;
    }
    let pushed = name.is_some();
    collector.push_function_scope(name.map(str::to_string));
    if let Some(scope) = collector.current_function() {
        collector.callable_scopes.insert(scope);
    }
    collector.add_type_parameter_names(method.value.type_parameters.as_deref());
    collector.add_formal_parameters(&method.value.params);
    walk::walk_function(
        collector,
        &method.value,
        oxc_syntax::scope::ScopeFlags::empty(),
    );
    collector.pop_function_scope(pushed);
}

fn visit_object_property_with_scope<'a>(
    collector: &mut ImportCollector,
    property: &ObjectProperty<'a>,
) {
    let name = crate::codebase::ts_source::static_property_key_name(&property.key);
    match &property.value {
        Expression::FunctionExpression(function) => {
            walk::walk_property_key(collector, &property.key);
            let pushed_syntactic_caller =
                collector.push_syntactic_caller(function_name(function));
            let pushed = name.is_some();
            collector.push_function_scope(name.map(str::to_string));
            if let Some(scope) = collector.current_function() {
                collector.callable_scopes.insert(scope);
            }
            collector.add_type_parameter_names(function.type_parameters.as_deref());
            collector.add_formal_parameters(&function.params);
            walk::walk_function(collector, function, oxc_syntax::scope::ScopeFlags::empty());
            collector.pop_function_scope(pushed);
            collector.pop_syntactic_caller(pushed_syntactic_caller);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            walk::walk_property_key(collector, &property.key);
            let pushed = name.is_some();
            collector.push_function_scope(name.map(str::to_string));
            if let Some(scope) = collector.current_function() {
                collector.callable_scopes.insert(scope);
            }
            collector.add_type_parameter_names(arrow.type_parameters.as_deref());
            collector.add_formal_parameters(&arrow.params);
            walk::walk_arrow_function_expression(collector, arrow);
            collector.pop_function_scope(pushed);
        }
        _ => walk::walk_object_property(collector, property),
    }
}
