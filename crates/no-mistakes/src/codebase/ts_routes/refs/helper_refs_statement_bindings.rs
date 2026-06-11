fn register_helper_bindings_from_statement(
    stmt: &Statement<'_>,
    bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
) {
    match stmt {
        Statement::VariableDeclaration(var_decl) => {
            remove_shadowed_helper_var_bindings(var_decl, bindings, local_helpers);
        }
        Statement::FunctionDeclaration(func) => {
            remove_shadowed_helper_function_binding(func, bindings, local_helpers);
        }
        Statement::ClassDeclaration(class) => {
            remove_shadowed_helper_class_binding(class, bindings);
        }
        Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
            Some(oxc::ast::ast::Declaration::VariableDeclaration(var_decl)) => {
                remove_shadowed_helper_var_bindings(var_decl, bindings, local_helpers);
            }
            Some(oxc::ast::ast::Declaration::FunctionDeclaration(func)) => {
                remove_shadowed_helper_function_binding(func, bindings, local_helpers);
            }
            Some(oxc::ast::ast::Declaration::ClassDeclaration(class)) => {
                remove_shadowed_helper_class_binding(class, bindings);
            }
            _ => {}
        },
        _ => {}
    }
}

fn collect_scope_helper_bindings(
    stmts: &[Statement<'_>],
    bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
) {
    for stmt in stmts {
        register_helper_bindings_from_statement(stmt, bindings, local_helpers);
    }
}
