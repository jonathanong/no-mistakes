fn visit_variable_declarator_with_scope<'a>(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'a>,
) {
    let name = binding_identifier_name(&declarator.id).map(str::to_string);
    match declarator.init.as_ref() {
        Some(Expression::ArrowFunctionExpression(arrow)) => {
            push_variable_function_scope(collector, declarator, name);
            collector.add_type_parameter_names(arrow.type_parameters.as_deref());
            collector.add_formal_parameters(&arrow.params);
            predeclare_arrow_body(collector, arrow);
            walk::walk_arrow_function_expression(collector, arrow);
            collector.pop_function_scope(true);
        }
        Some(Expression::FunctionExpression(function)) => {
            let scope_name = name.or_else(|| function_name(function));
            push_variable_function_scope(collector, declarator, scope_name);
            collector.add_type_parameter_names(function.type_parameters.as_deref());
            collector.add_formal_parameters(&function.params);
            predeclare_function_body(collector, function);
            walk::walk_function(
                collector,
                function,
                oxc_syntax::scope::ScopeFlags::empty(),
            );
            collector.pop_function_scope(true);
        }
        Some(Expression::ObjectExpression(object))
            if name.is_some() && collector.function_stack.is_empty() =>
        {
            if let Some(name) = name.as_deref() {
                record_object_member_calls(collector, name, object);
            }
            // Treat both inline `export const` and later `export { … }` named
            // object bindings as exported, so a registry written either way keeps
            // its dynamic-import edges reachable.
            let exported = collector.export_depth > 0
                || name
                    .as_deref()
                    .is_some_and(|name| collector.is_exported_top_level_name(name));
            if exported {
                if let Some(name) = name.as_deref() {
                    collector.record_exported_resource_root(name);
                    record_object_resource_scopes(collector, name, object);
                }
                visit_exported_variable_declarator_reference(collector, declarator, name.clone());
                if let Some(name) = name.as_deref() {
                    walk_object_values_with_parent_scope(collector, name, object);
                }
            } else {
                if let Some(name) = name.as_deref() {
                    record_object_value_references(collector, name, object);
                    walk_object_values_with_parent_scope(collector, name, object);
                } else {
                    walk::walk_variable_declarator(collector, declarator);
                }
            }
        }
        Some(Expression::ClassExpression(class)) if name.is_some() && collector.function_stack.is_empty() => {
            if let Some(name) = name.as_deref() {
                record_class_member_calls(collector, name, class);
                if collector.is_exported_top_level_name(name) {
                    collector.record_exported_resource_root(name);
                    record_class_resource_scopes(collector, name, class);
                }
            }
            visit_exported_variable_declarator_reference(collector, declarator, name);
            walk::walk_variable_declarator(collector, declarator);
        }
        _ if name.is_some()
            && collector.function_stack.is_empty()
            && declarator.init.is_some() =>
        {
            visit_exported_variable_declarator_reference(collector, declarator, name);
            walk::walk_variable_declarator(collector, declarator);
        }
        _ if collector.function_stack.is_empty() && declarator.init.is_some() =>
        {
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
            collector.add_function_binding_names(&declarator.id);
        } else {
            collector.add_binding_names(&declarator.id);
        }
    }
}

fn visit_block_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    block: &BlockStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    predeclare_function_declarations(collector, &block.body);
    predeclare_lexical_declarations(collector, &block.body);
    walk::walk_block_statement(collector, block);
    collector.pop_lexical_scope(pushed);
}

fn visit_catch_clause_with_scope<'a>(collector: &mut ImportCollector, clause: &CatchClause<'a>) {
    let pushed = collector.push_lexical_scope();
    if let Some(param) = &clause.param {
        collector.add_binding_names(&param.pattern);
    }
    walk::walk_catch_clause(collector, clause);
    collector.pop_lexical_scope(pushed);
}
