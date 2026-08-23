fn collect_from_default_export(kind: &ExportDefaultDeclarationKind<'_>, values: &mut Vec<String>) {
    match kind {
        ExportDefaultDeclarationKind::ObjectExpression(obj) => {
            for prop in &obj.properties {
                if let ObjectPropertyKind::ObjectProperty(p) = prop {
                    if !p.computed {
                        if let Expression::StringLiteral(lit) = &p.value {
                            values.push(lit.value.to_string());
                        }
                    }
                }
            }
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            if let Some(body) = &func.body {
                collect_returns_from_function_body(&body.statements, values);
            }
        }
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
            if let Some(expression) = arrow.body.as_expression() {
                values.extend(collect_string_leaves(expression));
            } else if let Some(statements) = crate::ast::arrow_function_body_statements(&arrow.body) {
                collect_returns_from_function_body(statements, values);
            }
        }
        _ => {}
    }
}

fn collect_from_named_declaration(
    decl: &Declaration<'_>,
    exported_name: &str,
    values: &mut Vec<String>,
) {
    match decl {
        Declaration::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                let Some(name) = binding_ident_name(&declarator.id) else {
                    continue;
                };
                if name != exported_name {
                    continue;
                }
                let Some(init) = declarator.init.as_ref() else {
                    continue;
                };
                let leaves = collect_string_leaves(init);
                if !leaves.is_empty() {
                    values.extend(leaves);
                    continue;
                }
                values.extend(collect_object_string_values(init));
            }
        }
        Declaration::FunctionDeclaration(func)
            if func.id.as_ref().is_some_and(|id| id.name == exported_name) =>
        {
            if let Some(body) = &func.body {
                collect_returns_from_function_body(&body.statements, values);
            }
        }
        _ => {}
    }
}

fn collect_returns_from_function_body(statements: &[Statement<'_>], values: &mut Vec<String>) {
    for stmt in statements {
        match stmt {
            Statement::ReturnStatement(ret) => {
                if let Some(expr) = &ret.argument {
                    values.extend(collect_string_leaves(expr));
                }
            }
            Statement::IfStatement(if_stmt) => {
                collect_returns_from_stmt(&if_stmt.consequent, values);
                if let Some(alt) = &if_stmt.alternate {
                    collect_returns_from_stmt(alt, values);
                }
            }
            Statement::BlockStatement(block) => {
                collect_returns_from_function_body(&block.body, values);
            }
            _ => {}
        }
    }
}

fn collect_returns_from_stmt(stmt: &Statement<'_>, values: &mut Vec<String>) {
    match stmt {
        Statement::ReturnStatement(ret) => {
            if let Some(expr) = &ret.argument {
                values.extend(collect_string_leaves(expr));
            }
        }
        Statement::BlockStatement(block) => {
            collect_returns_from_function_body(&block.body, values);
        }
        _ => {}
    }
}

fn binding_ident_name(pattern: &oxc_ast::ast::BindingPattern<'_>) -> Option<String> {
    match pattern {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
        _ => None,
    }
}

fn find_import_info(local_name: &str, program: &Program<'_>) -> Option<(String, String, bool)> {
    program.body.iter().find_map(|stmt| {
        let Statement::ImportDeclaration(import) = stmt else {
            return None;
        };

        import
            .specifiers
            .as_ref()?
            .iter()
            .find_map(|specifier| match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(named)
                    if named.local.name == local_name =>
                {
                    Some((
                        import.source.value.to_string(),
                        named.imported.name().to_string(),
                        false,
                    ))
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default)
                    if default.local.name == local_name =>
                {
                    Some((import.source.value.to_string(), "default".to_string(), true))
                }
                _ => None,
            })
    })
}
