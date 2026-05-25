use oxc_ast::ast::{Expression, Program, Statement};

pub fn collect_string_leaves(expression: &Expression<'_>) -> Vec<String> {
    match expression {
        Expression::StringLiteral(lit) => vec![lit.value.to_string()],
        Expression::ConditionalExpression(cond) => {
            let mut values = collect_string_leaves(&cond.consequent);
            values.extend(collect_string_leaves(&cond.alternate));
            values
        }
        Expression::LogicalExpression(logical) => {
            let mut values = collect_string_leaves(&logical.left);
            values.extend(collect_string_leaves(&logical.right));
            values
        }
        Expression::TSAsExpression(ts_as) => collect_string_leaves(&ts_as.expression),
        Expression::TSNonNullExpression(ts_non_null) => {
            collect_string_leaves(&ts_non_null.expression)
        }
        Expression::TSTypeAssertion(ts_assert) => collect_string_leaves(&ts_assert.expression),
        Expression::TSSatisfiesExpression(ts_sat) => collect_string_leaves(&ts_sat.expression),
        Expression::ParenthesizedExpression(paren) => collect_string_leaves(&paren.expression),
        _ => vec![],
    }
}

pub fn collect_object_string_values(expr: &Expression<'_>) -> Vec<String> {
    let obj = match expr {
        Expression::ObjectExpression(obj) => obj,
        Expression::TSAsExpression(ts_as) => {
            if let Expression::ObjectExpression(obj) = &ts_as.expression {
                obj
            } else {
                return vec![];
            }
        }
        _ => return vec![],
    };
    let mut values = Vec::new();
    for prop in &obj.properties {
        if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(prop) = prop {
            if prop.computed {
                continue;
            }
            if let Expression::StringLiteral(lit) = &prop.value {
                values.push(lit.value.to_string());
            }
        }
    }
    values
}

pub fn collect_function_return_strings(fn_name: &str, program: &Program<'_>) -> Vec<String> {
    let mut values = Vec::new();
    for stmt in &program.body {
        let function = match stmt {
            Statement::FunctionDeclaration(f) => f,
            Statement::ExportNamedDeclaration(export) => {
                if let Some(oxc_ast::ast::Declaration::FunctionDeclaration(f)) = &export.declaration
                {
                    f
                } else {
                    continue;
                }
            }
            _ => continue,
        };
        if function.id.as_ref().is_some_and(|id| id.name == fn_name) {
            if let Some(body) = &function.body {
                collect_returns_from_statements(&body.statements, &mut values);
            }
        }
    }
    values
}

pub fn collect_returns_from_statements(statements: &[Statement<'_>], values: &mut Vec<String>) {
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
                collect_returns_from_statements(&block.body, values);
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
            collect_returns_from_statements(&block.body, values);
        }
        _ => {}
    }
}

pub(super) fn binding_identifier_name(
    pattern: &oxc_ast::ast::BindingPattern<'_>,
) -> Option<String> {
    match pattern {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
        _ => None,
    }
}

pub(super) fn call_identifier_name(callee: &Expression<'_>) -> Option<String> {
    match callee {
        Expression::Identifier(id) => Some(id.name.to_string()),
        _ => None,
    }
}

pub(super) fn extract_computed_member_object_name<'a>(expr: &'a Expression<'_>) -> Option<&'a str> {
    match expr {
        Expression::ComputedMemberExpression(member) => {
            if let Expression::Identifier(obj_ident) = &member.object {
                Some(obj_ident.name.as_str())
            } else {
                None
            }
        }
        Expression::TSAsExpression(ts_as) => {
            if let Expression::ComputedMemberExpression(member) = &ts_as.expression {
                if let Expression::Identifier(obj_ident) = &member.object {
                    Some(obj_ident.name.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(super) fn collect_assignments_from_stmt<F>(stmt: &Statement<'_>, collector: &mut F)
where
    F: FnMut(&str, &str),
{
    match stmt {
        Statement::BlockStatement(block) => {
            for s in &block.body {
                collect_assignments_from_stmt(s, collector);
            }
        }
        Statement::ExpressionStatement(expr_stmt) => {
            if let Expression::AssignmentExpression(assignment) = &expr_stmt.expression {
                if assignment.operator == oxc_ast::ast::AssignmentOperator::Assign {
                    if let oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(ident) =
                        &assignment.left
                    {
                        if let Expression::StringLiteral(lit) = &assignment.right {
                            collector(ident.name.as_str(), lit.value.as_str());
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
