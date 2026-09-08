fn visit_method_definition_with_scope<'a>(
    collector: &mut ImportCollector,
    method: &MethodDefinition<'a>,
) {
    let name = crate::codebase::ts_source::static_property_key_name(&method.key);
    // Decorators and computed keys are evaluated where the class expression
    // appears, before entering the method's callable scope.
    walk_decorators_as_invocations(collector, &method.decorators);
    walk::walk_property_key(collector, &method.key);
    let pushed = name.is_some();
    collector.push_function_scope(name.map(str::to_string), CallableId(method.value.span.start));
    if let Some(scope) = collector.current_function() {
        collector.callable_scopes.insert(scope);
    }
    collector.add_type_parameter_names(method.value.type_parameters.as_deref());
    collector.add_formal_parameters(&method.value.params);
    walk_function_with_body_bindings(collector, &method.value);
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
            collector.push_function_scope(name.map(str::to_string), CallableId(function.span.start));
            if let Some(scope) = collector.current_function() {
                collector.callable_scopes.insert(scope);
            }
            collector.add_type_parameter_names(function.type_parameters.as_deref());
            collector.add_formal_parameters(&function.params);
            walk_function_with_body_bindings(collector, function);
            collector.pop_function_scope(pushed);
            collector.pop_syntactic_caller(pushed_syntactic_caller);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            walk::walk_property_key(collector, &property.key);
            let pushed = name.is_some();
            collector.push_function_scope(name.map(str::to_string), CallableId(arrow.span.start));
            if let Some(scope) = collector.current_function() {
                collector.callable_scopes.insert(scope);
            }
            collector.add_type_parameter_names(arrow.type_parameters.as_deref());
            collector.add_formal_parameters(&arrow.params);
            walk_arrow_function_with_body_bindings(collector, arrow);
            collector.pop_function_scope(pushed);
        }
        _ => walk::walk_object_property(collector, property),
    }
}
