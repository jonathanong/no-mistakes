fn collect_import_bindings<'a>(stmts: &'a [Statement<'a>]) -> RouterBindings<'a> {
    let mut bindings = RouterBindings::default();
    for stmt in stmts {
        let Statement::ImportDeclaration(import) = stmt else {
            continue;
        };
        if import.source.value.as_str() != "next/navigation" {
            mark_shadowed_fetch_import(import, &mut bindings);
            continue;
        }
        let Some(specifiers) = &import.specifiers else {
            continue;
        };
        for specifier in specifiers {
            let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                continue;
            };
            if specifier.imported.name().as_str() == "redirect" {
                bindings.redirects.insert(specifier.local.name.as_str());
            }
        }
    }
    bindings
}

fn mark_shadowed_fetch_import(import: &oxc_ast::ast::ImportDeclaration<'_>, bindings: &mut RouterBindings<'_>) {
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier)
                if specifier.local.name.as_str() == "fetch" =>
            {
                bindings.fetch_shadowed = true;
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier)
                if specifier.local.name.as_str() == "fetch" =>
            {
                bindings.fetch_shadowed = true;
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier)
                if specifier.local.name.as_str() == "fetch" =>
            {
                bindings.fetch_shadowed = true;
            }
            _ => {}
        }
    }
}

fn register_router_bindings_from_statement<'a>(
    stmt: &'a Statement<'a>,
    bindings: &mut RouterBindings<'a>,
) {
    match stmt {
        Statement::VariableDeclaration(var_decl) => {
            collect_router_bindings_from_var_decl(var_decl, bindings);
        }
        Statement::FunctionDeclaration(func) => {
            remove_shadowed_function_binding(func, bindings);
        }
        Statement::ClassDeclaration(class) => {
            remove_shadowed_class_binding(class, bindings);
        }
        Statement::ForStatement(for_stmt) => match &for_stmt.init {
            Some(ForStatementInit::VariableDeclaration(var_decl))
                if var_decl.kind == VariableDeclarationKind::Var =>
            {
                collect_router_bindings_from_var_decl(var_decl, bindings);
            }
            _ => {}
        },
        Statement::ForInStatement(for_stmt) => {
            collect_for_statement_left_var_bindings(&for_stmt.left, bindings);
        }
        Statement::ForOfStatement(for_stmt) => {
            collect_for_statement_left_var_bindings(&for_stmt.left, bindings);
        }
        Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
            Some(oxc_ast::ast::Declaration::VariableDeclaration(var_decl)) => {
                collect_router_bindings_from_var_decl(var_decl, bindings);
            }
            Some(oxc_ast::ast::Declaration::FunctionDeclaration(func)) => {
                remove_shadowed_function_binding(func, bindings);
            }
            Some(oxc_ast::ast::Declaration::ClassDeclaration(class)) => {
                remove_shadowed_class_binding(class, bindings);
            }
            _ => {}
        },
        _ => {}
    }
}

fn collect_scope_router_bindings<'a>(
    stmts: &'a [Statement<'a>],
    bindings: &mut RouterBindings<'a>,
) {
    for stmt in stmts {
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                collect_router_bindings_from_var_decl(var_decl, bindings);
            }
            Statement::FunctionDeclaration(func) => {
                remove_shadowed_function_binding(func, bindings);
            }
            Statement::ClassDeclaration(class) => {
                remove_shadowed_class_binding(class, bindings);
            }
            Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
                Some(oxc_ast::ast::Declaration::VariableDeclaration(var_decl)) => {
                    collect_router_bindings_from_var_decl(var_decl, bindings);
                }
                Some(oxc_ast::ast::Declaration::FunctionDeclaration(func)) => {
                    remove_shadowed_function_binding(func, bindings);
                }
                Some(oxc_ast::ast::Declaration::ClassDeclaration(class)) => {
                    remove_shadowed_class_binding(class, bindings);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
