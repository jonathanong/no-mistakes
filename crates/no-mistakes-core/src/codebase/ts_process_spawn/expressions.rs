fn collect_from_expr(
    expr: &Expression,
    source: &str,
    file_path: &Path,
    root: &Path,
    out: &mut Vec<SpawnEdge>,
) {
    let expr = unwrap_ts_wrappers(expr);
    match expr {
        Expression::CallExpression(call) => {
            // Check for spawn/exec/execFile/fork at top level
            if let Some(fn_name) = callee_name(&call.callee) {
                match fn_name {
                    "spawn" | "execFile" | "fork" => {
                        // First arg is the command/module path
                        let entry = string_or_template_arg(&call.arguments, 0).and_then(|cmd| {
                            let cwd = extract_cwd_from_opts(&call.arguments, 2);
                            resolve_entry_file(&cmd, cwd.as_deref(), file_path, root)
                        });
                        if let Some(entry) = entry {
                            out.push(SpawnEdge {
                                spawner: file_path.to_path_buf(),
                                entry,
                            });
                        }
                    }
                    "exec" => {
                        // exec takes a shell command string; extract the file from it
                        if let Some(cmd) = string_or_template_arg(&call.arguments, 0) {
                            let cwd = extract_cwd_from_opts(&call.arguments, 1);
                            if let Some(entry) =
                                resolve_entry_file_from_shell(&cmd, cwd.as_deref(), file_path, root)
                            {
                                out.push(SpawnEdge {
                                    spawner: file_path.to_path_buf(),
                                    entry,
                                });
                            }
                        }
                    }
                    "defineConfig" => extract_define_config_web_server(call, file_path, root, out),
                    _ => {}
                }
            }
            // Recurse into arguments regardless
            for arg in &call.arguments {
                collect_from_optional_expr(arg.as_expression(), source, file_path, root, out);
            }
            collect_from_expr(&call.callee, source, file_path, root, out);
        }
        Expression::AwaitExpression(a) => {
            collect_from_expr(&a.argument, source, file_path, root, out)
        }
        Expression::ArrowFunctionExpression(a) => {
            for s in &a.body.statements {
                collect_from_stmt(s, source, file_path, root, out);
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                if let ObjectPropertyKind::ObjectProperty(p) = prop {
                    // Look for a top-level webServer: [...] object property
                    if matches!(&p.key, PropertyKey::StaticIdentifier(id) if id.name.as_str() == "webServer")
                    {
                        extract_web_server(&p.value, file_path, root, out);
                    } else {
                        collect_from_expr(&p.value, source, file_path, root, out);
                    }
                }
            }
        }
        _ => {}
    }
}

