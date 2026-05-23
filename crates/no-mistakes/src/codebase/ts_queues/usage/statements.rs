fn scan_stmt(
    stmt: &Statement,
    source: &str,
    namespace_imports: &HashMap<String, String>,
    usage: &mut QueueUsage,
) {
    match stmt {
        Statement::ExpressionStatement(s) => {
            scan_expr(&s.expression, source, namespace_imports, usage);
        }
        Statement::VariableDeclaration(v) => {
            for decl in &v.declarations {
                if let Some(init) = &decl.init {
                    scan_expr(init, source, namespace_imports, usage);
                }
            }
        }
        Statement::ExportNamedDeclaration(e) => {
            if let Some(decl) = &e.declaration {
                match decl {
                    oxc::ast::ast::Declaration::VariableDeclaration(v) => {
                        for d in &v.declarations {
                            if let Some(init) = &d.init {
                                scan_expr(init, source, namespace_imports, usage);
                            }
                        }
                    }
                    oxc::ast::ast::Declaration::FunctionDeclaration(f) => {
                        scan_function_body(f.body.as_deref(), source, namespace_imports, usage);
                    }
                    _ => {}
                }
            }
        }
        Statement::ReturnStatement(r) => {
            if let Some(expr) = &r.argument {
                scan_expr(expr, source, namespace_imports, usage);
            }
        }
        Statement::BlockStatement(b) => {
            for s in &b.body {
                scan_stmt(s, source, namespace_imports, usage);
            }
        }
        Statement::FunctionDeclaration(f) => {
            scan_function_body(f.body.as_deref(), source, namespace_imports, usage);
        }
        Statement::IfStatement(i) => {
            scan_stmt(&i.consequent, source, namespace_imports, usage);
            if let Some(alt) = &i.alternate {
                scan_stmt(alt, source, namespace_imports, usage);
            }
        }
        Statement::TryStatement(t) => {
            for s in &t.block.body {
                scan_stmt(s, source, namespace_imports, usage);
            }
            if let Some(handler) = &t.handler {
                scan_statements(&handler.body.body, source, namespace_imports, usage);
            }
        }
        _ => {}
    }
}

fn scan_statements(
    statements: &[Statement],
    source: &str,
    namespace_imports: &HashMap<String, String>,
    usage: &mut QueueUsage,
) {
    for stmt in statements {
        scan_stmt(stmt, source, namespace_imports, usage);
    }
}

fn scan_function_body(
    body: Option<&FunctionBody>,
    source: &str,
    namespace_imports: &HashMap<String, String>,
    usage: &mut QueueUsage,
) {
    if let Some(body) = body {
        scan_statements(&body.statements, source, namespace_imports, usage);
    }
}
