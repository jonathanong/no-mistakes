fn exported_imported_helper_wrapper(
    declaration: &oxc::ast::ast::Declaration<'_>,
    imports: &[RouteHelperImport],
) -> Vec<RouteHelperImport> {
    match declaration {
        oxc::ast::ast::Declaration::FunctionDeclaration(func) => {
            func.id
                .as_ref()
                .zip(func.body.as_ref())
                .and_then(|(id, body)| {
                    single_return_call(body)
                        .and_then(|call| import_for_helper_call(id.name.as_str(), call, imports))
                })
                .into_iter()
                .collect()
        }
        oxc::ast::ast::Declaration::VariableDeclaration(var_decl) => {
            var_decl
                .declarations
                .iter()
                .filter_map(|decl| {
                    let local = binding_identifier_name(&decl.id)?;
                    let init = decl.init.as_ref()?;
                    imported_helper_wrapper_from_expression(local, init, imports)
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn imported_helper_wrapper_from_expression(
    local: &str,
    expr: &Expression,
    imports: &[RouteHelperImport],
) -> Option<RouteHelperImport> {
    match expr {
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            arrow
                .body
                .statements
                .first()
                .and_then(|stmt| match stmt { Statement::ExpressionStatement(expr_stmt) => Some(&expr_stmt.expression), _ => None })
                .and_then(|expr| match expr {
                    Expression::CallExpression(call) => Some(call),
                    _ => None,
                })
                .and_then(|call| import_for_helper_call(local, call, imports))
        }
        Expression::FunctionExpression(func) => {
            import_for_helper_call(local, single_return_call(func.body.as_ref()?)?, imports)
        }
        _ => None,
    }
}

fn single_return_call<'a>(
    body: &'a oxc::ast::ast::FunctionBody<'a>,
) -> Option<&'a oxc::ast::ast::CallExpression<'a>> {
    let Statement::ReturnStatement(ret) = body.statements.first()? else {
        return None;
    };
    let Expression::CallExpression(call) = ret.argument.as_ref()? else {
        return None;
    };
    Some(call)
}

fn import_for_helper_call(
    local: &str,
    call: &oxc::ast::ast::CallExpression<'_>,
    imports: &[RouteHelperImport],
) -> Option<RouteHelperImport> {
    let callee = route_helper_callee_name_from_callee(&call.callee)?;
    if let Some((namespace, member)) = callee.split_once('.') {
        let import = imports
            .iter()
            .find(|import| import.local == namespace && import.imported == "*")?;
        return Some(RouteHelperImport {
            local: local.to_string(),
            imported: member.to_string(),
            source: import.source.clone(),
        });
    }
    let import = imports
        .iter()
        .find(|import| import.local == callee && import.imported != "*")?;
    Some(RouteHelperImport {
        local: local.to_string(),
        imported: import.imported.clone(),
        source: import.source.clone(),
    })
}
