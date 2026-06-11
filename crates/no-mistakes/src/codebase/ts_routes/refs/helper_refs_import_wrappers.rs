fn exported_imported_helper_wrapper(
    declaration: Option<&oxc::ast::ast::Declaration<'_>>,
    imports: &[RouteHelperImport],
) -> Option<RouteHelperImport> {
    match declaration? {
        oxc::ast::ast::Declaration::FunctionDeclaration(func) => {
            let id = func.id.as_ref()?;
            let call = single_return_call(func.body.as_ref()?)?;
            import_for_helper_call(id.name.as_str(), call, imports)
        }
        oxc::ast::ast::Declaration::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                let Some(local) = binding_identifier_name(&decl.id) else {
                    continue;
                };
                let Some(init) = &decl.init else {
                    continue;
                };
                if let Some(import) = imported_helper_wrapper_from_expression(local, init, imports)
                {
                    return Some(import);
                }
            }
            None
        }
        _ => None,
    }
}

fn imported_helper_wrapper_from_expression(
    local: &str,
    expr: &Expression,
    imports: &[RouteHelperImport],
) -> Option<RouteHelperImport> {
    match expr {
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            let Statement::ExpressionStatement(expr_stmt) = arrow.body.statements.first()? else {
                return None;
            };
            let Expression::CallExpression(call) = &expr_stmt.expression else {
                return None;
            };
            import_for_helper_call(local, call, imports)
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
