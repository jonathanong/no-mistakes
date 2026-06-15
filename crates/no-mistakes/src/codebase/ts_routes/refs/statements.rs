fn is_use_router_call(expr: &Expression) -> bool {
    if let Expression::CallExpression(call) = expr {
        match &call.callee {
            Expression::Identifier(id) => id.name.as_str() == "useRouter",
            other => other
                .as_member_expression()
                .and_then(|m| m.static_property_name())
                .map(|n| n == "useRouter")
                .unwrap_or(false),
        }
    } else {
        false
    }
}

fn collect_from_statement<'a>(
    stmt: &'a Statement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    refs: &mut Vec<RouteRef>,
) {
    register_router_bindings_from_statement(stmt, router_bindings);

    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            collect_from_expression(&expr_stmt.expression, source, file, router_bindings, refs);
        }
        Statement::ReturnStatement(ret_stmt) => {
            if let Some(expr) = &ret_stmt.argument {
                collect_from_expression(expr, source, file, router_bindings, refs);
            }
        }
        Statement::BlockStatement(block) => {
            let mut scoped_bindings = router_bindings.clone();
            collect_router_bindings_for_scope(&block.body, &mut scoped_bindings);
            for s in &block.body {
                collect_from_statement(s, source, file, &mut scoped_bindings, refs);
            }
        }
        Statement::IfStatement(if_stmt) => {
            collect_from_expression(&if_stmt.test, source, file, router_bindings, refs);
            collect_from_statement(&if_stmt.consequent, source, file, router_bindings, refs);
            if let Some(alt) = &if_stmt.alternate {
                collect_from_statement(alt, source, file, router_bindings, refs);
            }
        }
        Statement::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                if let Some(init) = &decl.init {
                    collect_from_expression(init, source, file, router_bindings, refs);
                }
            }
        }
        Statement::FunctionDeclaration(func) => {
            if let Some(body) = &func.body {
                let mut scoped_bindings = router_bindings.clone();
                remove_shadowed_function_binding(func, &mut scoped_bindings);
                remove_shadowed_parameters(&func.params, &mut scoped_bindings);
                collect_router_bindings_for_scope(&body.statements, &mut scoped_bindings);
                for s in &body.statements {
                    collect_from_statement(s, source, file, &mut scoped_bindings, refs);
                }
            }
        }
        Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
            Some(oxc_ast::ast::Declaration::VariableDeclaration(var_decl)) => {
                for init in var_decl
                    .declarations
                    .iter()
                    .filter_map(|decl| decl.init.as_ref())
                {
                    collect_from_expression(init, source, file, router_bindings, refs);
                }
            }
            Some(oxc_ast::ast::Declaration::FunctionDeclaration(func)) => {
                collect_from_function_body(func, source, file, router_bindings, refs);
            }
            _ => {}
        },
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                collect_from_function_body(func, source, file, router_bindings, refs);
            }
            other => {
                if let Some(expr) = other.as_expression() {
                    collect_from_expression(expr, source, file, router_bindings, refs);
                }
            }
        },
        _ => {}
    }
}

fn collect_from_function_body<'a>(
    func: &'a oxc_ast::ast::Function<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    refs: &mut Vec<RouteRef>,
) {
    let Some(body) = &func.body else {
        return;
    };
    let mut scoped_bindings = router_bindings.clone();
    remove_shadowed_function_binding(func, &mut scoped_bindings);
    remove_shadowed_parameters(&func.params, &mut scoped_bindings);
    collect_router_bindings_for_scope(&body.statements, &mut scoped_bindings);
    for s in &body.statements {
        collect_from_statement(s, source, file, &mut scoped_bindings, refs);
    }
}
