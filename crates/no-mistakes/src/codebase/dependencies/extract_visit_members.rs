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
    collector.push_function_scope(
        name.map(str::to_string),
        CallableId(method.value.span.start),
    );
    if let Some(scope) = collector.current_function() {
        collector.callable_scopes.insert(scope);
    }
    collector.add_type_parameter_names(method.value.type_parameters.as_deref());
    collector.add_formal_parameters(&method.value.params);
    // A dynamic key may have no display scope; its body is still repeatable.
    with_repeat_execution(
        collector,
        method.value.as_ref(),
        walk_function_with_body_bindings,
    );
    collector.pop_function_scope(pushed);
}

/// A class field evaluates its decorators and key first, then its annotation and
/// initializer. Instance initializers cannot prove eager receiver replacement.
fn visit_property_definition_with_decorators<'a>(
    collector: &mut ImportCollector,
    property: &PropertyDefinition<'a>,
) {
    walk_decorators_as_invocations(collector, &property.decorators);
    collector.visit_property_key(&property.key);
    if let Some(type_annotation) = &property.type_annotation {
        collector.visit_ts_type_annotation(type_annotation);
    }
    if let Some(value) = &property.value {
        visit_receiver_field_initializer(collector, property.r#static, value);
    }
}

fn visit_accessor_property_with_decorators<'a>(
    collector: &mut ImportCollector,
    property: &AccessorProperty<'a>,
) {
    walk_decorators_as_invocations(collector, &property.decorators);
    collector.visit_property_key(&property.key);
    if let Some(type_annotation) = &property.type_annotation {
        collector.visit_ts_type_annotation(type_annotation);
    }
    if let Some(value) = &property.value {
        visit_receiver_field_initializer(collector, property.r#static, value);
    }
}

fn visit_object_property_with_scope<'a>(
    collector: &mut ImportCollector,
    property: &ObjectProperty<'a>,
) {
    let name = crate::codebase::ts_source::static_property_key_name(&property.key);
    match &property.value {
        Expression::FunctionExpression(function) => {
            walk::walk_property_key(collector, &property.key);
            let pushed_syntactic_caller = collector.push_syntactic_caller(function_name(function));
            let pushed = name.is_some();
            collector
                .push_function_scope(name.map(str::to_string), CallableId(function.span.start));
            if let Some(scope) = collector.current_function() {
                collector.callable_scopes.insert(scope);
            }
            collector.add_type_parameter_names(function.type_parameters.as_deref());
            collector.add_formal_parameters(&function.params);
            with_repeat_execution(
                collector,
                function.as_ref(),
                walk_function_with_body_bindings,
            );
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
            with_repeat_execution(
                collector,
                arrow.as_ref(),
                walk_arrow_function_with_body_bindings,
            );
            collector.pop_function_scope(pushed);
        }
        _ => walk::walk_object_property(collector, property),
    }
}

fn visit_receiver_field_initializer<'a>(
    collector: &mut ImportCollector,
    is_static: bool,
    value: &Expression<'a>,
) {
    if is_static {
        collector.visit_expression(value);
    } else {
        // Deferred fields can run once per instance even with module context.
        collector.repeat_depth += 1;
        let active = std::mem::replace(&mut collector.fresh_receiver_effects.active, false);
        let descriptor_call = std::mem::take(&mut collector.fresh_receiver_effects.descriptor_call);
        let epoch = std::mem::replace(
            &mut collector.fresh_receiver_effects.epoch,
            u64::from(value.span().start) + 1,
        );
        with_conditional_receiver_write(collector, |collector| collector.visit_expression(value));
        collector.fresh_receiver_effects.epoch = epoch;
        collector.fresh_receiver_effects.active = active;
        collector.fresh_receiver_effects.descriptor_call = descriptor_call;
        collector.repeat_depth -= 1;
    }
}
