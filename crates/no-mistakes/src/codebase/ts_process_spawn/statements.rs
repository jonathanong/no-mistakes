fn collect_from_stmt(
    stmt: &Statement,
    source: &str,
    file_path: &Path,
    root: &Path,
    out: &mut Vec<SpawnEdge>,
) {
    match stmt {
        Statement::ExpressionStatement(s) => {
            collect_from_expr(&s.expression, source, file_path, root, out);
        }
        Statement::VariableDeclaration(v) => {
            for decl in &v.declarations {
                collect_from_optional_expr(decl.init.as_ref(), source, file_path, root, out);
            }
        }
        Statement::ReturnStatement(r) => {
            collect_from_optional_expr(r.argument.as_ref(), source, file_path, root, out);
        }
        Statement::BlockStatement(b) => {
            for s in &b.body {
                collect_from_stmt(s, source, file_path, root, out);
            }
        }
        Statement::FunctionDeclaration(f) => {
            if let Some(body) = &f.body {
                for s in &body.statements {
                    collect_from_stmt(s, source, file_path, root, out);
                }
            }
        }
        Statement::ExportNamedDeclaration(e) => {
            collect_from_export_named(e, source, file_path, root, out);
        }
        Statement::ExportDefaultDeclaration(e) => {
            collect_from_export_default(&e.declaration, source, file_path, root, out);
        }
        Statement::IfStatement(i) => {
            collect_from_stmt(&i.consequent, source, file_path, root, out);
            if let Some(alt) = &i.alternate {
                collect_from_stmt(alt, source, file_path, root, out);
            }
        }
        Statement::TryStatement(t) => {
            collect_from_try_stmt(t, source, file_path, root, out);
        }
        Statement::WhileStatement(w) => {
            collect_from_stmt(&w.body, source, file_path, root, out);
        }
        Statement::ForStatement(f) => {
            collect_from_stmt(&f.body, source, file_path, root, out);
        }
        Statement::ForInStatement(f) => {
            collect_from_stmt(&f.body, source, file_path, root, out);
        }
        Statement::ForOfStatement(f) => {
            collect_from_stmt(&f.body, source, file_path, root, out);
        }
        _ => {}
    }
}

fn collect_from_optional_expr(
    expr: Option<&Expression>,
    source: &str,
    file_path: &Path,
    root: &Path,
    out: &mut Vec<SpawnEdge>,
) {
    let _ = expr.map(|expr| collect_from_expr(expr, source, file_path, root, out));
}

fn collect_from_export_default(
    kind: &oxc_ast::ast::ExportDefaultDeclarationKind,
    source: &str,
    file_path: &Path,
    root: &Path,
    out: &mut Vec<SpawnEdge>,
) {
    match kind {
        oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
            if let Some(body) = &f.body {
                for s in &body.statements {
                    collect_from_stmt(s, source, file_path, root, out);
                }
            }
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::ArrowFunctionExpression(a) => {
            for s in &a.body.statements {
                collect_from_stmt(s, source, file_path, root, out);
            }
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::CallExpression(call)
            if callee_name(&call.callee) == Some("defineConfig") =>
        {
            extract_define_config_web_server(call, file_path, root, out);
        }
        _ => {}
    }
}
