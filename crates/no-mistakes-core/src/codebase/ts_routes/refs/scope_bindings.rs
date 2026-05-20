fn collect_function_scope_var_bindings<'a>(
    stmts: &'a [Statement<'a>],
    bindings: &mut RouterBindings<'a>,
) {
    for stmt in stmts {
        match stmt {
            Statement::VariableDeclaration(var_decl)
                if var_decl.kind == VariableDeclarationKind::Var =>
            {
                collect_router_bindings_from_var_decl(var_decl, bindings);
            }
            Statement::BlockStatement(block) => {
                collect_function_scope_var_bindings(&block.body, bindings);
            }
            Statement::IfStatement(if_stmt) => {
                collect_function_scope_var_bindings(
                    std::slice::from_ref(&if_stmt.consequent),
                    bindings,
                );
                if let Some(alt) = &if_stmt.alternate {
                    collect_function_scope_var_bindings(std::slice::from_ref(alt), bindings);
                }
            }
            Statement::ForStatement(for_stmt) => {
                match &for_stmt.init {
                    Some(ForStatementInit::VariableDeclaration(var_decl))
                        if var_decl.kind == VariableDeclarationKind::Var =>
                    {
                        collect_router_bindings_from_var_decl(var_decl, bindings);
                    }
                    _ => {}
                }
                collect_function_scope_var_bindings(std::slice::from_ref(&for_stmt.body), bindings);
            }
            Statement::ForInStatement(for_stmt) => {
                collect_for_statement_left_var_bindings(&for_stmt.left, bindings);
                collect_function_scope_var_bindings(std::slice::from_ref(&for_stmt.body), bindings);
            }
            Statement::ForOfStatement(for_stmt) => {
                collect_for_statement_left_var_bindings(&for_stmt.left, bindings);
                collect_function_scope_var_bindings(std::slice::from_ref(&for_stmt.body), bindings);
            }
            Statement::WhileStatement(while_stmt) => {
                collect_function_scope_var_bindings(
                    std::slice::from_ref(&while_stmt.body),
                    bindings,
                );
            }
            Statement::DoWhileStatement(do_while_stmt) => {
                collect_function_scope_var_bindings(
                    std::slice::from_ref(&do_while_stmt.body),
                    bindings,
                );
            }
            Statement::SwitchStatement(switch_stmt) => {
                for case in &switch_stmt.cases {
                    collect_function_scope_var_bindings(&case.consequent, bindings);
                }
            }
            Statement::TryStatement(try_stmt) => {
                collect_function_scope_var_bindings(&try_stmt.block.body, bindings);
                if let Some(handler) = &try_stmt.handler {
                    collect_function_scope_var_bindings(&handler.body.body, bindings);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    collect_function_scope_var_bindings(&finalizer.body, bindings);
                }
            }
            _ => {}
        }
    }
}

fn collect_router_bindings_for_scope<'a>(
    stmts: &'a [Statement<'a>],
    bindings: &mut RouterBindings<'a>,
) {
    collect_scope_router_bindings(stmts, bindings);
    collect_function_scope_var_bindings(stmts, bindings);
}

fn collect_for_statement_left_var_bindings<'a>(
    left: &'a ForStatementLeft<'a>,
    bindings: &mut RouterBindings<'a>,
) {
    if let ForStatementLeft::VariableDeclaration(var_decl) = left {
        if var_decl.kind == VariableDeclarationKind::Var {
            collect_router_bindings_from_var_decl(var_decl, bindings);
        }
    }
}

fn collect_router_bindings_from_var_decl<'a>(
    var_decl: &'a oxc::ast::ast::VariableDeclaration<'a>,
    bindings: &mut RouterBindings<'a>,
) {
    for decl in &var_decl.declarations {
        remove_shadowed_binding(&decl.id, bindings);
        if decl
            .init
            .as_ref()
            .map(|init| is_use_router_call(init))
            .unwrap_or(false)
        {
            add_router_binding_pattern(&decl.id, bindings);
        }
    }
}

fn add_router_binding_pattern<'a>(
    pattern: &'a BindingPattern<'a>,
    bindings: &mut RouterBindings<'a>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            bindings.objects.insert(id.name.as_str());
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                let Some(key) = prop.key.static_name() else {
                    continue;
                };
                if !matches!(key.as_ref(), "push" | "replace" | "prefetch") {
                    continue;
                }
                if let Some(name) = router_method_binding_name(&prop.value) {
                    bindings.methods.insert(name);
                }
            }
        }
        _ => {}
    }
}
