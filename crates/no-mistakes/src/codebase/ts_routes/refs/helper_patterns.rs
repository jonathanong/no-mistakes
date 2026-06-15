#[derive(Clone, Copy)]
struct HelperDef<'a> {
    name: &'a str,
    params: &'a oxc_ast::ast::FormalParameters<'a>,
    body: &'a oxc_ast::ast::FunctionBody<'a>,
    expression_body: bool,
}

fn collect_route_helpers<'a>(
    program: &'a Program<'a>,
    imports: &[RouteHelperImport],
) -> Vec<RouteHelper> {
    let mut defs = HashMap::new();
    let mut default_alias = None;
    for stmt in &program.body {
        collect_helper_def_from_statement(stmt, &mut defs, &mut default_alias);
    }
    if let Some(def) = default_alias.and_then(|alias| defs.get(alias).copied()) {
        defs.insert(
            "default",
            HelperDef {
                name: "default",
                ..def
            },
        );
    }
    for stmt in &program.body {
        collect_helper_alias_exports_from_statement(stmt, &mut defs);
    }

    let imported_helpers = collect_route_helper_bindings(&[], imports);
    let mut helpers: Vec<RouteHelper> = defs
        .values()
        .filter_map(|def| {
            let patterns = evaluate_helper_def(def, &defs, &imported_helpers, &HashMap::new(), 0);
            (!patterns.is_empty()).then(|| RouteHelper {
                name: def.name.to_string(),
                patterns,
            })
        })
        .collect();
    helpers.sort_by(|a, b| a.name.cmp(&b.name));
    helpers
}

fn collect_helper_def_from_statement<'a>(
    stmt: &'a Statement<'a>,
    defs: &mut HashMap<&'a str, HelperDef<'a>>,
    default_alias: &mut Option<&'a str>,
) {
    match stmt {
        Statement::FunctionDeclaration(func) => {
            if let (Some(id), Some(body)) = (&func.id, &func.body) {
                defs.insert(
                    id.name.as_str(),
                    HelperDef {
                        name: id.name.as_str(),
                        params: &func.params,
                        body,
                        expression_body: false,
                    },
                );
            }
        }
        Statement::VariableDeclaration(var_decl) => {
            collect_helper_defs_from_var_decl(var_decl, defs);
        }
        Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
            Some(oxc_ast::ast::Declaration::FunctionDeclaration(func)) => {
                if let (Some(id), Some(body)) = (&func.id, &func.body) {
                    defs.insert(
                        id.name.as_str(),
                        HelperDef {
                            name: id.name.as_str(),
                            params: &func.params,
                            body,
                            expression_body: false,
                        },
                    );
                }
            }
            Some(oxc_ast::ast::Declaration::VariableDeclaration(var_decl)) => {
                collect_helper_defs_from_var_decl(var_decl, defs);
            }
            _ => {}
        },
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                if let Some(body) = &func.body {
                    if let Some(id) = &func.id {
                        defs.insert(
                            id.name.as_str(),
                            HelperDef {
                                name: id.name.as_str(),
                                params: &func.params,
                                body,
                                expression_body: false,
                            },
                        );
                    }
                    insert_default_helper_def(defs, &func.params, body, false);
                }
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                insert_default_helper_def(defs, &arrow.params, &arrow.body, arrow.expression);
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
                collect_default_helper_def_from_expression(&parenthesized.expression, defs);
            }
            other => {
                if let Some(Expression::Identifier(id)) = other.as_expression() {
                    *default_alias = Some(id.name.as_str());
                }
            }
        },
        _ => {}
    }
}

fn collect_default_helper_def_from_expression<'a>(
    expr: &'a Expression<'a>,
    defs: &mut HashMap<&'a str, HelperDef<'a>>,
) {
    match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            insert_default_helper_def(defs, &arrow.params, &arrow.body, arrow.expression);
        }
        Expression::FunctionExpression(func) => {
            if let Some(body) = &func.body {
                insert_default_helper_def(defs, &func.params, body, false);
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_default_helper_def_from_expression(&parenthesized.expression, defs);
        }
        _ => {}
    }
}

fn insert_default_helper_def<'a>(
    defs: &mut HashMap<&'a str, HelperDef<'a>>,
    params: &'a oxc_ast::ast::FormalParameters<'a>,
    body: &'a oxc_ast::ast::FunctionBody<'a>,
    expression_body: bool,
) {
    defs.insert(
        "default",
        HelperDef {
            name: "default",
            params,
            body,
            expression_body,
        },
    );
}

fn collect_helper_defs_from_var_decl<'a>(
    var_decl: &'a oxc_ast::ast::VariableDeclaration<'a>,
    defs: &mut HashMap<&'a str, HelperDef<'a>>,
) {
    for decl in &var_decl.declarations {
        let Some(name) = binding_identifier_name(&decl.id) else {
            continue;
        };
        let Some(init) = &decl.init else {
            continue;
        };
        match init {
            Expression::ArrowFunctionExpression(arrow) => {
                defs.insert(
                    name,
                    HelperDef {
                        name,
                        params: &arrow.params,
                        body: &arrow.body,
                        expression_body: arrow.expression,
                    },
                );
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    defs.insert(
                        name,
                        HelperDef {
                            name,
                            params: &func.params,
                            body,
                            expression_body: false,
                        },
                    );
                }
            }
            _ => {}
        }
    }
}
