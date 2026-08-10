fn insert_default_arrow_helper_def<'a>(
    defs: &mut HashMap<&'a str, HelperDef<'a>>,
    arrow: &'a oxc_ast::ast::ArrowFunctionExpression<'a>,
) {
    if let Some(body) = crate::ast::arrow_function_body(&arrow.body) {
        insert_default_helper_def(defs, &arrow.params, body);
    } else if let Some(expression) = arrow.body.as_expression() {
        defs.insert(
            "default",
            HelperDef {
                name: "default",
                params: &arrow.params,
                body: None,
                expression: Some(expression),
            },
        );
    }
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
                if let Some(body) = crate::ast::arrow_function_body(&arrow.body) {
                    defs.insert(
                        name,
                        HelperDef {
                            name,
                            params: &arrow.params,
                            body: Some(body),
                            expression: None,
                        },
                    );
                } else if let Some(expression) = arrow.body.as_expression() {
                    defs.insert(
                        name,
                        HelperDef {
                            name,
                            params: &arrow.params,
                            body: None,
                            expression: Some(expression),
                        },
                    );
                }
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    defs.insert(
                        name,
                        HelperDef {
                            name,
                            params: &func.params,
                            body: Some(body),
                            expression: None,
                        },
                    );
                }
            }
            _ => {}
        }
    }
}
