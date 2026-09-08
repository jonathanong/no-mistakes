fn walk_class_with_scoped_methods<'a>(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    class: &Class<'a>,
) {
    walk::walk_decorators(collector, &class.decorators);
    if let Some(type_parameters) = &class.type_parameters {
        collector.visit_ts_type_parameter_declaration(type_parameters);
    }
    if let Some(heritage) = &class.heritage {
        collector.visit_expression(&heritage.expression);
        if let Some(type_arguments) = &heritage.type_arguments {
            collector.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }
    collector.visit_ts_class_implements_list(&class.implements);
    for element in &class.body.body {
        if let ClassElement::MethodDefinition(method) = element {
            let method_id = class_method_callable_id(class, method);
            walk_class_method_with_scope(collector, class_name, class_id, method_id, method);
        } else {
            walk::walk_class_element(collector, element);
        }
    }
}

fn class_method_callable_id(class: &Class<'_>, method: &MethodDefinition<'_>) -> CallableId {
    if !matches!(
        method.kind,
        MethodDefinitionKind::Method | MethodDefinitionKind::Constructor
    ) {
        return CallableId(method.value.span.start);
    }
    let Some(name) = crate::codebase::ts_source::static_property_key_name(&method.key) else {
        return CallableId(method.value.span.start);
    };
    class
        .body
        .body
        .iter()
        .filter_map(|element| match element {
            ClassElement::MethodDefinition(candidate)
                if candidate.kind == method.kind
                    && candidate.r#static == method.r#static
                    && candidate.value.body.is_some()
                    && crate::codebase::ts_source::static_property_key_name(&candidate.key)
                        == Some(name) =>
            {
                Some(CallableId(candidate.value.span.start))
            }
            _ => None,
        })
        .next()
        .unwrap_or(CallableId(method.value.span.start))
}

fn walk_class_method_with_scope<'a>(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    method_id: CallableId,
    method: &MethodDefinition<'a>,
) {
    walk::walk_decorators(collector, &method.decorators);
    walk::walk_property_key(collector, &method.key);
    collector.push_function_scope(Some(class_name.to_string()), class_id);
    let name = crate::codebase::ts_source::static_property_key_name(&method.key);
    let pushed = name.is_some();
    collector.push_function_scope(name.map(str::to_string), method_id);
    if let Some(scope) = collector.current_function() {
        collector.callable_scopes.insert(scope);
    }
    collector.add_type_parameter_names(method.value.type_parameters.as_deref());
    collector.add_formal_parameters(&method.value.params);
    walk_function_with_body_bindings(collector, &method.value);
    collector.pop_function_scope(pushed);
    collector.pop_function_scope(true);
}
