use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    AssignmentTarget, BindingPattern, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    Program, Statement,
};
use std::collections::HashMap;

pub(in super::super) fn extract_config_objects<'a>(
    program: &'a Program<'a>,
) -> Vec<&'a ObjectExpression<'a>> {
    let bindings = collect_top_level_bindings(program);
    let mut out = Vec::new();
    for stmt in &program.body {
        match stmt {
            Statement::ExportDefaultDeclaration(decl) => {
                if let Some(expr) = decl.declaration.as_expression() {
                    collect_config_expr(expr, &bindings, 6, &mut out);
                } else if let ExportDefaultDeclarationKind::FunctionDeclaration(func) =
                    &decl.declaration
                {
                    if let Some(body) = &func.body {
                        collect_statements(&body.statements, &bindings, 6, &mut out);
                    }
                }
            }
            Statement::ExpressionStatement(expr_stmt) => {
                if let Expression::AssignmentExpression(assign) =
                    unwrap_ts_wrappers(&expr_stmt.expression)
                {
                    if is_module_exports(&assign.left) {
                        collect_config_expr(&assign.right, &bindings, 6, &mut out);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn collect_top_level_bindings<'a>(
    program: &'a Program<'a>,
) -> HashMap<&'a str, &'a Expression<'a>> {
    let mut map = HashMap::new();
    for stmt in &program.body {
        let Statement::VariableDeclaration(decl) = stmt else {
            continue;
        };
        for declarator in &decl.declarations {
            if let (BindingPattern::BindingIdentifier(id), Some(init)) =
                (&declarator.id, &declarator.init)
            {
                map.insert(id.name.as_str(), init);
            }
        }
    }
    map
}

fn is_module_exports(target: &AssignmentTarget<'_>) -> bool {
    matches!(
        target,
        AssignmentTarget::StaticMemberExpression(member)
            if matches!(&member.object, Expression::Identifier(id) if id.name == "module")
                && member.property.name == "exports"
    )
}

fn collect_config_expr<'a>(
    expr: &'a Expression<'a>,
    bindings: &HashMap<&'a str, &'a Expression<'a>>,
    depth: u8,
    out: &mut Vec<&'a ObjectExpression<'a>>,
) {
    if depth == 0 {
        return;
    }
    match unwrap_ts_wrappers(expr) {
        Expression::ObjectExpression(obj) => out.push(obj.as_ref()),
        Expression::Identifier(id) => {
            if let Some(bound) = bindings.get(id.name.as_str()) {
                collect_config_expr(bound, bindings, depth - 1, out);
            }
        }
        Expression::CallExpression(call) => {
            for arg in call.arguments.iter().filter_map(|a| a.as_expression()) {
                collect_config_expr(arg, bindings, depth - 1, out);
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if let Some(expr) = arrow.body.as_expression() {
                collect_config_expr(expr, bindings, depth - 1, out);
            } else if let Some(statements) = crate::ast::arrow_function_body_statements(&arrow.body)
            {
                collect_statements(statements, bindings, depth - 1, out);
            }
        }
        Expression::FunctionExpression(func) => {
            if let Some(body) = &func.body {
                collect_statements(&body.statements, bindings, depth - 1, out);
            }
        }
        _ => {}
    }
}

fn collect_statements<'a>(
    statements: &'a [Statement<'a>],
    bindings: &HashMap<&'a str, &'a Expression<'a>>,
    depth: u8,
    out: &mut Vec<&'a ObjectExpression<'a>>,
) {
    for stmt in statements {
        if let Statement::ReturnStatement(ret) = stmt {
            if let Some(arg) = &ret.argument {
                collect_config_expr(arg, bindings, depth, out);
            }
        }
    }
}
