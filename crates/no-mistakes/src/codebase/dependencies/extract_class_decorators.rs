fn record_class_member_calls(
    collector: &mut ImportCollector,
    class_name: Option<&str>,
    class_id: CallableId,
    class: &Class<'_>,
) -> bool {
    let mut decorated_members = false;
    for element in &class.body.body {
        decorated_members |= match element {
            ClassElement::MethodDefinition(method) => !method.decorators.is_empty(),
            ClassElement::PropertyDefinition(property) => !property.decorators.is_empty(),
            ClassElement::AccessorProperty(property) => !property.decorators.is_empty(),
            _ => false,
        };
        match element {
            ClassElement::MethodDefinition(method) => {
                let name = crate::codebase::ts_source::static_property_key_name(&method.key);
                if let Some(class_name) = class_name {
                    record_member_call(collector, class_name, class_id, name);
                }
            }
            ClassElement::PropertyDefinition(property)
                if property.r#static
                    && matches!(
                        property.value,
                        Some(
                            Expression::FunctionExpression(_)
                                | Expression::ArrowFunctionExpression(_)
                        )
                    ) =>
            {
                if let Some(class_name) = class_name {
                    record_member_call(
                        collector,
                        class_name,
                        class_id,
                        crate::codebase::ts_source::static_property_key_name(&property.key),
                    );
                }
            }
            _ => {}
        }
    }
    decorated_members
}

/// Decorators evaluate in the class's enclosing lexical scope, not inside the
/// class or decorated member. A bare static decorator is still an invocation
/// at runtime even though the AST represents it as an expression rather than
/// a `CallExpression`.
fn walk_decorators_as_invocations<'a>(
    collector: &mut ImportCollector,
    decorators: &oxc_allocator::Vec<'a, oxc_ast::ast::Decorator<'a>>,
) {
    collector.decorator_applications.member |= !decorators.is_empty();
    collector.decorator_applications.any |= !decorators.is_empty();
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
            collector.mark_call_namespace_head(&decorator.expression);
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

// Keep nested/deferred classes from lending their application state to the
// containing class. Receiver events retain the enclosing callable and epoch.
fn begin_decorator_applications(
    collector: &mut ImportCollector,
    class: &Class<'_>,
    member: bool,
) -> DecoratorApplications {
    std::mem::replace(
        &mut collector.decorator_applications,
        DecoratorApplications {
            member,
            any: member || !class.decorators.is_empty(),
            static_started: false,
        },
    )
}

fn finish_decorator_applications(
    collector: &mut ImportCollector,
    class: &Class<'_>,
    previous: DecoratorApplications,
) {
    if collector.decorator_applications.any {
        // This is application of the evaluated decorator, not evaluation of a
        // factory expression. Record it after captures created by class keys.
        collector.record_descriptor_call(class.span.end);
    }
    collector.decorator_applications = previous;
}

fn begin_static_decorator_applications(collector: &mut ImportCollector, offset: u32) {
    if collector.decorator_applications.member && !collector.decorator_applications.static_started {
        // Member decorators can run before eager static initialization.
        collector.record_descriptor_call(offset);
        collector.decorator_applications.static_started = true;
    }
}
