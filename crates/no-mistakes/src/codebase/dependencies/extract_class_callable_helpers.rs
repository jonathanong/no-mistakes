#[inline(never)]
fn static_callable_field(property: &PropertyDefinition<'_>) -> Option<(String, CallableId)> {
    if !property.r#static {
        return None;
    }
    let name = crate::codebase::ts_source::static_property_key_name(&property.key)?.to_string();
    let value = property.value.as_ref()?;
    let id = match value {
        Expression::FunctionExpression(function) => CallableId(function.span.start),
        Expression::ArrowFunctionExpression(arrow) => CallableId(arrow.span.start),
        _ => return None,
    };
    Some((name, id))
}

#[inline(never)]
fn walk_static_callable_field_with_scope<'a>(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    name: &str,
    callable_id: CallableId,
    property: &PropertyDefinition<'a>,
) {
    walk_decorators_as_invocations(collector, &property.decorators);
    collector.visit_property_key(&property.key);
    if let Some(type_annotation) = &property.type_annotation {
        collector.visit_ts_type_annotation(type_annotation);
    }
    collector.push_function_scope(Some(class_name.to_string()), class_id);
    collector.push_function_scope(Some(name.to_string()), callable_id);
    if let Some(scope) = collector.current_function() {
        collector.callable_scopes.insert(scope);
    }
    match property
        .value
        .as_ref()
        .expect("static callable field has a value")
    {
        Expression::FunctionExpression(function) => {
            collector.add_type_parameter_names(function.type_parameters.as_deref());
            collector.add_formal_parameters(&function.params);
            walk_function_with_body_bindings(collector, function);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            collector.add_type_parameter_names(arrow.type_parameters.as_deref());
            collector.add_formal_parameters(&arrow.params);
            walk_arrow_function_with_body_bindings(collector, arrow);
        }
        _ => unreachable!("static callable field was validated before walking"),
    }
    collector.pop_function_scope(true);
    collector.pop_function_scope(true);
}

#[inline(never)]
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

#[inline(never)]
fn walk_class_method_with_scope<'a>(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    method_id: CallableId,
    method: &MethodDefinition<'a>,
) {
    walk_decorators_as_invocations(collector, &method.decorators);
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
