fn walk_object_values_with_parent_scope<'a>(
    collector: &mut ImportCollector,
    parent: &str,
    object: &ObjectExpression<'a>,
) {
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                walk_object_property_value_with_parent_scope(collector, parent, property);
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                collector.visit_expression(&spread.argument);
            }
        }
    }
}

fn walk_object_property_value_with_parent_scope<'a>(
    collector: &mut ImportCollector,
    parent: &str,
    property: &ObjectProperty<'a>,
) {
    match &property.value {
        Expression::FunctionExpression(function) => {
            collector.push_function_scope(Some(parent.to_string()), CallableId(function.span.start));
            walk::walk_property_key(collector, &property.key);
            walk_function_property_value(collector, &property.key, function);
            collector.pop_function_scope(true);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            collector.push_function_scope(Some(parent.to_string()), CallableId(arrow.span.start));
            walk::walk_property_key(collector, &property.key);
            walk_arrow_property_value(collector, &property.key, arrow);
            collector.pop_function_scope(true);
        }
        _ => {
            walk::walk_property_key(collector, &property.key);
            collector.visit_expression(&property.value);
        }
    }
}

fn walk_function_property_value<'a>(
    collector: &mut ImportCollector,
    key: &oxc_ast::ast::PropertyKey<'a>,
    function: &oxc_ast::ast::Function<'a>,
) {
    let name = crate::codebase::ts_source::static_property_key_name(key);
    let pushed = name.is_some();
    collector.push_function_scope(name.map(str::to_string), CallableId(function.span.start));
    if let Some(scope) = collector.current_function() {
        collector.callable_scopes.insert(scope);
    }
    collector.add_type_parameter_names(function.type_parameters.as_deref());
    collector.add_formal_parameters(&function.params);
    walk_function_with_body_bindings(collector, function);
    collector.pop_function_scope(pushed);
}

fn walk_arrow_property_value<'a>(
    collector: &mut ImportCollector,
    key: &oxc_ast::ast::PropertyKey<'a>,
    arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
) {
    let name = crate::codebase::ts_source::static_property_key_name(key);
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
