fn visit_variable_declarator_with_scope<'a>(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'a>,
) {
    let name = binding_identifier_name(&declarator.id).map(str::to_string);
    match declarator.init.as_ref() {
        Some(Expression::ArrowFunctionExpression(arrow)) => {
            let pushed_syntactic_caller = collector.push_syntactic_caller(name.clone());
            push_variable_function_scope(collector, declarator, name, CallableId(arrow.span.start));
            collector.add_type_parameter_names(arrow.type_parameters.as_deref());
            collector.add_formal_parameters(&arrow.params);
            walk_arrow_function_with_body_bindings(collector, arrow);
            collector.pop_function_scope(true);
            collector.pop_syntactic_caller(pushed_syntactic_caller);
        }
        Some(Expression::FunctionExpression(function)) => {
            // The legacy source-occurrence owner is the variable binding even
            // when a function expression also has an internal name.
            let source_name = name.clone().or_else(|| function_name(function));
            let pushed_syntactic_caller = collector.push_syntactic_caller(source_name);
            let scope_name = name.or_else(|| function_name(function));
            push_variable_function_scope(
                collector,
                declarator,
                scope_name,
                CallableId(function.span.start),
            );
            if let Some(self_name) = function_name(function) {
                collector.add_binding_name(&self_name);
                collector.record_callable_binding(&self_name);
                collector.callable_aliases.push(CallableAliasBinding {
                    alias: CallableAlias {
                        scope: collector.current_function(),
                        scope_id: collector.current_function_id(),
                        local: self_name,
                        target: collector.current_function().expect("named function scope"),
                        binding_scope: collector.current_lexical_scope_id(),
                        declared_at: function.span.start,
                        invalidated_at: None,
                    },
                    lexical_scope_depth: collector.local_stack.len() - 1,
                });
            }
            collector.add_type_parameter_names(function.type_parameters.as_deref());
            collector.add_formal_parameters(&function.params);
            walk_function_with_body_bindings(collector, function);
            collector.pop_function_scope(true);
            collector.pop_syntactic_caller(pushed_syntactic_caller);
        }
        Some(Expression::ObjectExpression(object)) if name.is_some() => {
            let name = name.expect("object branch requires a binding name");
            let member_scope = collector.callable_scope_name(&name);
            record_object_member_calls(
                collector,
                &name,
                &member_scope,
                CallableId(declarator.span.start),
                object,
            );
            // Treat both inline `export const` and later `export { … }` named
            // object bindings as exported, so a registry written either way keeps
            // its dynamic-import edges reachable.
            let exported = collector.function_stack.is_empty()
                && (collector.export_depth > 0 || collector.is_exported_top_level_name(&name));
            if exported {
                collector.record_exported_resource_root(&name);
                record_object_resource_scopes(collector, &name, object);
                visit_exported_variable_declarator_reference(collector, declarator, Some(name));
            } else {
                record_object_value_references(collector, &member_scope, object);
                walk_object_values_with_parent_scope(collector, &name, object);
            }
        }
        Some(Expression::ClassExpression(class)) if name.is_some() => {
            if let Some(name) = name.as_deref() {
                let class_id = CallableId(class.span.start);
                // A named class expression has two bindings to one class:
                // `Public` is observable around the expression, while
                // `Internal` is the class body's self-reference. Keep method
                // scopes under the latter, but retain the outward binding so
                // calls through either spelling share the class identity.
                let class_name = class.id.as_ref().map_or(name, |id| id.name.as_str());
                let scope = collector.callable_scope_name(class_name);
                collector.record_callable_binding_id(name, class_id);
                record_class_member_calls(collector, &scope, class_id, class);
                record_class_base_construction(collector, &scope, class_id, class);
                collector.known_function_scopes.insert(scope.clone());
                collector
                    .callable_scope_ids
                    .insert((class_id, scope.clone()));
                collector.callable_scopes.insert(scope.clone());
                collector.class_scopes.insert(scope);
                if collector.function_stack.is_empty() && collector.is_exported_top_level_name(name)
                {
                    collector.record_exported_resource_root(name);
                    record_class_resource_scopes(collector, name, class);
                }
                collector.visit_binding_pattern(&declarator.id);
                walk_variable_type_annotation(collector, declarator);
                walk_class_with_scoped_methods(collector, class_name, class_id, class);
            }
        }
        _ if name.is_some() && collector.function_stack.is_empty() && declarator.init.is_some() => {
            visit_exported_variable_declarator_reference(collector, declarator, name);
            walk::walk_variable_declarator(collector, declarator);
        }
        _ if collector.function_stack.is_empty() && declarator.init.is_some() => {
            visit_variable_declarator_references_for_bindings(collector, declarator);
            walk::walk_variable_declarator(collector, declarator);
        }
        _ => walk::walk_variable_declarator(collector, declarator),
    }
}

fn visit_variable_declaration_with_bindings<'a>(
    collector: &mut ImportCollector,
    declaration: &VariableDeclaration<'a>,
) {
    for declarator in &declaration.declarations {
        if declaration.kind == VariableDeclarationKind::Var {
            collector.add_var_binding_names(&declarator.id);
        } else {
            collector.add_binding_names(&declarator.id);
        }
    }
}
