fn walk_class_with_scoped_methods<'a>(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    class: &Class<'a>,
) {
    walk_decorators_as_invocations(collector, &class.decorators);
    let pushed_class_scope = collector.push_lexical_scope();
    if let Some(name) = class.id.as_ref().map(|id| id.name.as_str()) {
        collector.add_binding_name(name);
        collector.record_callable_binding_id(name, class_id);
    }
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
            if let Some(name) =
                crate::codebase::ts_source::static_property_key_name(&method.key)
            {
                collector.record_class_callable_member_id(class_id, name, method_id);
            }
            if method.r#static {
                if let Some(name) = crate::codebase::ts_source::static_property_key_name(&method.key) {
                    collector.record_class_member_callable_id(class_id, name, method_id);
                }
            }
            walk_class_method_with_scope(collector, class_name, class_id, method_id, method);
        } else if let ClassElement::PropertyDefinition(property) = element {
            if let Some((name, callable_id)) = static_callable_field(property) {
                collector.record_class_member_callable_id(class_id, &name, callable_id);
                walk_static_callable_field_with_scope(
                    collector,
                    class_name,
                    class_id,
                    &name,
                    callable_id,
                    property,
                );
            } else if !property.r#static {
                walk_instance_property_with_class_scope(collector, class_name, class_id, property);
            } else {
                walk::walk_class_element(collector, element);
            }
        } else {
            walk::walk_class_element(collector, element);
        }
    }
    collector.pop_lexical_scope(pushed_class_scope);
}

fn walk_instance_property_with_class_scope<'a>(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    property: &PropertyDefinition<'a>,
) {
    walk_decorators_as_invocations(collector, &property.decorators);
    collector.visit_property_key(&property.key);
    if let Some(type_annotation) = &property.type_annotation {
        collector.visit_ts_type_annotation(type_annotation);
    }
    collector.push_function_scope(Some(class_name.to_string()), class_id);
    if let Some(value) = &property.value {
        collector.visit_expression(value);
    }
    collector.pop_function_scope(true);
}

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

fn visit_class_static_block_with_scope<'a>(
    collector: &mut ImportCollector,
    block: &StaticBlock<'a>,
) {
    let pushed = collector.push_lexical_scope();
    if pushed {
        collector.var_scope_stack.push(collector.local_stack.len() - 1);
    }
    predeclare_function_declarations(collector, &block.body);
    walk::walk_static_block(collector, block);
    if pushed {
        collector.var_scope_stack.pop();
    }
    collector.pop_lexical_scope(pushed);
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

/// Decorators evaluate in the class's enclosing lexical scope, not inside the
/// class or decorated member. A bare static decorator is still an invocation
/// at runtime even though the AST represents it as an expression rather than
/// a `CallExpression`.
fn walk_decorators_as_invocations<'a>(
    collector: &mut ImportCollector,
    decorators: &oxc_allocator::Vec<'a, oxc_ast::ast::Decorator<'a>>,
) {
    record_decorator_invocations(collector, decorators);
    walk::walk_decorators(collector, decorators);
}

fn record_decorator_invocations<'a>(
    collector: &mut ImportCollector,
    decorators: &oxc_allocator::Vec<'a, oxc_ast::ast::Decorator<'a>>,
) {
    for decorator in decorators {
        let line = import_line_at(&collector.line_starts, decorator.span.start as usize);
        if let Some(callee) = simple_callee_name(&decorator.expression) {
            let target_identity = collector.call_target_identity(&callee);
            let callee_binding_scope = collector.callee_binding_scope(&callee);
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                caller_id: collector.current_function_id(),
                syntactic_caller: collector.current_syntactic_caller(),
                callee,
                line,
                offset: decorator.span.start,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity,
                callee_binding_scope,
                static_arg: None,
                static_cwd: None,
            });
            if has_dynamic_static_member_receiver(&decorator.expression) {
                collector.record_unknown_call(line, decorator.span.start, InvocationKind::Call);
            }
        } else {
            // A decorator factory or computed member can execute, but its
            // resulting decorator cannot be named without guessing.
            collector.record_unknown_call(line, decorator.span.start, InvocationKind::Call);
        }
    }
}
