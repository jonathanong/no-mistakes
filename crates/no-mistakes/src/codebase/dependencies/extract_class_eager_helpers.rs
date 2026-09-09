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
        record_class_base_symbol_reference(collector, class_name, class_id, class);
        collector.visit_expression(&heritage.expression);
        if let Some(type_arguments) = &heritage.type_arguments {
            collector.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }
    collector.visit_ts_class_implements_list(&class.implements);
    for element in &class.body.body {
        if let ClassElement::MethodDefinition(method) = element {
            let method_id = class_method_callable_id(class, method);
            record_class_computed_key_symbol_reference(
                collector,
                class_name,
                class_id,
                &method.key,
            );
            if let Some(name) = crate::codebase::ts_source::static_property_key_name(&method.key) {
                collector.record_aggregate_callable_member_id(class_id, name, method_id);
            }
            if method.r#static {
                if let Some(name) =
                    crate::codebase::ts_source::static_property_key_name(&method.key)
                {
                    collector.record_class_member_callable_id(class_id, name, method_id);
                    if method.kind == MethodDefinitionKind::Get {
                        owner_name_insert(
                            &mut collector.static_getter_member_ids,
                            class_id,
                            name.to_string(),
                        );
                    } else if method.kind == MethodDefinitionKind::Set {
                        owner_name_insert(
                            &mut collector.static_setter_member_ids,
                            class_id,
                            name.to_string(),
                        );
                    }
                }
            }
            walk_class_method_with_scope(collector, class_name, class_id, method_id, method);
        } else if let ClassElement::PropertyDefinition(property) = element {
            record_class_computed_key_symbol_reference(
                collector,
                class_name,
                class_id,
                &property.key,
            );
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

fn record_class_computed_key_symbol_reference(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    key: &oxc_ast::ast::PropertyKey<'_>,
) {
    let Some(name) = key.as_expression().and_then(simple_callee_name) else {
        return;
    };
    if collector.callee_shadows_import(&name) {
        return;
    }
    collector.symbol_references.push(FunctionCall {
        caller: Some(class_name.to_string()),
        caller_id: Some(class_id),
        syntactic_caller: collector.current_syntactic_caller(),
        callee_binding_scope: collector.callee_binding_scope(&name),
        callee: name,
        line: 0,
        offset: 0,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: CallTargetIdentity::Unknown,
        static_arg: None,
        static_cwd: None,
    });
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

fn visit_class_static_block_with_scope<'a>(
    collector: &mut ImportCollector,
    block: &StaticBlock<'a>,
) {
    let pushed = collector.push_lexical_scope();
    if pushed {
        collector
            .var_scope_stack
            .push(collector.local_stack.len() - 1);
    }
    predeclare_function_declarations(collector, &block.body);
    walk::walk_static_block(collector, block);
    if pushed {
        collector.var_scope_stack.pop();
    }
    collector.pop_lexical_scope(pushed);
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
