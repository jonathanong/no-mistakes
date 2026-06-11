#[derive(Clone, Copy)]
struct HelperDef<'a> {
    name: &'a str,
    params: &'a oxc::ast::ast::FormalParameters<'a>,
    body: &'a oxc::ast::ast::FunctionBody<'a>,
    expression_body: bool,
}

fn collect_route_helpers<'a>(program: &'a Program<'a>) -> Vec<RouteHelper> {
    let mut defs = HashMap::new();
    for stmt in &program.body {
        collect_helper_def_from_statement(stmt, &mut defs);
    }

    let mut helpers: Vec<RouteHelper> = defs
        .values()
        .filter_map(|def| {
            let patterns = evaluate_helper_def(def, &defs, &HashMap::new(), 0);
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
            Some(oxc::ast::ast::Declaration::FunctionDeclaration(func)) => {
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
            Some(oxc::ast::ast::Declaration::VariableDeclaration(var_decl)) => {
                collect_helper_defs_from_var_decl(var_decl, defs);
            }
            _ => {}
        },
        _ => {}
    }
}

fn collect_helper_defs_from_var_decl<'a>(
    var_decl: &'a oxc::ast::ast::VariableDeclaration<'a>,
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
