fn extract_web_server(expr: &Expression, file_path: &Path, root: &Path, out: &mut Vec<SpawnEdge>) {
    let expr = unwrap_ts_wrappers(expr);
    match expr {
        Expression::ArrayExpression(arr) => {
            for item in &arr.elements {
                extract_optional_web_server_entry(item.as_expression(), file_path, root, out);
            }
        }
        _ => extract_web_server_entry(expr, file_path, root, out),
    }
}

fn extract_optional_web_server_entry(
    expr: Option<&Expression>,
    file_path: &Path,
    root: &Path,
    out: &mut Vec<SpawnEdge>,
) {
    let _ = expr.map(|expr| extract_web_server_entry(expr, file_path, root, out));
}

fn extract_define_config_web_server(
    call: &oxc_ast::ast::CallExpression,
    file_path: &Path,
    root: &Path,
    out: &mut Vec<SpawnEdge>,
) {
    let Some(Argument::ObjectExpression(obj)) = call.arguments.first() else {
        return;
    };
    for p in obj.properties.iter().filter_map(|prop| match prop {
        ObjectPropertyKind::ObjectProperty(p) => Some(p),
        _ => None,
    }) {
        if matches!(&p.key, PropertyKey::StaticIdentifier(id) if id.name.as_str() == "webServer") {
            extract_web_server(&p.value, file_path, root, out);
        }
    }
}

fn extract_web_server_entry(
    expr: &Expression,
    file_path: &Path,
    root: &Path,
    out: &mut Vec<SpawnEdge>,
) {
    let expr = unwrap_ts_wrappers(expr);
    if let Expression::ObjectExpression(obj) = expr {
        let mut command: Option<String> = None;
        let mut cwd: Option<PathBuf> = None;

        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(p) = prop {
                let key = match &p.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                    _ => continue,
                };
                match key {
                    "command" => {
                        command = string_or_template_literal(&p.value);
                    }
                    "cwd" => {
                        // cwd may be a dynamic join() — only take literal strings
                        assign_literal_cwd(&mut cwd, &p.value);
                    }
                    _ => {}
                }
            }
        }

        if let Some(cmd) = command {
            let cwd_str = cwd.as_deref().and_then(|p| p.to_str());
            if let Some(entry) = resolve_entry_file_from_shell(&cmd, cwd_str, file_path, root) {
                out.push(SpawnEdge {
                    spawner: file_path.to_path_buf(),
                    entry,
                });
            }
        }
    }
}

fn collect_from_export_named(
    e: &ExportNamedDeclaration,
    source: &str,
    file_path: &Path,
    root: &Path,
    out: &mut Vec<SpawnEdge>,
) {
    let Some(decl) = &e.declaration else { return };
    match decl {
        oxc_ast::ast::Declaration::VariableDeclaration(v) => {
            for d in &v.declarations {
                collect_from_optional_expr(d.init.as_ref(), source, file_path, root, out);
            }
        }
        oxc_ast::ast::Declaration::FunctionDeclaration(f) => {
            if let Some(body) = &f.body {
                for s in &body.statements {
                    collect_from_stmt(s, source, file_path, root, out);
                }
            }
        }
        _ => {}
    }
}

fn collect_from_try_stmt(
    t: &TryStatement,
    source: &str,
    file_path: &Path,
    root: &Path,
    out: &mut Vec<SpawnEdge>,
) {
    for s in &t.block.body {
        collect_from_stmt(s, source, file_path, root, out);
    }
    if let Some(handler) = &t.handler {
        for s in &handler.body.body {
            collect_from_stmt(s, source, file_path, root, out);
        }
    }
    if let Some(finalizer) = &t.finalizer {
        for s in &finalizer.body {
            collect_from_stmt(s, source, file_path, root, out);
        }
    }
}
