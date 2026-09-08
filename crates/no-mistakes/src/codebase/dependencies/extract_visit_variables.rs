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
        Some(Expression::ObjectExpression(object))
            if name.is_some() && collector.function_stack.is_empty() =>
        {
            if let Some(name) = name.as_deref() {
                record_object_member_calls(collector, name, CallableId(declarator.span.start), object);
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
                visit_exported_variable_declarator_reference(collector, declarator, name);
            } else {
                if let Some(name) = name.as_deref() {
                    record_object_value_references(collector, name, object);
                    walk_object_values_with_parent_scope(collector, name, object);
                } else {
                    walk::walk_variable_declarator(collector, declarator);
                }
            }
        }
        Some(Expression::ClassExpression(class))
            if name.is_some() && collector.function_stack.is_empty() =>
        {
            if let Some(name) = name.as_deref() {
                collector.record_callable_binding_id(name, CallableId(class.span.start));
                record_class_member_calls(collector, name, CallableId(class.span.start), class);
                if collector.is_exported_top_level_name(name) {
                    collector.record_exported_resource_root(name);
                    record_class_resource_scopes(collector, name, class);
                }
            }
            visit_exported_variable_declarator_reference(collector, declarator, name);
            walk::walk_variable_declarator(collector, declarator);
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

fn visit_block_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    block: &BlockStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    predeclare_function_declarations(collector, &block.body);
    walk::walk_block_statement(collector, block);
    collector.pop_lexical_scope(pushed);
}

fn visit_switch_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    switch: &oxc_ast::ast::SwitchStatement<'a>,
) {
    // Case consequents share one lexical environment, while the discriminant
    // is evaluated outside it.
    collector.visit_expression(&switch.discriminant);
    let pushed = collector.push_lexical_scope();
    for case in &switch.cases {
        predeclare_function_declarations(collector, &case.consequent);
    }
    collector.visit_switch_cases(&switch.cases);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    if let Some(init) = &statement.init {
        collector.visit_for_statement_init(init);
    }
    if let Some(test) = &statement.test {
        collector.visit_expression(test);
    }
    if let Some(update) = &statement.update {
        collector.visit_expression(update);
    }
    collector.visit_statement(&statement.body);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_in_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForInStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    collector.visit_for_statement_left(&statement.left);
    collector.visit_expression(&statement.right);
    collector.visit_statement(&statement.body);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_of_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForOfStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    collector.visit_for_statement_left(&statement.left);
    collector.visit_expression(&statement.right);
    collector.visit_statement(&statement.body);
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
